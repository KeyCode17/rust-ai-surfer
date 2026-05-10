use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use ras_cdp::BrowserPort;
use ras_errors::AppError;
use ras_events::EventBus;
use ras_llm::{ChatMessage, ChatResponse, InvokeOptions, LlmClient};
use ras_tools::domain::registry::{ActionRegistry, ToolContext};
use ras_types::{ActionResult, StepId};
use url::Url;

use crate::application::compute_action_hash::compute_action_hash;
use crate::application::detect_loop::{build_budget_warning, build_loop_nudge};
use crate::application::fallback_llm::should_switch_to_fallback;
use crate::application::parse_output::parse_agent_output;
use crate::domain::agent_history::StepRecord;
use crate::domain::loop_detector::ActionLoopDetector;
use crate::domain::step_metadata::StepMetadata;

pub struct RunStep {
    primary_llm: Arc<dyn LlmClient>,
    fallback_llm: Option<Arc<dyn LlmClient>>,
    registry: Arc<ActionRegistry>,
    browser: Arc<dyn BrowserPort>,
    events: Arc<dyn EventBus>,
}

impl RunStep {
    #[must_use]
    pub fn new(
        primary: Arc<dyn LlmClient>,
        fallback: Option<Arc<dyn LlmClient>>,
        registry: Arc<ActionRegistry>,
        browser: Arc<dyn BrowserPort>,
        events: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            primary_llm: primary,
            fallback_llm: fallback,
            registry,
            browser,
            events,
        }
    }

    pub async fn execute(
        &self,
        step: StepId,
        max_steps: u32,
        prompt: Vec<ChatMessage>,
        detector: &mut ActionLoopDetector,
    ) -> Result<StepRecord, AppError> {
        let started = Instant::now();
        let mut messages = prompt;
        if let Some(nudge) = build_loop_nudge(detector) {
            messages.push(nudge);
        }
        if let Some(warn) = build_budget_warning(step.0, max_steps) {
            messages.push(warn);
        }
        let response = self.invoke_with_fallback(messages).await?;
        let output = parse_agent_output(&response)?;

        let target = self.browser.focused_target().await.ok();
        let page_url = match &target {
            Some(t) => self
                .browser
                .evaluate(t, "location.href")
                .await
                .ok()
                .and_then(|v| v.as_str().and_then(|s| Url::parse(s).ok())),
            None => None,
        };

        let mut results = Vec::new();
        for action in &output.action {
            detector.record_action(compute_action_hash(action));
            let Some(reg) = self.registry.get(&action.name) else {
                results.push(ActionResult::err(format!(
                    "unknown action: {}",
                    action.name.0
                )));
                break;
            };
            let ctx = ToolContext {
                browser: self.browser.clone(),
                events: self.events.clone(),
                page_url: page_url.clone(),
                available_files: Vec::new(),
            };
            match reg.handler.execute(action.parameters.clone(), ctx).await {
                Ok(r) => {
                    let terminates = reg.metadata.terminates_sequence;
                    let is_done = r.is_done;
                    let is_err = r.is_error();
                    results.push(r);
                    if terminates || is_done || is_err {
                        break;
                    }
                }
                Err(e) => {
                    results.push(ActionResult::err(e.to_string()));
                    break;
                }
            }
        }

        let metadata = StepMetadata {
            duration_ms: started.elapsed().as_millis() as u64,
            step_interval_ms: None,
            usage: response.usage,
            model: Some(response.model.clone()),
            fallback_used: false,
        };
        Ok(StepRecord {
            step,
            started_at: Utc::now(),
            url: page_url,
            output,
            results,
            metadata,
        })
    }

    async fn invoke_with_fallback(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResponse, AppError> {
        let opts = InvokeOptions::default();
        match self
            .primary_llm
            .ainvoke(messages.clone(), opts.clone())
            .await
        {
            Ok(r) => Ok(r),
            Err(e) if should_switch_to_fallback(&e) => match &self.fallback_llm {
                Some(fb) => fb.ainvoke(messages, opts).await,
                None => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

#[must_use]
pub fn done_result(text: impl Into<String>) -> ActionResult {
    ActionResult::done(text)
}

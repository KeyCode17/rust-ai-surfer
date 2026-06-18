use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use ras_cdp::BrowserPort;
use ras_dom::DomExtractor;
use ras_errors::AppError;
use ras_events::EventBus;
use ras_llm::{ChatMessage, ChatResponse, InvokeOptions, LlmClient};
use ras_tools::domain::registry::{ActionRegistry, ToolContext};
use ras_types::{ActionResult, StepId, TargetId};

use crate::application::clickable_map::build_current_page_message;
use crate::application::compute_action_hash::compute_action_hash;
use crate::application::detect_loop::{build_budget_warning, build_loop_nudge};
use crate::application::fallback_llm::should_switch_to_fallback;
use crate::application::parse_output::parse_agent_output;
use crate::application::run_step_log::{log_action_err, log_action_ok, log_decision};
use crate::application::salvage::salvage_into;
use crate::domain::agent_history::StepRecord;
use crate::domain::loop_detector::ActionLoopDetector;
use crate::domain::step_metadata::StepMetadata;

pub struct RunStep {
    primary_llm: Arc<dyn LlmClient>,
    fallback_llm: Option<Arc<dyn LlmClient>>,
    registry: Arc<ActionRegistry>,
    browser: Arc<dyn BrowserPort>,
    events: Arc<dyn EventBus>,
    dom_extractor: Option<Arc<dyn DomExtractor>>,
    bound_target: Option<TargetId>,
}

impl RunStep {
    #[must_use]
    pub fn new(
        primary: Arc<dyn LlmClient>,
        fallback: Option<Arc<dyn LlmClient>>,
        registry: Arc<ActionRegistry>,
        browser: Arc<dyn BrowserPort>,
        events: Arc<dyn EventBus>,
        dom_extractor: Option<Arc<dyn DomExtractor>>,
        bound_target: Option<TargetId>,
    ) -> Self {
        Self {
            primary_llm: primary,
            fallback_llm: fallback,
            registry,
            browser,
            events,
            dom_extractor,
            bound_target,
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
        let target = match &self.bound_target {
            Some(t) => Some(t.clone()),
            None => self.browser.focused_target().await.ok(),
        };
        let current = match (&self.dom_extractor, &target) {
            (Some(extractor), Some(t)) => extractor.snapshot(t).await.ok(),
            _ => None,
        };
        if let Some(page_msg) = current.as_ref().and_then(build_current_page_message) {
            messages.push(page_msg);
        }
        let pre_clickables: Arc<Vec<ras_dom::ClickableElement>> = Arc::new(
            current
                .as_ref()
                .map(|s| s.clickables.clone())
                .unwrap_or_default(),
        );
        let page_url = current.as_ref().map(|s| s.url.clone());

        let response = self.invoke_with_fallback(messages).await?;
        let mut output = parse_agent_output(&response)?;
        salvage_into(&mut output, &self.registry);
        log_decision(step.0, &output);

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
                target: target.clone(),
                browser: self.browser.clone(),
                events: self.events.clone(),
                page_url: page_url.clone(),
                available_files: Vec::new(),
                clickables: pre_clickables.clone(),
            };
            match reg.handler.execute(action.parameters.clone(), ctx).await {
                Ok(r) => {
                    let terminates = reg.metadata.terminates_sequence;
                    let is_done = r.is_done;
                    let is_err = r.is_error();
                    log_action_ok(step.0, action, &r);
                    results.push(r);
                    if terminates || is_done || is_err {
                        break;
                    }
                }
                Err(e) => {
                    log_action_err(step.0, action, &e.to_string());
                    results.push(ActionResult::err(e.to_string()));
                    break;
                }
            }
        }

        let summary = match (&self.dom_extractor, &target) {
            (Some(extractor), Some(t)) => match extractor.snapshot(t).await {
                Ok(s) => Some(s),
                Err(e) => {
                    tracing::warn!(error = %e, "dom snapshot failed; continuing without grounding");
                    None
                }
            },
            _ => None,
        };

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
            summary,
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

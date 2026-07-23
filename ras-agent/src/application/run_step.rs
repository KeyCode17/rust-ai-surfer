use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use ras_cdp::BrowserPort;
use ras_dom::DomExtractor;
use ras_errors::AppError;
use ras_events::EventBus;
use ras_llm::{ChatMessage, ChatResponse, InvokeOptions, LlmClient};
use ras_tools::domain::registry::{ActionRegistry, ToolContext};
use ras_types::{ActionResult, AgentId, StepId, TargetId};

use crate::application::capture_step_screenshot::capture_step_screenshot;
use crate::application::clickable_map::build_current_page_message;
use crate::application::compute_action_hash::compute_action_hash;
use crate::application::detect_loop::{build_budget_warning, build_loop_nudge};
use crate::application::fallback_llm::should_switch_to_fallback;
use crate::application::parse_output::parse_agent_output;
use crate::application::run_step_log::{log_action_err, log_action_ok, log_decision};
use crate::application::salvage::salvage_into;
use crate::domain::agent_history::StepRecord;
use crate::domain::loop_detector::ActionLoopDetector;
use crate::domain::screenshot_sink::StepScreenshotSink;
use crate::domain::step_metadata::StepMetadata;

pub struct RunStepDeps {
    pub agent: AgentId,
    pub primary_llm: Arc<dyn LlmClient>,
    pub fallback_llm: Option<Arc<dyn LlmClient>>,
    pub registry: Arc<ActionRegistry>,
    pub browser: Arc<dyn BrowserPort>,
    pub events: Arc<dyn EventBus>,
    pub dom_extractor: Option<Arc<dyn DomExtractor>>,
    pub bound_target: Option<TargetId>,
    pub screenshot_sink: Option<Arc<dyn StepScreenshotSink>>,
}

pub struct RunStep {
    deps: RunStepDeps,
}

impl RunStep {
    #[must_use]
    pub fn new(deps: RunStepDeps) -> Self {
        Self { deps }
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
        let target = match &self.deps.bound_target {
            Some(t) => Some(t.clone()),
            None => self.deps.browser.focused_target().await.ok(),
        };
        let current = match (&self.deps.dom_extractor, &target) {
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
        salvage_into(&mut output, &self.deps.registry);
        log_decision(step.0, &output);

        let mut results = Vec::new();
        for action in &output.action {
            detector.record_action(compute_action_hash(action));
            let Some(reg) = self.deps.registry.get(&action.name) else {
                results.push(ActionResult::err(format!(
                    "unknown action: {}",
                    action.name.0
                )));
                break;
            };
            let ctx = ToolContext {
                target: target.clone(),
                browser: self.deps.browser.clone(),
                events: self.deps.events.clone(),
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

        let summary = match (&self.deps.dom_extractor, &target) {
            (Some(extractor), Some(t)) => match extractor.snapshot(t).await {
                Ok(s) => Some(s),
                Err(e) => {
                    tracing::warn!(error = %e, "dom snapshot failed; continuing without grounding");
                    None
                }
            },
            _ => None,
        };
        let screenshot = match (&self.deps.screenshot_sink, &target) {
            (Some(sink), Some(t)) => {
                capture_step_screenshot(&self.deps.browser, sink, self.deps.agent, step, t).await
            }
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
            screenshot,
        })
    }

    async fn invoke_with_fallback(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResponse, AppError> {
        let opts = InvokeOptions::default();
        match self
            .deps
            .primary_llm
            .ainvoke(messages.clone(), opts.clone())
            .await
        {
            Ok(r) => Ok(r),
            Err(e) if should_switch_to_fallback(&e) => match &self.deps.fallback_llm {
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

use std::sync::Arc;

use ras_cdp::BrowserPort;
use ras_dom::DomExtractor;
use ras_errors::AppError;
use ras_events::EventBus;
use ras_llm::{ChatMessage, LlmClient};
use ras_tools::domain::registry::ActionRegistry;
use ras_types::{AgentId, StepId, TargetId};

use crate::application::detect_loop::build_empty_action_nudge;
use crate::application::render_step_message::render_step_message;
use crate::application::run_step::{RunStep, RunStepDeps};
use crate::domain::agent_history::{AgentHistory, AgentHistoryList, StepRecord};
use crate::domain::loop_detector::ActionLoopDetector;
use crate::domain::screenshot_sink::StepScreenshotSink;

const AGENT_OUTPUT_SHAPE: &str = r#"{"current_state":{"evaluation_previous_goal":"...","memory":"...","next_goal":"..."},"action":[{"name":"<action_name>","parameters":{...}}]}"#;

pub struct RunAgent {
    pub agent: AgentId,
    pub task: String,
    pub max_steps: u32,
    pub bound_target: Option<TargetId>,
    pub primary_llm: Arc<dyn LlmClient>,
    pub fallback_llm: Option<Arc<dyn LlmClient>>,
    pub registry: Arc<ActionRegistry>,
    pub browser: Arc<dyn BrowserPort>,
    pub events: Arc<dyn EventBus>,
    pub dom_extractor: Option<Arc<dyn DomExtractor>>,
    pub screenshot_sink: Option<Arc<dyn StepScreenshotSink>>,
}

impl RunAgent {
    pub fn new(
        task: impl Into<String>,
        llm: Arc<dyn LlmClient>,
        registry: Arc<ActionRegistry>,
        browser: Arc<dyn BrowserPort>,
        events: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            agent: AgentId::new(),
            task: task.into(),
            max_steps: 25,
            bound_target: None,
            primary_llm: llm,
            fallback_llm: None,
            registry,
            browser,
            events,
            dom_extractor: None,
            screenshot_sink: None,
        }
    }

    /// Capture one screenshot per step and hand it to `sink`.
    #[must_use]
    pub fn with_screenshot_sink(mut self, sink: Arc<dyn StepScreenshotSink>) -> Self {
        self.screenshot_sink = Some(sink);
        self
    }

    #[must_use]
    pub fn with_target(mut self, target: TargetId) -> Self {
        self.bound_target = Some(target);
        self
    }

    #[must_use]
    pub fn with_dom_extractor(mut self, extractor: Arc<dyn DomExtractor>) -> Self {
        self.dom_extractor = Some(extractor);
        self
    }

    #[must_use]
    pub fn with_fallback(mut self, fallback: Arc<dyn LlmClient>) -> Self {
        self.fallback_llm = Some(fallback);
        self
    }

    #[must_use]
    pub fn with_max_steps(mut self, max_steps: u32) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub async fn execute(self) -> Result<AgentHistoryList, AppError> {
        let runner = RunStep::new(RunStepDeps {
            agent: self.agent,
            primary_llm: self.primary_llm.clone(),
            fallback_llm: self.fallback_llm.clone(),
            registry: self.registry.clone(),
            browser: self.browser.clone(),
            events: self.events.clone(),
            dom_extractor: self.dom_extractor.clone(),
            bound_target: self.bound_target.clone(),
            screenshot_sink: self.screenshot_sink.clone(),
        });
        let mut detector = ActionLoopDetector::new();
        let mut history = AgentHistory {
            agent: self.agent,
            task: self.task.clone(),
            steps: Vec::new(),
        };
        let mut last_step_ms: Option<u64> = None;
        let mut empty_streak: u32 = 0;
        let mut prev_empty = false;
        for n in 0..self.max_steps {
            let mut prompt = build_prompt(&self.task, &history.steps, &self.registry);
            if let Some(nudge) = build_empty_action_nudge(prev_empty) {
                prompt.push(nudge);
            }
            let mut record = runner
                .execute(StepId(n), self.max_steps, prompt, &mut detector)
                .await?;
            record.metadata.step_interval_ms = last_step_ms;
            last_step_ms = Some(record.metadata.duration_ms);

            let done = record.results.iter().any(|r| r.is_done);
            if record.output.action.is_empty() {
                empty_streak += 1;
                prev_empty = true;
                tracing::warn!(
                    step = n,
                    "model returned empty action list (streak={empty_streak}); re-prompting once, then stall"
                );
            } else {
                empty_streak = 0;
                prev_empty = false;
            }

            history.steps.push(record);
            if done {
                break;
            }
            if empty_streak >= 2 {
                tracing::error!("agent stalled: 2 consecutive empty action lists, aborting");
                break;
            }
        }
        let mut list = AgentHistoryList::default();
        list.push(history);
        Ok(list)
    }
}

fn build_prompt(task: &str, history: &[StepRecord], registry: &ActionRegistry) -> Vec<ChatMessage> {
    let catalog = render_action_catalog(registry);
    let system = format!(
        "You are a browsing agent. Your task: {task}\n\n\
         You drive a real browser via the actions listed below. Each step you must \
         emit ONE JSON object matching this exact shape (no prose, no markdown fences):\n\
         {AGENT_OUTPUT_SHAPE}\n\n\
         Available actions:\n{catalog}\n\n\
         Rules:\n\
         - Use only action names from the catalog above. Match parameters_schema exactly.\n\
         - Plan one or two atomic steps at a time; the runtime executes them in order \
           and feeds results back on the next turn.\n\
         - Call the `done` action when the task is complete; pass the final answer in \
           its parameters. Returning an empty action list is treated as a failure."
    );
    let mut out = vec![ChatMessage::system_cached(system)];
    for step in history.iter().rev().take(4).rev() {
        if let Ok(j) = serde_json::to_string(&step.output) {
            out.push(ChatMessage::assistant_text(j));
        }
        if let Some(msg) = render_step_message(step) {
            out.push(msg);
        }
    }
    out.push(ChatMessage::user_text(
        "Decide the next action(s). Respond with the JSON object only.",
    ));
    out
}

fn render_action_catalog(registry: &ActionRegistry) -> String {
    let mut buf = String::new();
    for (name, reg) in registry.iter() {
        let schema =
            serde_json::to_string(&reg.metadata.parameters_schema).unwrap_or_else(|_| "{}".into());
        buf.push_str("- ");
        buf.push_str(&name.0);
        buf.push_str(": ");
        buf.push_str(&reg.metadata.description);
        if reg.metadata.terminates_sequence {
            buf.push_str(" [terminates step]");
        }
        buf.push_str("\n  parameters_schema: ");
        buf.push_str(&schema);
        buf.push('\n');
    }
    if buf.is_empty() {
        buf.push_str("(no actions registered)\n");
    }
    buf
}

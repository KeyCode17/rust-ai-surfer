use std::sync::Arc;

use ras_cdp::BrowserPort;
use ras_errors::AppError;
use ras_events::EventBus;
use ras_llm::{ChatMessage, LlmClient};
use ras_tools::domain::registry::ActionRegistry;
use ras_types::{AgentId, StepId};

use crate::application::run_step::RunStep;
use crate::domain::agent_history::{AgentHistory, AgentHistoryList, StepRecord};
use crate::domain::loop_detector::ActionLoopDetector;

pub struct RunAgent {
    pub agent: AgentId,
    pub task: String,
    pub max_steps: u32,
    pub primary_llm: Arc<dyn LlmClient>,
    pub fallback_llm: Option<Arc<dyn LlmClient>>,
    pub registry: Arc<ActionRegistry>,
    pub browser: Arc<dyn BrowserPort>,
    pub events: Arc<dyn EventBus>,
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
            primary_llm: llm,
            fallback_llm: None,
            registry,
            browser,
            events,
        }
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
        let runner = RunStep::new(
            self.primary_llm.clone(),
            self.fallback_llm.clone(),
            self.registry.clone(),
            self.browser.clone(),
            self.events.clone(),
        );
        let mut detector = ActionLoopDetector::new();
        let mut history = AgentHistory {
            agent: self.agent,
            task: self.task.clone(),
            steps: Vec::new(),
        };
        let mut last_step_ms: Option<u64> = None;
        for n in 0..self.max_steps {
            let prompt = build_prompt(&self.task, &history.steps);
            let mut record = runner
                .execute(StepId(n), self.max_steps, prompt, &mut detector)
                .await?;
            record.metadata.step_interval_ms = last_step_ms;
            last_step_ms = Some(record.metadata.duration_ms);
            let done = record.results.iter().any(|r| r.is_done) || record.output.action.is_empty();
            history.steps.push(record);
            if done {
                break;
            }
        }
        let mut list = AgentHistoryList::default();
        list.push(history);
        Ok(list)
    }
}

fn build_prompt(task: &str, history: &[StepRecord]) -> Vec<ChatMessage> {
    let mut out = vec![ChatMessage::system(format!(
        "You are a browsing agent. Task: {task}\nReturn JSON with current_state and action[]."
    ))];
    for step in history.iter().rev().take(4).rev() {
        if let Ok(j) = serde_json::to_string(&step.output) {
            out.push(ChatMessage::assistant_text(j));
        }
    }
    out.push(ChatMessage::user_text(
        "Decide the next action(s). Return JSON {current_state, action: [...]}",
    ));
    out
}

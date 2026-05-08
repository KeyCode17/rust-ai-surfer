use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use ras_errors::AppError;
use ras_llm::{ChatMessage, ChatResponse, InvokeOptions, LlmClient};
use ras_types::{ActionResult, StepId};

use crate::application::compute_action_hash::compute_action_hash;
use crate::application::detect_loop::{build_budget_warning, build_loop_nudge};
use crate::application::fallback_llm::should_switch_to_fallback;
use crate::domain::agent_history::StepRecord;
use crate::domain::agent_output::{ActionInvocation, AgentBrain, AgentOutput};
use crate::domain::loop_detector::ActionLoopDetector;
use crate::domain::step_metadata::StepMetadata;

pub struct RunStep {
    primary_llm: Arc<dyn LlmClient>,
    fallback_llm: Option<Arc<dyn LlmClient>>,
}

impl RunStep {
    #[must_use]
    pub fn new(primary: Arc<dyn LlmClient>, fallback: Option<Arc<dyn LlmClient>>) -> Self {
        Self {
            primary_llm: primary,
            fallback_llm: fallback,
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
        for action in &output.action {
            detector.record_action(compute_action_hash(action));
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
            url: None,
            output,
            results: Vec::new(),
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

fn parse_agent_output(response: &ChatResponse) -> Result<AgentOutput, AppError> {
    if let Some(content) = &response.content {
        if let Ok(parsed) = serde_json::from_str::<AgentOutput>(content) {
            return Ok(parsed);
        }
    }
    Ok(AgentOutput {
        current_state: AgentBrain {
            evaluation_previous_goal: String::new(),
            memory: String::new(),
            next_goal: response.content.clone().unwrap_or_default(),
        },
        action: tool_calls_to_actions(&response.tool_calls),
        plan: None,
        current_plan_item: None,
    })
}

fn tool_calls_to_actions(calls: &[ras_llm::ToolCall]) -> Vec<ActionInvocation> {
    calls
        .iter()
        .map(|c| ActionInvocation {
            name: ras_types::ActionName(c.name.clone().into()),
            parameters: c.arguments.clone(),
        })
        .collect()
}

#[must_use]
pub fn done_result(text: impl Into<String>) -> ActionResult {
    ActionResult::done(text)
}

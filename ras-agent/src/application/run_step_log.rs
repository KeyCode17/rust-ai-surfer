use ras_types::ActionResult;

use crate::domain::agent_output::{ActionInvocation, AgentOutput};

pub(crate) fn log_decision(step: u32, output: &AgentOutput) {
    tracing::info!(
        target: "ras_agent::brain",
        step = step,
        eval = %output.current_state.evaluation_previous_goal,
        memory = %output.current_state.memory,
        next_goal = %output.current_state.next_goal,
        actions = ?output.action.iter().map(|a| a.name.0.as_str()).collect::<Vec<_>>(),
        "agent decision"
    );
    for a in &output.action {
        tracing::debug!(
            target: "ras_agent::brain",
            step = step,
            action = %a.name.0,
            params = %a.parameters,
            "agent action params"
        );
    }
}

pub(crate) fn log_action_ok(step: u32, action: &ActionInvocation, r: &ActionResult) {
    tracing::info!(
        target: "ras_agent::brain",
        step = step,
        action = %action.name.0,
        ok = r.error.is_none(),
        done = r.is_done,
        result = %r.extracted_content.as_deref().unwrap_or(""),
        error = %r.error.as_deref().unwrap_or(""),
        "action result"
    );
}

pub(crate) fn log_action_err(step: u32, action: &ActionInvocation, err: &str) {
    tracing::warn!(
        target: "ras_agent::brain",
        step = step,
        action = %action.name.0,
        error = err,
        "action failed"
    );
}

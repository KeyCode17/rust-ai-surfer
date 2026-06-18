use ras_llm::ChatMessage;

use crate::domain::loop_detector::ActionLoopDetector;

#[must_use]
pub fn build_loop_nudge(detector: &ActionLoopDetector) -> Option<ChatMessage> {
    let mut parts: Vec<String> = Vec::new();
    if detector.action_loop_detected() {
        parts.push(
            "Heads up: the same action repeated several times. Try a different element or strategy."
                .into(),
        );
    }
    if detector.page_stagnation_detected() {
        parts.push(
            "Heads up: the page state has not changed for several steps. Reassess your approach."
                .into(),
        );
    }
    if parts.is_empty() {
        return None;
    }
    Some(ChatMessage::system(parts.join("\n\n")))
}

#[must_use]
pub fn build_budget_warning(step: u32, max_steps: u32) -> Option<ChatMessage> {
    if max_steps == 0 {
        return None;
    }
    let pct = (step * 100) / max_steps.max(1);
    if pct < 95 {
        return None;
    }
    Some(ChatMessage::system(format!(
        "You are at {pct}% of your step budget. Wrap up: call done if you have an answer, or pivot decisively."
    )))
}

/// One-time re-prompt issued the step AFTER an empty action list: the model put
/// its intent in `next_goal`/memory instead of emitting an action. It must emit
/// exactly one action — or, if the record it was looking for is genuinely not
/// found, call `done` with a not-found result — BEFORE a second empty trips the
/// stall abort. Not a salvage: nothing is read out of `next_goal`.
#[must_use]
pub fn build_empty_action_nudge(prev_empty: bool) -> Option<ChatMessage> {
    if !prev_empty {
        return None;
    }
    Some(ChatMessage::system(
        "Your previous response had NO action — that is not allowed. You MUST emit exactly one \
         action this step. If the record you were searching for is genuinely not found, call the \
         `done` action with a not-found result (its `text` set to a JSON object such as \
         {\"found\": false}). Do NOT put your decision only in next_goal, memory, or the plan.",
    ))
}

#[cfg(test)]
mod tests {
    use super::build_empty_action_nudge;
    use ras_llm::ChatMessage;

    #[test]
    fn no_nudge_when_previous_step_had_an_action() {
        assert!(build_empty_action_nudge(false).is_none());
    }

    #[test]
    fn nudge_after_empty_demands_action_or_done_not_found() {
        let text = match build_empty_action_nudge(true) {
            Some(ChatMessage::System(s)) => s.content,
            _ => String::new(),
        };
        assert!(!text.is_empty(), "an empty action must trigger a re-prompt");
        let lo = text.to_lowercase();
        assert!(lo.contains("action"), "demands an action");
        assert!(lo.contains("done"), "offers the done not-found escape");
        assert!(
            lo.contains("next_goal") || lo.contains("not found"),
            "addresses the next_goal stall"
        );
    }
}

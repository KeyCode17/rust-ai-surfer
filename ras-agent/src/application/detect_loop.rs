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

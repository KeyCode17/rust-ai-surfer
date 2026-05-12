use ras_agent::application::detect_loop::{build_budget_warning, build_loop_nudge};
use ras_agent::domain::loop_detector::ActionLoopDetector;
use ras_llm::ChatMessage;

#[test]
fn no_nudge_when_actions_diverse() {
    let mut d = ActionLoopDetector::new();
    for i in 0..5 {
        d.record_action(format!("hash-{i}"));
    }
    assert!(build_loop_nudge(&d).is_none());
}

#[test]
fn nudge_triggers_at_five_repetitions() {
    let mut d = ActionLoopDetector::new();
    for _ in 0..5 {
        d.record_action("same");
    }
    let n = build_loop_nudge(&d).expect("nudge");
    match n {
        ChatMessage::System(m) => assert!(m.content.contains("repeated")),
        _ => panic!("expected system message"),
    }
}

#[test]
fn budget_warning_at_95_percent() {
    let n = build_budget_warning(95, 100).expect("warning");
    match n {
        ChatMessage::System(m) => {
            assert!(m.content.contains("95%"));
            assert!(m.content.contains("Wrap up"));
        }
        _ => panic!("expected system"),
    }
}

#[test]
fn no_budget_warning_under_95_percent() {
    assert!(build_budget_warning(70, 100).is_none());
    assert!(build_budget_warning(90, 100).is_none());
    assert!(build_budget_warning(94, 100).is_none());
}

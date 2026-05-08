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
fn budget_warning_at_75_percent() {
    let n = build_budget_warning(8, 10).expect("warning");
    match n {
        ChatMessage::System(m) => {
            assert!(m.content.contains("80%"));
            assert!(m.content.contains(">= 75%"));
        }
        _ => panic!("expected system"),
    }
}

#[test]
fn budget_warning_escalates_at_90_percent() {
    let n = build_budget_warning(9, 10).expect("warning");
    match n {
        ChatMessage::System(m) => {
            assert!(m.content.contains("90%"));
            assert!(m.content.contains(">= 90%"));
        }
        _ => panic!("expected system"),
    }
}

#[test]
fn no_budget_warning_under_75_percent() {
    assert!(build_budget_warning(5, 10).is_none());
    assert!(build_budget_warning(7, 10).is_none());
}

use ras_tools::{ActionRegistry, register_default_actions};
use ras_types::ActionName;

#[test]
fn default_set_registers_eight_actions() {
    let mut r = ActionRegistry::new();
    register_default_actions(&mut r).expect("register");
    assert_eq!(r.len(), 8);
    for name in [
        "navigate",
        "click_element",
        "click_coordinate",
        "type_text",
        "scroll",
        "screenshot",
        "wait",
        "done",
    ] {
        assert!(r.get(&ActionName(name.into())).is_some(), "missing {name}");
    }
}

#[test]
fn duplicate_registration_errors() {
    let mut r = ActionRegistry::new();
    register_default_actions(&mut r).expect("register once");
    let err = register_default_actions(&mut r).expect_err("duplicate should err");
    assert!(format!("{err:?}").contains("already registered"));
}

#[test]
fn exclude_removes_action() {
    let mut r = ActionRegistry::new();
    register_default_actions(&mut r).expect("register");
    let n = ActionName("done".into());
    r.exclude(&n);
    assert_eq!(r.len(), 7);
    assert!(r.get(&n).is_none());
}

#[test]
fn done_action_metadata_terminates_sequence() {
    let mut r = ActionRegistry::new();
    register_default_actions(&mut r).expect("register");
    let done = r.get(&ActionName("done".into())).expect("done");
    assert!(done.metadata.terminates_sequence);
    assert!(r
        .get(&ActionName("type_text".into()))
        .is_some_and(|a| !a.metadata.terminates_sequence));
}

#[test]
fn navigate_terminates_sequence() {
    let mut r = ActionRegistry::new();
    register_default_actions(&mut r).expect("register");
    let nav = r.get(&ActionName("navigate".into())).expect("navigate");
    assert!(nav.metadata.terminates_sequence);
}

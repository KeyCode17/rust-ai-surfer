use ras_dom::application::dynamic_class_filter::filter_dynamic_classes;

#[test]
fn keeps_semantic_classes() {
    let kept = filter_dynamic_classes("button primary card");
    assert_eq!(kept, vec!["button", "card", "primary"]);
}

#[test]
fn drops_state_classes() {
    let kept = filter_dynamic_classes("button is-active hover focus selected");
    assert_eq!(kept, vec!["button"]);
}

#[test]
fn drops_hashed_classes() {
    let kept = filter_dynamic_classes("button css-12ab34 sc-abcd1234");
    assert_eq!(kept, vec!["button"]);
}

#[test]
fn drops_long_digit_runs() {
    let kept = filter_dynamic_classes("a1234 b1 c");
    assert!(!kept.contains(&"a1234".to_string()));
    assert!(kept.contains(&"b1".to_string()));
    assert!(kept.contains(&"c".to_string()));
}

#[test]
fn dedup_and_sort() {
    let kept = filter_dynamic_classes("z a b a c");
    assert_eq!(kept, vec!["a", "b", "c", "z"]);
}

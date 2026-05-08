use ras_agent::application::compute_action_hash::compute_action_hash;
use ras_agent::domain::agent_output::ActionInvocation;
use ras_types::ActionName;

fn invocation(name: &str, params: serde_json::Value) -> ActionInvocation {
    ActionInvocation {
        name: ActionName(name.into()),
        parameters: params,
    }
}

#[test]
fn different_actions_have_different_hashes() {
    let a = compute_action_hash(&invocation("click", serde_json::json!({"index": 1})));
    let b = compute_action_hash(&invocation("scroll", serde_json::json!({"index": 1})));
    assert_ne!(a, b);
}

#[test]
fn same_action_same_hash() {
    let a = compute_action_hash(&invocation("click", serde_json::json!({"index": 1})));
    let b = compute_action_hash(&invocation("click", serde_json::json!({"index": 1})));
    assert_eq!(a, b);
}

#[test]
fn search_strips_pagination_fields() {
    let a = compute_action_hash(&invocation(
        "search_page",
        serde_json::json!({"query": "q", "max_results": 10}),
    ));
    let b = compute_action_hash(&invocation(
        "search_page",
        serde_json::json!({"query": "q", "max_results": 50}),
    ));
    assert_eq!(a, b);
}

#[test]
fn navigate_strips_query_string() {
    let a = compute_action_hash(&invocation(
        "navigate",
        serde_json::json!({"url": "https://example.com/x?a=1"}),
    ));
    let b = compute_action_hash(&invocation(
        "navigate",
        serde_json::json!({"url": "https://example.com/x?a=2"}),
    ));
    assert_eq!(a, b);
}

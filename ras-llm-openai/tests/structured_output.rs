//! Structured-output plumbing: an InvokeOptions.response_schema must be wrapped
//! as an OpenAI/OpenRouter `response_format` json_schema envelope, so the model
//! is constrained to emit the agent `action` field (a weak model otherwise
//! buries the action in `next_goal`).

use ras_llm_openai::infrastructure::http::chat_openai_compatible::response_format;
use serde_json::json;

#[test]
fn none_schema_yields_no_response_format() {
    assert!(response_format(None).is_none());
}

#[test]
fn wraps_schema_in_a_json_schema_envelope() {
    let schema = json!({ "type": "object", "required": ["action"] });
    let rf = response_format(Some(&schema)).expect("response_format");
    assert_eq!(rf["type"], "json_schema");
    assert_eq!(rf["json_schema"]["name"], "agent_output");
    assert_eq!(rf["json_schema"]["schema"], schema);
}

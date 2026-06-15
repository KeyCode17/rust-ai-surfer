//! Integration tests for the OpenRouter/OpenAI DTO prompt-cache wiring:
//! - a `cache: true` system message serialises to the array form carrying an
//!   ephemeral `cache_control` breakpoint (so OpenRouter caches the prefix);
//! - a `cache: false` system message stays a plain string (no behaviour change);
//! - cache token counts in the response `usage` (OpenAI `prompt_tokens_details`
//!   OR Anthropic-style top-level fields) are parsed back into `Usage`.

use ras_llm::ChatMessage;
use ras_llm_openai::infrastructure::http::dto::{
    ChatCompletionResponse, response_to_chat, to_dto_messages,
};

#[test]
fn cached_system_message_emits_cache_control_breakpoint() {
    let dto = to_dto_messages(vec![ChatMessage::system_cached("BIG STATIC PREFIX")]);
    let value = serde_json::to_value(&dto[0]).expect("serialise dto");

    assert_eq!(value["role"], "system");
    let parts = value["content"].as_array().expect("content is an array");
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0]["type"], "text");
    assert_eq!(parts[0]["text"], "BIG STATIC PREFIX");
    assert_eq!(
        parts[0]["cache_control"]["type"], "ephemeral",
        "a cached system message must carry an ephemeral cache_control breakpoint"
    );
}

#[test]
fn uncached_system_message_stays_a_plain_string() {
    let dto = to_dto_messages(vec![ChatMessage::system("plain")]);
    let value = serde_json::to_value(&dto[0]).expect("serialise dto");

    assert_eq!(value["role"], "system");
    assert_eq!(
        value["content"], "plain",
        "without caching the system content stays a plain string (unchanged wire format)"
    );
}

#[test]
fn parses_openai_style_cached_tokens() {
    let body = r#"{
        "model": "anthropic/claude-sonnet-4.5",
        "choices": [{"message": {"content": "ok"}, "finish_reason": "stop"}],
        "usage": {"prompt_tokens": 1000, "completion_tokens": 10,
                  "prompt_tokens_details": {"cached_tokens": 800}}
    }"#;
    let parsed: ChatCompletionResponse = serde_json::from_str(body).expect("parse");
    let resp = response_to_chat(parsed);

    assert_eq!(resp.usage.input_tokens, 1000);
    assert_eq!(resp.usage.output_tokens, 10);
    assert_eq!(
        resp.usage.cache_read_input_tokens, 800,
        "prompt_tokens_details.cached_tokens must map to cache_read_input_tokens"
    );
}

#[test]
fn parses_anthropic_style_top_level_cache_fields() {
    let body = r#"{
        "model": "anthropic/claude-sonnet-4.5",
        "choices": [{"message": {"content": "ok"}, "finish_reason": "stop"}],
        "usage": {"prompt_tokens": 1000, "completion_tokens": 10,
                  "cache_read_input_tokens": 700, "cache_creation_input_tokens": 200}
    }"#;
    let parsed: ChatCompletionResponse = serde_json::from_str(body).expect("parse");
    let resp = response_to_chat(parsed);

    assert_eq!(resp.usage.cache_read_input_tokens, 700);
    assert_eq!(resp.usage.cache_creation_input_tokens, 200);
}

#[test]
fn no_cache_fields_yields_zero() {
    let body = r#"{
        "model": "x",
        "choices": [{"message": {"content": "ok"}, "finish_reason": "stop"}],
        "usage": {"prompt_tokens": 50, "completion_tokens": 5}
    }"#;
    let parsed: ChatCompletionResponse = serde_json::from_str(body).expect("parse");
    let resp = response_to_chat(parsed);

    assert_eq!(resp.usage.cache_read_input_tokens, 0);
    assert_eq!(resp.usage.cache_creation_input_tokens, 0);
}

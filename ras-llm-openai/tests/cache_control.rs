//! Integration tests for the OpenRouter/OpenAI DTO prompt-cache wiring:
//! - a `cache: true` system message serialises to the array form carrying an
//!   ephemeral `cache_control` breakpoint ONLY when the target accepts it
//!   (Anthropic-routed); otherwise it stays a plain string so a non-Anthropic
//!   provider never 400s on the unknown field;
//! - a `cache: false` system message stays a plain string (no behaviour change);
//! - cache token counts in the response `usage` (OpenAI `prompt_tokens_details`
//!   OR Anthropic-style top-level fields) are parsed back into `Usage`.

use ras_llm::ChatMessage;
use ras_llm_openai::infrastructure::http::chat_openai_compatible::model_supports_cache_control;
use ras_llm_openai::infrastructure::http::dto::{
    ChatCompletionResponse, response_to_chat, to_dto_messages,
};

fn has_cache_control(msg: ChatMessage, allow: bool) -> bool {
    let dto = to_dto_messages(vec![msg], allow);
    let value = serde_json::to_value(&dto[0]).expect("serialise dto");
    value["content"]
        .as_array()
        .and_then(|parts| parts.first())
        .map(|p| p["cache_control"]["type"] == "ephemeral")
        .unwrap_or(false)
}

#[test]
fn cached_system_message_emits_cache_control_when_supported() {
    let dto = to_dto_messages(vec![ChatMessage::system_cached("BIG STATIC PREFIX")], true);
    let value = serde_json::to_value(&dto[0]).expect("serialise dto");

    assert_eq!(value["role"], "system");
    let parts = value["content"].as_array().expect("content is an array");
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0]["type"], "text");
    assert_eq!(parts[0]["text"], "BIG STATIC PREFIX");
    assert_eq!(
        parts[0]["cache_control"]["type"], "ephemeral",
        "a cached system message on a supported target must carry an ephemeral breakpoint"
    );
}

#[test]
fn cached_system_message_is_plain_string_when_unsupported() {
    // The BLOCKER regression: cache:true on a NON-Anthropic target must NOT emit
    // the cache_control breakpoint (OpenAI/Groq/etc. reject the unknown field).
    let dto = to_dto_messages(vec![ChatMessage::system_cached("BIG STATIC PREFIX")], false);
    let value = serde_json::to_value(&dto[0]).expect("serialise dto");

    assert_eq!(value["role"], "system");
    assert_eq!(
        value["content"], "BIG STATIC PREFIX",
        "an unsupported target must receive plain-string content, no cache_control"
    );
}

#[test]
fn uncached_system_message_stays_a_plain_string() {
    let dto = to_dto_messages(vec![ChatMessage::system("plain")], true);
    let value = serde_json::to_value(&dto[0]).expect("serialise dto");

    assert_eq!(value["role"], "system");
    assert_eq!(
        value["content"], "plain",
        "without caching the system content stays a plain string (unchanged wire format)"
    );
}

#[test]
fn cache_control_gate_keys_on_the_model() {
    // Anthropic-routed models accept the breakpoint; everything else does not.
    for anthropic in [
        "anthropic/claude-sonnet-4.5",
        "anthropic/claude-3.5-haiku",
        "claude-3-5-sonnet",
    ] {
        assert!(
            model_supports_cache_control(anthropic),
            "{anthropic} should support cache_control"
        );
    }
    for other in ["openai/gpt-4o", "gpt-4", "groq/llama-3.1", "deepseek-chat"] {
        assert!(
            !model_supports_cache_control(other),
            "{other} must NOT get a cache_control breakpoint"
        );
    }

    // End-to-end through the gate: a cache:true system message keyed on a real
    // model id only carries the breakpoint for the Anthropic route.
    assert!(has_cache_control(
        ChatMessage::system_cached("x"),
        model_supports_cache_control("anthropic/claude-sonnet-4.5")
    ));
    assert!(!has_cache_control(
        ChatMessage::system_cached("x"),
        model_supports_cache_control("openai/gpt-4o")
    ));
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

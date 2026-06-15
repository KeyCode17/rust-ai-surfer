//! Live (network + paid key) smoke test proving end-to-end prompt caching
//! through OpenRouter: two identical calls carrying a large cached system
//! prefix; the second must report `cache_read_input_tokens > 0`.
//!
//! Ignored by default (the offline `cache_control` tests cover the wire format
//! and parsing deterministically). Run it where the key lives, e.g. staging:
//!
//!   OPENROUTER_API_KEY=sk-... \
//!     cargo test -p ras-llm-openai --test live_cache -- --ignored --nocapture
//!
//! Optionally override the model with OPENROUTER_MODEL (default
//! anthropic/claude-sonnet-4.5).

use ras_llm::{ChatMessage, InvokeOptions, LlmClient};
use ras_llm_openai::{ChatOpenAICompatible, OpenAiAuth};

#[tokio::test]
#[ignore = "live network + paid OpenRouter key; run manually with --ignored"]
async fn second_identical_call_reads_the_cached_prefix() {
    let key = std::env::var("OPENROUTER_API_KEY")
        .expect("set OPENROUTER_API_KEY to run the live cache probe");
    let model = std::env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| "anthropic/claude-sonnet-4.5".to_string());

    let client = ChatOpenAICompatible::new(
        "openrouter",
        model,
        "https://openrouter.ai/api",
        OpenAiAuth::Bearer(key),
    )
    .expect("build client");

    // A large, STABLE prefix so it clears Anthropic's per-model cache minimum
    // (~1024 tokens). ~400 repetitions is comfortably over the threshold.
    let big_prefix = "You are a deterministic test fixture. Ignore this line. ".repeat(400);
    let messages = || {
        vec![
            ChatMessage::system_cached(big_prefix.clone()),
            ChatMessage::user_text("Reply with the single word: ok"),
        ]
    };
    let opts = InvokeOptions {
        max_tokens: Some(8),
        ..InvokeOptions::default()
    };

    let first = client
        .ainvoke(messages(), opts.clone())
        .await
        .expect("first call");
    let second = client.ainvoke(messages(), opts).await.expect("second call");

    println!(
        "cache tokens -> first: read={} creation={} | second: read={} creation={}",
        first.usage.cache_read_input_tokens,
        first.usage.cache_creation_input_tokens,
        second.usage.cache_read_input_tokens,
        second.usage.cache_creation_input_tokens,
    );
    assert!(
        second.usage.cache_read_input_tokens > 0,
        "the second identical call must read the cached prefix (cache_read_input_tokens > 0)"
    );
}

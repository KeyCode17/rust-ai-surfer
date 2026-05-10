use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use ras_agent::application::run_agent::RunAgent;
use ras_cdp::BrowserPort;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use ras_events::{BroadcastBus, EventBus};
use ras_llm::LlmClient;
use ras_llm_anthropic::infrastructure::http::chat_anthropic_claude_code::ChatAnthropicClaudeCode;
use ras_tools::domain::registry::ActionRegistry;
use ras_tools::infrastructure::builtin::register::register_default_actions;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let model = std::env::var("RAS_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".into());
    let cdp_url_str = std::env::var("CDP_URL").unwrap_or_else(|_| "http://127.0.0.1:9222".into());
    let task = std::env::var("TASK")
        .unwrap_or_else(|_| "Open https://example.com and report the page heading.".into());

    let llm: Arc<dyn LlmClient> = Arc::new(
        ChatAnthropicClaudeCode::new(model.clone())
            .await
            .context("Claude Code OAuth resolution failed - run `claude` to log in")?,
    );
    println!("[ok] Claude Code OAuth resolved (model={model})");

    let cdp_http = Url::parse(&cdp_url_str).context("invalid CDP_URL")?;
    let ws_url = ras_cosmium::infrastructure::attach::resolve_attach_url(&cdp_http)
        .await
        .context("failed to resolve CDP websocket URL - is cosmium running?")?;
    let browser: Arc<dyn BrowserPort> = Arc::new(
        ChromiumoxideAdapter::connect(ws_url.clone(), Duration::from_secs(60))
            .await
            .context("failed to attach to CDP")?,
    );
    println!("[ok] BrowserSession attached to {ws_url}");

    let mut registry = ActionRegistry::new();
    register_default_actions(&mut registry).context("failed to register default actions")?;
    let registry = Arc::new(registry);
    let events: Arc<dyn EventBus> = Arc::new(BroadcastBus::new(64));

    let history = RunAgent::new(task, llm, registry, browser, events)
        .with_max_steps(10)
        .execute()
        .await?;
    let final_text = history.final_result().unwrap_or("(no final result returned)");
    println!("[done] {final_text}");
    Ok(())
}

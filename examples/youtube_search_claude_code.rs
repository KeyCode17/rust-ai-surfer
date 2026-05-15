use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use ras_agent::application::run_agent::RunAgent;
use ras_cdp::BrowserPort;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use ras_dom::{ChromiumoxideDomExtractor, DomExtractor};
use ras_events::{BroadcastBus, EventBus};
use ras_llm::LlmClient;
use ras_llm_anthropic::infrastructure::http::chat_anthropic_claude_code::ChatAnthropicClaudeCode;
use ras_tools::domain::registry::ActionRegistry;
use ras_tools::infrastructure::builtin::register::register_default_actions;
use url::Url;

const TASK: &str = r#"
You are a browsing agent. Your task:
1. Navigate to https://www.youtube.com/?gl=US&hl=en
2. If a cookie consent banner appears, accept or dismiss it.
3. Click the search input field. Type the exact text: how to backflip
4. Click the search submit button (NOT the URL bar - use the button).
5. Wait for the results page to load.
6. Call done with text="ok" if the final URL is the YouTube results
   page with search_query containing 'backflip'. Otherwise call done
   with text="failed: <reason>".
"#;

#[tokio::main]
async fn main() -> Result<()> {
    let mut p = std::env::current_dir()?;
    loop {
        let candidate = p.join(".env");
        if candidate.exists() {
            let _ = dotenvy::from_path(&candidate);
            break;
        }
        if !p.pop() {
            break;
        }
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let model = std::env::var("RAS_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".into());
    let cdp_url: Url = std::env::var("CDP_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:9222".into())
        .parse()?;

    let llm: Arc<dyn LlmClient> = Arc::new(
        ChatAnthropicClaudeCode::new(model.clone())
            .await
            .context("Claude Code OAuth resolution failed - run `claude` to log in")?,
    );
    eprintln!("[smoke] Claude Code OAuth resolved (model={model})");

    let ws = ras_cosmium::infrastructure::attach::resolve_attach_url(&cdp_url).await?;
    let adapter = ChromiumoxideAdapter::connect(ws, Duration::from_secs(60)).await?;
    let browser_arc = adapter.browser_arc();
    let browser: Arc<dyn BrowserPort> = Arc::new(adapter);
    let extractor: Arc<dyn DomExtractor> = Arc::new(ChromiumoxideDomExtractor::new(
        browser_arc,
        Duration::from_secs(30),
    ));

    let mut registry = ActionRegistry::new();
    register_default_actions(&mut registry)?;
    let registry = Arc::new(registry);
    let events: Arc<dyn EventBus> = Arc::new(BroadcastBus::default());

    let history = RunAgent::new(TASK.to_string(), llm, registry, browser.clone(), events)
        .with_max_steps(25)
        .with_dom_extractor(extractor)
        .execute()
        .await?;

    let final_text = history.final_result().unwrap_or_default().to_string();
    let target = browser.focused_target().await?;
    let url_value = browser.evaluate(&target, "location.href").await?;
    let final_url = url_value.as_str().unwrap_or_default().to_string();

    eprintln!("\n[smoke] agent done text: {final_text}");
    eprintln!("[smoke] final url       : {final_url}");

    let url_ok = final_url.contains("/results")
        && final_url.contains("search_query=")
        && (final_url.contains("backflip") || final_url.contains("how"));

    if !url_ok {
        bail!(
            "youtube search failed: url did not reach /results?search_query=*backflip*\n  url={final_url}\n  done={final_text}"
        );
    }
    eprintln!("[smoke] PASS");
    Ok(())
}

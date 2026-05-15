use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use ras_agent::application::run_agent::RunAgent;
use ras_cdp::BrowserPort;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use ras_dom::{ChromiumoxideDomExtractor, DomExtractor};
use ras_events::{BroadcastBus, EventBus};
use ras_llm::LlmClient;
use ras_llm_openrouter::{ChatOpenAICompatible, OpenAiAuth};
use ras_tools::domain::registry::ActionRegistry;
use ras_tools::infrastructure::builtin::register::register_default_actions;
use url::Url;

const TASK: &str = r#"
You are a browsing agent on pedidosya.com.ar (Argentine food delivery).
The site is protected by PerimeterX / HUMAN Security bot detection.

Two distinct PX outcomes — you MUST distinguish them by the PRESENCE OR
ABSENCE OF A PRESS-AND-HOLD BUTTON, NOT by surrounding text:
  (A) Solvable challenge — page shows a button labelled "Pulsar y
      mantener pulsado" / "Press & Hold" / "Mantén presionado". Even
      when the page ALSO says "Acceso ha sido denegado" or "Por favor
      confirma que eres un humano", the presence of the button means
      it IS solvable. Use the press_and_hold tools below.
  (B) Hard deny — no interactive button anywhere on the page; just
      static text and maybe a reference id. THIS is unsolvable.
      Only conclude (B) after a screenshot confirms NO press-and-hold
      button is present.

Tools to know about:
  - press_and_hold_element { index, ms? } — resolves the bbox center
    of a clickable in the DOM snapshot and dispatches a humanized CDP
    press (approach + press + jitter + release). ms default 12000, max
    60000. Use this when the hold button IS in the clickable list.
  - press_and_hold_coordinate { x, y, ms? } — same gesture at raw pixels.
    Use this when the hold button is in a cross-origin iframe and the
    clickable list does NOT show it; first take a screenshot, identify
    the button center in pixels, then call this tool.
  - PerimeterX usually needs 15–25 seconds of sustained press. Try
    ms=20000 first. If still blocked, ms=30000, then ms=45000.

Your task:
1. Navigate to https://www.pedidosya.com.ar/
2. CRITICAL: After navigate, your FIRST action MUST be wait with
   seconds=5. The PerimeterX widget loads its JavaScript over ~3 seconds
   and renders the button slightly later. A blank/grey area where the
   button should be is NOT proof of hard deny — it means the widget
   hasn't painted yet. NEVER call done="px_hard_deny" without first
   doing wait(5) and then re-inspecting.
3. After the wait, take a screenshot and inspect the page:
   - If the screenshot shows the normal PedidosYa homepage (city
     selector, cuisines, search box) → call done with text="ok: cleared".
   - If you see ANY button-shaped element labelled "Pulsar y mantener
     pulsado" / "Press & Hold" / "Mantén presionado" anywhere on the
     page (even when the page ALSO says "Acceso ha sido denegado" or
     "Por favor confirma que eres un humano") → proceed to step 4.
     The button is usually a rounded pill in the center of the page.
   - If the screenshot STILL shows just text and a grey/empty area,
     do ANOTHER wait(5) before deciding. Only conclude px_hard_deny
     after at least 10 seconds total of waiting AND a clear screenshot
     with no button-shaped element anywhere.
4. Solve the Press & Hold challenge:
   a. First try press_and_hold_element: scan the clickable list for
      labels containing "Press", "Hold", "Pulsar", "Mantener",
      "Mantén". If found, call press_and_hold_element with that
      index and ms=20000.
   b. If the button is NOT in the clickable list (often happens
      because PX renders inside an iframe), fall back: identify the
      button center coordinates from the screenshot (it sits near the
      visual center of the page), call press_and_hold_coordinate with
      those (x, y) and ms=20000.
   c. wait 4 seconds, then take another screenshot.
   d. If still blocked, retry the same hold tool with ms=30000.
   e. If still blocked, retry once more with ms=45000.
   f. If still blocked after three attempts, call done with
      text="px_blocked: hold rejected after 3 attempts".
5. Hard ceiling: do NOT exceed 18 steps.
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

    let api_key = std::env::var("OPENROUTER_API_KEY")
        .context("OPENROUTER_API_KEY not set in env or .env")?;
    let model =
        std::env::var("RAS_MODEL").unwrap_or_else(|_| "google/gemini-2.5-flash".into());
    let cdp_url: Url = std::env::var("CDP_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:9222".into())
        .parse()?;

    let mut inner = ChatOpenAICompatible::new(
        "openrouter",
        model.clone(),
        "https://openrouter.ai/api",
        OpenAiAuth::Bearer(api_key),
    )?;
    inner.extra_headers = vec![
        (
            "HTTP-Referer".into(),
            "https://github.com/KeyCode17/rust-ai-surfer".into(),
        ),
        ("X-Title".into(), "pedidosya_perimeterx_smoke".into()),
    ];
    let llm: Arc<dyn LlmClient> = Arc::new(inner);

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
        .with_max_steps(20)
        .with_dom_extractor(extractor)
        .execute()
        .await?;

    let final_text = history.final_result().unwrap_or_default().to_string();
    let target = browser.focused_target().await?;
    let url_value = browser.evaluate(&target, "location.href").await?;
    let final_url = url_value.as_str().unwrap_or_default().to_string();
    let title_value = browser.evaluate(&target, "document.title").await?;
    let final_title = title_value.as_str().unwrap_or_default().to_string();
    let px_probe = browser
        .evaluate(
            &target,
            r#"(function(){
                var html = document.documentElement ? document.documentElement.innerHTML : '';
                var hit = /px-captcha|_pxCaptcha|Press\s*&\s*Hold|Mant[eé]n\s*presionado|Access\s*to\s*this\s*page\s*has\s*been\s*denied/i.test(html);
                return hit ? 'px_present' : 'px_clear';
            })()"#,
        )
        .await?;
    let px_state = px_probe.as_str().unwrap_or("unknown").to_string();

    eprintln!("\n[smoke] agent done text : {final_text}");
    eprintln!("[smoke] final url       : {final_url}");
    eprintln!("[smoke] final title     : {final_title}");
    eprintln!("[smoke] px probe        : {px_state}");

    let host_ok = final_url.contains("pedidosya.com.ar");
    let cleared = px_state == "px_clear" && !final_title.is_empty();

    if !host_ok {
        bail!(
            "pedidosya run failed: never reached pedidosya.com.ar\n  url={final_url}\n  done={final_text}"
        );
    }
    if !cleared {
        eprintln!(
            "[smoke] WARN: PerimeterX challenge still present after agent run.\n\
             This is expected: the built-in click action cannot perform a sustained\n\
             press-and-hold mouse gesture, which is what most PX challenges require."
        );
        bail!(
            "px_blocked: challenge not cleared\n  url={final_url}\n  title={final_title}\n  done={final_text}"
        );
    }
    eprintln!("[smoke] PASS");
    Ok(())
}

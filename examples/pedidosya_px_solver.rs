// Pure-CDP PerimeterX "Pulsar y mantener pulsado" solver. No LLM involved.
//
// 1. Navigate to pedidosya.com.ar
// 2. Wait for PX widget to paint, then locate the press-and-hold button by
//    searching the DOM for its label text in every same-origin frame.
// 3. mouse_hold its bbox center for N ms (default 18000), with humanized
//    approach + jittered MouseMoved events during the hold.
// 4. Wait + verify whether the deny page has cleared.
//
// Run:
//   cargo run --example pedidosya_px_solver
//
// Optional env:
//   PX_HOLD_MS=20000     hold duration (clamped 100..60000)
//   PX_RETRIES=3         number of hold attempts before giving up
//   CDP_URL=...          CDP HTTP endpoint (default 127.0.0.1:9222)

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use ras_cdp::BrowserPort;
use ras_cdp::domain::repository::ScreenshotFormat;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use url::Url;

const TARGET_URL: &str = "https://www.pedidosya.com.ar/";

// The PerimeterX press-and-hold widget lives inside a CLOSED Shadow DOM
// hosted by `<div id="px-captcha">`, with the actual button rendered in an
// iframe inside that shadow root. Closed shadow roots are NOT reachable
// from `querySelectorAll`, `elementFromPoint`, or iframe descent — but the
// shadow host element (`#px-captcha`) IS in the light DOM, and its
// bounding rect tells us exactly where the iframe paints in viewport
// pixels. That's the hold target.
//
// We still also scan for label text + iframe metadata as fallback diagnostics.
const FIND_BUTTON_JS: &str = r#"(function () {
    try {
        var w = window.innerWidth, h = window.innerHeight;
        var host = document.querySelector('#px-captcha')
                || document.querySelector('.px-captcha-container');
        if (!host) {
            return { ok: false, why: 'no #px-captcha host yet', w: w, h: h };
        }
        var r = host.getBoundingClientRect();
        if (!r || r.width < 50) {
            return {
                ok: false,
                why: 'host present but not painted',
                w: w, h: h,
                host_w: r ? r.width : -1,
                host_h: r ? r.height : -1
            };
        }
        return {
            ok: true,
            w: w, h: h,
            x: r.left,
            y: r.top,
            host_w: r.width,
            host_h: r.height,
            host_id: host.id || '',
            host_cls: (host.className || '').toString().slice(0, 80)
        };
    } catch (e) {
        return { ok: false, why: String(e && e.message || e) };
    }
})()"#;

// Lightweight probe: does the page currently still look like the PX deny /
// challenge screen? True = still blocked. Used as a "did we clear it" check.
const PX_PROBE_JS: &str = r#"(function(){
    var html = document.documentElement ? document.documentElement.innerHTML : '';
    var titleHit = /Acceso\s+ha\s+sido\s+denegado|Access\s+to\s+this\s+page\s+has\s+been\s+denied/i.test(document.title || '');
    var bodyHit  = /Pulsar\s+y\s+mantener|Press\s*&\s*Hold|Mant[eé]n\s+presionado|px-captcha|_pxCaptcha/i.test(html);
    return titleHit || bodyHit;
})()"#;

#[tokio::main]
async fn main() -> Result<()> {
    // load .env
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

    let hold_ms: u64 = std::env::var("PX_HOLD_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(18_000);
    let retries: u32 = std::env::var("PX_RETRIES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    let cdp_url: Url = std::env::var("CDP_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:9222".into())
        .parse()?;
    let ws = ras_cosmium::infrastructure::attach::resolve_attach_url(&cdp_url).await?;
    let adapter = ChromiumoxideAdapter::connect(ws, Duration::from_secs(60)).await?;
    let browser: Arc<dyn BrowserPort> = Arc::new(adapter);

    let target = match browser.focused_target().await {
        Ok(t) => t,
        Err(_) => {
            eprintln!("[px] no existing tab — creating one");
            browser
                .create_target(&Url::parse("about:blank")?)
                .await
                .context("create new tab")?
        }
    };

    // Clear any prior PerimeterX clearance cookie (`_px3`/`_pxhd`) so every
    // run forces a fresh challenge. PX caches successful holds for ~30 min
    // and would otherwise let us straight through, hiding regressions.
    if let Err(e) = browser
        .clear_cookies(&target, "https://www.pedidosya.com.ar")
        .await
    {
        eprintln!("[px] clear_cookies (pre-navigate) failed: {e} (continuing)");
    } else {
        eprintln!("[px] cleared pedidosya cookies + storage");
    }

    browser
        .navigate(&target, &Url::parse(TARGET_URL)?)
        .await
        .context("navigate pedidosya")?;
    eprintln!("[px] navigated to {TARGET_URL}");

    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        if attempt > retries {
            eprintln!("[px] all {retries} attempts exhausted");
            break;
        }
        eprintln!("[px] attempt {attempt}/{retries}: waiting for button to render...");

        // Wait for PX widget to paint, then look up the `#px-captcha` host
        // element. The press-and-hold iframe lives inside its closed shadow
        // root; a coordinate-based mouse_down on the host's bbox hit-tests
        // straight into the iframe via blink, so we don't need to pierce
        // the shadow boundary to trigger the hold.
        tokio::time::sleep(Duration::from_secs(4)).await;
        let probe = browser.evaluate(&target, FIND_BUTTON_JS).await?;
        let vw = probe.get("w").and_then(|n| n.as_f64()).unwrap_or(0.0);
        let vh = probe.get("h").and_then(|n| n.as_f64()).unwrap_or(0.0);
        let ok = probe.get("ok").and_then(|b| b.as_bool()).unwrap_or(false);
        let why = probe.get("why").and_then(|s| s.as_str()).unwrap_or("");
        if !ok {
            eprintln!("[px] probe: ok=false  why={why}  viewport={vw:.0}x{vh:.0}");
        }
        let mut button: Option<(f64, f64)> = None;
        if ok {
            let x = probe.get("x").and_then(|n| n.as_f64()).unwrap_or(0.0);
            let y = probe.get("y").and_then(|n| n.as_f64()).unwrap_or(0.0);
            let hw = probe.get("host_w").and_then(|n| n.as_f64()).unwrap_or(0.0);
            let hh = probe.get("host_h").and_then(|n| n.as_f64()).unwrap_or(0.0);
            let hid = probe
                .get("host_id")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            let hcls = probe
                .get("host_cls")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            // PX's visible iframe is fixed-height 52px at the TOP of the
            // host; the rest of the host's height is the error-alert <p>
            // BELOW it. Targeting host_h/2 lands at or below the button.
            // Bias toward the iframe center: 26px down from host top.
            let iframe_h = 52.0_f64.min(hh);
            let cx = x + hw / 2.0;
            let cy = y + iframe_h / 2.0;
            eprintln!(
                "[px] PX HOST: id=\"{hid}\" cls=\"{hcls}\" bbox={hw:.0}x{hh:.0}  iframe_top_h={iframe_h:.0}  hold target=({cx:.0},{cy:.0})  viewport={vw:.0}x{vh:.0}"
            );
            button = Some((cx, cy));
        }
        if button.is_none() && vw > 0.0 && vh > 0.0 {
            let fx = vw / 2.0;
            let fy = vh / 2.0 + 70.0;
            eprintln!(
                "[px] no #px-captcha host yet — falling back to (vp_center + 70): ({fx:.0},{fy:.0})"
            );
            button = Some((fx, fy));
        }

        // probe state
        let still_blocked = browser
            .evaluate(&target, PX_PROBE_JS)
            .await?
            .as_bool()
            .unwrap_or(true);

        if !still_blocked {
            eprintln!("[px] page no longer shows PX challenge — CLEARED");
            break;
        }

        let (cx, cy) = match button {
            Some(b) => b,
            None => {
                eprintln!("[px] no target coords; still_blocked={still_blocked}");
                if attempt >= retries {
                    anyhow::bail!("could not derive hold target");
                }
                continue;
            }
        };

        // small pre-move to look human before hitting it
        let _ = browser
            .mouse_move(&target, cx as i32 - 80, cy as i32 - 60, 0)
            .await;
        tokio::time::sleep(Duration::from_millis(150)).await;

        eprintln!(
            "[px] HOLDING ({:.0},{:.0}) for {} ms (attempt {}/{})",
            cx, cy, hold_ms, attempt, retries
        );
        browser
            .mouse_hold(&target, cx as i32, cy as i32, hold_ms)
            .await
            .context("mouse_hold")?;
        eprintln!("[px] release fired; settling 5s for redirect...");
        tokio::time::sleep(Duration::from_secs(5)).await;

        // After a successful PX clear the page navigates away, which
        // destroys our JS execution context — evaluate then returns
        // "Cannot find context with specified id". Treat that as
        // "probably cleared" and re-probe a moment later.
        let after_blocked = match browser.evaluate(&target, PX_PROBE_JS).await {
            Ok(v) => v.as_bool().unwrap_or(true),
            Err(e) => {
                eprintln!("[px] post-hold evaluate failed ({e}); page likely navigated. Re-checking in 3s...");
                tokio::time::sleep(Duration::from_secs(3)).await;
                browser
                    .evaluate(&target, PX_PROBE_JS)
                    .await
                    .map(|v| v.as_bool().unwrap_or(true))
                    .unwrap_or(false)
            }
        };
        if !after_blocked {
            eprintln!("[px] CLEARED on attempt {attempt}");
            break;
        }
        eprintln!("[px] still blocked after attempt {attempt}");
        if attempt >= retries {
            break;
        }
    }

    // final state + screenshot (tolerant of stale context after navigate)
    let title = browser
        .evaluate(&target, "document.title")
        .await
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    let url = browser
        .evaluate(&target, "location.href")
        .await
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    let still = browser
        .evaluate(&target, PX_PROBE_JS)
        .await
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(false);
    let png = browser.screenshot(&target, ScreenshotFormat::Png).await?;
    let out = "/tmp/pedidosya_px_solver.png";
    std::fs::write(out, &png).context("write screenshot")?;

    eprintln!("\n[px] final state:");
    eprintln!("[px]   title       : {title}");
    eprintln!("[px]   url         : {url}");
    eprintln!("[px]   still_blocked: {still}");
    eprintln!("[px]   screenshot  : {out}");

    if still {
        anyhow::bail!(
            "PX challenge NOT cleared after {} attempt(s) of {}ms hold each",
            attempt,
            hold_ms
        );
    }
    eprintln!("[px] PASS");
    Ok(())
}

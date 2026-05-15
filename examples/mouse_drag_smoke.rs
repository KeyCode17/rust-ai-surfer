// Drives CDP directly (no LLM) to prove that mouse_down + mouse_move(buttons=1)
// + mouse_up actually produce a drag the page can observe.
//
// Target: https://www.autodraw.com/  — Google's ML drawing tool. After
// clicking "Start Drawing", the viewport becomes one big `<canvas>` that
// records pointer trails. If our CDP synthetic drag works, the canvas pixel
// content will go from blank to drawn-on; a screenshot is written to
// /tmp/mouse_drag_smoke.png.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use ras_cdp::BrowserPort;
use ras_cdp::domain::repository::ScreenshotFormat;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use url::Url;

const TARGET_URL: &str = "https://www.autodraw.com/";

// JS: click the "Start Drawing" / splash button if it's still visible.
// AutoDraw shows a landing splash with a big "Start Drawing" call-to-action;
// after click, the canvas takes the viewport. Idempotent — does nothing if
// already past the splash.
const ENTER_DRAW_MODE_JS: &str = r#"(function () {
    var btn = Array.from(document.querySelectorAll('button, a, div, span'))
        .find(function (n) {
            var t = (n.textContent || '').trim();
            return /^start\s+drawing\s*\.?$/i.test(t) && t.length < 40;
        });
    if (btn) {
        btn.click();
        return 'clicked';
    }
    return 'no_splash';
})()"#;

// JS: return the largest visible <canvas> bbox. AutoDraw uses a single fullscreen
// canvas once you enter draw mode.
const FIND_CANVAS_JS: &str = r#"(function () {
    var best = null;
    var canvases = document.querySelectorAll('canvas');
    for (var i = 0; i < canvases.length; i++) {
        var c = canvases[i];
        var r = c.getBoundingClientRect();
        if (r.width < 100 || r.height < 100) continue;
        if (!best || r.width * r.height > best.w * best.h) {
            best = { x: r.left, y: r.top, w: r.width, h: r.height };
        }
    }
    return best;
})()"#;

// JS: count how many non-background pixels exist in the largest canvas. The
// AutoDraw canvas paints strokes in dark gray (~rgb(60,60,60)) on white. We
// sample every 4th pixel and tally any that aren't near-white. Returns
// { sampled, marks, total_pixels }.
const COUNT_INK_JS: &str = r#"(function () {
    var best = null;
    var canvases = document.querySelectorAll('canvas');
    for (var i = 0; i < canvases.length; i++) {
        var c = canvases[i];
        var r = c.getBoundingClientRect();
        if (r.width < 100 || r.height < 100) continue;
        if (!best || r.width * r.height > best.w * best.h) best = c;
    }
    if (!best) return { error: 'no canvas' };
    try {
        var ctx = best.getContext('2d');
        if (!ctx) return { error: 'no 2d context' };
        var w = best.width, h = best.height;
        // sample only a 400x400 window in the center for speed
        var sw = Math.min(400, w), sh = Math.min(400, h);
        var sx = Math.max(0, Math.floor((w - sw) / 2));
        var sy = Math.max(0, Math.floor((h - sh) / 2));
        var img = ctx.getImageData(sx, sy, sw, sh).data;
        var sampled = 0, marks = 0;
        for (var i = 0; i < img.length; i += 16) { // every 4th pixel
            var r2 = img[i], g2 = img[i+1], b2 = img[i+2], a2 = img[i+3];
            if (a2 < 8) { sampled++; continue; }
            sampled++;
            // anything noticeably darker than near-white counts as ink
            if (r2 < 230 || g2 < 230 || b2 < 230) marks++;
        }
        return { sampled: sampled, marks: marks, canvas_w: w, canvas_h: h };
    } catch (e) {
        return { error: String(e && e.message || e) };
    }
})()"#;

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

    let cdp_url: Url = std::env::var("CDP_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:9222".into())
        .parse()?;
    let ws = ras_cosmium::infrastructure::attach::resolve_attach_url(&cdp_url).await?;
    let adapter = ChromiumoxideAdapter::connect(ws, Duration::from_secs(60)).await?;
    let browser: Arc<dyn BrowserPort> = Arc::new(adapter);

    let target = match browser.focused_target().await {
        Ok(t) => t,
        Err(_) => {
            eprintln!("[smoke] no existing tab — creating one");
            browser
                .create_target(&Url::parse("about:blank")?)
                .await
                .context("create new tab")?
        }
    };

    browser
        .navigate(&target, &Url::parse(TARGET_URL)?)
        .await
        .context("navigate autodraw")?;
    eprintln!("[smoke] navigated to {TARGET_URL}");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // dismiss the splash if it's there
    let splash = browser
        .evaluate(&target, ENTER_DRAW_MODE_JS)
        .await
        .unwrap_or(serde_json::Value::Null);
    eprintln!(
        "[smoke] splash: {}",
        splash.as_str().unwrap_or("unknown")
    );
    tokio::time::sleep(Duration::from_millis(800)).await;

    // locate the drawing canvas
    let mut canvas: Option<(f64, f64, f64, f64)> = None;
    for attempt in 0..20_i32 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let v = browser.evaluate(&target, FIND_CANVAS_JS).await?;
        if let Some(obj) = v.as_object() {
            let x = obj.get("x").and_then(|n| n.as_f64());
            let y = obj.get("y").and_then(|n| n.as_f64());
            let w = obj.get("w").and_then(|n| n.as_f64());
            let h = obj.get("h").and_then(|n| n.as_f64());
            if let (Some(x), Some(y), Some(w), Some(h)) = (x, y, w, h) {
                eprintln!(
                    "[smoke] canvas found @{}ms: {:.0}x{:.0} at ({:.0},{:.0})",
                    attempt * 500,
                    w,
                    h,
                    x,
                    y
                );
                canvas = Some((x, y, w, h));
                break;
            }
        }
    }
    let (cx, cy, cw, ch) = canvas.context("AutoDraw canvas not found after 10s")?;

    // baseline ink count BEFORE drawing
    let baseline = browser.evaluate(&target, COUNT_INK_JS).await?;
    let baseline_marks = baseline
        .as_object()
        .and_then(|o| o.get("marks").and_then(|n| n.as_i64()))
        .unwrap_or(-1);
    eprintln!("[smoke] baseline ink marks (before drag): {baseline_marks}");

    // draw a zig-zag inside the canvas centered around the middle, well away
    // from the left toolbar (~80px wide) and top app-bar.
    let pad_left = 120.0_f64;
    let pad_top = 80.0_f64;
    let pad_right = 40.0_f64;
    let pad_bot = 40.0_f64;
    let usable_x0 = cx + pad_left;
    let usable_y0 = cy + pad_top;
    let usable_w = (cw - pad_left - pad_right).max(200.0);
    let usable_h = (ch - pad_top - pad_bot).max(200.0);

    let center_x = usable_x0 + usable_w / 2.0;
    let center_y = usable_y0 + usable_h / 2.0;
    let radius_x = (usable_w * 0.30).min(220.0);
    let radius_y = (usable_h * 0.30).min(220.0);
    let steps: i32 = 80;
    let start_x = (center_x + radius_x).round() as i32;
    let start_y = center_y.round() as i32;

    eprintln!(
        "[smoke] drawing circle center=({:.0},{:.0}) r=({:.0},{:.0}) steps={steps}",
        center_x, center_y, radius_x, radius_y
    );

    // approach without button so canvas sees natural pointer movement first
    for i in 1..=6_i32 {
        let t = f64::from(i) / 6.0;
        let x = (center_x + radius_x * 1.4 - radius_x * 0.4 * t).round() as i32;
        let y = (center_y - 30.0 + 30.0 * t).round() as i32;
        browser.mouse_move(&target, x, y, 0).await?;
        tokio::time::sleep(Duration::from_millis(15)).await;
    }
    browser.mouse_down(&target, start_x, start_y).await?;
    // trace a full ellipse
    for i in 1..=steps {
        let theta = (f64::from(i) / f64::from(steps)) * std::f64::consts::TAU;
        let x = (center_x + radius_x * theta.cos()).round() as i32;
        let y = (center_y + radius_y * theta.sin()).round() as i32;
        browser.mouse_move(&target, x, y, 1).await?;
        tokio::time::sleep(Duration::from_millis(18)).await;
    }
    browser.mouse_up(&target, start_x, start_y).await?;

    // settle + read pixel delta
    tokio::time::sleep(Duration::from_millis(700)).await;
    let after = browser.evaluate(&target, COUNT_INK_JS).await?;
    let after_marks = after
        .as_object()
        .and_then(|o| o.get("marks").and_then(|n| n.as_i64()))
        .unwrap_or(-1);
    let sampled = after
        .as_object()
        .and_then(|o| o.get("sampled").and_then(|n| n.as_i64()))
        .unwrap_or(-1);
    let canvas_pixels = after
        .as_object()
        .and_then(|o| {
            let w = o.get("canvas_w").and_then(|n| n.as_i64())?;
            let h = o.get("canvas_h").and_then(|n| n.as_i64())?;
            Some((w, h))
        });
    eprintln!(
        "[smoke] ink AFTER drag: marks={after_marks}  sampled={sampled}  canvas_pixels={canvas_pixels:?}"
    );

    // screenshot for visual confirmation
    let png = browser.screenshot(&target, ScreenshotFormat::Png).await?;
    let out = "/tmp/mouse_drag_smoke.png";
    std::fs::write(out, &png).context("write screenshot")?;
    eprintln!("[smoke] screenshot saved -> {out} ({} bytes)", png.len());

    let delta = if baseline_marks >= 0 && after_marks >= 0 {
        after_marks - baseline_marks
    } else {
        after_marks
    };
    eprintln!("[smoke] ink delta: {delta} marks");

    if delta < 50 {
        bail!(
            "drag did NOT mark the canvas (delta={delta}, baseline={baseline_marks}, after={after_marks}) — synthetic drag pipeline broken"
        );
    }
    eprintln!("[smoke] PASS  (canvas inked +{delta} marks; check {out})");
    Ok(())
}

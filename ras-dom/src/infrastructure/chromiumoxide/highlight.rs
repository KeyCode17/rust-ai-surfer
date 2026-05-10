use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::Browser;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use ras_errors::AppError;
use ras_types::TargetId as RasTargetId;
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::domain::repository::HighlightOptions;
use crate::infrastructure::chromiumoxide::snapshot::page_for_public;

const OVERLAY_ID: &str = "__ras_dom_highlight_overlay__";

pub(crate) async fn capture_with_overlay(
    browser: &Arc<Mutex<Browser>>,
    target: &RasTargetId,
    options: &HighlightOptions,
    request_timeout: Duration,
) -> Result<Vec<u8>, AppError> {
    timeout(
        request_timeout,
        capture_with_overlay_inner(browser, target, options),
    )
    .await
    .map_err(|_| AppError::ActionFailed("highlight timed out".into()))?
}

async fn capture_with_overlay_inner(
    browser: &Arc<Mutex<Browser>>,
    target: &RasTargetId,
    options: &HighlightOptions,
) -> Result<Vec<u8>, AppError> {
    let page = page_for_public(browser, target).await?;

    let install_script = build_install_script(options);
    page.evaluate(install_script.as_str())
        .await
        .map_err(|e| AppError::ActionFailed(format!("install overlay: {e}")))?;

    let bytes = page
        .screenshot(
            chromiumoxide::page::ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .full_page(false)
                .build(),
        )
        .await;

    let _ = page.evaluate(remove_script().as_str()).await;

    bytes.map_err(|e| AppError::ActionFailed(format!("screenshot: {e}")))
}

fn build_install_script(options: &HighlightOptions) -> String {
    let max = options.max_index;
    let draw = options.draw_bounding_boxes;
    let labels = options.include_text_labels;
    format!(
        r#"
(function() {{
  const id = "{OVERLAY_ID}";
  const existing = document.getElementById(id);
  if (existing) existing.remove();

  const overlay = document.createElement('div');
  overlay.id = id;
  Object.assign(overlay.style, {{
    position: 'fixed', top: '0', left: '0', width: '100vw', height: '100vh',
    pointerEvents: 'none', zIndex: '2147483647'
  }});

  const selector = 'a, button, input, select, textarea, summary, label, [onclick], [role]';
  const els = Array.from(document.querySelectorAll(selector)).slice(0, {max});
  els.forEach((el, i) => {{
    const r = el.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) return;
    if ({draw}) {{
      const box = document.createElement('div');
      Object.assign(box.style, {{
        position: 'absolute',
        left: r.left + 'px', top: r.top + 'px',
        width: r.width + 'px', height: r.height + 'px',
        border: '2px solid #ff3366',
        boxSizing: 'border-box'
      }});
      overlay.appendChild(box);
    }}
    if ({labels}) {{
      const tag = document.createElement('div');
      tag.textContent = '[' + i + ']';
      Object.assign(tag.style, {{
        position: 'absolute',
        left: r.left + 'px', top: Math.max(0, r.top - 16) + 'px',
        background: '#ff3366', color: 'white',
        font: 'bold 11px sans-serif',
        padding: '1px 4px', borderRadius: '2px',
        lineHeight: '14px'
      }});
      overlay.appendChild(tag);
    }}
  }});
  document.documentElement.appendChild(overlay);
}})();
"#
    )
}

fn remove_script() -> String {
    format!(
        r#"(function() {{ const el = document.getElementById("{OVERLAY_ID}"); if (el) el.remove(); }})();"#
    )
}

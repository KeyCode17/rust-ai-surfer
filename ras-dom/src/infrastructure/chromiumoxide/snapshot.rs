use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::Browser;
use chromiumoxide::cdp::browser_protocol::dom_snapshot::CaptureSnapshotParams;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::cdp::browser_protocol::target::{GetTargetsParams, TargetInfo};
use ras_errors::AppError;
use ras_types::TargetId as RasTargetId;
use tokio::sync::Mutex;
use tokio::time::timeout;
use url::Url;

use crate::domain::state_summary::{BrowserStateSummary, TabInfo};
use crate::infrastructure::chromiumoxide::snapshot_parser::parse_snapshot;

pub(crate) async fn capture_snapshot(
    browser: &Arc<Mutex<Browser>>,
    target: &RasTargetId,
    request_timeout: Duration,
) -> Result<BrowserStateSummary, AppError> {
    let result = timeout(request_timeout, capture_snapshot_inner(browser, target))
        .await
        .map_err(|_| AppError::ActionFailed("snapshot timed out".into()))?;
    result
}

async fn capture_snapshot_inner(
    browser: &Arc<Mutex<Browser>>,
    target: &RasTargetId,
) -> Result<BrowserStateSummary, AppError> {
    let page = page_for(browser, target).await?;

    let params = CaptureSnapshotParams::builder()
        .include_paint_order(true)
        .include_dom_rects(true)
        .build()
        .map_err(|e| AppError::ActionFailed(format!("snapshot params: {e}")))?;
    let snapshot_resp = page
        .execute(params)
        .await
        .map_err(|e| AppError::ActionFailed(format!("captureSnapshot: {e}")))?;

    let (url, title, clickables, page_stats) = parse_snapshot(&snapshot_resp.result)
        .map_err(|e| AppError::ActionFailed(format!("snapshot parse: {e}")))?;

    let screenshot_b64 = capture_screenshot_b64(&page).await.ok();
    let tabs = list_tabs(browser).await.unwrap_or_default();

    Ok(BrowserStateSummary {
        target: target.clone(),
        url,
        title,
        tree: None,
        clickables,
        screenshot_b64,
        tabs,
        page_stats,
    })
}

async fn capture_screenshot_b64(page: &chromiumoxide::Page) -> Result<String, AppError> {
    let bytes = page
        .screenshot(
            chromiumoxide::page::ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .full_page(false)
                .build(),
        )
        .await
        .map_err(|e| AppError::ActionFailed(format!("screenshot: {e}")))?;
    Ok(base64_encode(&bytes))
}

async fn list_tabs(browser: &Arc<Mutex<Browser>>) -> Result<Vec<TabInfo>, AppError> {
    let guard = browser.lock().await;
    let resp = guard
        .execute(GetTargetsParams::default())
        .await
        .map_err(|e| AppError::ActionFailed(format!("getTargets: {e}")))?;
    drop(guard);
    Ok(resp
        .result
        .target_infos
        .iter()
        .filter(|t: &&TargetInfo| t.r#type == "page")
        .filter_map(|t| {
            let url = Url::parse(&t.url).ok()?;
            Some(TabInfo {
                target: RasTargetId(t.target_id.inner().as_str().into()),
                url,
                title: t.title.clone(),
                focused: t.attached,
            })
        })
        .collect())
}

async fn page_for(
    browser: &Arc<Mutex<Browser>>,
    target: &RasTargetId,
) -> Result<chromiumoxide::Page, AppError> {
    let guard = browser.lock().await;
    let pages = guard
        .pages()
        .await
        .map_err(|e| AppError::ActionFailed(format!("list pages: {e}")))?;
    for p in pages {
        if p.target_id().inner().as_str() == target.0.as_str() {
            return Ok(p);
        }
    }
    Err(AppError::NotFound(format!("page for target {}", target.0)))
}

fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let n =
            (u32::from(bytes[i]) << 16) | (u32::from(bytes[i + 1]) << 8) | u32::from(bytes[i + 2]);
        out.push(CHARS[((n >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 6) & 0x3f) as usize] as char);
        out.push(CHARS[(n & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let n = u32::from(bytes[i]) << 16;
        out.push(CHARS[((n >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3f) as usize] as char);
        out.push_str("==");
    } else if rem == 2 {
        let n = (u32::from(bytes[i]) << 16) | (u32::from(bytes[i + 1]) << 8);
        out.push(CHARS[((n >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 6) & 0x3f) as usize] as char);
        out.push('=');
    }
    out
}

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::network::EnableParams as NetworkEnableParams;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::cdp::browser_protocol::storage::ClearDataForOriginParams;
use chromiumoxide::types::MethodId;
use chromiumoxide::{Browser, Command, Method, Page};
use ras_errors::AppError;
use ras_types::TargetId;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::domain::repository::ScreenshotFormat;
use crate::infrastructure::chromiumoxide_helpers::page_for;
use crate::infrastructure::timeout::within;

pub(crate) async fn with_page<F, Fut, T>(
    browser: &Arc<Mutex<Browser>>,
    target: &TargetId,
    timeout: Duration,
    label: &'static str,
    f: F,
) -> Result<T, AppError>
where
    F: FnOnce(Page) -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let page = page_for(browser, target).await?;
    within(label, timeout, f(page)).await
}

#[derive(Serialize)]
pub(crate) struct LegacyBlockUrls {
    pub urls: Vec<String>,
}

impl Method for LegacyBlockUrls {
    fn identifier(&self) -> MethodId {
        "Network.setBlockedURLs".into()
    }
}

impl Command for LegacyBlockUrls {
    type Response = serde_json::Value;
}

pub(crate) async fn cdp_block_urls(page: &Page, urls: Vec<String>) -> Result<(), AppError> {
    page.execute(NetworkEnableParams::default())
        .await
        .map_err(|e| AppError::ActionFailed(format!("Network.enable: {e}")))?;
    page.execute(LegacyBlockUrls { urls })
        .await
        .map_err(|e| AppError::ActionFailed(format!("Network.setBlockedURLs: {e}")))?;
    Ok(())
}

pub(crate) async fn cdp_clear_cookies(page: &Page, origin: &str) -> Result<(), AppError> {
    let params = ClearDataForOriginParams::new(
        origin.to_string(),
        "cookies,local_storage,session_storage,indexeddb,service_workers,cache_storage",
    );
    page.execute(params)
        .await
        .map_err(|e| AppError::ActionFailed(format!("Storage.clearDataForOrigin: {e}")))?;
    Ok(())
}

pub(crate) async fn cdp_navigate(page: &Page, url_str: String) -> Result<(), AppError> {
    page.goto(url_str.clone())
        .await
        .map_err(|e| AppError::ActionFailed(format!("goto {url_str}: {e}")))?;
    Ok(())
}

pub(crate) async fn cdp_evaluate(
    page: &Page,
    expression: String,
) -> Result<serde_json::Value, AppError> {
    let v = page
        .evaluate(expression.as_str())
        .await
        .map_err(|e| AppError::ActionFailed(format!("evaluate: {e}")))?;
    Ok(v.into_value::<serde_json::Value>()
        .unwrap_or(serde_json::Value::Null))
}

pub(crate) async fn cdp_click_at(page: &Page, x: i32, y: i32) -> Result<(), AppError> {
    page.click(chromiumoxide::layout::Point {
        x: f64::from(x),
        y: f64::from(y),
    })
    .await
    .map_err(|e| AppError::ActionFailed(format!("click_at: {e}")))?;
    Ok(())
}

pub(crate) async fn cdp_screenshot(
    page: &Page,
    format: ScreenshotFormat,
) -> Result<Vec<u8>, AppError> {
    let cdp_format = match format {
        ScreenshotFormat::Png => CaptureScreenshotFormat::Png,
        ScreenshotFormat::Jpeg => CaptureScreenshotFormat::Jpeg,
    };
    page.screenshot(
        chromiumoxide::page::ScreenshotParams::builder()
            .format(cdp_format)
            .full_page(false)
            .build(),
    )
    .await
    .map_err(|e| AppError::ActionFailed(format!("screenshot: {e}")))
}

pub(crate) async fn cdp_close(page: Page) -> Result<(), AppError> {
    page.close()
        .await
        .map_err(|e| AppError::ActionFailed(format!("close_target: {e}")))?;
    Ok(())
}

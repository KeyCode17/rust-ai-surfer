use std::sync::Arc;

use chromiumoxide::Browser;
use chromiumoxide::cdp::browser_protocol::browser::{
    BrowserContextId, SetDownloadBehaviorBehavior, SetDownloadBehaviorParams,
};
use chromiumoxide::cdp::browser_protocol::storage::ClearCookiesParams;
use chromiumoxide::cdp::browser_protocol::target::{
    CreateBrowserContextParams, CreateTargetParams, GetTargetsParams,
};
use ras_errors::AppError;
use ras_types::{ContextId, TargetId};
use tokio::sync::Mutex;
use url::Url;

fn to_cdp(ctx: &ContextId) -> BrowserContextId {
    BrowserContextId::new(ctx.0.to_string())
}

/// Create an isolated context and point its downloads at a per-context directory.
pub(crate) async fn create_context(browser: &Arc<Mutex<Browser>>) -> Result<ContextId, AppError> {
    let guard = browser.lock().await;
    let id = guard
        .create_browser_context(CreateBrowserContextParams::default())
        .await
        .map_err(|e| AppError::ActionFailed(format!("createBrowserContext: {e}")))?;
    let ctx = ContextId(id.inner().as_str().into());
    drop(guard);
    set_context_download_dir(browser, &ctx).await?;
    Ok(ctx)
}

pub(crate) async fn dispose_context(
    browser: &Arc<Mutex<Browser>>,
    ctx: &ContextId,
) -> Result<(), AppError> {
    let guard = browser.lock().await;
    guard
        .dispose_browser_context(to_cdp(ctx))
        .await
        .map_err(|e| AppError::ActionFailed(format!("disposeBrowserContext: {e}")))
}

pub(crate) async fn new_target_in(
    browser: &Arc<Mutex<Browser>>,
    ctx: &ContextId,
    url: &Url,
) -> Result<TargetId, AppError> {
    let mut params = CreateTargetParams::new(url.as_str());
    params.browser_context_id = Some(to_cdp(ctx));
    let guard = browser.lock().await;
    let page = guard
        .new_page(params)
        .await
        .map_err(|e| AppError::ActionFailed(format!("new_page in context: {e}")))?;
    Ok(TargetId(page.target_id().as_ref().into()))
}

#[allow(dead_code)]
pub(crate) async fn list_targets_in(
    browser: &Arc<Mutex<Browser>>,
    ctx: &ContextId,
) -> Result<Vec<TargetId>, AppError> {
    let wanted = to_cdp(ctx);
    let guard = browser.lock().await;
    let resp = guard
        .execute(GetTargetsParams::default())
        .await
        .map_err(|e| AppError::BrowserDisconnected(format!("getTargets: {e}")))?;
    Ok(resp
        .result
        .target_infos
        .iter()
        .filter(|t| t.browser_context_id.as_ref() == Some(&wanted))
        .filter(|t| t.r#type == "page")
        .map(|t| TargetId(t.target_id.as_ref().into()))
        .collect())
}

#[allow(dead_code)]
pub(crate) async fn clear_context_cookies(
    browser: &Arc<Mutex<Browser>>,
    ctx: &ContextId,
) -> Result<(), AppError> {
    let params = ClearCookiesParams::builder()
        .browser_context_id(to_cdp(ctx))
        .build();
    let guard = browser.lock().await;
    guard
        .execute(params)
        .await
        .map_err(|e| AppError::ActionFailed(format!("storage.clearCookies: {e}")))?;
    Ok(())
}

async fn set_context_download_dir(
    browser: &Arc<Mutex<Browser>>,
    ctx: &ContextId,
) -> Result<(), AppError> {
    let dir = std::env::temp_dir()
        .join("ras-downloads")
        .join(ctx.0.as_str());
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::ActionFailed(format!("download dir: {e}")))?;
    let mut params = SetDownloadBehaviorParams::new(SetDownloadBehaviorBehavior::AllowAndName);
    params.browser_context_id = Some(to_cdp(ctx));
    params.download_path = Some(dir.to_string_lossy().into_owned());
    let guard = browser.lock().await;
    guard
        .execute(params)
        .await
        .map_err(|e| AppError::ActionFailed(format!("setDownloadBehavior: {e}")))?;
    Ok(())
}

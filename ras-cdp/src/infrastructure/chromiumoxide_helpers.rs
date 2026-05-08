use std::sync::Arc;

use chromiumoxide::Browser;
use ras_errors::AppError;
use ras_types::TargetId;
use tokio::sync::Mutex;
use url::Url;

pub(crate) async fn page_for(
    browser: &Arc<Mutex<Browser>>,
    target: &TargetId,
) -> Result<chromiumoxide::Page, AppError> {
    let browser = browser.lock().await;
    let pages = browser
        .pages()
        .await
        .map_err(|e| AppError::BrowserDisconnected(format!("list pages: {e}")))?;
    for p in pages {
        if p.target_id().as_ref() == target.0.as_str() {
            return Ok(p);
        }
    }
    Err(AppError::NotFound(format!("target {} not found", target.0)))
}

pub(crate) async fn list_target_ids(
    browser: &Arc<Mutex<Browser>>,
) -> Result<Vec<TargetId>, AppError> {
    let browser = browser.lock().await;
    let pages = browser
        .pages()
        .await
        .map_err(|e| AppError::BrowserDisconnected(format!("list pages: {e}")))?;
    Ok(pages
        .into_iter()
        .map(|p| TargetId(p.target_id().as_ref().into()))
        .collect())
}

pub(crate) async fn new_target(
    browser: &Arc<Mutex<Browser>>,
    url: &Url,
) -> Result<TargetId, AppError> {
    let browser = browser.lock().await;
    let page = browser
        .new_page(url.as_str())
        .await
        .map_err(|e| AppError::ActionFailed(format!("new_page: {e}")))?;
    Ok(TargetId(page.target_id().as_ref().into()))
}

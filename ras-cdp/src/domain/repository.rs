use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{BackendNodeId, ContextId, TargetId};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::viewport::Viewport;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotFormat {
    #[default]
    Png,
    Jpeg,
}

#[async_trait]
pub trait BrowserPort: Send + Sync + 'static {
    async fn cdp_url(&self) -> Result<Url, AppError>;
    async fn list_targets(&self) -> Result<Vec<TargetId>, AppError>;
    async fn focused_target(&self) -> Result<TargetId, AppError>;
    async fn navigate(&self, target: &TargetId, url: &Url) -> Result<(), AppError>;
    async fn evaluate(
        &self,
        target: &TargetId,
        expression: &str,
    ) -> Result<serde_json::Value, AppError>;
    async fn click_at(&self, target: &TargetId, x: i32, y: i32) -> Result<(), AppError>;
    async fn click_node(&self, target: &TargetId, node: BackendNodeId) -> Result<(), AppError>;
    async fn mouse_down(&self, target: &TargetId, x: i32, y: i32) -> Result<(), AppError>;
    async fn mouse_up(&self, target: &TargetId, x: i32, y: i32) -> Result<(), AppError>;
    /// Dispatch a single MouseMoved event. `buttons_mask` follows the W3C
    /// buttons bitfield: 0 = none, 1 = left held (drag), 2 = right, 4 = middle.
    async fn mouse_move(
        &self,
        target: &TargetId,
        x: i32,
        y: i32,
        buttons_mask: i64,
    ) -> Result<(), AppError>;
    /// Humanized press-and-hold at (x, y): pre-approach moves, press, jittered
    /// drag-style MouseMoved events with buttons=1 during the hold, release.
    /// `ms` is the hold duration (excluding approach overhead).
    async fn mouse_hold(&self, target: &TargetId, x: i32, y: i32, ms: u64) -> Result<(), AppError>;
    async fn type_text(&self, target: &TargetId, text: &str) -> Result<(), AppError>;
    async fn screenshot(
        &self,
        target: &TargetId,
        format: ScreenshotFormat,
    ) -> Result<Vec<u8>, AppError>;
    async fn set_viewport(&self, target: &TargetId, viewport: Viewport) -> Result<(), AppError>;
    /// Enable Network domain and block requests whose URL matches any of the
    /// given glob patterns (Chrome's `Network.setBlockedURLs.urls` accepts
    /// `*` as the only wildcard — e.g. `*googlesyndication.com*`). Used to
    /// suppress ads and trackers so the page layout settles deterministically.
    async fn block_urls(
        &self,
        target: &TargetId,
        url_patterns: Vec<String>,
    ) -> Result<(), AppError>;
    /// Clear cookies + local storage + session storage for the given origin.
    /// `origin` is a scheme+host (e.g. `https://www.pedidosya.com.ar`). Used
    /// to reset bot-detection clearance cookies between test runs.
    async fn clear_cookies(&self, target: &TargetId, origin: &str) -> Result<(), AppError>;
    async fn close_target(&self, target: &TargetId) -> Result<(), AppError>;
    async fn create_target(&self, url: &Url) -> Result<TargetId, AppError>;

    /// Create a fresh isolated browser context (separate cookies/storage).
    async fn create_context(&self) -> Result<ContextId, AppError> {
        Err(AppError::ActionFailed(
            "create_context not supported by this BrowserPort".into(),
        ))
    }

    /// Dispose a context, freeing its cookies/storage/tabs.
    async fn close_context(&self, _ctx: &ContextId) -> Result<(), AppError> {
        Err(AppError::ActionFailed(
            "close_context not supported by this BrowserPort".into(),
        ))
    }

    /// Open a new tab inside a specific context.
    async fn new_target_in(&self, _ctx: &ContextId, _url: &Url) -> Result<TargetId, AppError> {
        Err(AppError::ActionFailed(
            "new_target_in not supported by this BrowserPort".into(),
        ))
    }

    /// List only the tabs belonging to a specific context.
    async fn list_targets_in(&self, _ctx: &ContextId) -> Result<Vec<TargetId>, AppError> {
        Err(AppError::ActionFailed(
            "list_targets_in not supported by this BrowserPort".into(),
        ))
    }
}

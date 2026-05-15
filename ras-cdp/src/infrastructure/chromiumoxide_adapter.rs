use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chromiumoxide::Browser;
use chromiumoxide::handler::HandlerConfig;
use futures::StreamExt;
use ras_errors::AppError;
use ras_types::{BackendNodeId, TargetId};
use tokio::sync::Mutex;
use tracing::warn;
use url::Url;

use crate::domain::repository::{BrowserPort, ScreenshotFormat};
use crate::domain::viewport::Viewport;
use crate::infrastructure::cdp_ext::{
    cdp_block_urls, cdp_clear_cookies, cdp_click_at, cdp_close, cdp_evaluate, cdp_navigate,
    cdp_screenshot, with_page,
};
use crate::infrastructure::chromiumoxide_helpers::{list_target_ids, new_target, page_for};
use crate::infrastructure::chromiumoxide_input::{click_backend_node, type_chars};
use crate::infrastructure::mouse_input::{
    dispatch_mouse_hold, dispatch_mouse_move, dispatch_mouse_press, dispatch_mouse_release,
};
use crate::infrastructure::timeout::within;

pub struct ChromiumoxideAdapter {
    browser: Arc<Mutex<Browser>>,
    cdp_url: Url,
    request_timeout: Duration,
}

impl std::fmt::Debug for ChromiumoxideAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChromiumoxideAdapter")
            .field("cdp_url", &self.cdp_url)
            .finish()
    }
}

impl ChromiumoxideAdapter {
    #[must_use]
    pub fn browser_arc(&self) -> Arc<Mutex<Browser>> {
        Arc::clone(&self.browser)
    }

    pub async fn connect(cdp_url: Url, request_timeout: Duration) -> Result<Self, AppError> {
        let cfg = HandlerConfig::default();
        let (browser, mut handler) = Browser::connect_with_config(cdp_url.as_str(), cfg)
            .await
            .map_err(|e| AppError::BrowserDisconnected(format!("cdp connect: {e}")))?;
        tokio::spawn(async move {
            while let Some(ev) = handler.next().await {
                if let Err(e) = ev {
                    warn!(error = %e, "cdp handler event error");
                }
            }
        });
        Ok(Self {
            browser: Arc::new(Mutex::new(browser)),
            cdp_url,
            request_timeout,
        })
    }
}

macro_rules! op {
    ($self:expr, $target:expr, $label:literal, |$p:ident| $body:expr) => {
        with_page(
            &$self.browser,
            $target,
            $self.request_timeout,
            $label,
            |$p| async move { $body },
        )
        .await
    };
}

#[async_trait]
impl BrowserPort for ChromiumoxideAdapter {
    async fn cdp_url(&self) -> Result<Url, AppError> {
        Ok(self.cdp_url.clone())
    }
    async fn list_targets(&self) -> Result<Vec<TargetId>, AppError> {
        list_target_ids(&self.browser).await
    }
    async fn focused_target(&self) -> Result<TargetId, AppError> {
        let mut targets = self.list_targets().await?;
        targets
            .pop()
            .ok_or_else(|| AppError::NotFound("no focused target".into()))
    }
    async fn navigate(&self, target: &TargetId, url: &Url) -> Result<(), AppError> {
        let u = url.as_str().to_string();
        op!(self, target, "navigate", |p| cdp_navigate(&p, u).await)
    }
    async fn evaluate(
        &self,
        target: &TargetId,
        expression: &str,
    ) -> Result<serde_json::Value, AppError> {
        let e = expression.to_string();
        op!(self, target, "evaluate", |p| cdp_evaluate(&p, e).await)
    }
    async fn click_at(&self, target: &TargetId, x: i32, y: i32) -> Result<(), AppError> {
        op!(self, target, "click_at", |p| cdp_click_at(&p, x, y).await)
    }
    async fn click_node(&self, target: &TargetId, node: BackendNodeId) -> Result<(), AppError> {
        op!(self, target, "click_node", |p| click_backend_node(&p, node)
            .await)
    }
    async fn mouse_down(&self, target: &TargetId, x: i32, y: i32) -> Result<(), AppError> {
        op!(self, target, "mouse_down", |p| dispatch_mouse_press(
            &p, x, y
        )
        .await)
    }
    async fn mouse_up(&self, target: &TargetId, x: i32, y: i32) -> Result<(), AppError> {
        op!(self, target, "mouse_up", |p| dispatch_mouse_release(
            &p, x, y
        )
        .await)
    }
    async fn mouse_move(
        &self,
        target: &TargetId,
        x: i32,
        y: i32,
        buttons_mask: i64,
    ) -> Result<(), AppError> {
        op!(self, target, "mouse_move", |p| dispatch_mouse_move(
            &p,
            x,
            y,
            buttons_mask
        )
        .await)
    }
    async fn mouse_hold(&self, target: &TargetId, x: i32, y: i32, ms: u64) -> Result<(), AppError> {
        let page = page_for(&self.browser, target).await?;
        let budget = self.request_timeout + Duration::from_millis(ms + 5_000);
        within("mouse_hold", budget, async move {
            dispatch_mouse_hold(&page, x, y, ms).await
        })
        .await
    }
    async fn type_text(&self, target: &TargetId, text: &str) -> Result<(), AppError> {
        let t = text.to_string();
        op!(self, target, "type_text", |p| type_chars(&p, &t).await)
    }
    async fn screenshot(
        &self,
        target: &TargetId,
        format: ScreenshotFormat,
    ) -> Result<Vec<u8>, AppError> {
        op!(self, target, "screenshot", |p| cdp_screenshot(&p, format)
            .await)
    }
    async fn set_viewport(&self, _target: &TargetId, _viewport: Viewport) -> Result<(), AppError> {
        Ok(())
    }
    async fn block_urls(
        &self,
        target: &TargetId,
        url_patterns: Vec<String>,
    ) -> Result<(), AppError> {
        op!(self, target, "block_urls", |p| cdp_block_urls(
            &p,
            url_patterns
        )
        .await)
    }
    async fn clear_cookies(&self, target: &TargetId, origin: &str) -> Result<(), AppError> {
        let o = origin.to_string();
        op!(self, target, "clear_cookies", |p| cdp_clear_cookies(&p, &o)
            .await)
    }
    async fn close_target(&self, target: &TargetId) -> Result<(), AppError> {
        op!(self, target, "close_target", |p| cdp_close(p).await)
    }
    async fn create_target(&self, url: &Url) -> Result<TargetId, AppError> {
        new_target(&self.browser, url).await
    }
}

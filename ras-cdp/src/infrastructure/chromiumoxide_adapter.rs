use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chromiumoxide::Browser;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::handler::HandlerConfig;
use futures::StreamExt;
use ras_errors::AppError;
use ras_types::{BackendNodeId, TargetId};
use tokio::sync::Mutex;
use tracing::warn;
use url::Url;

use crate::domain::repository::{BrowserPort, ScreenshotFormat};
use crate::domain::viewport::Viewport;
use crate::infrastructure::timeout::within;

pub struct ChromiumoxideAdapter {
    browser: Arc<Mutex<Browser>>,
    cdp_url: Url,
    request_timeout: Duration,
}

impl std::fmt::Debug for ChromiumoxideAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChromiumoxideAdapter").field("cdp_url", &self.cdp_url).finish()
    }
}

impl ChromiumoxideAdapter {
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
        Ok(Self { browser: Arc::new(Mutex::new(browser)), cdp_url, request_timeout })
    }

    async fn page_for(&self, target: &TargetId) -> Result<chromiumoxide::Page, AppError> {
        let browser = self.browser.lock().await;
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
}

#[async_trait]
impl BrowserPort for ChromiumoxideAdapter {
    async fn cdp_url(&self) -> Result<Url, AppError> {
        Ok(self.cdp_url.clone())
    }

    async fn list_targets(&self) -> Result<Vec<TargetId>, AppError> {
        let browser = self.browser.lock().await;
        let pages = browser
            .pages()
            .await
            .map_err(|e| AppError::BrowserDisconnected(format!("list pages: {e}")))?;
        Ok(pages.into_iter().map(|p| TargetId(p.target_id().as_ref().into())).collect())
    }

    async fn focused_target(&self) -> Result<TargetId, AppError> {
        let mut targets = self.list_targets().await?;
        targets
            .pop()
            .ok_or_else(|| AppError::NotFound("no focused target".into()))
    }

    async fn navigate(&self, target: &TargetId, url: &Url) -> Result<(), AppError> {
        let page = self.page_for(target).await?;
        let url_str = url.as_str().to_string();
        within("navigate", self.request_timeout, async move {
            page.goto(url_str.clone())
                .await
                .map_err(|e| AppError::ActionFailed(format!("goto {url_str}: {e}")))?;
            Ok(())
        })
        .await
    }

    async fn evaluate(
        &self,
        target: &TargetId,
        expression: &str,
    ) -> Result<serde_json::Value, AppError> {
        let page = self.page_for(target).await?;
        let expr = expression.to_string();
        within("evaluate", self.request_timeout, async move {
            let v = page
                .evaluate(expr.as_str())
                .await
                .map_err(|e| AppError::ActionFailed(format!("evaluate: {e}")))?;
            Ok(v.into_value::<serde_json::Value>().unwrap_or(serde_json::Value::Null))
        })
        .await
    }

    async fn click_at(&self, target: &TargetId, x: i32, y: i32) -> Result<(), AppError> {
        let page = self.page_for(target).await?;
        within("click_at", self.request_timeout, async move {
            page.click(chromiumoxide::layout::Point { x: f64::from(x), y: f64::from(y) })
                .await
                .map_err(|e| AppError::ActionFailed(format!("click_at: {e}")))?;
            Ok(())
        })
        .await
    }

    async fn click_node(&self, target: &TargetId, node: BackendNodeId) -> Result<(), AppError> {
        let page = self.page_for(target).await?;
        within("click_node", self.request_timeout, async move {
            let js = format!(
                "(() => {{ const el = document.body && document.querySelectorAll('*')[{}]; if (el) el.click(); }})()",
                node.0
            );
            page.evaluate(js.as_str())
                .await
                .map_err(|e| AppError::ActionFailed(format!("click_node: {e}")))?;
            Ok(())
        })
        .await
    }

    async fn type_text(&self, target: &TargetId, text: &str) -> Result<(), AppError> {
        let page = self.page_for(target).await?;
        let text = text.to_string();
        within("type_text", self.request_timeout, async move {
            for ch in text.chars() {
                page.evaluate(format!(
                    "document.activeElement && document.activeElement.dispatchEvent(new KeyboardEvent('keydown', {{key: '{}'}}))",
                    ch
                ).as_str())
                    .await
                    .map_err(|e| AppError::ActionFailed(format!("type_text: {e}")))?;
            }
            Ok(())
        })
        .await
    }

    async fn screenshot(
        &self,
        target: &TargetId,
        format: ScreenshotFormat,
    ) -> Result<Vec<u8>, AppError> {
        let page = self.page_for(target).await?;
        let cdp_format = match format {
            ScreenshotFormat::Png => CaptureScreenshotFormat::Png,
            ScreenshotFormat::Jpeg => CaptureScreenshotFormat::Jpeg,
        };
        within("screenshot", self.request_timeout, async move {
            page.screenshot(
                chromiumoxide::page::ScreenshotParams::builder()
                    .format(cdp_format)
                    .full_page(false)
                    .build(),
            )
            .await
            .map_err(|e| AppError::ActionFailed(format!("screenshot: {e}")))
        })
        .await
    }

    async fn set_viewport(&self, _target: &TargetId, _viewport: Viewport) -> Result<(), AppError> {
        Ok(())
    }

    async fn close_target(&self, target: &TargetId) -> Result<(), AppError> {
        let page = self.page_for(target).await?;
        page.close()
            .await
            .map_err(|e| AppError::ActionFailed(format!("close_target: {e}")))?;
        Ok(())
    }

    async fn create_target(&self, url: &Url) -> Result<TargetId, AppError> {
        let browser = self.browser.lock().await;
        let page = browser
            .new_page(url.as_str())
            .await
            .map_err(|e| AppError::ActionFailed(format!("new_page: {e}")))?;
        Ok(TargetId(page.target_id().as_ref().into()))
    }
}

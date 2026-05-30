//! Minimal `BrowserPort` stub for integration tests.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ras_cdp::{BrowserPort, ScreenshotFormat, Viewport};
use ras_errors::AppError;
use ras_events::EventBus;
use ras_types::{BackendNodeId, ContextId, TargetId};
use url::Url;

/// Records which CDP context operations were called.
#[derive(Debug, Default, Clone)]
pub struct CallLog(pub Arc<Mutex<Vec<String>>>);

impl CallLog {
    pub fn push(&self, s: impl Into<String>) {
        self.0.lock().expect("lock").push(s.into());
    }

    pub fn snapshot(&self) -> Vec<String> {
        self.0.lock().expect("lock").clone()
    }
}

/// Stub browser that captures `create_context` / `close_context` calls.
pub struct MockBrowser {
    pub log: CallLog,
    pub context_id: ContextId,
}

impl MockBrowser {
    pub fn new(ctx: &str) -> Self {
        Self {
            log: CallLog::default(),
            context_id: ContextId(ctx.into()),
        }
    }
}

#[async_trait]
impl BrowserPort for MockBrowser {
    async fn cdp_url(&self) -> Result<Url, AppError> {
        Url::parse("http://localhost:9222").map_err(|e| AppError::ActionFailed(e.to_string()))
    }

    async fn list_targets(&self) -> Result<Vec<TargetId>, AppError> {
        Ok(vec![])
    }

    async fn focused_target(&self) -> Result<TargetId, AppError> {
        Ok(TargetId("stub-target".into()))
    }

    async fn navigate(&self, _t: &TargetId, _url: &Url) -> Result<(), AppError> {
        Ok(())
    }

    async fn evaluate(&self, _t: &TargetId, _expr: &str) -> Result<serde_json::Value, AppError> {
        Ok(serde_json::Value::Null)
    }

    async fn click_at(&self, _t: &TargetId, _x: i32, _y: i32) -> Result<(), AppError> {
        Ok(())
    }

    async fn click_node(&self, _t: &TargetId, _n: BackendNodeId) -> Result<(), AppError> {
        Ok(())
    }

    async fn mouse_down(&self, _t: &TargetId, _x: i32, _y: i32) -> Result<(), AppError> {
        Ok(())
    }

    async fn mouse_up(&self, _t: &TargetId, _x: i32, _y: i32) -> Result<(), AppError> {
        Ok(())
    }

    async fn mouse_move(
        &self,
        _t: &TargetId,
        _x: i32,
        _y: i32,
        _buttons: i64,
    ) -> Result<(), AppError> {
        Ok(())
    }

    async fn mouse_hold(&self, _t: &TargetId, _x: i32, _y: i32, _ms: u64) -> Result<(), AppError> {
        Ok(())
    }

    async fn type_text(&self, _t: &TargetId, _text: &str) -> Result<(), AppError> {
        Ok(())
    }

    async fn screenshot(&self, _t: &TargetId, _fmt: ScreenshotFormat) -> Result<Vec<u8>, AppError> {
        Ok(vec![])
    }

    async fn set_viewport(&self, _t: &TargetId, _v: Viewport) -> Result<(), AppError> {
        Ok(())
    }

    async fn block_urls(&self, _t: &TargetId, _patterns: Vec<String>) -> Result<(), AppError> {
        Ok(())
    }

    async fn clear_cookies(&self, _t: &TargetId, _origin: &str) -> Result<(), AppError> {
        Ok(())
    }

    async fn close_target(&self, _t: &TargetId) -> Result<(), AppError> {
        Ok(())
    }

    async fn create_target(&self, _url: &Url) -> Result<TargetId, AppError> {
        Ok(TargetId("stub-target".into()))
    }

    async fn create_context(&self) -> Result<ContextId, AppError> {
        self.log.push("create_context");
        Ok(self.context_id.clone())
    }

    async fn close_context(&self, ctx: &ContextId) -> Result<(), AppError> {
        self.log.push(format!("close_context:{}", ctx.0));
        Ok(())
    }

    async fn attach_events(&self, _t: &TargetId, _bus: Arc<dyn EventBus>) -> Result<(), AppError> {
        Ok(())
    }
}

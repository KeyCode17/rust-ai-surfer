use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{BackendNodeId, TargetId};
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
    async fn type_text(&self, target: &TargetId, text: &str) -> Result<(), AppError>;
    async fn screenshot(
        &self,
        target: &TargetId,
        format: ScreenshotFormat,
    ) -> Result<Vec<u8>, AppError>;
    async fn set_viewport(&self, target: &TargetId, viewport: Viewport) -> Result<(), AppError>;
    async fn close_target(&self, target: &TargetId) -> Result<(), AppError>;
    async fn create_target(&self, url: &Url) -> Result<TargetId, AppError>;
}

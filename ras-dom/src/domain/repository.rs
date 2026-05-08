use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::TargetId;
use serde::{Deserialize, Serialize};

use crate::domain::state_summary::BrowserStateSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightOptions {
    pub draw_bounding_boxes: bool,
    pub include_text_labels: bool,
    pub max_index: u32,
}

impl Default for HighlightOptions {
    fn default() -> Self {
        Self { draw_bounding_boxes: true, include_text_labels: true, max_index: 200 }
    }
}

#[async_trait]
pub trait DomExtractor: Send + Sync + 'static {
    async fn snapshot(&self, target: &TargetId) -> Result<BrowserStateSummary, AppError>;
    async fn highlight(
        &self,
        target: &TargetId,
        options: &HighlightOptions,
    ) -> Result<Vec<u8>, AppError>;
}

use std::path::Path;

use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::recording::{RecordingFormat, RecordingState};

#[async_trait]
pub trait RecorderPort: Send + Sync + 'static {
    async fn start(
        &self,
        output: &Path,
        format: RecordingFormat,
    ) -> Result<RecordingState, AppError>;
    async fn frame(&self, png_bytes: &[u8]) -> Result<(), AppError>;
    async fn stop(&self) -> Result<RecordingState, AppError>;
}

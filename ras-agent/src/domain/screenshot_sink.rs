use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::step_screenshot::{StepScreenshot, StepScreenshotRequest};

/// Destination for per-step screenshots.
///
/// Implement this to store step artifacts anywhere the host project wants — a
/// database, object storage, an in-memory buffer. [`crate::FolderScreenshotSink`]
/// is the batteries-included local-directory implementation.
///
/// # Errors
///
/// Returns [`AppError`] when the screenshot cannot be persisted. The agent loop
/// logs and continues, so a failing sink never aborts a run.
#[async_trait]
pub trait StepScreenshotSink: Send + Sync + 'static {
    async fn save(
        &self,
        request: StepScreenshotRequest,
        bytes: &[u8],
    ) -> Result<StepScreenshot, AppError>;
}

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use ras_errors::AppError;

use crate::domain::screenshot_sink::StepScreenshotSink;
use crate::domain::step_screenshot::{StepScreenshot, StepScreenshotRequest, screenshot_extension};

const CREATE_DIR_FAILED: &str = "Failed to create screenshot directory";
const WRITE_FAILED: &str = "Failed to write screenshot";

/// Writes each step screenshot to `{root}/{agent_id}/step-{step:04}.{ext}`.
pub struct FolderScreenshotSink {
    root: PathBuf,
}

impl FolderScreenshotSink {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn path_for(&self, request: StepScreenshotRequest) -> PathBuf {
        self.root.join(request.agent.0.to_string()).join(format!(
            "step-{:04}.{}",
            request.step.0,
            screenshot_extension(request.format)
        ))
    }
}

#[async_trait]
impl StepScreenshotSink for FolderScreenshotSink {
    async fn save(
        &self,
        request: StepScreenshotRequest,
        bytes: &[u8],
    ) -> Result<StepScreenshot, AppError> {
        let path = self.path_for(request);
        let dir = path
            .parent()
            .ok_or_else(|| AppError::InternalError(CREATE_DIR_FAILED.into()))?;
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(|e| AppError::InternalError(format!("{CREATE_DIR_FAILED}: {e}")))?;
        tokio::fs::write(&path, bytes)
            .await
            .map_err(|e| AppError::InternalError(format!("{WRITE_FAILED}: {e}")))?;
        Ok(StepScreenshot {
            location: path.to_string_lossy().into_owned(),
            format: request.format,
            size_bytes: bytes.len() as u64,
            captured_at: Utc::now(),
        })
    }
}

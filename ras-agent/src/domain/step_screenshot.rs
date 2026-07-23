use chrono::{DateTime, Utc};
use ras_cdp::ScreenshotFormat;
use ras_types::{AgentId, StepId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct StepScreenshotRequest {
    pub agent: AgentId,
    pub step: StepId,
    pub format: ScreenshotFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepScreenshot {
    pub location: String,
    pub format: ScreenshotFormat,
    pub size_bytes: u64,
    pub captured_at: DateTime<Utc>,
}

#[must_use]
pub fn screenshot_extension(format: ScreenshotFormat) -> &'static str {
    match format {
        ScreenshotFormat::Png => "png",
        ScreenshotFormat::Jpeg => "jpeg",
    }
}

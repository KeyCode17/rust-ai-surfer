use std::sync::Arc;

use ras_cdp::{BrowserPort, ScreenshotFormat};
use ras_types::{AgentId, StepId, TargetId};

use crate::domain::screenshot_sink::StepScreenshotSink;
use crate::domain::step_screenshot::{StepScreenshot, StepScreenshotRequest};

pub async fn capture_step_screenshot(
    browser: &Arc<dyn BrowserPort>,
    sink: &Arc<dyn StepScreenshotSink>,
    agent: AgentId,
    step: StepId,
    target: &TargetId,
) -> Option<StepScreenshot> {
    let request = StepScreenshotRequest {
        agent,
        step,
        format: ScreenshotFormat::Png,
    };
    let bytes = match browser.screenshot(target, request.format).await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, step = step.0, "step screenshot capture failed; continuing");
            return None;
        }
    };
    match sink.save(request, &bytes).await {
        Ok(saved) => {
            tracing::debug!(step = step.0, location = %saved.location, "step screenshot saved");
            Some(saved)
        }
        Err(e) => {
            tracing::warn!(error = %e, step = step.0, "step screenshot save failed; continuing");
            None
        }
    }
}

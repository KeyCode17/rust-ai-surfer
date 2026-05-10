use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::Browser;
use ras_errors::AppError;
use ras_types::TargetId;
use tokio::sync::Mutex;

use crate::domain::repository::HighlightOptions;

pub(crate) async fn capture_with_overlay(
    _browser: &Arc<Mutex<Browser>>,
    _target: &TargetId,
    _options: &HighlightOptions,
    _timeout: Duration,
) -> Result<Vec<u8>, AppError> {
    Err(AppError::ActionFailed(
        "ChromiumoxideDomExtractor::highlight not implemented in this scaffold (C1); land in C3"
            .into(),
    ))
}

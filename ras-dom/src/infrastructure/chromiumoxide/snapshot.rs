use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::Browser;
use ras_errors::AppError;
use ras_types::TargetId;
use tokio::sync::Mutex;

use crate::domain::state_summary::BrowserStateSummary;

pub(crate) async fn capture_snapshot(
    _browser: &Arc<Mutex<Browser>>,
    _target: &TargetId,
    _timeout: Duration,
) -> Result<BrowserStateSummary, AppError> {
    Err(AppError::ActionFailed(
        "ChromiumoxideDomExtractor::snapshot not implemented in this scaffold (C1); land in C2"
            .into(),
    ))
}

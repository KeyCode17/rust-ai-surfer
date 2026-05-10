use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chromiumoxide::Browser;
use ras_errors::AppError;
use ras_types::TargetId;
use tokio::sync::Mutex;

use crate::domain::repository::{DomExtractor, HighlightOptions};
use crate::domain::state_summary::BrowserStateSummary;
use crate::infrastructure::chromiumoxide::highlight::capture_with_overlay;
use crate::infrastructure::chromiumoxide::snapshot::capture_snapshot;

pub struct ChromiumoxideDomExtractor {
    browser: Arc<Mutex<Browser>>,
    request_timeout: Duration,
}

impl std::fmt::Debug for ChromiumoxideDomExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChromiumoxideDomExtractor")
            .field("request_timeout", &self.request_timeout)
            .finish()
    }
}

impl ChromiumoxideDomExtractor {
    #[must_use]
    pub fn new(browser: Arc<Mutex<Browser>>, request_timeout: Duration) -> Self {
        Self {
            browser,
            request_timeout,
        }
    }
}

#[async_trait]
impl DomExtractor for ChromiumoxideDomExtractor {
    async fn snapshot(&self, target: &TargetId) -> Result<BrowserStateSummary, AppError> {
        capture_snapshot(&self.browser, target, self.request_timeout).await
    }

    async fn highlight(
        &self,
        target: &TargetId,
        options: &HighlightOptions,
    ) -> Result<Vec<u8>, AppError> {
        capture_with_overlay(&self.browser, target, options, self.request_timeout).await
    }
}

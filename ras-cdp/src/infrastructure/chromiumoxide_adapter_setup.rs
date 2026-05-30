//! Construction and [`std::fmt::Debug`] implementation for [`ChromiumoxideAdapter`].

use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::Browser;
use chromiumoxide::handler::HandlerConfig;
use futures::StreamExt;
use ras_errors::AppError;
use tokio::sync::Mutex;
use tracing::warn;
use url::Url;

use super::chromiumoxide_adapter::ChromiumoxideAdapter;

impl std::fmt::Debug for ChromiumoxideAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChromiumoxideAdapter")
            .field("cdp_url", &self.cdp_url)
            .finish()
    }
}

impl ChromiumoxideAdapter {
    #[must_use]
    pub fn browser_arc(&self) -> Arc<Mutex<Browser>> {
        Arc::clone(&self.browser)
    }

    pub async fn connect(cdp_url: Url, request_timeout: Duration) -> Result<Self, AppError> {
        let cfg = HandlerConfig::default();
        let (browser, mut handler) = Browser::connect_with_config(cdp_url.as_str(), cfg)
            .await
            .map_err(|e| AppError::BrowserDisconnected(format!("cdp connect: {e}")))?;
        tokio::spawn(async move {
            while let Some(ev) = handler.next().await {
                if let Err(e) = ev {
                    warn!(error = %e, "cdp handler event error");
                }
            }
        });
        Ok(Self {
            browser: Arc::new(Mutex::new(browser)),
            cdp_url,
            request_timeout,
        })
    }
}

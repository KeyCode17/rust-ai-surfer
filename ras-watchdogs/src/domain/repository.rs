use std::sync::Arc;

use async_trait::async_trait;
use ras_cdp::BrowserPort;
use ras_errors::AppError;
use ras_events::EventBus;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct WatchdogContext {
    pub browser: Arc<dyn BrowserPort>,
    pub events: Arc<dyn EventBus>,
    pub cancel: CancellationToken,
}

impl std::fmt::Debug for WatchdogContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WatchdogContext").finish()
    }
}

#[derive(Debug)]
pub struct WatchdogHandle {
    pub name: &'static str,
    pub cancel: CancellationToken,
}

#[async_trait]
pub trait Watchdog: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    async fn attach(&self, ctx: WatchdogContext) -> Result<WatchdogHandle, AppError>;
}

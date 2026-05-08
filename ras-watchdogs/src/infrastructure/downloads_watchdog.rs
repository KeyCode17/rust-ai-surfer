use async_trait::async_trait;
use ras_errors::AppError;
use ras_events::BrowserEvent;
use tokio::sync::Mutex;
use tracing::info;

use crate::domain::repository::{Watchdog, WatchdogContext, WatchdogHandle};

#[derive(Default)]
pub struct DownloadsWatchdog {
    downloaded: Mutex<Vec<String>>,
}

impl std::fmt::Debug for DownloadsWatchdog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadsWatchdog").finish()
    }
}

#[async_trait]
impl Watchdog for DownloadsWatchdog {
    fn name(&self) -> &'static str {
        "downloads"
    }

    async fn attach(&self, ctx: WatchdogContext) -> Result<WatchdogHandle, AppError> {
        let cancel = ctx.cancel.clone();
        let mut rx = ctx.events.subscribe();
        let inner_cancel = cancel.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = inner_cancel.cancelled() => break,
                    ev = rx.recv() => match ev {
                        Ok(BrowserEvent::DownloadCompleted { path }) => {
                            info!(%path, "download completed");
                        }
                        Err(_) => break,
                        _ => {}
                    }
                }
            }
        });
        Ok(WatchdogHandle {
            name: "downloads",
            cancel,
        })
    }
}

impl DownloadsWatchdog {
    pub async fn record(&self, path: impl Into<String>) {
        self.downloaded.lock().await.push(path.into());
    }

    pub async fn list(&self) -> Vec<String> {
        self.downloaded.lock().await.clone()
    }
}

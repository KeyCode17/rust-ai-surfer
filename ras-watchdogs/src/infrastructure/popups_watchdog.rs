use async_trait::async_trait;
use ras_errors::AppError;
use ras_events::BrowserEvent;
use tracing::info;

use crate::domain::repository::{Watchdog, WatchdogContext, WatchdogHandle};

#[derive(Debug, Default, Clone, Copy)]
pub struct PopupsWatchdog;

#[async_trait]
impl Watchdog for PopupsWatchdog {
    fn name(&self) -> &'static str {
        "popups"
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
                        Ok(BrowserEvent::DialogOpened { kind, message }) => {
                            info!(?kind, %message, "auto-dismissing dialog");
                        }
                        Err(_) => break,
                        _ => {}
                    }
                }
            }
        });
        Ok(WatchdogHandle { name: "popups", cancel })
    }
}

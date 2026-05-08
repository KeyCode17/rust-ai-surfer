use async_trait::async_trait;
use ras_errors::AppError;
use ras_events::BrowserEvent;
use tracing::warn;

use crate::domain::repository::{Watchdog, WatchdogContext, WatchdogHandle};

#[derive(Debug, Default, Clone, Copy)]
pub struct CrashWatchdog;

#[async_trait]
impl Watchdog for CrashWatchdog {
    fn name(&self) -> &'static str {
        "crash"
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
                        Ok(BrowserEvent::TargetCrashed { target }) => {
                            warn!(?target, "target crashed");
                        }
                        Ok(BrowserEvent::BrowserDisconnected { reason }) => {
                            warn!(%reason, "browser disconnected");
                        }
                        Err(_) => break,
                        _ => {}
                    }
                }
            }
        });
        Ok(WatchdogHandle { name: "crash", cancel })
    }
}

use std::net::IpAddr;

use async_trait::async_trait;
use ras_browser::AllowedDomains;
use ras_errors::AppError;
use ras_events::BrowserEvent;
use ras_types::DomainPattern;
use tracing::warn;
use url::Url;

use crate::domain::repository::{Watchdog, WatchdogContext, WatchdogHandle};

pub struct SecurityWatchdog {
    pub allowed: AllowedDomains,
    pub prohibited: Vec<DomainPattern>,
    pub block_ip_addresses: bool,
}

impl std::fmt::Debug for SecurityWatchdog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecurityWatchdog")
            .field("block_ip_addresses", &self.block_ip_addresses)
            .field("prohibited_count", &self.prohibited.len())
            .finish()
    }
}

impl SecurityWatchdog {
    #[must_use]
    pub fn permits(&self, url: &Url) -> bool {
        if self.block_ip_addresses && is_ip(url) {
            return false;
        }
        if self.prohibited.iter().any(|p| p.matches_url(url)) {
            return false;
        }
        if !self.allowed.is_empty() && !self.allowed.allows(url) {
            return false;
        }
        true
    }
}

fn is_ip(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    let trimmed = host.trim_start_matches('[').trim_end_matches(']');
    trimmed.parse::<IpAddr>().is_ok()
}

#[async_trait]
impl Watchdog for SecurityWatchdog {
    fn name(&self) -> &'static str {
        "security"
    }

    async fn attach(&self, ctx: WatchdogContext) -> Result<WatchdogHandle, AppError> {
        let cancel = ctx.cancel.clone();
        let mut rx = ctx.events.subscribe();
        let allowed = self.allowed.clone();
        let prohibited = self.prohibited.clone();
        let block_ip = self.block_ip_addresses;
        let inner = SecurityWatchdog {
            allowed,
            prohibited,
            block_ip_addresses: block_ip,
        };
        let task_cancel = cancel.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = task_cancel.cancelled() => break,
                    ev = rx.recv() => match ev {
                        Ok(BrowserEvent::NavigationStarted { url, .. })
                            if !inner.permits(&url) =>
                        {
                            warn!(%url, "navigation blocked by SecurityWatchdog");
                        }
                        Err(_) => break,
                        _ => {}
                    }
                }
            }
        });
        Ok(WatchdogHandle {
            name: "security",
            cancel,
        })
    }
}

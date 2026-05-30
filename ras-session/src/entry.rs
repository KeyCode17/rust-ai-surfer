//! Shared per-session record and the owner-erasing registry trait.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use ras_cdp::BrowserPort;
use ras_events::EventBus;
use ras_types::{ContextId, TargetId};

use crate::config::AgentSessionId;
use crate::spawn_params::SpawnParams;

/// Cloneable (via `Arc`) state for a single live session.
pub(crate) struct SessionEntry {
    pub(crate) id: AgentSessionId,
    pub(crate) ctx: ContextId,
    pub(crate) tab: TargetId,
    pub(crate) browser: Arc<dyn BrowserPort>,
    pub(crate) bus: Arc<dyn EventBus>,
    pub(crate) params: SpawnParams,
    pub(crate) running: AtomicBool,
    pub(crate) last_active: Mutex<Instant>,
}

impl SessionEntry {
    /// Update `last_active` to now, ignoring a poisoned lock.
    pub(crate) fn touch(&self) {
        if let Ok(mut g) = self.last_active.lock() {
            *g = Instant::now();
        }
    }
}

/// Owner-erasing removal hook so `SessionHandle` need not be generic.
#[async_trait]
pub(crate) trait SessionRegistry: Send + Sync {
    /// Remove a session id from the manager's maps.
    async fn forget(&self, id: &AgentSessionId);
}

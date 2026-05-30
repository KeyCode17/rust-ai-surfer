//! `SessionManager`: spawns, tracks, and reaps tenant sessions.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use ras_errors::AppError;
use ras_events::{BroadcastBus, EventBus};
use tokio::sync::Mutex as AsyncMutex;
use url::Url;

use crate::config::{AgentSessionId, OnFull, SessionConfig, SessionError};
use crate::entry::{SessionEntry, SessionRegistry};
use crate::handle::SessionHandle;
use crate::provider::BrowserProvider;
use crate::spawn_params::SpawnParams;

pub(crate) struct Inner<Owner> {
    pub(crate) sessions: HashMap<AgentSessionId, Arc<SessionEntry>>,
    pub(crate) by_owner: HashMap<Owner, AgentSessionId>,
}

/// Manages the pool of live tenant sessions for one `Owner` key type.
pub struct SessionManager<Owner> {
    provider: Arc<dyn BrowserProvider>,
    cfg: SessionConfig,
    inner: Arc<AsyncMutex<Inner<Owner>>>,
}

impl<Owner> SessionManager<Owner>
where
    Owner: Eq + Hash + Clone + Send + Sync + 'static,
{
    /// Create a manager and launch its background idle-reaper task.
    pub fn new(provider: Arc<dyn BrowserProvider>, cfg: SessionConfig) -> Self {
        let inner = Arc::new(AsyncMutex::new(Inner {
            sessions: HashMap::new(),
            by_owner: HashMap::new(),
        }));
        crate::manager_reaper::spawn_reaper(inner.clone(), provider.clone(), cfg.idle_timeout);
        Self {
            provider,
            cfg,
            inner,
        }
    }

    fn handle_for(&self, entry: Arc<SessionEntry>) -> SessionHandle {
        let registry: Arc<dyn SessionRegistry> = self.inner.clone();
        SessionHandle::new(entry, self.provider.clone(), registry)
    }

    /// Spawn a new session for `owner`, honoring capacity and reuse policy.
    pub async fn spawn(
        &self,
        owner: Owner,
        params: SpawnParams,
    ) -> Result<SessionHandle, SessionError> {
        let mut guard = self.inner.lock().await;

        if !self.cfg.allow_multi_per_owner
            && let Some(existing) = guard.by_owner.get(&owner)
            && let Some(entry) = guard.sessions.get(existing)
        {
            return Ok(self.handle_for(entry.clone()));
        }

        if guard.sessions.len() >= self.cfg.max_sessions {
            match self.cfg.on_full {
                OnFull::Reject => return Err(SessionError::AtCapacity),
                OnFull::EvictOldest => self.evict_oldest(&mut guard).await?,
            }
        }

        let (browser, ctx) = self.provider.acquire().await?;
        let url = Url::parse("about:blank")
            .map_err(|e| SessionError::Browser(AppError::ActionFailed(e.to_string())))?;
        let tab = browser.new_target_in(&ctx, &url).await?;
        let bus: Arc<dyn EventBus> = Arc::new(BroadcastBus::default());
        browser.attach_events(&tab, bus.clone()).await?;

        let entry = Arc::new(SessionEntry {
            id: AgentSessionId::next(),
            ctx,
            tab,
            browser,
            bus,
            params,
            running: AtomicBool::new(false),
            last_active: Mutex::new(Instant::now()),
        });
        guard.sessions.insert(entry.id.clone(), entry.clone());
        guard.by_owner.insert(owner, entry.id.clone());
        Ok(self.handle_for(entry))
    }

    async fn evict_oldest(&self, guard: &mut Inner<Owner>) -> Result<(), SessionError> {
        let victim = guard
            .sessions
            .values()
            .filter(|e| !e.running.load(std::sync::atomic::Ordering::SeqCst))
            .min_by_key(|e| {
                e.last_active
                    .lock()
                    .map(|g| *g)
                    .unwrap_or_else(|_| Instant::now())
            })
            .map(|e| (e.id.clone(), e.ctx.clone()));
        let Some((id, ctx)) = victim else {
            return Err(SessionError::AtCapacity);
        };
        self.provider.release(&ctx).await?;
        guard.sessions.remove(&id);
        guard.by_owner.retain(|_, v| v != &id);
        Ok(())
    }

    /// Look up a live session by id.
    pub async fn get(&self, id: &AgentSessionId) -> Option<SessionHandle> {
        let guard = self.inner.lock().await;
        guard.sessions.get(id).cloned().map(|e| self.handle_for(e))
    }

    /// List ids of all live sessions.
    pub async fn list(&self) -> Vec<AgentSessionId> {
        let guard = self.inner.lock().await;
        guard.sessions.keys().cloned().collect()
    }
}

#[async_trait]
impl<Owner> SessionRegistry for AsyncMutex<Inner<Owner>>
where
    Owner: Eq + Hash + Clone + Send + Sync + 'static,
{
    async fn forget(&self, id: &AgentSessionId) {
        let mut guard = self.lock().await;
        guard.sessions.remove(id);
        guard.by_owner.retain(|_, v| v != id);
    }
}

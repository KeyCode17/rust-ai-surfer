//! Background reaper that releases idle, non-running sessions.

use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;

use ras_types::ContextId;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::interval;

use crate::config::AgentSessionId;
use crate::manager::Inner;
use crate::provider::BrowserProvider;

const MAX_TICK: Duration = Duration::from_secs(30);

/// Launch the periodic reaper task. Ticks every `min(idle_timeout, 30s)`.
pub(crate) fn spawn_reaper<Owner>(
    inner: Arc<AsyncMutex<Inner<Owner>>>,
    provider: Arc<dyn BrowserProvider>,
    idle_timeout: Duration,
) where
    Owner: Eq + Hash + Clone + Send + Sync + 'static,
{
    let tick = idle_timeout.min(MAX_TICK).max(Duration::from_millis(10));
    tokio::spawn(async move {
        let mut ticker = interval(tick);
        loop {
            ticker.tick().await;
            reap_once(&inner, &provider, idle_timeout).await;
        }
    });
}

async fn reap_once<Owner>(
    inner: &Arc<AsyncMutex<Inner<Owner>>>,
    provider: &Arc<dyn BrowserProvider>,
    idle_timeout: Duration,
) where
    Owner: Eq + Hash + Clone + Send + Sync + 'static,
{
    let victims = collect_victims(inner, idle_timeout).await;
    for (id, ctx) in victims {
        let _ = provider.release(&ctx).await;
        let mut guard = inner.lock().await;
        guard.sessions.remove(&id);
        guard.by_owner.retain(|_, v| v != &id);
    }
}

async fn collect_victims<Owner>(
    inner: &Arc<AsyncMutex<Inner<Owner>>>,
    idle_timeout: Duration,
) -> Vec<(AgentSessionId, ContextId)>
where
    Owner: Eq + Hash + Clone + Send + Sync + 'static,
{
    let guard = inner.lock().await;
    guard
        .sessions
        .values()
        .filter(|e| !e.running.load(SeqCst))
        .filter(|e| {
            let elapsed = e
                .last_active
                .lock()
                .map(|g| g.elapsed())
                .unwrap_or_else(|_| Duration::ZERO);
            elapsed > idle_timeout
        })
        .map(|e| (e.id.clone(), e.ctx.clone()))
        .collect()
}

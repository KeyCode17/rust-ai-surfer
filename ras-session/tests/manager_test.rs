//! Integration tests for `SessionManager` and `SessionHandle`.

mod mock_browser;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use mock_browser::MockBrowser;
use ras_errors::AppError;
use ras_llm::{ChatMessage, ChatResponse, InvokeOptions, LlmClient, ProviderName};
use ras_session::provider::BrowserProvider;
use ras_session::{OnFull, SessionConfig, SessionManager, SpawnParams};
use ras_tools::domain::registry::ActionRegistry;
use ras_types::ContextId;

/// LLM stub; `ainvoke` is never called in these unit tests.
struct StubLlm;

#[async_trait]
impl LlmClient for StubLlm {
    fn provider(&self) -> ProviderName {
        ProviderName("stub".into())
    }

    fn model(&self) -> &str {
        "stub"
    }

    async fn ainvoke(
        &self,
        _messages: Vec<ChatMessage>,
        _options: InvokeOptions,
    ) -> Result<ChatResponse, AppError> {
        Err(AppError::ActionFailed("stub".into()))
    }
}

/// Provider backed by a single `MockBrowser`, recording releases and acquires.
struct MockProvider {
    browser: Arc<MockBrowser>,
    released: Arc<Mutex<Vec<ContextId>>>,
    acquires: Arc<AtomicU64>,
    counter: AtomicU64,
}

impl MockProvider {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            browser: Arc::new(MockBrowser::new("ctx")),
            released: Arc::new(Mutex::new(Vec::new())),
            acquires: Arc::new(AtomicU64::new(0)),
            counter: AtomicU64::new(0),
        })
    }
}

#[async_trait]
impl BrowserProvider for MockProvider {
    async fn acquire(&self) -> Result<(Arc<dyn ras_cdp::BrowserPort>, ContextId), AppError> {
        self.acquires.fetch_add(1, Ordering::SeqCst);
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        Ok((self.browser.clone(), ContextId(format!("ctx-{n}").into())))
    }

    async fn release(&self, ctx: &ContextId) -> Result<(), AppError> {
        self.released.lock().expect("lock").push(ctx.clone());
        Ok(())
    }
}

fn params() -> SpawnParams {
    SpawnParams {
        llm: Arc::new(StubLlm),
        registry: Arc::new(ActionRegistry::new()),
        dom_extractor: None,
        screenshot_sink: None,
        max_steps: 5,
    }
}

#[tokio::test]
async fn spawn_under_max_returns_ok_and_is_listed() {
    let provider = MockProvider::new();
    let mgr: SessionManager<String> = SessionManager::new(provider, SessionConfig::default());

    let handle = mgr.spawn("owner-a".into(), params()).await.expect("spawn");
    let id = handle.id();

    assert!(mgr.get(&id).await.is_some());
    assert_eq!(mgr.list().await.len(), 1);
}

#[tokio::test]
async fn reject_policy_returns_at_capacity_on_second_owner() {
    let provider = MockProvider::new();
    let cfg = SessionConfig {
        max_sessions: 1,
        on_full: OnFull::Reject,
        allow_multi_per_owner: true,
        ..SessionConfig::default()
    };
    let mgr: SessionManager<String> = SessionManager::new(provider, cfg);

    mgr.spawn("a".into(), params()).await.expect("first");
    let err = mgr.spawn("b".into(), params()).await;

    assert!(matches!(err, Err(ras_session::SessionError::AtCapacity)));
}

#[tokio::test]
async fn evict_oldest_releases_and_removes_first() {
    let provider = MockProvider::new();
    let released = provider.released.clone();
    let cfg = SessionConfig {
        max_sessions: 1,
        on_full: OnFull::EvictOldest,
        allow_multi_per_owner: true,
        ..SessionConfig::default()
    };
    let mgr: SessionManager<String> = SessionManager::new(provider, cfg);

    let first = mgr.spawn("a".into(), params()).await.expect("first");
    let first_id = first.id();
    mgr.spawn("b".into(), params()).await.expect("second");

    assert!(mgr.get(&first_id).await.is_none());
    assert_eq!(released.lock().expect("lock").len(), 1);
}

#[tokio::test]
async fn one_per_owner_reuses_session() {
    let provider = MockProvider::new();
    let acquires = provider.acquires.clone();
    let cfg = SessionConfig {
        allow_multi_per_owner: false,
        ..SessionConfig::default()
    };
    let mgr: SessionManager<String> = SessionManager::new(provider, cfg);

    let a = mgr.spawn("owner".into(), params()).await.expect("a");
    let b = mgr.spawn("owner".into(), params()).await.expect("b");

    assert_eq!(a.id(), b.id());
    assert_eq!(acquires.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn close_releases_ctx_and_forgets() {
    let provider = MockProvider::new();
    let released = provider.released.clone();
    let mgr: SessionManager<String> = SessionManager::new(provider, SessionConfig::default());

    let handle = mgr.spawn("a".into(), params()).await.expect("spawn");
    let id = handle.id();
    handle.close().await.expect("close");

    assert!(mgr.get(&id).await.is_none());
    assert_eq!(released.lock().expect("lock").len(), 1);
}

#[tokio::test]
async fn idle_reaper_releases_idle_session() {
    let provider = MockProvider::new();
    let released = provider.released.clone();
    let cfg = SessionConfig {
        idle_timeout: Duration::from_millis(50),
        ..SessionConfig::default()
    };
    let mgr: SessionManager<String> = SessionManager::new(provider, cfg);

    let handle = mgr.spawn("a".into(), params()).await.expect("spawn");
    let id = handle.id();

    tokio::time::sleep(Duration::from_millis(300)).await;

    assert!(mgr.get(&id).await.is_none());
    assert_eq!(released.lock().expect("lock").len(), 1);
}

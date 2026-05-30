//! Integration tests for `SharedBrowserProvider`.

mod mock_browser;

use std::sync::Arc;

use mock_browser::MockBrowser;
use ras_session::provider::{BrowserProvider, SharedBrowserProvider};

#[tokio::test]
async fn acquire_returns_context_id_from_create_context() {
    let mock = Arc::new(MockBrowser::new("ctx-1"));
    let provider = SharedBrowserProvider::new(mock.clone());

    let (_, ctx) = provider.acquire().await.expect("acquire");

    assert_eq!(ctx.0.as_str(), "ctx-1");
    let calls = mock.log.snapshot();
    assert_eq!(calls, vec!["create_context"]);
}

#[tokio::test]
async fn acquire_returns_same_browser_arc() {
    let mock = Arc::new(MockBrowser::new("ctx-2"));
    let provider = SharedBrowserProvider::new(mock.clone());

    let (browser_arc, _) = provider.acquire().await.expect("acquire");

    assert!(Arc::ptr_eq(
        &(browser_arc as Arc<dyn ras_cdp::BrowserPort>),
        &(mock.clone() as Arc<dyn ras_cdp::BrowserPort>),
    ));
}

#[tokio::test]
async fn release_calls_close_context_with_ctx() {
    let mock = Arc::new(MockBrowser::new("ctx-3"));
    let provider = SharedBrowserProvider::new(mock.clone());

    let (_, ctx) = provider.acquire().await.expect("acquire");
    provider.release(&ctx).await.expect("release");

    let calls = mock.log.snapshot();
    assert_eq!(calls, vec!["create_context", "close_context:ctx-3"]);
}

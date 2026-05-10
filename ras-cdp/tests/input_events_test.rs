use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::dom::{
    BackendNodeId as CdpBackendNodeId, GetDocumentParams, QuerySelectorParams,
};
use ras_cdp::BrowserPort;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use ras_types::BackendNodeId;
use url::Url;

const SETUP_HINT: &str = "
chromium --remote-debugging-port=9222 --user-data-dir=/tmp/test &
CDP_URL=http://127.0.0.1:9222 cargo test -p ras-cdp -- --ignored
";

fn cdp_url() -> Option<Url> {
    let raw = std::env::var("CDP_URL").ok()?;
    raw.parse().ok()
}

#[tokio::test]
#[ignore]
async fn click_node_resolves_real_backend_id() {
    let url = cdp_url().expect(SETUP_HINT);
    let adapter = ChromiumoxideAdapter::connect(url, Duration::from_secs(30))
        .await
        .expect("attach");
    let target = adapter.focused_target().await.expect("target");

    let html = "data:text/html,<button id=t onclick=\"document.title='clicked'\">x</button>";
    adapter
        .navigate(&target, &html.parse().expect("data url"))
        .await
        .expect("navigate");

    tokio::time::sleep(Duration::from_millis(500)).await;

    let backend_id = resolve_backend_id(&adapter, &target, "#t").await;
    adapter
        .click_node(&target, backend_id)
        .await
        .expect("click_node");

    tokio::time::sleep(Duration::from_millis(200)).await;
    let title = adapter
        .evaluate(&target, "document.title")
        .await
        .expect("eval title");
    assert_eq!(
        title.as_str(),
        Some("clicked"),
        "click_node must trigger the real onclick (got {title:?})"
    );
}

#[tokio::test]
#[ignore]
async fn type_text_mutates_input_value() {
    let url = cdp_url().expect(SETUP_HINT);
    let adapter = ChromiumoxideAdapter::connect(url, Duration::from_secs(30))
        .await
        .expect("attach");
    let target = adapter.focused_target().await.expect("target");

    let html = "data:text/html,<input id=t autofocus>";
    adapter
        .navigate(&target, &html.parse().expect("data url"))
        .await
        .expect("navigate");
    tokio::time::sleep(Duration::from_millis(500)).await;

    adapter
        .type_text(&target, "hello")
        .await
        .expect("type_text");

    let value = adapter
        .evaluate(&target, "document.getElementById('t').value")
        .await
        .expect("eval value");
    assert_eq!(
        value.as_str(),
        Some("hello"),
        "type_text must mutate input.value via Input.dispatchKeyEvent type=Char (got {value:?})"
    );
}

async fn resolve_backend_id(
    adapter: &ChromiumoxideAdapter,
    target: &ras_types::TargetId,
    selector: &str,
) -> BackendNodeId {
    use chromiumoxide::cdp::browser_protocol::dom::DescribeNodeParams;

    let browser = adapter.browser_arc();
    let guard = browser.lock().await;
    let pages = guard.pages().await.expect("pages");
    let page = pages
        .into_iter()
        .find(|p| p.target_id().inner().as_str() == target.0.as_str())
        .expect("page");
    drop(guard);

    let doc = page
        .execute(GetDocumentParams::default())
        .await
        .expect("getDocument");
    let root_node_id = doc.result.root.node_id;
    let q = page
        .execute(
            QuerySelectorParams::builder()
                .node_id(root_node_id)
                .selector(selector.to_string())
                .build()
                .expect("qs params"),
        )
        .await
        .expect("querySelector");
    let target_node_id = q.result.node_id;
    let described = page
        .execute(
            DescribeNodeParams::builder()
                .node_id(target_node_id)
                .build(),
        )
        .await
        .expect("describeNode");
    let cdp_backend: CdpBackendNodeId = described.result.node.backend_node_id;
    BackendNodeId(*cdp_backend.inner())
}

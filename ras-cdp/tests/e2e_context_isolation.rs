use std::time::Duration;

use ras_cdp::BrowserPort;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use ras_types::{ContextId, TargetId};
use url::Url;

fn cdp_url() -> Option<Url> {
    std::env::var("CDP_URL").ok()?.parse().ok()
}

fn test_url() -> Url {
    std::env::var("TEST_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8731/".to_string())
        .parse()
        .expect("TEST_URL parse")
}

async fn adapter() -> ChromiumoxideAdapter {
    let url = cdp_url().expect("set CDP_URL to a running chrome --remote-debugging-port");
    ChromiumoxideAdapter::connect(url, Duration::from_secs(30))
        .await
        .expect("connect")
}

async fn cookie_len(a: &ChromiumoxideAdapter, target: &TargetId) -> i64 {
    let v = a
        .evaluate(target, "document.cookie.length")
        .await
        .expect("eval cookie len");
    v.as_i64().unwrap_or(-1)
}

#[tokio::test]
#[ignore]
async fn contexts_isolate_cookies() {
    let a = adapter().await;
    let ctx_a: ContextId = a.create_context().await.expect("ctx a");
    let ctx_b: ContextId = a.create_context().await.expect("ctx b");
    let url = test_url();

    let tab_a = a.new_target_in(&ctx_a, &url).await.expect("tab a");
    let tab_b = a.new_target_in(&ctx_b, &url).await.expect("tab b");
    tokio::time::sleep(Duration::from_millis(900)).await;

    a.evaluate(&tab_a, "document.cookie = 'ras_iso=1; path=/'")
        .await
        .expect("set cookie in A");

    assert!(
        cookie_len(&a, &tab_a).await > 0,
        "A should see its own cookie"
    );
    assert_eq!(cookie_len(&a, &tab_b).await, 0, "B must NOT see A's cookie");

    a.close_context(&ctx_a).await.expect("close a");
    let after = a.list_targets_in(&ctx_a).await.unwrap_or_default();
    assert!(
        !after.contains(&tab_a),
        "closed context must no longer list its tab"
    );
    a.close_context(&ctx_b).await.expect("close b");
}

#[tokio::test]
#[ignore]
async fn list_targets_in_is_context_scoped() {
    let a = adapter().await;
    let ctx_a = a.create_context().await.expect("ctx a");
    let ctx_b = a.create_context().await.expect("ctx b");
    let url = test_url();
    let tab_a = a.new_target_in(&ctx_a, &url).await.expect("tab a");
    let tab_b = a.new_target_in(&ctx_b, &url).await.expect("tab b");

    let in_a = a.list_targets_in(&ctx_a).await.expect("list a");
    let in_b = a.list_targets_in(&ctx_b).await.expect("list b");

    assert!(
        in_a.contains(&tab_a) && !in_a.contains(&tab_b),
        "A lists only A's tab"
    );
    assert!(
        in_b.contains(&tab_b) && !in_b.contains(&tab_a),
        "B lists only B's tab"
    );

    a.close_context(&ctx_a).await.expect("close a");
    a.close_context(&ctx_b).await.expect("close b");
}

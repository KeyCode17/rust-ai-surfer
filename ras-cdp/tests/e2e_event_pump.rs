use std::sync::Arc;
use std::time::Duration;

use ras_cdp::BrowserPort;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use ras_events::{BroadcastBus, BrowserEvent, EventBus};
use url::Url;

fn cdp_url() -> Option<Url> {
    std::env::var("CDP_URL").ok()?.parse().ok()
}

#[tokio::test]
#[ignore]
async fn navigation_event_reaches_the_bus() {
    let url = cdp_url().expect("set CDP_URL");
    let a = ChromiumoxideAdapter::connect(url, Duration::from_secs(30))
        .await
        .expect("connect");

    let ctx = a.create_context().await.expect("ctx");
    let tab = a
        .new_target_in(&ctx, &"about:blank".parse().expect("about"))
        .await
        .expect("tab");

    let bus: Arc<dyn EventBus> = Arc::new(BroadcastBus::default());
    let mut rx = bus.subscribe();
    a.attach_events(&tab, bus.clone()).await.expect("attach");

    let dest: Url = std::env::var("TEST_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8732/".to_string())
        .parse()
        .expect("dest");
    a.navigate(&tab, &dest).await.expect("navigate");

    let got = tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            match rx.recv().await {
                Ok(BrowserEvent::NavigationCompleted { target, .. }) if target == tab => {
                    break true;
                }
                Ok(_) => continue,
                Err(_) => break false,
            }
        }
    })
    .await
    .unwrap_or(false);

    assert!(
        got,
        "expected a NavigationCompleted event for the tab on the bus"
    );

    a.close_context(&ctx).await.expect("close");
}

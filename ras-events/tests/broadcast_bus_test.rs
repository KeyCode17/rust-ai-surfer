use std::sync::Arc;

use ras_events::{BroadcastBus, BrowserEvent, EventBus};
use ras_types::TargetId;
use url::Url;

#[tokio::test]
async fn publish_and_subscribe_round_trip() {
    let bus = Arc::new(BroadcastBus::new(16));
    let mut rx = bus.subscribe();
    let url = Url::parse("https://example.com/").expect("url");
    bus.publish(BrowserEvent::NavigationStarted {
        target: TargetId("t1".into()),
        url: url.clone(),
    })
    .await
    .expect("publish");
    let received = rx.recv().await.expect("recv");
    match received {
        BrowserEvent::NavigationStarted { target, url: u } => {
            assert_eq!(target.0.as_str(), "t1");
            assert_eq!(u, url);
        }
        _ => panic!("wrong event"),
    }
}

#[tokio::test]
async fn multiple_subscribers_each_receive() {
    let bus = Arc::new(BroadcastBus::new(16));
    let mut a = bus.subscribe();
    let mut b = bus.subscribe();
    bus.publish(BrowserEvent::DialogDismissed).await.expect("publish");
    let _ = a.recv().await.expect("a recv");
    let _ = b.recv().await.expect("b recv");
}

use std::sync::Arc;
use std::time::Duration;

use ras_cdp::BrowserPort;
use ras_cdp::infrastructure::chromiumoxide_adapter::ChromiumoxideAdapter;
use ras_dom::{ChromiumoxideDomExtractor, DomExtractor};
use ras_events::BroadcastBus;
use ras_tools::domain::action::ToolHandler;
use ras_tools::domain::registry::ToolContext;
use ras_tools::infrastructure::builtin::click::ClickElementAction;
use ras_tools::infrastructure::builtin::type_text::TypeTextAction;
use serde_json::json;
use url::Url;

const SETUP_HINT: &str = "
chromium --remote-debugging-port=9222 --user-data-dir=/tmp/test &
CDP_URL=http://127.0.0.1:9222 cargo test -p ras-tools --test e2e_grounding -- --ignored
";

fn cdp_url() -> Option<Url> {
    std::env::var("CDP_URL").ok()?.parse().ok()
}

#[tokio::test]
#[ignore]
async fn click_and_type_resolve_through_clickable_map() {
    let url = cdp_url().expect(SETUP_HINT);
    let adapter = ChromiumoxideAdapter::connect(url, Duration::from_secs(30))
        .await
        .expect("connect");
    let browser_arc = adapter.browser_arc();
    let browser: Arc<dyn BrowserPort> = Arc::new(adapter);
    let extractor: Arc<dyn DomExtractor> = Arc::new(ChromiumoxideDomExtractor::new(
        browser_arc,
        Duration::from_secs(30),
    ));

    let html = "data:text/html,<input id=u autofocus><input id=p type=password>\
        <button id=b onclick=\"document.title='clicked'\">Go</button>";
    let target = browser
        .create_target(&html.parse().expect("data url"))
        .await
        .expect("create target");
    tokio::time::sleep(Duration::from_millis(700)).await;

    let summary = extractor.snapshot(&target).await.expect("snapshot");
    assert!(
        !summary.clickables.is_empty(),
        "snapshot must produce clickables for the fixture page"
    );
    let clickables = Arc::new(summary.clickables.clone());

    let input_idx = clickables
        .iter()
        .find(|c| c.tag.eq_ignore_ascii_case("input"))
        .map(|c| c.index)
        .expect("input clickable");
    let button_idx = clickables
        .iter()
        .find(|c| c.tag.eq_ignore_ascii_case("button"))
        .map(|c| c.index)
        .expect("button clickable");

    let ctx = ToolContext {
        target: Some(target.clone()),
        browser: browser.clone(),
        events: Arc::new(BroadcastBus::default()),
        page_url: None,
        available_files: Vec::new(),
        clickables: clickables.clone(),
    };

    ClickElementAction
        .execute(json!({"index": input_idx}), ctx.clone())
        .await
        .expect("click input");
    tokio::time::sleep(Duration::from_millis(150)).await;

    TypeTextAction
        .execute(json!({"text": "hello-world"}), ctx.clone())
        .await
        .expect("type");
    tokio::time::sleep(Duration::from_millis(150)).await;

    let value = browser
        .evaluate(&target, "document.getElementById('u').value")
        .await
        .expect("eval value");
    assert_eq!(
        value.as_str(),
        Some("hello-world"),
        "type_text must mutate input.value (got {value:?})"
    );

    ClickElementAction
        .execute(json!({"index": button_idx}), ctx)
        .await
        .expect("click button");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let title = browser
        .evaluate(&target, "document.title")
        .await
        .expect("eval title");
    assert_eq!(
        title.as_str(),
        Some("clicked"),
        "button click must trigger onclick (got {title:?})"
    );
}

/// Live check that the clickable map names elements from descendant text, onclick handlers,
/// and FontAwesome icon classes, and that `role` overrides the displayed tag.
#[tokio::test]
#[ignore]
async fn clickable_names_resolve_text_onclick_icon_and_role() {
    let url = cdp_url().expect(SETUP_HINT);
    let adapter = ChromiumoxideAdapter::connect(url, Duration::from_secs(30))
        .await
        .expect("connect");
    let browser_arc = adapter.browser_arc();
    let browser: Arc<dyn BrowserPort> = Arc::new(adapter);
    let extractor: Arc<dyn DomExtractor> = Arc::new(ChromiumoxideDomExtractor::new(
        browser_arc,
        Duration::from_secs(30),
    ));

    let html = concat!(
        "data:text/html,",
        "<ul>",
        "<li><a href='/private/dashboard-statistic'>Dashboard</a></li>",
        "<li><a href='/sub'><span class='menu'>Home</span></a></li>",
        "</ul>",
        "<i role='button' class='fas fa-eye text-secondary' onclick=\"viewBankAccount('0')\"></i>",
        "<i role='button' class='fas fa-plus-circle text-primary' ",
        "onclick=\"addBankAccountModal('0','135','BANL BAHLIL')\"></i>",
    );
    let target = browser
        .create_target(&html.parse().expect("data url"))
        .await
        .expect("create target");
    tokio::time::sleep(Duration::from_millis(700)).await;

    let summary = extractor.snapshot(&target).await.expect("snapshot");
    let names: Vec<(String, Option<String>)> = summary
        .clickables
        .iter()
        .map(|c| (c.tag.clone(), c.ax_name.clone()))
        .collect();
    let has = |name: &str| {
        summary
            .clickables
            .iter()
            .any(|c| c.ax_name.as_deref() == Some(name))
    };

    // 1. anchor text — direct child and nested <span>.
    assert!(has("Dashboard"), "direct anchor text; got {names:?}");
    assert!(has("Home"), "nested span text; got {names:?}");

    // 2. onclick handler name wins, and role=button overrides the <i> tag.
    let eye = summary
        .clickables
        .iter()
        .find(|c| c.ax_name.as_deref() == Some("view bank account"))
        .unwrap_or_else(|| panic!("onclick-derived name; got {names:?}"));
    assert_eq!(eye.tag, "button", "role overrides tag; got {names:?}");
    assert!(
        has("add bank account modal"),
        "second handler; got {names:?}"
    );
}

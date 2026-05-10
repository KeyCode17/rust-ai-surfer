use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ras_cdp::BrowserPort;
use ras_cdp::domain::repository::ScreenshotFormat;
use ras_cdp::domain::viewport::Viewport;
use ras_dom::ClickableElement;
use ras_dom::domain::node::BoundingBox;
use ras_errors::AppError;
use ras_events::BroadcastBus;
use ras_tools::domain::action::ToolHandler;
use ras_tools::domain::registry::ToolContext;
use ras_tools::infrastructure::builtin::click::ClickElementAction;
use ras_types::{BackendNodeId, TargetId};
use serde_json::json;
use url::Url;

#[derive(Default)]
struct CaptureBrowser {
    clicked: Mutex<Vec<BackendNodeId>>,
}

#[async_trait]
impl BrowserPort for CaptureBrowser {
    async fn cdp_url(&self) -> Result<Url, AppError> {
        unimplemented!()
    }
    async fn list_targets(&self) -> Result<Vec<TargetId>, AppError> {
        unimplemented!()
    }
    async fn focused_target(&self) -> Result<TargetId, AppError> {
        Ok(TargetId("t-1".into()))
    }
    async fn navigate(&self, _t: &TargetId, _u: &Url) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn evaluate(&self, _t: &TargetId, _e: &str) -> Result<serde_json::Value, AppError> {
        unimplemented!()
    }
    async fn click_at(&self, _t: &TargetId, _x: i32, _y: i32) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn click_node(&self, _t: &TargetId, n: BackendNodeId) -> Result<(), AppError> {
        self.clicked.lock().expect("lock").push(n);
        Ok(())
    }
    async fn type_text(&self, _t: &TargetId, _s: &str) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn screenshot(&self, _t: &TargetId, _f: ScreenshotFormat) -> Result<Vec<u8>, AppError> {
        unimplemented!()
    }
    async fn set_viewport(&self, _t: &TargetId, _v: Viewport) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn close_target(&self, _t: &TargetId) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn create_target(&self, _u: &Url) -> Result<TargetId, AppError> {
        unimplemented!()
    }
}

fn clickable(index: u32, backend: i64) -> ClickableElement {
    ClickableElement {
        index,
        backend_node_id: BackendNodeId(backend),
        bbox: BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        },
        xpath: format!("/html/body/*[{index}]"),
        stable_hash: format!("h{index}"),
        ax_name: None,
        tag: "button".into(),
        label: None,
    }
}

fn ctx_with(browser: Arc<CaptureBrowser>, clickables: Vec<ClickableElement>) -> ToolContext {
    ToolContext {
        browser,
        events: Arc::new(BroadcastBus::default()),
        page_url: None,
        available_files: Vec::new(),
        clickables: Arc::new(clickables),
    }
}

#[tokio::test]
async fn click_handler_translates_list_index_to_backend_node_id() {
    let browser = Arc::new(CaptureBrowser::default());
    let clickables = vec![
        clickable(0, 11111),
        clickable(1, 98765),
        clickable(2, 22222),
    ];
    let ctx = ctx_with(browser.clone(), clickables);
    let action = ClickElementAction;
    action
        .execute(json!({"index": 1}), ctx)
        .await
        .expect("click ok");
    let calls = browser.clicked.lock().expect("lock");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], BackendNodeId(98765));
}

#[tokio::test]
async fn click_handler_errors_on_unknown_index() {
    let browser = Arc::new(CaptureBrowser::default());
    let ctx = ctx_with(browser.clone(), Vec::new());
    let action = ClickElementAction;
    let err = action
        .execute(json!({"index": 5}), ctx)
        .await
        .expect_err("must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("no clickable with index 5"),
        "wrong error: {msg}"
    );
    assert!(
        browser.clicked.lock().expect("lock").is_empty(),
        "click_node must not be called"
    );
}

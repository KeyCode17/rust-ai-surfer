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
use ras_tools::infrastructure::builtin::select_option::SelectOptionAction;
use ras_types::{BackendNodeId, TargetId};
use serde_json::json;
use url::Url;

struct ScriptedBrowser {
    eval_calls: Mutex<Vec<String>>,
    result: serde_json::Value,
}

impl ScriptedBrowser {
    fn returning(result: serde_json::Value) -> Self {
        Self {
            eval_calls: Mutex::new(Vec::new()),
            result,
        }
    }
}

#[async_trait]
impl BrowserPort for ScriptedBrowser {
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
    async fn evaluate(&self, _t: &TargetId, e: &str) -> Result<serde_json::Value, AppError> {
        self.eval_calls.lock().expect("lock").push(e.to_string());
        Ok(self.result.clone())
    }
    async fn click_at(&self, _t: &TargetId, _x: i32, _y: i32) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn click_node(&self, _t: &TargetId, _n: BackendNodeId) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn mouse_down(&self, _t: &TargetId, _x: i32, _y: i32) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn mouse_up(&self, _t: &TargetId, _x: i32, _y: i32) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn mouse_hold(&self, _t: &TargetId, _x: i32, _y: i32, _ms: u64) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn mouse_move(&self, _t: &TargetId, _x: i32, _y: i32, _b: i64) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn block_urls(&self, _t: &TargetId, _patterns: Vec<String>) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn clear_cookies(&self, _t: &TargetId, _origin: &str) -> Result<(), AppError> {
        unimplemented!()
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

fn clickable(index: u32, backend: i64, xpath: &str, tag: &str) -> ClickableElement {
    ClickableElement {
        index,
        backend_node_id: BackendNodeId(backend),
        bbox: BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        },
        xpath: xpath.into(),
        stable_hash: format!("h{index}"),
        ax_name: None,
        tag: tag.into(),
        label: None,
    }
}

fn ctx_with(browser: Arc<ScriptedBrowser>, clickables: Vec<ClickableElement>) -> ToolContext {
    ToolContext {
        target: Some(TargetId("t-1".into())),
        browser,
        events: Arc::new(BroadcastBus::default()),
        page_url: None,
        available_files: Vec::new(),
        clickables: Arc::new(clickables),
    }
}

#[tokio::test]
async fn native_select_set_value_succeeds() {
    let browser = Arc::new(ScriptedBrowser::returning(json!({
        "ok": true, "kind": "select", "selected": "BCA Bank"
    })));
    let clickables = vec![clickable(0, 11111, "/html/body/form/select[1]", "select")];
    let ctx = ctx_with(browser.clone(), clickables);

    let res = SelectOptionAction
        .execute(json!({"index": 0, "text": "BCA Bank"}), ctx)
        .await
        .expect("select ok");
    assert!(
        res.extracted_content
            .as_deref()
            .unwrap_or_default()
            .contains("BCA Bank"),
        "result should name the selected option"
    );

    let calls = browser.eval_calls.lock().expect("lock");
    assert_eq!(calls.len(), 1, "evaluate called once");
    let js = &calls[0];
    assert!(
        js.contains("/html/body/form/select[1]"),
        "JS must target the resolved element xpath"
    );
    assert!(js.contains("BCA Bank"), "JS must carry the requested text");
    assert!(
        js.contains("SELECT"),
        "JS must branch on native select tagName"
    );
}

#[tokio::test]
async fn no_matching_option_errors_with_available_options() {
    let browser = Arc::new(ScriptedBrowser::returning(json!({
        "ok": false, "kind": "select", "options": ["BNI", "BRI"]
    })));
    let clickables = vec![clickable(3, 222, "/html/body/select", "select")];
    let ctx = ctx_with(browser.clone(), clickables);

    let err = SelectOptionAction
        .execute(json!({"index": 3, "text": "Nonexistent"}), ctx)
        .await
        .expect_err("must fail when no option matches");
    let msg = err.to_string();
    assert!(
        msg.contains("Nonexistent"),
        "error names the wanted option: {msg}"
    );
    assert!(
        msg.contains("BNI") && msg.contains("BRI"),
        "error lists available: {msg}"
    );
}

#[tokio::test]
async fn unknown_index_errors_before_evaluate() {
    let browser = Arc::new(ScriptedBrowser::returning(json!({"ok": true})));
    let ctx = ctx_with(browser.clone(), Vec::new());

    let err = SelectOptionAction
        .execute(json!({"index": 9, "text": "X"}), ctx)
        .await
        .expect_err("must fail on unknown index");
    assert!(
        err.to_string().contains("no clickable with index 9"),
        "wrong error: {err}"
    );
    assert!(
        browser.eval_calls.lock().expect("lock").is_empty(),
        "evaluate must not run for an unresolved index"
    );
}

#[tokio::test]
async fn omitted_index_searches_the_document() {
    let browser = Arc::new(ScriptedBrowser::returning(json!({
        "ok": true, "kind": "custom", "selected": "Savings"
    })));
    let ctx = ctx_with(browser.clone(), Vec::new());

    SelectOptionAction
        .execute(json!({"text": "Savings"}), ctx)
        .await
        .expect("select ok without an index");

    let calls = browser.eval_calls.lock().expect("lock");
    assert_eq!(calls.len(), 1);
    assert!(
        !calls[0].contains("document.evaluate("),
        "without an index the JS must not resolve an xpath; it searches the document"
    );
}

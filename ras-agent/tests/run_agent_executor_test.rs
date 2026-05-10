use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use ras_agent::application::run_agent::RunAgent;
use ras_cdp::{BrowserPort, ScreenshotFormat, Viewport};
use ras_errors::AppError;
use ras_events::{BroadcastBus, EventBus};
use ras_llm::{
    ChatMessage, ChatResponse, FinishReason, InvokeOptions, LlmClient, ProviderName, Usage,
};
use ras_tools::domain::registry::ActionRegistry;
use ras_tools::infrastructure::builtin::register::register_default_actions;
use ras_types::{BackendNodeId, TargetId};
use url::Url;

struct MockBrowser {
    pub navigations: Mutex<Vec<Url>>,
    pub target: TargetId,
}

impl Default for MockBrowser {
    fn default() -> Self {
        Self {
            navigations: Mutex::new(Vec::new()),
            target: TargetId("mock-target".into()),
        }
    }
}

#[async_trait]
impl BrowserPort for MockBrowser {
    async fn cdp_url(&self) -> Result<Url, AppError> {
        Ok(Url::parse("ws://mock/devtools").expect("test invariant"))
    }
    async fn list_targets(&self) -> Result<Vec<TargetId>, AppError> {
        Ok(vec![self.target.clone()])
    }
    async fn focused_target(&self) -> Result<TargetId, AppError> {
        Ok(self.target.clone())
    }
    async fn navigate(&self, _t: &TargetId, url: &Url) -> Result<(), AppError> {
        self.navigations
            .lock()
            .expect("test invariant")
            .push(url.clone());
        Ok(())
    }
    async fn evaluate(
        &self,
        _t: &TargetId,
        expression: &str,
    ) -> Result<serde_json::Value, AppError> {
        if expression == "location.href" {
            Ok(serde_json::Value::String("https://example.com/".into()))
        } else {
            Ok(serde_json::Value::Null)
        }
    }
    async fn click_at(&self, _t: &TargetId, _x: i32, _y: i32) -> Result<(), AppError> {
        Ok(())
    }
    async fn click_node(&self, _t: &TargetId, _n: BackendNodeId) -> Result<(), AppError> {
        Ok(())
    }
    async fn type_text(&self, _t: &TargetId, _text: &str) -> Result<(), AppError> {
        Ok(())
    }
    async fn screenshot(
        &self,
        _t: &TargetId,
        _format: ScreenshotFormat,
    ) -> Result<Vec<u8>, AppError> {
        Ok(vec![0xFF, 0xD8, 0xFF])
    }
    async fn set_viewport(&self, _t: &TargetId, _v: Viewport) -> Result<(), AppError> {
        Ok(())
    }
    async fn close_target(&self, _t: &TargetId) -> Result<(), AppError> {
        Ok(())
    }
    async fn create_target(&self, _url: &Url) -> Result<TargetId, AppError> {
        Ok(self.target.clone())
    }
}

struct ScriptedLlm {
    responses: Mutex<Vec<String>>,
}

impl ScriptedLlm {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().rev().map(String::from).collect()),
        }
    }
}

#[async_trait]
impl LlmClient for ScriptedLlm {
    fn provider(&self) -> ProviderName {
        ProviderName("scripted".into())
    }
    fn model(&self) -> &str {
        "scripted-1"
    }
    async fn ainvoke(
        &self,
        _messages: Vec<ChatMessage>,
        _options: InvokeOptions,
    ) -> Result<ChatResponse, AppError> {
        let next = self
            .responses
            .lock()
            .expect("test invariant")
            .pop()
            .unwrap_or_else(|| r#"{"current_state":{"evaluation_previous_goal":"","memory":"","next_goal":""},"action":[]}"#.into());
        Ok(ChatResponse {
            content: Some(next),
            tool_calls: vec![],
            usage: Usage::default(),
            model: "scripted-1".into(),
            finish_reason: FinishReason::Stop,
        })
    }
}

#[tokio::test]
async fn agent_navigates_then_completes() {
    let mut registry = ActionRegistry::new();
    register_default_actions(&mut registry).expect("test invariant");
    let registry = Arc::new(registry);

    let browser = Arc::new(MockBrowser::default());
    let browser_port: Arc<dyn BrowserPort> = browser.clone();
    let events: Arc<dyn EventBus> = Arc::new(BroadcastBus::new(16));

    let plan_step1 = r#"{"current_state":{"evaluation_previous_goal":"start","memory":"","next_goal":"open example"},"action":[{"name":"navigate","parameters":{"url":"https://example.com/"}}]}"#;
    let plan_step2 = r#"{"current_state":{"evaluation_previous_goal":"navigated","memory":"on example.com","next_goal":"finish"},"action":[{"name":"done","parameters":{"text":"Done."}}]}"#;
    let llm: Arc<dyn LlmClient> = Arc::new(ScriptedLlm::new(vec![plan_step1, plan_step2]));

    let history = RunAgent::new("open example.com", llm, registry, browser_port, events)
        .with_max_steps(5)
        .execute()
        .await
        .expect("agent run");

    let navs = browser.navigations.lock().expect("test invariant").clone();
    assert_eq!(navs.len(), 1, "navigate should be called once");
    assert_eq!(navs[0].as_str(), "https://example.com/");

    let final_text = history.final_result().expect("final result present");
    assert!(final_text.contains("Done"), "final result: {final_text}");

    // Drill into the underlying steps via the public iterator.
    let mut total_steps = 0usize;
    let mut saw_done = false;
    for hist in &history.histories {
        for step in &hist.steps {
            total_steps += 1;
            if step.results.iter().any(|r| r.is_done) {
                saw_done = true;
            }
        }
    }
    assert_eq!(total_steps, 2, "expected 2 steps, got {total_steps}");
    assert!(saw_done, "expected one step to terminate via done action");
}

#[tokio::test]
async fn agent_aborts_after_consecutive_empty_actions() {
    let mut registry = ActionRegistry::new();
    register_default_actions(&mut registry).expect("test invariant");
    let registry = Arc::new(registry);

    let browser: Arc<dyn BrowserPort> = Arc::new(MockBrowser::default());
    let events: Arc<dyn EventBus> = Arc::new(BroadcastBus::new(16));

    let empty = r#"{"current_state":{"evaluation_previous_goal":"","memory":"","next_goal":""},"action":[]}"#;
    let llm: Arc<dyn LlmClient> = Arc::new(ScriptedLlm::new(vec![empty, empty, empty]));

    let history = RunAgent::new("noop", llm, registry, browser, events)
        .with_max_steps(10)
        .execute()
        .await
        .expect("agent run");

    let mut total = 0;
    for h in &history.histories {
        total += h.steps.len();
    }
    assert!(
        total <= 2,
        "agent should bail after 2 empty streaks, ran {total} steps"
    );
}

#[tokio::test]
async fn agent_recovers_from_markdown_fenced_response() {
    let mut registry = ActionRegistry::new();
    register_default_actions(&mut registry).expect("test invariant");
    let registry = Arc::new(registry);

    let browser = Arc::new(MockBrowser::default());
    let browser_port: Arc<dyn BrowserPort> = browser.clone();
    let events: Arc<dyn EventBus> = Arc::new(BroadcastBus::new(16));

    let fenced_step1 = "```json\n{\"current_state\":{\"evaluation_previous_goal\":\"\",\"memory\":\"\",\"next_goal\":\"go\"},\"action\":[{\"name\":\"navigate\",\"parameters\":{\"url\":\"https://example.com/\"}}]}\n```";
    let plain_step2 = r#"{"current_state":{"evaluation_previous_goal":"navigated","memory":"","next_goal":"finish"},"action":[{"name":"done","parameters":{"text":"Done."}}]}"#;
    let llm: Arc<dyn LlmClient> = Arc::new(ScriptedLlm::new(vec![fenced_step1, plain_step2]));

    let history = RunAgent::new("fenced task", llm, registry, browser_port, events)
        .with_max_steps(5)
        .execute()
        .await
        .expect("agent run");

    let first_step = history
        .histories
        .first()
        .and_then(|h| h.steps.first())
        .expect("at least one step");
    assert!(
        !first_step.output.action.is_empty(),
        "fenced JSON must parse into a non-empty action list"
    );

    let navs = browser.navigations.lock().expect("test invariant").clone();
    assert_eq!(navs.len(), 1, "navigate should reach browser through fence");
    assert_eq!(navs[0].as_str(), "https://example.com/");
}

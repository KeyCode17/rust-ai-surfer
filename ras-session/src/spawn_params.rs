//! Parameters carried into each spawned agent session.

use std::sync::Arc;

use ras_agent::StepScreenshotSink;
use ras_dom::DomExtractor;
use ras_llm::LlmClient;
use ras_tools::domain::registry::ActionRegistry;

/// Per-session wiring used to build a `RunAgent` when a task runs.
pub struct SpawnParams {
    /// LLM client driving the agent.
    pub llm: Arc<dyn LlmClient>,
    /// Action registry exposed to the agent.
    pub registry: Arc<ActionRegistry>,
    /// Optional DOM extractor for richer page state.
    pub dom_extractor: Option<Arc<dyn DomExtractor>>,
    /// Optional destination for one screenshot per agent step.
    pub screenshot_sink: Option<Arc<dyn StepScreenshotSink>>,
    /// Maximum agent steps per task.
    pub max_steps: u32,
}

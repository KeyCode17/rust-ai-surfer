use ras_types::AgentId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetrySource {
    Agent,
    Cli,
    McpServer,
    McpClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TelemetryEvent {
    AgentStarted {
        agent: AgentId,
        model: String,
    },
    AgentStepCompleted {
        agent: AgentId,
        step: u32,
        success: bool,
    },
    AgentFinished {
        agent: AgentId,
        total_steps: u32,
        success: bool,
    },
    Error {
        source: TelemetrySource,
        message: String,
    },
}

use ras_types::ActionName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBrain {
    pub evaluation_previous_goal: String,
    pub memory: String,
    pub next_goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub step: u32,
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub current_state: AgentBrain,
    pub action: Vec<ActionInvocation>,
    pub plan: Option<Vec<PlanItem>>,
    pub current_plan_item: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInvocation {
    pub name: ActionName,
    pub parameters: serde_json::Value,
}

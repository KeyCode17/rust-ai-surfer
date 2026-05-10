use ras_types::ActionName;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBrain {
    #[serde(default, deserialize_with = "null_to_empty")]
    pub evaluation_previous_goal: String,
    #[serde(default, deserialize_with = "null_to_empty")]
    pub memory: String,
    #[serde(default, deserialize_with = "null_to_empty")]
    pub next_goal: String,
}

fn null_to_empty<'de, D>(de: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(de)?.unwrap_or_default())
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

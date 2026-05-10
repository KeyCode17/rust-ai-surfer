use chrono::{DateTime, Utc};
use ras_dom::BrowserStateSummary;
use ras_types::{ActionResult, AgentId, StepId};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::agent_output::AgentOutput;
use crate::domain::step_metadata::StepMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub step: StepId,
    pub started_at: DateTime<Utc>,
    pub url: Option<Url>,
    pub output: AgentOutput,
    pub results: Vec<ActionResult>,
    pub metadata: StepMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<BrowserStateSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHistory {
    pub agent: AgentId,
    pub task: String,
    pub steps: Vec<StepRecord>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentHistoryList {
    pub histories: Vec<AgentHistory>,
}

impl AgentHistoryList {
    pub fn push(&mut self, history: AgentHistory) {
        self.histories.push(history);
    }

    #[must_use]
    pub fn last(&self) -> Option<&AgentHistory> {
        self.histories.last()
    }

    #[must_use]
    pub fn final_result(&self) -> Option<&str> {
        self.histories
            .iter()
            .flat_map(|h| h.steps.iter())
            .flat_map(|s| s.results.iter())
            .rfind(|r| r.is_done)
            .and_then(|r| r.extracted_content.as_deref())
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        self.final_result().is_some()
    }
}

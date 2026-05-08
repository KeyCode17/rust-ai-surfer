use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Success,
    PartialSuccess,
    Failure,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgementResult {
    pub verdict: Verdict,
    pub reasoning: String,
    pub confidence: f32,
    pub flagged_steps: Vec<u32>,
}

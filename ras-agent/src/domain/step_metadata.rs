use std::time::Duration;

use ras_llm::Usage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepMetadata {
    pub duration_ms: u64,
    pub step_interval_ms: Option<u64>,
    pub usage: Usage,
    pub model: Option<String>,
    pub fallback_used: bool,
}

impl StepMetadata {
    #[must_use]
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.duration_ms)
    }
}

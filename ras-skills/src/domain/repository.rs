use async_trait::async_trait;
use ras_errors::AppError;
use serde::{Deserialize, Serialize};

use crate::domain::skill::{SkillDefinition, SkillId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionRequest {
    pub skill: SkillId,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    pub skill: SkillId,
    pub output: serde_json::Value,
    pub duration_ms: u64,
}

#[async_trait]
pub trait SkillsPort: Send + Sync + 'static {
    async fn list(&self) -> Result<Vec<SkillDefinition>, AppError>;
    async fn get(&self, id: &SkillId) -> Result<SkillDefinition, AppError>;
    async fn execute(
        &self,
        request: SkillExecutionRequest,
    ) -> Result<SkillExecutionResult, AppError>;
}

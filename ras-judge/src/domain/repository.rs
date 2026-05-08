use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::judgement::JudgementResult;

#[async_trait]
pub trait JudgePort: Send + Sync + 'static {
    async fn evaluate(
        &self,
        task: &str,
        screenshot_b64: Option<&str>,
        history_json: serde_json::Value,
    ) -> Result<JudgementResult, AppError>;
}

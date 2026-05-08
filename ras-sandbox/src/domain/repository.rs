use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::execution::{ExecutionRequest, ExecutionResult};

#[async_trait]
pub trait SandboxRunner: Send + Sync + 'static {
    async fn run(&self, request: ExecutionRequest) -> Result<ExecutionResult, AppError>;
}

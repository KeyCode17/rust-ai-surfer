use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::tool::{McpToolDefinition, McpToolResult};

#[async_trait]
pub trait McpClientPort: Send + Sync + 'static {
    async fn list_tools(&self) -> Result<Vec<McpToolDefinition>, AppError>;
    async fn invoke(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolResult, AppError>;
    async fn disconnect(&self) -> Result<(), AppError>;
}

#[async_trait]
pub trait McpServerPort: Send + Sync + 'static {
    async fn serve(&self) -> Result<(), AppError>;
    async fn shutdown(&self) -> Result<(), AppError>;
}

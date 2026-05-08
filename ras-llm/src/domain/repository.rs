use async_trait::async_trait;
use ras_errors::AppError;
use serde::{Deserialize, Serialize};

use crate::domain::message::{ChatMessage, ToolCall};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvokeOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Vec<String>,
    pub tools: Vec<serde_json::Value>,
    pub response_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
    pub model: String,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    #[default]
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_input_tokens: u32,
    pub cache_creation_input_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderName(pub String);

#[async_trait]
pub trait LlmClient: Send + Sync + 'static {
    fn provider(&self) -> ProviderName;
    fn model(&self) -> &str;
    async fn ainvoke(
        &self,
        messages: Vec<ChatMessage>,
        options: InvokeOptions,
    ) -> Result<ChatResponse, AppError>;
}

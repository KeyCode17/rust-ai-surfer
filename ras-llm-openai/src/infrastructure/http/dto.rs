use ras_llm::{
    AssistantMessage, ChatMessage, ChatResponse, ContentPart, FinishReason, ToolCall, Usage,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessageDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatMessageDto {
    pub role: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<UsageDto>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChoiceMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChoiceMessage {
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<ToolCallDto>,
}

#[derive(Debug, Deserialize)]
pub struct ToolCallDto {
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    pub function: FunctionCallDto,
}

#[derive(Debug, Deserialize)]
pub struct FunctionCallDto {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UsageDto {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    #[serde(default)]
    pub total_tokens: u32,
}

#[must_use]
pub fn to_dto_messages(messages: Vec<ChatMessage>) -> Vec<ChatMessageDto> {
    messages.into_iter().map(map_one).collect()
}

fn map_one(m: ChatMessage) -> ChatMessageDto {
    match m {
        ChatMessage::System(s) => ChatMessageDto { role: "system".into(), content: serde_json::Value::String(s.content) },
        ChatMessage::User(u) => {
            let parts: Vec<serde_json::Value> = u
                .content
                .into_iter()
                .map(|p| match p {
                    ContentPart::Text { text } => serde_json::json!({"type": "text", "text": text}),
                    ContentPart::ImageBase64 { media_type, data } => serde_json::json!({
                        "type": "image_url",
                        "image_url": { "url": format!("data:{media_type};base64,{data}") }
                    }),
                    ContentPart::ImageUrl { url } => serde_json::json!({
                        "type": "image_url",
                        "image_url": { "url": url }
                    }),
                })
                .collect();
            ChatMessageDto { role: "user".into(), content: serde_json::Value::Array(parts) }
        }
        ChatMessage::Assistant(a) => ChatMessageDto {
            role: "assistant".into(),
            content: serde_json::Value::String(a.content.unwrap_or_default()),
        },
        ChatMessage::Tool(t) => ChatMessageDto {
            role: "tool".into(),
            content: serde_json::Value::String(t.content),
        },
    }
}

#[must_use]
pub fn response_to_chat(r: ChatCompletionResponse) -> ChatResponse {
    let model = r.model.clone();
    let usage = r.usage.unwrap_or_default();
    let mut content = None;
    let mut tool_calls = Vec::new();
    let mut finish = FinishReason::Stop;
    if let Some(c) = r.choices.into_iter().next() {
        content = c.message.content;
        for t in c.message.tool_calls {
            let args: serde_json::Value =
                serde_json::from_str(&t.function.arguments).unwrap_or(serde_json::Value::Null);
            tool_calls.push(ToolCall { id: t.id, name: t.function.name, arguments: args });
        }
        finish = match c.finish_reason.as_deref() {
            Some("stop") | None => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolCalls,
            Some("content_filter") => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        };
    }
    let _assistant = AssistantMessage { content: content.clone(), tool_calls: tool_calls.clone(), cache: false };
    ChatResponse {
        content,
        tool_calls,
        usage: Usage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
            cache_read_input_tokens: 0,
            cache_creation_input_tokens: 0,
        },
        model,
        finish_reason: finish,
    }
}

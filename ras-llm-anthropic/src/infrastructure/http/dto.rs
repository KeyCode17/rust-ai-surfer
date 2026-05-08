use ras_llm::{AssistantMessage, ChatMessage, ContentPart, FinishReason, ToolCall, Usage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AnthropicMessagesRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: Vec<AnthropicContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContent {
    Text { text: String },
    Image { source: AnthropicImageSource },
}

#[derive(Debug, Serialize)]
pub struct AnthropicImageSource {
    #[serde(rename = "type")]
    pub kind: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicMessagesResponse {
    pub id: String,
    pub model: String,
    pub stop_reason: Option<String>,
    pub content: Vec<AnthropicResponseContent>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicResponseContent {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
}

#[must_use]
pub fn split_messages(messages: Vec<ChatMessage>) -> (Option<String>, Vec<AnthropicMessage>) {
    let mut system = None;
    let mut out = Vec::new();
    for m in messages {
        match m {
            ChatMessage::System(s) => {
                let prev: String = system.take().unwrap_or_default();
                let merged = if prev.is_empty() {
                    s.content
                } else {
                    format!("{prev}\n\n{}", s.content)
                };
                system = Some(merged);
            }
            ChatMessage::User(u) => out.push(AnthropicMessage {
                role: "user".into(),
                content: u
                    .content
                    .into_iter()
                    .map(content_part_to_anthropic)
                    .collect(),
            }),
            ChatMessage::Assistant(a) => {
                let mut parts = Vec::new();
                if let Some(t) = a.content {
                    parts.push(AnthropicContent::Text { text: t });
                }
                out.push(AnthropicMessage {
                    role: "assistant".into(),
                    content: parts,
                });
            }
            ChatMessage::Tool(t) => out.push(AnthropicMessage {
                role: "user".into(),
                content: vec![AnthropicContent::Text { text: t.content }],
            }),
        }
    }
    (system, out)
}

fn content_part_to_anthropic(part: ContentPart) -> AnthropicContent {
    match part {
        ContentPart::Text { text } => AnthropicContent::Text { text },
        ContentPart::ImageBase64 { media_type, data } => AnthropicContent::Image {
            source: AnthropicImageSource {
                kind: "base64".into(),
                media_type,
                data,
            },
        },
        ContentPart::ImageUrl { url } => AnthropicContent::Text { text: url },
    }
}

#[must_use]
pub fn response_to_chat(
    response: AnthropicMessagesResponse,
) -> (AssistantMessage, Vec<ToolCall>, Usage, FinishReason) {
    let mut text = String::new();
    let mut tool_calls = Vec::new();
    for c in response.content {
        match c {
            AnthropicResponseContent::Text { text: t } => text.push_str(&t),
            AnthropicResponseContent::ToolUse { id, name, input } => {
                tool_calls.push(ToolCall {
                    id,
                    name,
                    arguments: input,
                });
            }
        }
    }
    let finish = match response.stop_reason.as_deref() {
        Some("end_turn") | Some("stop_sequence") | None => FinishReason::Stop,
        Some("max_tokens") => FinishReason::Length,
        Some("tool_use") => FinishReason::ToolCalls,
        Some(_) => FinishReason::Stop,
    };
    let usage = Usage {
        input_tokens: response.usage.input_tokens,
        output_tokens: response.usage.output_tokens,
        cache_read_input_tokens: response.usage.cache_read_input_tokens,
        cache_creation_input_tokens: response.usage.cache_creation_input_tokens,
    };
    let assistant = AssistantMessage {
        content: if text.is_empty() { None } else { Some(text) },
        tool_calls: tool_calls.clone(),
        cache: false,
    };
    (assistant, tool_calls, usage, finish)
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum ChatMessage {
    System(SystemMessage),
    User(UserMessage),
    Assistant(AssistantMessage),
    Tool(ToolResultMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub content: String,
    #[serde(default)]
    pub cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub content: Vec<ContentPart>,
    #[serde(default)]
    pub cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default)]
    pub cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultMessage {
    pub tool_call_id: String,
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageBase64 { media_type: String, data: String },
    ImageUrl { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

impl ChatMessage {
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self::System(SystemMessage {
            content: content.into(),
            cache: false,
        })
    }

    /// A system message marked for prompt caching. Use for a large, STABLE
    /// prefix (a task/system prompt re-sent every step): providers that support
    /// it (Anthropic via OpenRouter) read the prefix from cache instead of
    /// re-prefilling it, cutting per-step latency and cost with no output change.
    #[must_use]
    pub fn system_cached(content: impl Into<String>) -> Self {
        Self::System(SystemMessage {
            content: content.into(),
            cache: true,
        })
    }

    #[must_use]
    pub fn user_text(text: impl Into<String>) -> Self {
        Self::User(UserMessage {
            content: vec![ContentPart::Text { text: text.into() }],
            cache: false,
        })
    }

    #[must_use]
    pub fn user_parts(parts: Vec<ContentPart>) -> Self {
        Self::User(UserMessage {
            content: parts,
            cache: false,
        })
    }

    #[must_use]
    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self::Assistant(AssistantMessage {
            content: Some(text.into()),
            tool_calls: Vec::new(),
            cache: false,
        })
    }
}

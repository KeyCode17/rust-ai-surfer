use async_trait::async_trait;
use ras_errors::AppError;
use ras_llm::{ChatMessage, ChatResponse, InvokeOptions, LlmClient, ProviderName};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tracing::debug;

use crate::domain::billing_header::ANTHROPIC_BASE_URL;
use crate::infrastructure::http::dto::{
    AnthropicMessagesRequest, AnthropicMessagesResponse, response_to_chat, split_messages,
};

pub struct ChatAnthropic {
    pub model: String,
    pub api_key: Option<String>,
    pub auth_token: Option<String>,
    pub base_url: String,
    pub default_headers: Vec<(String, String)>,
    pub client: Client,
}

impl std::fmt::Debug for ChatAnthropic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatAnthropic")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .field(
                "auth_kind",
                &self
                    .auth_token
                    .as_ref()
                    .map(|_| "bearer")
                    .unwrap_or("api_key"),
            )
            .finish()
    }
}

impl ChatAnthropic {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Result<Self, AppError> {
        let client = Client::builder()
            .build()
            .map_err(|e| AppError::InternalError(format!("http client: {e}")))?;
        Ok(Self {
            model: model.into(),
            api_key: Some(api_key.into()),
            auth_token: None,
            base_url: ANTHROPIC_BASE_URL.into(),
            default_headers: Vec::new(),
            client,
        })
    }
}

#[async_trait]
impl LlmClient for ChatAnthropic {
    fn provider(&self) -> ProviderName {
        ProviderName("anthropic".into())
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn ainvoke(
        &self,
        messages: Vec<ChatMessage>,
        options: InvokeOptions,
    ) -> Result<ChatResponse, AppError> {
        let (system, anthropic_messages) = split_messages(messages);
        let req = AnthropicMessagesRequest {
            model: self.model.clone(),
            max_tokens: options.max_tokens.unwrap_or(4096),
            messages: anthropic_messages,
            system,
            temperature: options.temperature,
            stop_sequences: options.stop_sequences,
        };
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let mut headers = HeaderMap::new();
        for (k, v) in &self.default_headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                headers.insert(name, val);
            }
        }
        if let Some(token) = &self.auth_token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {token}")) {
                headers.insert(reqwest::header::AUTHORIZATION, val);
            }
        } else if let Some(key) = &self.api_key
            && let Ok(val) = HeaderValue::from_str(key)
        {
            headers.insert(HeaderName::from_static("x-api-key"), val);
        }
        debug!(url = %url, model = %self.model, "anthropic post");
        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::LlmProviderError(format!("anthropic send: {e}")))?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::LlmProviderError(format!("anthropic body: {e}")))?;
        if !status.is_success() {
            return Err(map_anthropic_error(status, &body));
        }
        let parsed: AnthropicMessagesResponse = serde_json::from_str(&body)
            .map_err(|e| AppError::LlmProviderError(format!("anthropic parse: {e}")))?;
        let model = parsed.model.clone();
        let (assistant, tool_calls, usage, finish_reason) = response_to_chat(parsed);
        Ok(ChatResponse {
            content: assistant.content,
            tool_calls,
            usage,
            model,
            finish_reason,
        })
    }
}

fn map_anthropic_error(status: reqwest::StatusCode, body: &str) -> AppError {
    let snippet = body.chars().take(400).collect::<String>();
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return AppError::LlmRateLimited(snippet);
    }
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return AppError::LlmAuthExpired(snippet);
    }
    if status.is_server_error() {
        return AppError::LlmProviderError(format!("{status}: {snippet}"));
    }
    AppError::LlmProviderError(format!("{status}: {snippet}"))
}

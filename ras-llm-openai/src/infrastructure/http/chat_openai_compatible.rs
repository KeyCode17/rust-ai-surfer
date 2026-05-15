use async_trait::async_trait;
use ras_errors::AppError;
use ras_llm::{ChatMessage, ChatResponse, InvokeOptions, LlmClient, ProviderName};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tracing::{debug, trace};

use crate::infrastructure::http::dto::{
    ChatCompletionResponse, ChatRequest, response_to_chat, to_dto_messages,
};

#[derive(Debug, Clone)]
pub enum OpenAiAuth {
    Bearer(String),
    Header(String, String),
}

pub struct ChatOpenAICompatible {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub auth: OpenAiAuth,
    pub extra_headers: Vec<(String, String)>,
    pub client: Client,
}

impl ChatOpenAICompatible {
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
        auth: OpenAiAuth,
    ) -> Result<Self, AppError> {
        let client = Client::builder()
            .build()
            .map_err(|e| AppError::InternalError(format!("http client: {e}")))?;
        Ok(Self {
            provider: provider.into(),
            model: model.into(),
            base_url: base_url.into(),
            auth,
            extra_headers: Vec::new(),
            client,
        })
    }
}

impl std::fmt::Debug for ChatOpenAICompatible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatOpenAICompatible")
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[async_trait]
impl LlmClient for ChatOpenAICompatible {
    fn provider(&self) -> ProviderName {
        ProviderName(self.provider.clone())
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn ainvoke(
        &self,
        messages: Vec<ChatMessage>,
        options: InvokeOptions,
    ) -> Result<ChatResponse, AppError> {
        let req = ChatRequest {
            model: self.model.clone(),
            messages: to_dto_messages(messages),
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stop: options.stop_sequences,
        };
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let mut headers = HeaderMap::new();
        match &self.auth {
            OpenAiAuth::Bearer(token) => {
                if let Ok(v) = HeaderValue::from_str(&format!("Bearer {token}")) {
                    headers.insert(reqwest::header::AUTHORIZATION, v);
                }
            }
            OpenAiAuth::Header(name, value) => {
                if let (Ok(n), Ok(v)) = (
                    HeaderName::from_bytes(name.as_bytes()),
                    HeaderValue::from_str(value),
                ) {
                    headers.insert(n, v);
                }
            }
        }
        for (k, v) in &self.extra_headers {
            if let (Ok(n), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                headers.insert(n, val);
            }
        }
        debug!(provider = %self.provider, model = %self.model, "chat post");
        if tracing::enabled!(tracing::Level::TRACE) {
            match serde_json::to_string(&req) {
                Ok(s) => trace!(provider = %self.provider, body = %s, "llm request"),
                Err(e) => trace!(provider = %self.provider, err = %e, "llm request serialize"),
            }
        }
        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::LlmProviderError(format!("{} send: {e}", self.provider)))?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::LlmProviderError(format!("{} body: {e}", self.provider)))?;
        if tracing::enabled!(tracing::Level::TRACE) {
            trace!(provider = %self.provider, status = %status, body = %body, "llm response");
        }
        if !status.is_success() {
            return Err(map_status(status, &body));
        }
        let parsed: ChatCompletionResponse = serde_json::from_str(&body)
            .map_err(|e| AppError::LlmProviderError(format!("{} parse: {e}", self.provider)))?;
        Ok(response_to_chat(parsed))
    }
}

fn map_status(status: reqwest::StatusCode, body: &str) -> AppError {
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

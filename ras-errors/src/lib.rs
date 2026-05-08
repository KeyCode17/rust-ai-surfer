use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    InternalError(String),

    #[error("browser disconnected: {0}")]
    BrowserDisconnected(String),

    #[error("cdp timeout: {0}")]
    CdpTimeout(String),

    #[error("llm rate limited: {0}")]
    LlmRateLimited(String),

    #[error("llm auth expired: {0}")]
    LlmAuthExpired(String),

    #[error("llm provider error: {0}")]
    LlmProviderError(String),

    #[error("action failed: {0}")]
    ActionFailed(String),

    #[error("element not found: {0}")]
    ElementNotFound(String),
}

impl AppError {
    #[must_use]
    pub fn cli_exit_code(&self) -> i32 {
        match self {
            Self::NotFound(_) | Self::ElementNotFound(_) => 4,
            Self::BadRequest(_) | Self::ValidationError(_) => 2,
            Self::Unauthorized(_) | Self::LlmAuthExpired(_) => 5,
            Self::Forbidden(_) => 6,
            Self::Conflict(_) => 7,
            Self::BrowserDisconnected(_) | Self::CdpTimeout(_) => 8,
            Self::LlmRateLimited(_) | Self::LlmProviderError(_) => 9,
            Self::ActionFailed(_) | Self::InternalError(_) => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

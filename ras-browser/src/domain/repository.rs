use std::path::PathBuf;

use async_trait::async_trait;
use ras_errors::AppError;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::session::{BrowserSession, SessionState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionMode {
    Attach {
        cdp_url: Url,
    },
    Launch {
        binary: PathBuf,
        port: Option<u16>,
    },
    Cloud {
        api_key: String,
        region: Option<String>,
    },
}

#[async_trait]
pub trait BrowserSessionPort: Send + Sync + 'static {
    async fn start(&self, mode: SessionMode) -> Result<BrowserSession, AppError>;
    async fn current_state(&self) -> Result<SessionState, AppError>;
    async fn close(&self) -> Result<(), AppError>;
}

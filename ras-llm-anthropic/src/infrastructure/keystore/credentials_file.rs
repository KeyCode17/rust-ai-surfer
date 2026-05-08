use std::path::PathBuf;

use async_trait::async_trait;
use ras_errors::AppError;
use serde::Deserialize;

use crate::domain::claude_code_credentials::{ClaudeCodeCredentials, CredentialSource};
use crate::domain::repository::CredentialsFileRepository;

#[derive(Debug, Deserialize)]
struct OauthBlob {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<TokenBlob>,
}

#[derive(Debug, Deserialize)]
struct TokenBlob {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<i64>,
}

pub struct CredentialsFileReader {
    path: PathBuf,
}

impl CredentialsFileReader {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn default_path() -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"));
        Self { path: home.join(".claude").join(".credentials.json") }
    }
}

#[async_trait]
impl CredentialsFileRepository for CredentialsFileReader {
    async fn read(&self) -> Result<Option<ClaudeCodeCredentials>, AppError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let bytes = tokio::fs::read(&self.path)
            .await
            .map_err(|e| AppError::InternalError(format!("read credentials.json: {e}")))?;
        let parsed: OauthBlob = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let Some(blob) = parsed.claude_ai_oauth else {
            return Ok(None);
        };
        let Some(token) = blob.access_token else {
            return Ok(None);
        };
        let expires_at = blob.expires_at.unwrap_or(0);
        Ok(Some(ClaudeCodeCredentials::new(token, expires_at, CredentialSource::CredentialsFile)))
    }
}

use std::path::PathBuf;

use async_trait::async_trait;
use ras_errors::AppError;
use serde::Deserialize;

use crate::domain::claude_code_credentials::{ClaudeCodeCredentials, CredentialSource};
use crate::domain::repository::SettingsFileRepository;

#[derive(Debug, Deserialize, Default)]
struct SettingsRoot {
    #[serde(default)]
    env: SettingsEnv,
}

#[derive(Debug, Deserialize, Default)]
struct SettingsEnv {
    #[serde(rename = "ANTHROPIC_AUTH_TOKEN")]
    anthropic_auth_token: Option<String>,
    #[serde(rename = "ANTHROPIC_BASE_URL")]
    anthropic_base_url: Option<String>,
}

pub struct SettingsFileReader {
    path: PathBuf,
}

impl SettingsFileReader {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn default_path() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        Self {
            path: home.join(".claude").join("settings.json"),
        }
    }
}

#[async_trait]
impl SettingsFileRepository for SettingsFileReader {
    async fn read(&self) -> Result<Option<ClaudeCodeCredentials>, AppError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let bytes = tokio::fs::read(&self.path)
            .await
            .map_err(|e| AppError::InternalError(format!("read settings.json: {e}")))?;
        let parsed: SettingsRoot = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let Some(token) = parsed.env.anthropic_auth_token else {
            return Ok(None);
        };
        let mut creds = ClaudeCodeCredentials::new(token, 0, CredentialSource::SettingsFile);
        if let Some(base) = parsed.env.anthropic_base_url {
            creds = creds.with_base_url(base.trim_end_matches('/'));
        }
        Ok(Some(creds))
    }
}

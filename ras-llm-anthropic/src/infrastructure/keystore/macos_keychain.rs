use async_trait::async_trait;
use ras_errors::AppError;
use serde::Deserialize;

use crate::domain::claude_code_credentials::{ClaudeCodeCredentials, CredentialSource};
use crate::domain::repository::KeychainRepository;

#[derive(Debug, Deserialize)]
struct KeychainBlob {
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

#[derive(Debug, Default, Clone, Copy)]
pub struct MacosKeychain;

#[async_trait]
impl KeychainRepository for MacosKeychain {
    async fn read(&self) -> Result<Option<ClaudeCodeCredentials>, AppError> {
        if !cfg!(target_os = "macos") {
            return Ok(None);
        }
        let output = match tokio::process::Command::new("security")
            .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
            .output()
            .await
        {
            Ok(o) => o,
            Err(_) => return Ok(None),
        };
        if !output.status.success() {
            return Ok(None);
        }
        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if raw.is_empty() {
            return Ok(None);
        }
        let parsed: KeychainBlob = match serde_json::from_str(&raw) {
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
        Ok(Some(ClaudeCodeCredentials::new(token, expires_at, CredentialSource::Keychain)))
    }
}

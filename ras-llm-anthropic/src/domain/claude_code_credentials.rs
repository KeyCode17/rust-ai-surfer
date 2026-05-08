use std::time::{SystemTime, UNIX_EPOCH};

use ras_errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialSource {
    Env,
    Keychain,
    CredentialsFile,
    SettingsFile,
}

impl std::fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Env => "env",
            Self::Keychain => "keychain",
            Self::CredentialsFile => "credentials.json",
            Self::SettingsFile => "settings.json",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeCredentials {
    access_token: String,
    expires_at_ms: i64,
    source: CredentialSource,
    base_url: Option<String>,
}

impl ClaudeCodeCredentials {
    #[must_use]
    pub fn new(
        access_token: impl Into<String>,
        expires_at_ms: i64,
        source: CredentialSource,
    ) -> Self {
        Self { access_token: access_token.into(), expires_at_ms, source, base_url: None }
    }

    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    #[must_use]
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    #[must_use]
    pub fn source(&self) -> CredentialSource {
        self.source
    }

    #[must_use]
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    pub fn ensure_not_expired(self) -> Result<Self, AppError> {
        if self.expires_at_ms == 0 {
            return Ok(self);
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        if now > self.expires_at_ms {
            return Err(AppError::LlmAuthExpired(format!(
                "Claude Code OAuth token from {} is expired. Run `claude` to refresh.",
                self.source
            )));
        }
        Ok(self)
    }
}

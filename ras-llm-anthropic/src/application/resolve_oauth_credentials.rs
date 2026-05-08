use std::sync::Arc;

use ras_errors::AppError;

use crate::domain::claude_code_credentials::ClaudeCodeCredentials;
use crate::domain::repository::{
    CredentialsFileRepository, EnvCredentialRepository, KeychainRepository, SettingsFileRepository,
};

pub struct ResolveOauthCredentials {
    env_repo: Arc<dyn EnvCredentialRepository>,
    keychain_repo: Arc<dyn KeychainRepository>,
    cred_file_repo: Arc<dyn CredentialsFileRepository>,
    settings_repo: Arc<dyn SettingsFileRepository>,
}

impl ResolveOauthCredentials {
    pub fn new(
        env_repo: Arc<dyn EnvCredentialRepository>,
        keychain_repo: Arc<dyn KeychainRepository>,
        cred_file_repo: Arc<dyn CredentialsFileRepository>,
        settings_repo: Arc<dyn SettingsFileRepository>,
    ) -> Self {
        Self {
            env_repo,
            keychain_repo,
            cred_file_repo,
            settings_repo,
        }
    }

    pub async fn execute(&self) -> Result<ClaudeCodeCredentials, AppError> {
        if self.env_repo.has_anthropic_api_key().await? {
            return Err(AppError::Conflict(
                "ANTHROPIC_API_KEY is set; use ChatAnthropic for the regular API path.".into(),
            ));
        }
        if let Some(c) = self.keychain_repo.read().await? {
            return c.ensure_not_expired();
        }
        if let Some(c) = self.cred_file_repo.read().await? {
            return c.ensure_not_expired();
        }
        if let Some(c) = self.settings_repo.read().await? {
            return Ok(c);
        }
        Err(AppError::Unauthorized(
            "No Claude Code OAuth credentials. Run `claude` to log in.".into(),
        ))
    }
}

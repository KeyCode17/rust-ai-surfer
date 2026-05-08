use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::claude_code_credentials::ClaudeCodeCredentials;

#[async_trait]
pub trait EnvCredentialRepository: Send + Sync + 'static {
    async fn has_anthropic_api_key(&self) -> Result<bool, AppError>;
}

#[async_trait]
pub trait KeychainRepository: Send + Sync + 'static {
    async fn read(&self) -> Result<Option<ClaudeCodeCredentials>, AppError>;
}

#[async_trait]
pub trait CredentialsFileRepository: Send + Sync + 'static {
    async fn read(&self) -> Result<Option<ClaudeCodeCredentials>, AppError>;
}

#[async_trait]
pub trait SettingsFileRepository: Send + Sync + 'static {
    async fn read(&self) -> Result<Option<ClaudeCodeCredentials>, AppError>;
}

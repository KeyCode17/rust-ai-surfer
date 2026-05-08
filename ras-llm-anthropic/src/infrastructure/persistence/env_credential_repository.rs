use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::repository::EnvCredentialRepository;

#[derive(Debug, Default, Clone, Copy)]
pub struct EnvAnthropicApiKey;

#[async_trait]
impl EnvCredentialRepository for EnvAnthropicApiKey {
    async fn has_anthropic_api_key(&self) -> Result<bool, AppError> {
        Ok(std::env::var("ANTHROPIC_API_KEY").is_ok())
    }
}

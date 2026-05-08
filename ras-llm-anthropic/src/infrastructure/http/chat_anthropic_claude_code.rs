use std::sync::Arc;

use async_trait::async_trait;
use ras_errors::AppError;
use ras_llm::{ChatMessage, ChatResponse, InvokeOptions, LlmClient, ProviderName};
use reqwest::Client;

use crate::application::invoke_claude_code::inject_billing_header;
use crate::application::resolve_cc_version::resolve_cc_version;
use crate::application::resolve_oauth_credentials::ResolveOauthCredentials;
use crate::domain::billing_header::{ANTHROPIC_BASE_URL, BillingHeader, default_headers};
use crate::domain::cc_version::CcVersion;
use crate::domain::repository::{
    CredentialsFileRepository, EnvCredentialRepository, KeychainRepository, SettingsFileRepository,
};
use crate::infrastructure::http::chat_anthropic::ChatAnthropic;
use crate::infrastructure::keystore::credentials_file::CredentialsFileReader;
use crate::infrastructure::keystore::macos_keychain::MacosKeychain;
use crate::infrastructure::keystore::settings_file::SettingsFileReader;
use crate::infrastructure::persistence::env_credential_repository::EnvAnthropicApiKey;

pub struct ChatAnthropicClaudeCode {
    inner: ChatAnthropic,
    cc_version: CcVersion,
    billing: BillingHeader,
}

impl std::fmt::Debug for ChatAnthropicClaudeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatAnthropicClaudeCode")
            .field("inner", &self.inner)
            .field("cc_version", &self.cc_version.as_str())
            .finish()
    }
}

impl ChatAnthropicClaudeCode {
    pub async fn new(model: impl Into<String>) -> Result<Self, AppError> {
        Self::new_with_repos(
            model,
            Arc::new(EnvAnthropicApiKey),
            Arc::new(MacosKeychain),
            Arc::new(CredentialsFileReader::default_path()),
            Arc::new(SettingsFileReader::default_path()),
        )
        .await
    }

    pub async fn new_with_repos(
        model: impl Into<String>,
        env_repo: Arc<dyn EnvCredentialRepository>,
        keychain_repo: Arc<dyn KeychainRepository>,
        cred_repo: Arc<dyn CredentialsFileRepository>,
        settings_repo: Arc<dyn SettingsFileRepository>,
    ) -> Result<Self, AppError> {
        let resolver =
            ResolveOauthCredentials::new(env_repo, keychain_repo, cred_repo, settings_repo);
        let creds = resolver.execute().await?;
        let cc_version = resolve_cc_version().await;
        let billing = BillingHeader::for_cli(&cc_version);
        let headers = default_headers(&cc_version);
        let mut inner = ChatAnthropic {
            model: model.into(),
            api_key: None,
            auth_token: Some(creds.access_token().to_string()),
            base_url: creds.base_url().unwrap_or(ANTHROPIC_BASE_URL).to_string(),
            default_headers: headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            client: Client::builder()
                .build()
                .map_err(|e| AppError::InternalError(format!("http client: {e}")))?,
        };
        inner.api_key = None;
        Ok(Self {
            inner,
            cc_version,
            billing,
        })
    }

    #[must_use]
    pub fn cc_version(&self) -> &CcVersion {
        &self.cc_version
    }
}

#[async_trait]
impl LlmClient for ChatAnthropicClaudeCode {
    fn provider(&self) -> ProviderName {
        ProviderName("anthropic_claude_code".into())
    }

    fn model(&self) -> &str {
        self.inner.model()
    }

    async fn ainvoke(
        &self,
        messages: Vec<ChatMessage>,
        options: InvokeOptions,
    ) -> Result<ChatResponse, AppError> {
        let with_billing = inject_billing_header(messages, &self.billing);
        self.inner.ainvoke(with_billing, options).await
    }
}

use async_trait::async_trait;
use ras_errors::AppError;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::domain::auth::DeviceAuth;
use crate::domain::repository::{CloudClient, CloudSession};

pub struct HttpCloudClient {
    base_url: Url,
    api_key: Option<String>,
    client: Client,
}

impl HttpCloudClient {
    pub fn new(base_url: Url, api_key: Option<String>) -> Result<Self, AppError> {
        let client = Client::builder()
            .build()
            .map_err(|e| AppError::InternalError(format!("http: {e}")))?;
        Ok(Self { base_url, api_key, client })
    }
}

impl std::fmt::Debug for HttpCloudClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpCloudClient").field("base_url", &self.base_url).finish()
    }
}

#[derive(Deserialize)]
struct ProvisionResponse {
    session_id: String,
    cdp_url: String,
    region: Option<String>,
    expires_at_unix_ms: i64,
}

#[async_trait]
impl CloudClient for HttpCloudClient {
    async fn start_device_auth(&self) -> Result<DeviceAuth, AppError> {
        Err(AppError::NotFound("device auth not provisioned in this build".into()))
    }

    async fn poll_token(&self, _device_code: &str) -> Result<String, AppError> {
        Err(AppError::NotFound("device auth not provisioned in this build".into()))
    }

    async fn provision_browser(
        &self,
        region: Option<String>,
    ) -> Result<CloudSession, AppError> {
        let key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Unauthorized("api key missing".into()))?;
        let url = self
            .base_url
            .join("v1/browsers")
            .map_err(|e| AppError::ValidationError(format!("base_url: {e}")))?;
        let mut body = serde_json::json!({});
        if let Some(r) = &region {
            body["region"] = serde_json::Value::String(r.clone());
        }
        let resp = self
            .client
            .post(url.as_str())
            .bearer_auth(key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::LlmProviderError(format!("cloud send: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::LlmProviderError(format!("cloud status: {e}")))?
            .json::<ProvisionResponse>()
            .await
            .map_err(|e| AppError::LlmProviderError(format!("cloud parse: {e}")))?;
        let cdp_url = Url::parse(&resp.cdp_url)
            .map_err(|e| AppError::ValidationError(format!("cdp_url: {e}")))?;
        Ok(CloudSession {
            session_id: resp.session_id,
            cdp_url,
            region: resp.region,
            expires_at_unix_ms: resp.expires_at_unix_ms,
        })
    }

    async fn release_browser(&self, session_id: &str) -> Result<(), AppError> {
        let key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Unauthorized("api key missing".into()))?;
        let url = self
            .base_url
            .join(&format!("v1/browsers/{session_id}"))
            .map_err(|e| AppError::ValidationError(format!("url: {e}")))?;
        self.client
            .delete(url.as_str())
            .bearer_auth(key)
            .send()
            .await
            .map_err(|e| AppError::LlmProviderError(format!("cloud delete: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::LlmProviderError(format!("cloud delete status: {e}")))?;
        Ok(())
    }
}

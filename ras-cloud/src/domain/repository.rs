use async_trait::async_trait;
use ras_errors::AppError;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::auth::DeviceAuth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSession {
    pub session_id: String,
    pub cdp_url: Url,
    pub region: Option<String>,
    pub expires_at_unix_ms: i64,
}

#[async_trait]
pub trait CloudClient: Send + Sync + 'static {
    async fn start_device_auth(&self) -> Result<DeviceAuth, AppError>;
    async fn poll_token(&self, device_code: &str) -> Result<String, AppError>;
    async fn provision_browser(
        &self,
        region: Option<String>,
    ) -> Result<CloudSession, AppError>;
    async fn release_browser(&self, session_id: &str) -> Result<(), AppError>;
}

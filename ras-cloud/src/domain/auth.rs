use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAuthConfig {
    pub api_key: String,
    pub base_url: Url,
    pub device_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuth {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: Url,
    pub verification_uri_complete: Option<Url>,
    pub expires_in: u64,
    pub interval: u64,
}

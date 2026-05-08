use std::path::PathBuf;

use async_trait::async_trait;
use ras_errors::AppError;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::profile::CosmiumProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchedBrowser {
    pub cdp_url: Url,
    pub user_data_dir: PathBuf,
    pub pid: Option<u32>,
}

#[async_trait]
pub trait BrowserLauncher: Send + Sync + 'static {
    async fn launch(
        &self,
        binary: &std::path::Path,
        profile: &CosmiumProfile,
    ) -> Result<LaunchedBrowser, AppError>;
    async fn shutdown(&self, browser: &LaunchedBrowser) -> Result<(), AppError>;
}

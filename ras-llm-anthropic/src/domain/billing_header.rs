use serde::{Deserialize, Serialize};

use crate::domain::cc_version::CcVersion;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingHeader(String);

impl BillingHeader {
    #[must_use]
    pub fn for_cli(version: &CcVersion) -> Self {
        Self(format!(
            "x-anthropic-billing-header: cc_version={}; cc_entrypoint=cli;",
            version.as_str()
        ))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const OAUTH_BETA: &str = "oauth-2025-04-20,interleaved-thinking-2025-05-14,claude-code-20250219,prompt-caching-2024-07-31";

#[must_use]
pub fn default_headers(version: &CcVersion) -> Vec<(&'static str, String)> {
    vec![
        ("anthropic-version", ANTHROPIC_VERSION.to_string()),
        ("anthropic-beta", OAUTH_BETA.to_string()),
        (
            "anthropic-dangerous-direct-browser-access",
            "true".to_string(),
        ),
        (
            "user-agent",
            format!("claude-cli/{} (external, cli)", version.as_str()),
        ),
        ("x-app", "cli".to_string()),
    ]
}

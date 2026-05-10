use std::collections::HashSet;
use std::path::PathBuf;

use ras_types::{ActionTimeout, DomainPattern};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrowserProfile {
    pub user_data_dir: Option<PathBuf>,
    pub headless: bool,
    pub allowed_domains: AllowedDomains,
    pub prohibited_domains: Vec<DomainPattern>,
    pub block_ip_addresses: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub headers: Vec<(String, String)>,
    pub proxy: Option<Url>,
    pub action_timeout: ActionTimeout,
    pub stealth: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllowedDomains {
    patterns: Vec<DomainPattern>,
    cache: HashSet<String>,
}

impl AllowedDomains {
    #[must_use]
    pub fn new(patterns: Vec<DomainPattern>) -> Self {
        let cache = patterns.iter().map(|p| p.as_str().to_owned()).collect();
        Self { patterns, cache }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    #[must_use]
    pub fn allows(&self, url: &Url) -> bool {
        if self.patterns.is_empty() {
            return true;
        }
        if let Some(host) = url.host_str()
            && self.cache.contains(host)
        {
            return true;
        }
        self.patterns.iter().any(|p| p.matches_url(url))
    }
}

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DomainPattern(String);

#[derive(Debug, Error)]
pub enum DomainPatternError {
    #[error("empty pattern")]
    Empty,
    #[error("invalid pattern: {0}")]
    Invalid(String),
}

impl DomainPattern {
    pub fn new(pattern: impl Into<String>) -> Result<Self, DomainPatternError> {
        let s = pattern.into();
        if s.trim().is_empty() {
            return Err(DomainPatternError::Empty);
        }
        Ok(Self(s.to_lowercase()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn matches_url(&self, url: &Url) -> bool {
        let Some(host) = url.host_str() else {
            return false;
        };
        let host = host.to_lowercase();
        let pattern = &self.0;
        if pattern.starts_with("http://") || pattern.starts_with("https://") {
            return url.as_str().starts_with(pattern.as_str());
        }
        if let Some(rest) = pattern.strip_prefix("*.") {
            return host == rest || host.ends_with(&format!(".{rest}"));
        }
        host == *pattern || host == format!("www.{pattern}")
    }
}

impl FromStr for DomainPattern {
    type Err = DomainPatternError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchLevel {
    Hash,
    Xpath,
    AxName,
    Attributes,
}

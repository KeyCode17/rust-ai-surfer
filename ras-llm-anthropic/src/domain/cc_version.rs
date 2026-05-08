use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const FALLBACK_CC_VERSION: &str = "2.1.133";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CcVersion(String);

#[derive(Debug, Error)]
pub enum CcVersionError {
    #[error("invalid semver: {0}")]
    Invalid(String),
}

impl CcVersion {
    pub fn new(s: impl Into<String>) -> Result<Self, CcVersionError> {
        let s = s.into();
        let mut parts = s.split('.');
        let _major: u32 = parts
            .next()
            .ok_or_else(|| CcVersionError::Invalid(s.clone()))?
            .parse()
            .map_err(|_| CcVersionError::Invalid(s.clone()))?;
        let _minor: u32 = parts
            .next()
            .ok_or_else(|| CcVersionError::Invalid(s.clone()))?
            .parse()
            .map_err(|_| CcVersionError::Invalid(s.clone()))?;
        let _patch: u32 = parts
            .next()
            .ok_or_else(|| CcVersionError::Invalid(s.clone()))?
            .parse()
            .map_err(|_| CcVersionError::Invalid(s.clone()))?;
        Ok(Self(s))
    }

    #[must_use]
    pub fn fallback() -> Self {
        Self(FALLBACK_CC_VERSION.to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for CcVersion {
    type Err = CcVersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Default for CcVersion {
    fn default() -> Self {
        Self::fallback()
    }
}

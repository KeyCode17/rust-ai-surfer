use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionTimeout(Duration);

#[derive(Debug, Error)]
pub enum ActionTimeoutError {
    #[error("timeout must be > 0 and finite")]
    Invalid,
}

impl ActionTimeout {
    pub fn new(d: Duration) -> Result<Self, ActionTimeoutError> {
        if d.is_zero() {
            return Err(ActionTimeoutError::Invalid);
        }
        Ok(Self(d))
    }

    #[must_use]
    pub fn from_secs(s: u64) -> Self {
        Self(Duration::from_secs(s.max(1)))
    }

    #[must_use]
    pub fn as_duration(self) -> Duration {
        self.0
    }
}

impl Default for ActionTimeout {
    fn default() -> Self {
        Self(Duration::from_secs(180))
    }
}

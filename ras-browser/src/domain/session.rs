use ras_types::{AgentId, SessionId};
use serde::{Deserialize, Serialize};

use crate::domain::browser_profile::BrowserProfile;
use crate::domain::repository::SessionMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Disconnected,
    Connecting,
    Ready,
    Crashed,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSession {
    pub agent: AgentId,
    pub session: SessionId,
    pub mode: SessionMode,
    pub profile: BrowserProfile,
    pub state: SessionState,
}

impl BrowserSession {
    #[must_use]
    pub fn new(
        agent: AgentId,
        session: SessionId,
        mode: SessionMode,
        profile: BrowserProfile,
    ) -> Self {
        Self {
            agent,
            session,
            mode,
            profile,
            state: SessionState::Disconnected,
        }
    }
}

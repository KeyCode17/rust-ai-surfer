//! Session configuration types and error definitions.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use ras_errors::AppError;

/// Tenant session id (distinct from CDP `ras_types::SessionId`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentSessionId(pub String);

impl AgentSessionId {
    /// Allocate a process-unique session id.
    pub fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(format!("sess-{}", COUNTER.fetch_add(1, Ordering::Relaxed)))
    }
}

/// Policy when the session pool is at capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnFull {
    /// Reject new sessions with [`SessionError::AtCapacity`].
    Reject,
    /// Evict the least-recently-used session to make room.
    EvictOldest,
}

/// Runtime configuration for the session manager.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum number of concurrent sessions.
    pub max_sessions: usize,
    /// Duration after which an idle session is reaped.
    pub idle_timeout: Duration,
    /// What to do when the pool is full.
    pub on_full: OnFull,
    /// Whether one owner may hold more than one active session.
    pub allow_multi_per_owner: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 100,
            idle_timeout: Duration::from_secs(600),
            on_full: OnFull::Reject,
            allow_multi_per_owner: false,
        }
    }
}

/// Errors produced by the session manager.
#[derive(Debug)]
pub enum SessionError {
    /// Pool has reached `max_sessions` and the policy is `Reject`.
    AtCapacity,
    /// No session with the given id exists.
    NotFound,
    /// Session is currently executing a task and cannot be preempted.
    Busy,
    /// An underlying browser operation failed.
    Browser(AppError),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AtCapacity => write!(f, "session manager at capacity"),
            Self::NotFound => write!(f, "session not found"),
            Self::Busy => write!(f, "session already running a task"),
            Self::Browser(e) => write!(f, "browser error: {e}"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<AppError> for SessionError {
    fn from(e: AppError) -> Self {
        Self::Browser(e)
    }
}

#[cfg(test)]
mod config_tests {
    use super::{AgentSessionId, OnFull, SessionConfig, SessionError};
    use ras_errors::AppError;

    #[test]
    fn agent_session_id_increments() {
        let a = AgentSessionId::next();
        let b = AgentSessionId::next();
        assert!(a.0.starts_with("sess-"));
        assert_ne!(a, b);
    }

    #[test]
    fn session_config_defaults_are_sane() {
        let cfg = SessionConfig::default();
        assert_eq!(cfg.max_sessions, 100);
        assert_eq!(cfg.on_full, OnFull::Reject);
        assert!(!cfg.allow_multi_per_owner);
    }

    #[test]
    fn session_error_display_covers_variants() {
        assert!(SessionError::AtCapacity.to_string().contains("capacity"));
        assert!(SessionError::NotFound.to_string().contains("not found"));
        assert!(SessionError::Busy.to_string().contains("running"));
        let inner = AppError::ActionFailed("oops".into());
        assert!(SessionError::Browser(inner).to_string().contains("browser"));
    }

    #[test]
    fn session_error_from_app_error() {
        let e: SessionError = AppError::ActionFailed("x".into()).into();
        assert!(matches!(e, SessionError::Browser(_)));
    }
}

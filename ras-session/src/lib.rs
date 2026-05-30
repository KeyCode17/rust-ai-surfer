//! `ras-session` — tenant session lifecycle, config, and browser-provider abstraction.

pub mod config;
pub mod provider;

mod entry;
pub mod handle;
pub mod manager;
mod manager_reaper;
pub mod spawn_params;

pub use config::{AgentSessionId, OnFull, SessionConfig, SessionError};
pub use handle::SessionHandle;
pub use manager::SessionManager;
pub use provider::{BrowserProvider, SharedBrowserProvider};
pub use spawn_params::SpawnParams;

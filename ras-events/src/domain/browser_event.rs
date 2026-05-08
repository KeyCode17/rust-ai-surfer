use ras_types::{SessionId, TargetId};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BrowserEvent {
    BrowserConnected {
        cdp_url: String,
    },
    BrowserLaunched {
        pid: u32,
    },
    BrowserKilled {
        reason: String,
    },
    BrowserDisconnected {
        reason: String,
    },
    BrowserReconnecting {
        attempt: u32,
    },
    TargetCreated {
        target: TargetId,
    },
    TargetDestroyed {
        target: TargetId,
    },
    TargetCrashed {
        target: TargetId,
    },
    AgentFocusChanged {
        target: TargetId,
    },
    NavigationStarted {
        target: TargetId,
        url: Url,
    },
    NavigationCompleted {
        target: TargetId,
        url: Url,
    },
    DownloadStarted {
        url: Url,
        suggested_name: Option<String>,
    },
    DownloadCompleted {
        path: String,
    },
    DialogOpened {
        kind: DialogKind,
        message: String,
    },
    DialogDismissed,
    NetworkRequest {
        request_id: String,
        url: Url,
    },
    NetworkResponse {
        request_id: String,
        status: u16,
    },
    SessionDetached {
        session: SessionId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DialogKind {
    Alert,
    Confirm,
    Prompt,
    BeforeUnload,
}

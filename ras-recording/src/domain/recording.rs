use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingFormat {
    Mp4,
    Webm,
    Gif,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingState {
    pub output: PathBuf,
    pub format: RecordingFormat,
    pub frame_count: u64,
    pub started_at_unix_ms: i64,
}

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use ras_errors::AppError;
use tokio::sync::Mutex;

use crate::domain::recording::{RecordingFormat, RecordingState};
use crate::domain::repository::RecorderPort;

#[derive(Default)]
pub struct InMemoryRecorder {
    state: Mutex<Option<RecordingState>>,
    frames: Mutex<Vec<Vec<u8>>>,
}

impl std::fmt::Debug for InMemoryRecorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryRecorder").finish()
    }
}

impl InMemoryRecorder {
    pub async fn frame_count(&self) -> usize {
        self.frames.lock().await.len()
    }
}

#[async_trait]
impl RecorderPort for InMemoryRecorder {
    async fn start(
        &self,
        output: &Path,
        format: RecordingFormat,
    ) -> Result<RecordingState, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let state = RecordingState {
            output: PathBuf::from(output),
            format,
            frame_count: 0,
            started_at_unix_ms: now,
        };
        *self.state.lock().await = Some(state.clone());
        self.frames.lock().await.clear();
        Ok(state)
    }

    async fn frame(&self, png_bytes: &[u8]) -> Result<(), AppError> {
        let mut guard = self.state.lock().await;
        let Some(state) = guard.as_mut() else {
            return Err(AppError::Conflict("recorder not started".into()));
        };
        state.frame_count += 1;
        self.frames.lock().await.push(png_bytes.to_vec());
        Ok(())
    }

    async fn stop(&self) -> Result<RecordingState, AppError> {
        let mut guard = self.state.lock().await;
        guard
            .take()
            .ok_or_else(|| AppError::Conflict("recorder not started".into()))
    }
}

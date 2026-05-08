use std::time::Instant;

use async_trait::async_trait;
use ras_errors::AppError;
use tokio::process::Command;
use tokio::time::timeout;

use crate::domain::execution::{ExecutionRequest, ExecutionResult};
use crate::domain::repository::SandboxRunner;

#[derive(Debug, Default, Clone, Copy)]
pub struct ShellSandbox;

#[async_trait]
impl SandboxRunner for ShellSandbox {
    async fn run(&self, request: ExecutionRequest) -> Result<ExecutionResult, AppError> {
        let started = Instant::now();
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&request.script);
        for (k, v) in &request.env {
            cmd.env(k, v);
        }
        let result = timeout(request.timeout, cmd.output()).await;
        let elapsed = started.elapsed().as_millis() as u64;
        let output = match result {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => return Err(AppError::InternalError(format!("spawn: {e}"))),
            Err(_) => return Err(AppError::ActionFailed("sandbox timed out".into())),
        };
        Ok(ExecutionResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            duration_ms: elapsed,
        })
    }
}

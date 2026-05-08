use async_trait::async_trait;
use ras_errors::AppError;

use crate::domain::event::TelemetryEvent;

#[async_trait]
pub trait TelemetryClient: Send + Sync + 'static {
    async fn capture(&self, event: TelemetryEvent) -> Result<(), AppError>;
    async fn flush(&self) -> Result<(), AppError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopTelemetry;

#[async_trait]
impl TelemetryClient for NoopTelemetry {
    async fn capture(&self, _event: TelemetryEvent) -> Result<(), AppError> {
        Ok(())
    }
    async fn flush(&self) -> Result<(), AppError> {
        Ok(())
    }
}

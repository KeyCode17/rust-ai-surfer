use std::future::Future;
use std::time::Duration;

use ras_errors::AppError;
use tokio::time::timeout;

pub async fn within<T, F>(label: &str, dur: Duration, fut: F) -> Result<T, AppError>
where
    F: Future<Output = Result<T, AppError>>,
{
    match timeout(dur, fut).await {
        Ok(inner) => inner,
        Err(_) => Err(AppError::CdpTimeout(format!(
            "{label} exceeded {}ms",
            dur.as_millis()
        ))),
    }
}

use ras_errors::AppError;

#[must_use]
pub fn should_switch_to_fallback(error: &AppError) -> bool {
    matches!(
        error,
        AppError::LlmRateLimited(_)
            | AppError::LlmAuthExpired(_)
            | AppError::LlmProviderError(_)
            | AppError::CdpTimeout(_)
            | AppError::BrowserDisconnected(_)
    )
}

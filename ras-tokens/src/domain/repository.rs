use async_trait::async_trait;
use ras_errors::AppError;
use ras_llm::Usage;

use crate::domain::pricing::{ModelPricing, ModelUsageStats};

#[async_trait]
pub trait TokenCostService: Send + Sync + 'static {
    async fn pricing(&self, model: &str) -> Result<ModelPricing, AppError>;
    async fn record(&self, model: &str, usage: Usage) -> Result<(), AppError>;
    async fn aggregate(&self) -> Result<Vec<ModelUsageStats>, AppError>;
}

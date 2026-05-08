use std::collections::HashMap;

use async_trait::async_trait;
use ras_errors::AppError;
use ras_llm::Usage;
use tokio::sync::RwLock;

use crate::domain::pricing::{ModelPricing, ModelUsageStats};
use crate::domain::repository::TokenCostService;

pub struct InMemoryTokenCost {
    pricing: HashMap<String, ModelPricing>,
    usage: RwLock<HashMap<String, ModelUsageStats>>,
}

impl InMemoryTokenCost {
    #[must_use]
    pub fn new() -> Self {
        Self::with_pricing(default_pricing())
    }

    #[must_use]
    pub fn with_pricing(pricing: Vec<ModelPricing>) -> Self {
        let map = pricing.into_iter().map(|p| (p.model.clone(), p)).collect();
        Self {
            pricing: map,
            usage: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryTokenCost {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for InMemoryTokenCost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryTokenCost")
            .field("models", &self.pricing.len())
            .finish()
    }
}

#[async_trait]
impl TokenCostService for InMemoryTokenCost {
    async fn pricing(&self, model: &str) -> Result<ModelPricing, AppError> {
        self.pricing
            .get(model)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("no pricing for model {model}")))
    }

    async fn record(&self, model: &str, usage: Usage) -> Result<(), AppError> {
        let pricing = self.pricing.get(model).cloned().unwrap_or_default();
        let cost = pricing.cost_for(
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_read_input_tokens,
            usage.cache_creation_input_tokens,
        );
        let mut map = self.usage.write().await;
        let stats = map
            .entry(model.to_string())
            .or_insert_with(|| ModelUsageStats {
                model: model.to_string(),
                ..Default::default()
            });
        stats.input_tokens += u64::from(usage.input_tokens);
        stats.output_tokens += u64::from(usage.output_tokens);
        stats.cache_read_tokens += u64::from(usage.cache_read_input_tokens);
        stats.cache_write_tokens += u64::from(usage.cache_creation_input_tokens);
        stats.usd_cost += cost;
        Ok(())
    }

    async fn aggregate(&self) -> Result<Vec<ModelUsageStats>, AppError> {
        Ok(self.usage.read().await.values().cloned().collect())
    }
}

fn default_pricing() -> Vec<ModelPricing> {
    vec![
        ModelPricing {
            model: "claude-sonnet-4-5".into(),
            input_per_million: 3.0,
            output_per_million: 15.0,
            cache_read_per_million: 0.3,
            cache_write_per_million: 3.75,
        },
        ModelPricing {
            model: "claude-haiku-4-5".into(),
            input_per_million: 0.8,
            output_per_million: 4.0,
            cache_read_per_million: 0.08,
            cache_write_per_million: 1.0,
        },
        ModelPricing {
            model: "gpt-4o".into(),
            input_per_million: 2.5,
            output_per_million: 10.0,
            cache_read_per_million: 1.25,
            cache_write_per_million: 0.0,
        },
        ModelPricing {
            model: "gemini-2.0-flash".into(),
            input_per_million: 0.1,
            output_per_million: 0.4,
            cache_read_per_million: 0.025,
            cache_write_per_million: 0.0,
        },
    ]
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model: String,
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_read_per_million: f64,
    pub cache_write_per_million: f64,
}

impl ModelPricing {
    #[must_use]
    pub fn cost_for(
        &self,
        input_tokens: u32,
        output_tokens: u32,
        cache_read: u32,
        cache_write: u32,
    ) -> f64 {
        let one = 1_000_000.0;
        f64::from(input_tokens) / one * self.input_per_million
            + f64::from(output_tokens) / one * self.output_per_million
            + f64::from(cache_read) / one * self.cache_read_per_million
            + f64::from(cache_write) / one * self.cache_write_per_million
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsageStats {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub usd_cost: f64,
}

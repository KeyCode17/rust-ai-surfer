use ras_llm::Usage;
use ras_tokens::ModelPricing;
use ras_tokens::TokenCostService;
use ras_tokens::infrastructure::in_memory_token_cost::InMemoryTokenCost;

#[tokio::test]
async fn computes_cost_per_pricing_table() {
    let p = ModelPricing {
        model: "test-model".into(),
        input_per_million: 10.0,
        output_per_million: 20.0,
        cache_read_per_million: 1.0,
        cache_write_per_million: 5.0,
    };
    let cost = p.cost_for(1_000_000, 500_000, 0, 0);
    assert!((cost - 20.0).abs() < 1e-6);
}

#[tokio::test]
async fn records_usage_and_aggregates() {
    let svc = InMemoryTokenCost::new();
    svc.record(
        "claude-sonnet-4-5",
        Usage { input_tokens: 1000, output_tokens: 500, cache_read_input_tokens: 0, cache_creation_input_tokens: 0 },
    )
    .await
    .expect("record");
    let stats = svc.aggregate().await.expect("aggregate");
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].input_tokens, 1000);
    assert_eq!(stats[0].output_tokens, 500);
    assert!(stats[0].usd_cost > 0.0);
}

#[tokio::test]
async fn unknown_model_record_is_zero_cost_but_tracked() {
    let svc = InMemoryTokenCost::new();
    svc.record(
        "unknown-model",
        Usage { input_tokens: 1, output_tokens: 1, cache_read_input_tokens: 0, cache_creation_input_tokens: 0 },
    )
    .await
    .expect("record");
    let stats = svc.aggregate().await.expect("aggregate");
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].usd_cost, 0.0);
}

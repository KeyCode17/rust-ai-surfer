use std::time::Duration;

use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout};
use serde::Deserialize;
use serde_json::json;

use crate::domain::action::{ActionMetadata, ToolHandler};
use crate::domain::registry::ToolContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct WaitAction;

#[derive(Deserialize)]
struct Params {
    seconds: f32,
}

#[async_trait]
impl ToolHandler for WaitAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("wait".into()),
            description: "Wait for the given number of seconds.".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {"seconds": {"type": "number"}},
                "required": ["seconds"]
            }),
            domain_filter: Vec::new(),
            terminates_sequence: false,
            timeout: ActionTimeout::default(),
        }
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: ToolContext,
    ) -> Result<ActionResult, AppError> {
        let p: Params = serde_json::from_value(params)
            .map_err(|e| AppError::ValidationError(format!("wait params: {e}")))?;
        let secs = p.seconds.clamp(0.0, 60.0);
        tokio::time::sleep(Duration::from_secs_f32(secs)).await;
        Ok(ActionResult::ok(format!("waited {secs:.1}s")))
    }
}

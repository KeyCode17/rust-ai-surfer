use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout};
use serde::Deserialize;
use serde_json::json;

use crate::domain::action::{ActionMetadata, ToolHandler};
use crate::domain::registry::ToolContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct ScrollAction;

#[derive(Deserialize)]
struct Params {
    #[serde(default)]
    direction: String,
    #[serde(default = "default_amount")]
    amount: i32,
}

fn default_amount() -> i32 {
    600
}

#[async_trait]
impl ToolHandler for ScrollAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("scroll".into()),
            description: "Scroll the focused tab vertically.".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "direction": {"type": "string", "enum": ["up", "down"], "default": "down"},
                    "amount": {"type": "integer", "default": 600}
                }
            }),
            domain_filter: Vec::new(),
            terminates_sequence: false,
            timeout: ActionTimeout::default(),
        }
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: ToolContext,
    ) -> Result<ActionResult, AppError> {
        let p: Params = serde_json::from_value(params)
            .map_err(|e| AppError::ValidationError(format!("scroll params: {e}")))?;
        let dir = if p.direction == "up" { -1 } else { 1 };
        let dy = p.amount * dir;
        let target = ctx.browser.focused_target().await?;
        let _ = ctx
            .browser
            .evaluate(&target, &format!("window.scrollBy(0, {dy});"))
            .await?;
        Ok(ActionResult::ok(format!("scrolled {dy}px")))
    }
}

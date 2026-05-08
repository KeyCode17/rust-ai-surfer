use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout};
use serde::Deserialize;
use serde_json::json;

use crate::domain::action::{ActionMetadata, ToolHandler};
use crate::domain::registry::ToolContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct TypeTextAction;

#[derive(Deserialize)]
struct Params {
    text: String,
    #[serde(default)]
    sensitive: bool,
}

#[async_trait]
impl ToolHandler for TypeTextAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("type_text".into()),
            description: "Type text into the focused element.".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "sensitive": {"type": "boolean", "default": false}
                },
                "required": ["text"]
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
            .map_err(|e| AppError::ValidationError(format!("type params: {e}")))?;
        let target = ctx.browser.focused_target().await?;
        ctx.browser.type_text(&target, &p.text).await?;
        let display = if p.sensitive {
            "[redacted]".to_string()
        } else {
            p.text.chars().take(80).collect::<String>()
        };
        Ok(ActionResult::ok(format!("typed {display:?}")))
    }
}

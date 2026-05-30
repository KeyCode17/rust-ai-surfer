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
    #[serde(default, alias = "H_index", alias = "element_index")]
    index: Option<i64>,
}

#[async_trait]
impl ToolHandler for TypeTextAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("type_text".into()),
            description: "Type text into an input. If `index` is given, focus that clickable first; otherwise type into the currently focused element.".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "sensitive": {"type": "boolean", "default": false},
                    "index": {"type": "integer", "description": "Clickable index to focus before typing."}
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
        let target = ctx
            .target
            .clone()
            .ok_or_else(|| AppError::NotFound("no active target".into()))?;
        if let Some(idx) = p.index {
            let element = ctx
                .clickables
                .iter()
                .find(|c| i64::from(c.index) == idx)
                .ok_or_else(|| {
                    AppError::ValidationError(format!(
                        "no clickable with index {} in current snapshot ({} available)",
                        idx,
                        ctx.clickables.len()
                    ))
                })?;
            tracing::debug!(
                target: "ras_tools::type_text",
                list_index = idx,
                backend_node_id = ?element.backend_node_id,
                "type_text focusing element before typing"
            );
            ctx.browser
                .click_node(&target, element.backend_node_id)
                .await?;
        }
        ctx.browser.type_text(&target, &p.text).await?;
        let display = if p.sensitive {
            "[redacted]".to_string()
        } else {
            p.text.chars().take(80).collect::<String>()
        };
        Ok(ActionResult::ok(format!("typed {display:?}")))
    }
}

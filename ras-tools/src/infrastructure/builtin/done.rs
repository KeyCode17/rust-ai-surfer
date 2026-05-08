use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout};
use serde::Deserialize;
use serde_json::json;

use crate::domain::action::{ActionMetadata, ToolHandler};
use crate::domain::registry::ToolContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct DoneAction;

#[derive(Deserialize)]
struct Params {
    text: String,
    #[serde(default)]
    files_to_display: Vec<String>,
}

#[async_trait]
impl ToolHandler for DoneAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("done".into()),
            description: "Mark the task complete and return the final answer.".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "files_to_display": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["text"]
            }),
            domain_filter: Vec::new(),
            terminates_sequence: true,
            timeout: ActionTimeout::default(),
        }
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: ToolContext,
    ) -> Result<ActionResult, AppError> {
        let p: Params = serde_json::from_value(params)
            .map_err(|e| AppError::ValidationError(format!("done params: {e}")))?;
        let mut r = ActionResult::done(p.text);
        for f in p.files_to_display {
            r = r.with_attachment(f);
        }
        Ok(r)
    }
}

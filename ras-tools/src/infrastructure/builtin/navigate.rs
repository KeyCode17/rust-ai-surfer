use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout};
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::domain::action::{ActionMetadata, ToolHandler};
use crate::domain::registry::ToolContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct NavigateAction;

#[derive(Deserialize)]
struct Params {
    url: String,
}

#[async_trait]
impl ToolHandler for NavigateAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("navigate".into()),
            description: "Navigate the focused tab to a URL.".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {"url": {"type": "string"}},
                "required": ["url"]
            }),
            domain_filter: Vec::new(),
            terminates_sequence: true,
            timeout: ActionTimeout::default(),
        }
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: ToolContext,
    ) -> Result<ActionResult, AppError> {
        let p: Params = serde_json::from_value(params)
            .map_err(|e| AppError::ValidationError(format!("navigate params: {e}")))?;
        let url = Url::parse(&p.url)
            .map_err(|e| AppError::ValidationError(format!("invalid url: {e}")))?;
        let target = ctx.browser.focused_target().await?;
        ctx.browser.navigate(&target, &url).await?;
        Ok(ActionResult::ok(format!("navigated to {url}")))
    }
}

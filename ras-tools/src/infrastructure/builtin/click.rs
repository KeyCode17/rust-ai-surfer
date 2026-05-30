use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout};
use serde::Deserialize;
use serde_json::json;

use crate::domain::action::{ActionMetadata, ToolHandler};
use crate::domain::registry::ToolContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct ClickElementAction;

#[derive(Deserialize)]
struct IndexParams {
    index: i64,
}

#[async_trait]
impl ToolHandler for ClickElementAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("click_element".into()),
            description: "Click the interactive element with the given index.".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {"index": {"type": "integer"}},
                "required": ["index"]
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
        let p: IndexParams = serde_json::from_value(params)
            .map_err(|e| AppError::ValidationError(format!("click params: {e}")))?;
        let element = ctx
            .clickables
            .iter()
            .find(|c| i64::from(c.index) == p.index)
            .ok_or_else(|| {
                AppError::ValidationError(format!(
                    "no clickable with index {} in current snapshot ({} available)",
                    p.index,
                    ctx.clickables.len()
                ))
            })?;
        tracing::debug!(
            target: "ras_tools::click",
            list_index = p.index,
            backend_node_id = ?element.backend_node_id,
            xpath = %element.xpath,
            "click_element resolved"
        );
        let target = ctx
            .target
            .clone()
            .ok_or_else(|| AppError::NotFound("no active target".into()))?;
        ctx.browser
            .click_node(&target, element.backend_node_id)
            .await?;
        Ok(ActionResult::ok(format!("clicked element {}", p.index)))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ClickCoordinateAction;

#[derive(Deserialize)]
struct CoordParams {
    x: i32,
    y: i32,
}

#[async_trait]
impl ToolHandler for ClickCoordinateAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("click_coordinate".into()),
            description: "Click at a screen coordinate (x, y in CSS pixels).".into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "x": {"type": "integer"},
                    "y": {"type": "integer"}
                },
                "required": ["x", "y"]
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
        let p: CoordParams = serde_json::from_value(params)
            .map_err(|e| AppError::ValidationError(format!("click_coord params: {e}")))?;
        let target = ctx
            .target
            .clone()
            .ok_or_else(|| AppError::NotFound("no active target".into()))?;
        ctx.browser.click_at(&target, p.x, p.y).await?;
        Ok(ActionResult::ok(format!("clicked at ({}, {})", p.x, p.y)))
    }
}

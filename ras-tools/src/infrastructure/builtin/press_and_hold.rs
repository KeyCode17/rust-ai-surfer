use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout};
use serde::Deserialize;
use serde_json::json;

use crate::domain::action::{ActionMetadata, ToolHandler};
use crate::domain::registry::ToolContext;

const DEFAULT_HOLD_MS: u64 = 12_000;
const MAX_HOLD_MS: u64 = 60_000;

fn clamp_hold(ms: u64) -> u64 {
    ms.clamp(100, MAX_HOLD_MS)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PressAndHoldCoordinateAction;

#[derive(Deserialize)]
struct CoordParams {
    x: i32,
    y: i32,
    #[serde(default)]
    ms: Option<u64>,
}

#[async_trait]
impl ToolHandler for PressAndHoldCoordinateAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("press_and_hold_coordinate".into()),
            description: "Press the left mouse button at (x, y) in CSS pixels, hold for `ms` \
                          milliseconds (default 12000, max 60000), then release. Designed for \
                          PerimeterX / HUMAN \"Press & Hold\" challenges. Uses CDP \
                          Input.dispatchMouseEvent so events are isTrusted=true."
                .into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "x":  {"type": "integer"},
                    "y":  {"type": "integer"},
                    "ms": {"type": "integer", "minimum": 100, "maximum": MAX_HOLD_MS}
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
            .map_err(|e| AppError::ValidationError(format!("press_and_hold_coordinate: {e}")))?;
        let hold = clamp_hold(p.ms.unwrap_or(DEFAULT_HOLD_MS));
        let target = ctx
            .target
            .clone()
            .ok_or_else(|| AppError::NotFound("no active target".into()))?;
        ctx.browser.mouse_hold(&target, p.x, p.y, hold).await?;
        Ok(ActionResult::ok(format!(
            "held ({}, {}) for {} ms (humanized: approach + jitter)",
            p.x, p.y, hold
        )))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PressAndHoldElementAction;

#[derive(Deserialize)]
struct IndexParams {
    index: i64,
    #[serde(default)]
    ms: Option<u64>,
}

#[async_trait]
impl ToolHandler for PressAndHoldElementAction {
    fn metadata(&self) -> ActionMetadata {
        ActionMetadata {
            name: ActionName("press_and_hold_element".into()),
            description: "Press-and-hold the interactive element with the given index. Resolves \
                          the element's bbox center, dispatches a real CDP mouse press, sleeps \
                          for `ms` (default 12000, max 60000), then releases. Use for PerimeterX \
                          / HUMAN \"Press & Hold\" buttons."
                .into(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "index": {"type": "integer"},
                    "ms":    {"type": "integer", "minimum": 100, "maximum": MAX_HOLD_MS}
                },
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
            .map_err(|e| AppError::ValidationError(format!("press_and_hold_element: {e}")))?;
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
        let cx = (element.bbox.x + element.bbox.width / 2.0).round() as i32;
        let cy = (element.bbox.y + element.bbox.height / 2.0).round() as i32;
        let hold = clamp_hold(p.ms.unwrap_or(DEFAULT_HOLD_MS));
        tracing::debug!(
            target: "ras_tools::press_and_hold",
            list_index = p.index,
            backend_node_id = ?element.backend_node_id,
            xpath = %element.xpath,
            x = cx,
            y = cy,
            ms = hold,
            "press_and_hold_element resolved"
        );
        let target = ctx
            .target
            .clone()
            .ok_or_else(|| AppError::NotFound("no active target".into()))?;
        ctx.browser.mouse_hold(&target, cx, cy, hold).await?;
        Ok(ActionResult::ok(format!(
            "held element {} at ({}, {}) for {} ms (humanized: approach + jitter)",
            p.index, cx, cy, hold
        )))
    }
}

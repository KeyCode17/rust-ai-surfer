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

fn parse_and_check(raw: &str) -> Result<Url, AppError> {
    let url =
        Url::parse(raw).map_err(|e| AppError::ValidationError(format!("invalid url: {e}")))?;
    ras_validation::EgressPolicy::default()
        .check(&url)
        .map_err(|e| AppError::ValidationError(format!("blocked by egress policy: {e:?}")))?;
    Ok(url)
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
        let url = parse_and_check(&p.url)?;
        let target = ctx
            .target
            .clone()
            .ok_or_else(|| AppError::NotFound("no active target".into()))?;
        ctx.browser.navigate(&target, &url).await?;
        Ok(ActionResult::ok(format!("navigated to {url}")))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_and_check;

    #[test]
    fn allows_public_https() {
        assert!(parse_and_check("https://example.com/").is_ok());
    }

    #[test]
    fn blocks_metadata_ip() {
        assert!(parse_and_check("http://169.254.169.254/latest/meta-data/").is_err());
    }

    #[test]
    fn blocks_localhost() {
        assert!(parse_and_check("http://localhost:9222/json").is_err());
    }

    #[test]
    fn blocks_file_scheme() {
        assert!(parse_and_check("file:///etc/passwd").is_err());
    }

    #[test]
    fn rejects_garbage_url() {
        assert!(parse_and_check("not a url").is_err());
    }
}

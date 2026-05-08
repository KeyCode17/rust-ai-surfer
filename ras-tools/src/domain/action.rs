use std::sync::Arc;

use async_trait::async_trait;
use ras_errors::AppError;
use ras_types::{ActionName, ActionResult, ActionTimeout, DomainPattern};
use serde::{Deserialize, Serialize};

use crate::domain::registry::ToolContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    pub name: ActionName,
    pub description: String,
    pub parameters_schema: serde_json::Value,
    pub domain_filter: Vec<DomainPattern>,
    pub terminates_sequence: bool,
    pub timeout: ActionTimeout,
}

#[async_trait]
pub trait ToolHandler: Send + Sync + 'static {
    fn metadata(&self) -> ActionMetadata;
    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: ToolContext,
    ) -> Result<ActionResult, AppError>;
}

#[derive(Clone)]
pub struct RegisteredAction {
    pub metadata: ActionMetadata,
    pub handler: Arc<dyn ToolHandler>,
}

impl std::fmt::Debug for RegisteredAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredAction")
            .field("metadata", &self.metadata)
            .finish()
    }
}

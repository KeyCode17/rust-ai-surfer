use std::sync::Arc;

use indexmap::IndexMap;
use ras_cdp::BrowserPort;
use ras_errors::AppError;
use ras_events::EventBus;
use ras_types::ActionName;
use url::Url;

use crate::domain::action::{RegisteredAction, ToolHandler};

#[derive(Clone)]
pub struct ToolContext {
    pub browser: Arc<dyn BrowserPort>,
    pub events: Arc<dyn EventBus>,
    pub page_url: Option<Url>,
    pub available_files: Vec<String>,
}

impl std::fmt::Debug for ToolContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolContext")
            .field("page_url", &self.page_url)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct ActionRegistry {
    actions: IndexMap<ActionName, RegisteredAction>,
}

impl ActionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: Arc<dyn ToolHandler>) -> Result<(), AppError> {
        let metadata = handler.metadata();
        let name = metadata.name.clone();
        if self.actions.contains_key(&name) {
            return Err(AppError::Conflict(format!(
                "action already registered: {}",
                name.0
            )));
        }
        self.actions
            .insert(name, RegisteredAction { metadata, handler });
        Ok(())
    }

    pub fn exclude(&mut self, name: &ActionName) {
        self.actions.shift_remove(name);
    }

    #[must_use]
    pub fn get(&self, name: &ActionName) -> Option<&RegisteredAction> {
        self.actions.get(name)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, ActionName, RegisteredAction> {
        self.actions.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

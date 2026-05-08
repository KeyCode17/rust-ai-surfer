pub mod domain;
pub mod application;
pub mod infrastructure;

pub use domain::action::{ActionMetadata, RegisteredAction, ToolHandler};
pub use domain::registry::{ActionRegistry, ToolContext};
pub use infrastructure::builtin::register::register_default_actions;

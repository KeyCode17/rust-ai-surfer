pub mod domain;
pub mod application;
pub mod infrastructure;

pub use domain::repository::{McpClientPort, McpServerPort};
pub use domain::tool::{McpContent, McpToolDefinition, McpToolResult};

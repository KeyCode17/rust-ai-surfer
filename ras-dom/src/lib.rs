pub mod application;
pub mod domain;
pub mod infrastructure;

pub use domain::clickable::{ClickableElement, PaintOrderRect};
pub use domain::node::{BoundingBox, EnhancedDomTreeNode, NodeAttributes, NodeKind};
pub use domain::repository::{DomExtractor, HighlightOptions};
pub use domain::state_summary::{BrowserStateSummary, PageStatistics, TabInfo};
pub use infrastructure::chromiumoxide::ChromiumoxideDomExtractor;

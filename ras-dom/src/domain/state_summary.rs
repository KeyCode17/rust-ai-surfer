use ras_types::TargetId;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::clickable::ClickableElement;
use crate::domain::node::EnhancedDomTreeNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateSummary {
    pub target: TargetId,
    pub url: Url,
    pub title: String,
    pub tree: Option<EnhancedDomTreeNode>,
    pub clickables: Vec<ClickableElement>,
    pub screenshot_b64: Option<String>,
    pub tabs: Vec<TabInfo>,
    pub page_stats: PageStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub target: TargetId,
    pub url: Url,
    pub title: String,
    pub focused: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageStatistics {
    pub total_elements: u32,
    pub visible_elements: u32,
    pub text_chars: u32,
    pub is_skeleton: bool,
}

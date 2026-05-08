use ras_types::BackendNodeId;
use serde::{Deserialize, Serialize};

use crate::domain::node::BoundingBox;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickableElement {
    pub index: u32,
    pub backend_node_id: BackendNodeId,
    pub bbox: BoundingBox,
    pub xpath: String,
    pub stable_hash: String,
    pub ax_name: Option<String>,
    pub tag: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PaintOrderRect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl PaintOrderRect {
    #[must_use]
    pub fn from_bbox(b: BoundingBox) -> Self {
        Self {
            x1: b.x,
            y1: b.y,
            x2: b.x + b.width,
            y2: b.y + b.height,
        }
    }

    #[must_use]
    pub fn covers(self, other: Self) -> bool {
        self.x1 <= other.x1 && self.y1 <= other.y1 && self.x2 >= other.x2 && self.y2 >= other.y2
    }
}

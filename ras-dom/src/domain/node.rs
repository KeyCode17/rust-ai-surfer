use indexmap::IndexMap;
use ras_types::BackendNodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Element,
    Text,
    Document,
    DocumentFragment,
    Comment,
    Other,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeAttributes {
    pub map: IndexMap<String, String>,
}

impl NodeAttributes {
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(String::as_str)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDomTreeNode {
    pub backend_node_id: BackendNodeId,
    pub kind: NodeKind,
    pub tag_name: Option<String>,
    pub attributes: NodeAttributes,
    pub text: Option<String>,
    pub xpath: String,
    pub ax_role: Option<String>,
    pub ax_name: Option<String>,
    pub is_clickable: bool,
    pub is_visible: bool,
    pub bbox: Option<BoundingBox>,
    pub children: Vec<EnhancedDomTreeNode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

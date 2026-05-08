use sha2::{Digest, Sha256};

use crate::application::dynamic_class_filter::filter_dynamic_classes;
use crate::domain::node::EnhancedDomTreeNode;

#[must_use]
pub fn stable_hash(node: &EnhancedDomTreeNode, parent_xpath: &str) -> String {
    let tag = node.tag_name.as_deref().unwrap_or("");
    let class_attr = node.attributes.get("class").unwrap_or("");
    let classes = filter_dynamic_classes(class_attr);
    let id = node.attributes.get("id").unwrap_or("");
    let role = node.attributes.get("role").unwrap_or("");
    let ax_name = node.ax_name.as_deref().unwrap_or("");
    let mut h = Sha256::new();
    h.update(parent_xpath.as_bytes());
    h.update(b"|");
    h.update(tag.as_bytes());
    h.update(b"|");
    h.update(id.as_bytes());
    h.update(b"|");
    h.update(role.as_bytes());
    h.update(b"|");
    h.update(classes.join(" ").as_bytes());
    h.update(b"|");
    h.update(ax_name.as_bytes());
    format!("{:x}", h.finalize())
}

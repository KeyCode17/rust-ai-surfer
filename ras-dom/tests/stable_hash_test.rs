use indexmap::IndexMap;
use ras_dom::application::stable_hash::stable_hash;
use ras_dom::domain::node::{EnhancedDomTreeNode, NodeAttributes, NodeKind};
use ras_types::BackendNodeId;

fn node(class: &str, ax: Option<&str>) -> EnhancedDomTreeNode {
    let mut map = IndexMap::new();
    map.insert("class".into(), class.into());
    map.insert("id".into(), "submit".into());
    EnhancedDomTreeNode {
        backend_node_id: BackendNodeId(1),
        kind: NodeKind::Element,
        tag_name: Some("button".into()),
        attributes: NodeAttributes { map },
        text: None,
        xpath: "/html/body/button".into(),
        ax_role: Some("button".into()),
        ax_name: ax.map(String::from),
        is_clickable: true,
        is_visible: true,
        bbox: None,
        children: Vec::new(),
    }
}

#[test]
fn hash_stable_across_dynamic_class_swap() {
    let a = stable_hash(&node("primary is-active css-12ab34", Some("Submit")), "/html/body");
    let b = stable_hash(&node("primary hover sc-abcd1234", Some("Submit")), "/html/body");
    assert_eq!(a, b);
}

#[test]
fn hash_changes_with_ax_name() {
    let a = stable_hash(&node("primary", Some("Submit")), "/html/body");
    let b = stable_hash(&node("primary", Some("Cancel")), "/html/body");
    assert_ne!(a, b);
}

#[test]
fn hash_changes_with_parent_xpath() {
    let a = stable_hash(&node("primary", None), "/html/body");
    let b = stable_hash(&node("primary", None), "/html/body/section");
    assert_ne!(a, b);
}

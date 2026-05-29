use std::collections::HashMap;

use chromiumoxide::cdp::browser_protocol::dom_snapshot::{
    DocumentSnapshot, NodeTreeSnapshot, StringIndex,
};
use ras_types::BackendNodeId;

use crate::domain::clickable::ClickableElement;
use crate::domain::node::BoundingBox;
use crate::infrastructure::chromiumoxide::clickable_naming::{
    collapse_ws, derive_ax_name, derive_label, icon_name, onclick_name, role_value,
};
use crate::infrastructure::chromiumoxide::snapshot_parser::{
    decode_attrs, is_clickable, lookup_index,
};

const TEXT_CLIMB_DEPTH: usize = 6;
const TEXT_NODE_TYPE: i64 = 3;

pub(crate) fn extract_clickables(
    doc: &DocumentSnapshot,
    strings: &[String],
    layout_bbox: &HashMap<i64, BoundingBox>,
) -> Vec<ClickableElement> {
    let nodes = &doc.nodes;
    let Some(node_names) = &nodes.node_name else {
        return Vec::new();
    };
    let backend_ids = nodes.backend_node_id.as_ref();
    let attrs = nodes.attributes.as_ref();
    let n = node_names.len();

    let empty: Vec<StringIndex> = Vec::new();
    let tags: Vec<String> = node_names
        .iter()
        .map(|name| lookup_index(strings, *name.inner()).to_lowercase())
        .collect();
    let attr_pairs: Vec<Vec<(String, String)>> = (0..n)
        .map(|i| {
            let idxs = attrs
                .and_then(|a| a.get(i))
                .map(|a| a.inner().as_slice())
                .unwrap_or(&empty);
            decode_attrs(idxs, strings)
        })
        .collect();

    let mut clickable = vec![false; n];
    if let Some(rb) = &nodes.is_clickable {
        for idx in &rb.index {
            if *idx >= 0 && (*idx as usize) < n {
                clickable[*idx as usize] = true;
            }
        }
    }
    for i in 0..n {
        clickable[i] = clickable[i] || is_clickable(&tags[i], &attr_pairs[i]);
    }

    let text_map = build_text_map(nodes, strings, &clickable, n);

    let mut out = Vec::new();
    let mut clickable_index: u32 = 0;
    for i in 0..n {
        if !clickable[i] {
            continue;
        }
        let pairs = &attr_pairs[i];
        let backend_node_id = backend_ids
            .and_then(|b| b.get(i))
            .map(|v| BackendNodeId(*v.inner()))
            .unwrap_or(BackendNodeId(0));
        let bbox = layout_bbox
            .get(&(i as i64))
            .copied()
            .unwrap_or(BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            });
        let ax_name = derive_ax_name(pairs)
            .or_else(|| text_map.get(&i).cloned())
            .or_else(|| onclick_name(pairs))
            .or_else(|| icon_name(pairs))
            .or_else(|| role_value(pairs));
        let label = derive_label(pairs);
        let tag = role_value(pairs).unwrap_or_else(|| tags[i].clone());
        out.push(ClickableElement {
            index: clickable_index,
            backend_node_id,
            bbox,
            xpath: format!("//*[backendNodeId={}]", backend_node_id.0),
            stable_hash: String::new(),
            ax_name,
            tag,
            label,
        });
        clickable_index += 1;
    }
    out
}

/// Join visible text descendants to their nearest clickable ancestor, so anchors like
/// `<a><span>Home</span></a>` surface "Home" even though the text lives in a child node.
fn build_text_map(
    nodes: &NodeTreeSnapshot,
    strings: &[String],
    clickable: &[bool],
    n: usize,
) -> HashMap<usize, String> {
    let mut map: HashMap<usize, String> = HashMap::new();
    let (Some(parents), Some(types), Some(values)) =
        (&nodes.parent_index, &nodes.node_type, &nodes.node_value)
    else {
        return map;
    };
    for i in 0..n {
        if types.get(i).copied() != Some(TEXT_NODE_TYPE) {
            continue;
        }
        let raw = values.get(i).map(|v| lookup_index(strings, *v.inner()));
        let text = raw.as_deref().map(str::trim).unwrap_or("");
        if text.is_empty() {
            continue;
        }
        let mut cur = i;
        for _ in 0..TEXT_CLIMB_DEPTH {
            let parent = parents.get(cur).copied().unwrap_or(-1);
            if parent < 0 || parent as usize >= n {
                break;
            }
            let parent = parent as usize;
            if clickable[parent] {
                let entry = map.entry(parent).or_default();
                if !entry.is_empty() {
                    entry.push(' ');
                }
                entry.push_str(&collapse_ws(text));
                break;
            }
            cur = parent;
        }
    }
    map
}

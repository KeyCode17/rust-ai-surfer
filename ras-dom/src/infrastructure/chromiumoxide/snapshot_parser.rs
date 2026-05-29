use chromiumoxide::cdp::browser_protocol::dom_snapshot::{
    CaptureSnapshotReturns, DocumentSnapshot, NodeTreeSnapshot, StringIndex,
};
use url::Url;

use crate::domain::clickable::ClickableElement;
use crate::domain::node::BoundingBox;
use crate::domain::state_summary::PageStatistics;
use crate::infrastructure::chromiumoxide::clickables::extract_clickables;

const CLICKABLE_TAGS: &[&str] = &[
    "a", "button", "input", "select", "textarea", "summary", "label", "details",
];

pub(crate) fn parse_snapshot(
    resp: &CaptureSnapshotReturns,
) -> Result<(Url, String, Vec<ClickableElement>, PageStatistics), String> {
    let strings = &resp.strings;
    let doc = resp
        .documents
        .first()
        .ok_or_else(|| "no documents in snapshot".to_string())?;
    let url_str = lookup_index(strings, *doc.document_url.inner());
    let url = Url::parse(&url_str)
        .or_else(|_| Url::parse("about:blank"))
        .map_err(|e| format!("url parse: {e}"))?;
    let title = lookup_index(strings, *doc.title.inner());

    let layout_bbox = build_layout_index(doc);
    let clickables = extract_clickables(doc, strings, &layout_bbox);
    let page_stats = build_page_stats(&doc.nodes, &clickables);

    Ok((url, title, clickables, page_stats))
}

pub(crate) fn lookup_index(strings: &[String], idx: i64) -> String {
    if idx >= 0 && (idx as usize) < strings.len() {
        strings[idx as usize].clone()
    } else {
        String::new()
    }
}

fn build_layout_index(doc: &DocumentSnapshot) -> std::collections::HashMap<i64, BoundingBox> {
    let mut map = std::collections::HashMap::new();
    let layout = &doc.layout;
    for (i, node_idx) in layout.node_index.iter().enumerate() {
        if let Some(rect) = layout.bounds.get(i) {
            let inner = rect.inner();
            if inner.len() == 4 {
                map.insert(
                    *node_idx,
                    BoundingBox {
                        x: inner[0] as f32,
                        y: inner[1] as f32,
                        width: inner[2] as f32,
                        height: inner[3] as f32,
                    },
                );
            }
        }
    }
    map
}

pub(crate) fn decode_attrs(attr_idxs: &[StringIndex], strings: &[String]) -> Vec<(String, String)> {
    let mut pairs = Vec::with_capacity(attr_idxs.len() / 2);
    let mut i = 0;
    while i + 1 < attr_idxs.len() {
        pairs.push((
            lookup_index(strings, *attr_idxs[i].inner()),
            lookup_index(strings, *attr_idxs[i + 1].inner()),
        ));
        i += 2;
    }
    pairs
}

pub(crate) fn is_clickable(tag: &str, attrs: &[(String, String)]) -> bool {
    if CLICKABLE_TAGS.contains(&tag) {
        return true;
    }
    for (k, _) in attrs {
        if matches!(
            k.as_str(),
            "onclick" | "tabindex" | "role" | "aria-pressed" | "aria-checked"
        ) {
            return true;
        }
    }
    false
}

fn build_page_stats(nodes: &NodeTreeSnapshot, clickables: &[ClickableElement]) -> PageStatistics {
    let total = nodes.node_name.as_ref().map(|v| v.len()).unwrap_or(0) as u32;
    let visible = clickables
        .iter()
        .filter(|c| c.bbox.width > 0.0 && c.bbox.height > 0.0)
        .count() as u32;
    PageStatistics {
        total_elements: total,
        visible_elements: visible,
        text_chars: 0,
        is_skeleton: total < 5,
    }
}

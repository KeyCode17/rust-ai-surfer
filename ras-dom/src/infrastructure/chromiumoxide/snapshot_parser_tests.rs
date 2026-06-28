use chromiumoxide::cdp::browser_protocol::dom::BackendNodeId;
use chromiumoxide::cdp::browser_protocol::dom_snapshot::{
    CaptureSnapshotReturns, DocumentSnapshot, LayoutTreeSnapshot, NodeTreeSnapshot,
    RareBooleanData, StringIndex, TextBoxSnapshot,
};

use crate::infrastructure::chromiumoxide::snapshot_parser::parse_snapshot;

fn string_table() -> Vec<String> {
    ["", "html", "frameset", "frame", "button", "input", "a"]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

fn empty_layout() -> LayoutTreeSnapshot {
    LayoutTreeSnapshot {
        node_index: Vec::new(),
        styles: Vec::new(),
        bounds: Vec::new(),
        text: Vec::new(),
        stacking_contexts: RareBooleanData { index: Vec::new() },
        paint_orders: None,
        offset_rects: None,
        scroll_rects: None,
        client_rects: None,
        blended_background_colors: None,
        text_color_opacities: None,
    }
}

fn document(name_indices: &[i64], backend_ids: &[i64]) -> DocumentSnapshot {
    let nodes = NodeTreeSnapshot {
        node_name: Some(name_indices.iter().copied().map(StringIndex::new).collect()),
        backend_node_id: Some(
            backend_ids
                .iter()
                .copied()
                .map(BackendNodeId::new)
                .collect(),
        ),
        ..Default::default()
    };
    DocumentSnapshot {
        document_url: StringIndex::new(0),
        title: StringIndex::new(0),
        base_url: StringIndex::new(0),
        content_language: StringIndex::new(0),
        encoding_name: StringIndex::new(0),
        public_id: StringIndex::new(0),
        system_id: StringIndex::new(0),
        frame_id: StringIndex::new(0),
        nodes,
        layout: empty_layout(),
        text_boxes: TextBoxSnapshot::new(Vec::new(), Vec::new(), Vec::new(), Vec::new()),
        scroll_offset_x: None,
        scroll_offset_y: None,
        content_width: None,
        content_height: None,
    }
}

#[test]
fn aggregates_clickables_across_frame_documents() -> Result<(), String> {
    let shell = document(&[1, 2, 3, 3], &[10, 11, 12, 13]);
    let frame_a = document(&[4], &[101]);
    let frame_b = document(&[5], &[102]);
    let resp = CaptureSnapshotReturns::new(vec![shell, frame_a, frame_b], string_table());

    let (url, _title, clickables, stats) = parse_snapshot(&resp)?;

    assert_eq!(url.as_str(), "about:blank");
    assert_eq!(clickables.len(), 2);
    assert_eq!(clickables[0].index, 0);
    assert_eq!(clickables[1].index, 1);
    let ids: Vec<i64> = clickables.iter().map(|c| c.backend_node_id.0).collect();
    assert_eq!(ids, vec![101, 102]);
    assert_eq!(stats.total_elements, 6);
    Ok(())
}

#[test]
fn single_document_behavior_unchanged() -> Result<(), String> {
    let only = document(&[6, 4], &[201, 202]);
    let resp = CaptureSnapshotReturns::new(vec![only], string_table());

    let (_url, _title, clickables, stats) = parse_snapshot(&resp)?;

    assert_eq!(clickables.len(), 2);
    assert_eq!(clickables[0].index, 0);
    assert_eq!(clickables[1].index, 1);
    assert_eq!(clickables[0].backend_node_id.0, 201);
    assert_eq!(clickables[1].backend_node_id.0, 202);
    assert_eq!(stats.total_elements, 2);
    Ok(())
}

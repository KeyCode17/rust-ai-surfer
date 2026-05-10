use chromiumoxide::cdp::browser_protocol::dom_snapshot::CaptureSnapshotParams;

#[test]
fn snapshot_params_serialize_includes_computed_styles_field() {
    let params = CaptureSnapshotParams::builder()
        .computed_styles(["display".to_string()])
        .include_paint_order(true)
        .include_dom_rects(true)
        .build()
        .expect("build");
    let json = serde_json::to_value(&params).expect("serialize");
    assert!(
        json.get("computedStyles").is_some(),
        "computedStyles must appear in JSON or CDP rejects with -32602: {json:?}"
    );
    let arr = json
        .get("computedStyles")
        .and_then(|v| v.as_array())
        .expect("computedStyles array");
    assert_eq!(
        arr.len(),
        1,
        "expected one entry to defeat skip_serializing_if"
    );
    assert_eq!(arr[0], "display");
}

#[test]
fn snapshot_params_with_empty_computed_styles_drops_field_on_wire() {
    let params = CaptureSnapshotParams::builder()
        .computed_styles(std::iter::empty::<String>())
        .include_paint_order(true)
        .include_dom_rects(true)
        .build()
        .expect("build");
    let json = serde_json::to_value(&params).expect("serialize");
    assert!(
        json.get("computedStyles").is_none(),
        "regression sentinel: chromiumoxide_cdp 0.6.0 has \
         #[serde(skip_serializing_if = \"Vec::is_empty\")] on computedStyles, \
         which drops the field for the empty case and triggers CDP -32602. \
         If this assertion ever flips, the upstream bug is fixed and the \
         workaround in snapshot.rs can revert to std::iter::empty."
    );
}

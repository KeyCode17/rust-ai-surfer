use ras_dom::application::paint_order::remove_occluded;
use ras_dom::domain::node::BoundingBox;

fn b(x: f32, y: f32, w: f32, h: f32) -> BoundingBox {
    BoundingBox {
        x,
        y,
        width: w,
        height: h,
    }
}

#[test]
fn fully_covered_box_is_dropped() {
    let boxes = vec![b(10.0, 10.0, 50.0, 50.0), b(0.0, 0.0, 100.0, 100.0)];
    let kept = remove_occluded(&boxes);
    assert!(kept.contains(&1));
    assert!(!kept.contains(&0));
}

#[test]
fn disjoint_boxes_all_kept() {
    let boxes = vec![b(0.0, 0.0, 50.0, 50.0), b(60.0, 0.0, 50.0, 50.0)];
    let kept = remove_occluded(&boxes);
    assert_eq!(kept, vec![0, 1]);
}

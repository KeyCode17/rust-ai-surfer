use crate::domain::clickable::PaintOrderRect;
use crate::domain::node::BoundingBox;

#[must_use]
pub fn remove_occluded(boxes: &[BoundingBox]) -> Vec<usize> {
    let mut visible = Vec::with_capacity(boxes.len());
    let mut painted: Vec<PaintOrderRect> = Vec::new();
    for (idx, b) in boxes.iter().enumerate().rev() {
        let rect = PaintOrderRect::from_bbox(*b);
        if !painted.iter().any(|p| p.covers(rect)) {
            visible.push(idx);
            painted.push(rect);
        }
    }
    visible.reverse();
    visible
}

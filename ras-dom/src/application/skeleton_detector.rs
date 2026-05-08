use crate::domain::state_summary::PageStatistics;

#[must_use]
pub fn detect_skeleton(stats: &PageStatistics) -> bool {
    stats.total_elements > 20 && stats.text_chars < stats.total_elements * 5
}

pub fn annotate(stats: &mut PageStatistics) {
    stats.is_skeleton = detect_skeleton(stats);
}

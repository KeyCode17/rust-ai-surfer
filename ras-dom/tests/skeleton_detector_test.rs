use ras_dom::application::skeleton_detector::detect_skeleton;
use ras_dom::domain::state_summary::PageStatistics;

#[test]
fn flags_high_element_low_text_pages() {
    let s = PageStatistics {
        total_elements: 50,
        visible_elements: 50,
        text_chars: 100,
        is_skeleton: false,
    };
    assert!(detect_skeleton(&s));
}

#[test]
fn passes_normal_pages() {
    let s = PageStatistics {
        total_elements: 50,
        visible_elements: 50,
        text_chars: 1000,
        is_skeleton: false,
    };
    assert!(!detect_skeleton(&s));
}

#[test]
fn passes_small_pages() {
    let s = PageStatistics {
        total_elements: 5,
        visible_elements: 5,
        text_chars: 0,
        is_skeleton: false,
    };
    assert!(!detect_skeleton(&s));
}

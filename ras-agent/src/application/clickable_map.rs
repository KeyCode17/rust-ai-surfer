use ras_dom::BrowserStateSummary;

const CLICKABLE_LIMIT: usize = 80;
const NAME_BUDGET: usize = 80;

pub(crate) fn render_clickable_map(summary: &BrowserStateSummary) -> String {
    if summary.clickables.is_empty() {
        return String::new();
    }
    let mut buf = String::from("clickable_elements:\n");
    for c in summary.clickables.iter().take(CLICKABLE_LIMIT) {
        buf.push_str(&format!("  [{}] {}", c.index, c.tag));
        if let Some(name) = &c.ax_name {
            buf.push_str(&format!(" \"{}\"", truncate(name, NAME_BUDGET)));
        } else if let Some(label) = &c.label {
            buf.push_str(&format!(" \"{}\"", truncate(label, NAME_BUDGET)));
        }
        buf.push('\n');
    }
    if summary.clickables.len() > CLICKABLE_LIMIT {
        buf.push_str(&format!(
            "  …and {} more (truncated)\n",
            summary.clickables.len() - CLICKABLE_LIMIT
        ));
    }
    buf
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ras_dom::{BoundingBox, ClickableElement, PageStatistics};
    use ras_types::{BackendNodeId, TargetId};

    fn summary_with(clickables: Vec<ClickableElement>) -> BrowserStateSummary {
        BrowserStateSummary {
            target: TargetId("mock".into()),
            url: "https://example.com/".parse().expect("url"),
            title: "T".into(),
            tree: None,
            clickables,
            screenshot_b64: None,
            tabs: vec![],
            page_stats: PageStatistics::default(),
        }
    }

    fn click(idx: u32, tag: &str, ax_name: Option<&str>, label: Option<&str>) -> ClickableElement {
        ClickableElement {
            index: idx,
            backend_node_id: BackendNodeId(idx as i64),
            bbox: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            xpath: String::new(),
            stable_hash: String::new(),
            ax_name: ax_name.map(String::from),
            tag: tag.into(),
            label: label.map(String::from),
        }
    }

    #[test]
    fn empty_clickables_yields_empty_string() {
        assert_eq!(render_clickable_map(&summary_with(vec![])), "");
    }

    #[test]
    fn ax_name_takes_precedence_over_label() {
        let s = summary_with(vec![click(0, "button", Some("Sign in"), Some("submit"))]);
        let out = render_clickable_map(&s);
        assert!(out.contains("[0] button \"Sign in\""));
        assert!(!out.contains("submit"));
    }

    #[test]
    fn label_used_when_no_ax_name() {
        let s = summary_with(vec![click(0, "input", None, Some("user@example.com"))]);
        let out = render_clickable_map(&s);
        assert!(out.contains("[0] input \"user@example.com\""));
    }

    #[test]
    fn no_quotes_when_neither_ax_nor_label() {
        let s = summary_with(vec![click(0, "a", None, None)]);
        let out = render_clickable_map(&s);
        assert!(out.contains("[0] a\n"));
        assert!(!out.contains('"'));
    }

    #[test]
    fn truncates_beyond_limit_with_more_marker() {
        let many: Vec<ClickableElement> = (0..(CLICKABLE_LIMIT as u32 + 5))
            .map(|i| click(i, "div", None, None))
            .collect();
        let s = summary_with(many);
        let out = render_clickable_map(&s);
        assert!(out.contains(&format!("[{}]", CLICKABLE_LIMIT - 1)));
        assert!(!out.contains(&format!("[{}]", CLICKABLE_LIMIT)));
        assert!(out.contains("…and 5 more (truncated)"));
    }
}

use ras_llm::{ChatMessage, ContentPart};

use crate::domain::agent_history::StepRecord;

const TEXT_BUDGET: usize = 480;
const ERROR_BUDGET: usize = 240;
const SCREENSHOT_MEDIA_TYPE: &str = "image/png";

pub(crate) fn render_step_message(step: &StepRecord) -> Option<ChatMessage> {
    if step.results.is_empty() {
        return None;
    }
    let mut text = format!("Step {} result:\n", step.step.0);
    if let Some(url) = &step.url {
        text.push_str(&format!("url: {url}\n"));
    }
    text.push_str("action results:\n");
    for (i, r) in step.results.iter().enumerate() {
        text.push_str(&format!("  [{i}]"));
        if r.is_done {
            text.push_str(" done");
        }
        if let Some(err) = &r.error {
            text.push_str(&format!(" error: {}", truncate(err, ERROR_BUDGET)));
        } else if let Some(c) = &r.extracted_content {
            text.push_str(&format!(" {}", truncate(c, TEXT_BUDGET)));
        }
        text.push('\n');
    }

    let mut parts: Vec<ContentPart> = Vec::with_capacity(1 + image_count(step));
    parts.push(ContentPart::Text { text });
    for r in &step.results {
        for img_b64 in &r.images {
            parts.push(ContentPart::ImageBase64 {
                media_type: SCREENSHOT_MEDIA_TYPE.into(),
                data: img_b64.clone(),
            });
        }
    }
    Some(ChatMessage::user_parts(parts))
}

fn image_count(step: &StepRecord) -> usize {
    step.results.iter().map(|r| r.images.len()).sum()
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push_str("…");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ras_llm::ChatMessage;
    use ras_types::{ActionResult, StepId};

    use crate::domain::agent_output::{AgentBrain, AgentOutput};
    use crate::domain::step_metadata::StepMetadata;

    fn step(results: Vec<ActionResult>) -> StepRecord {
        StepRecord {
            step: StepId(7),
            started_at: Utc::now(),
            url: Some("https://example.com/login".parse().expect("test url")),
            output: AgentOutput {
                current_state: AgentBrain {
                    evaluation_previous_goal: String::new(),
                    memory: String::new(),
                    next_goal: String::new(),
                },
                action: vec![],
                plan: None,
                current_plan_item: None,
            },
            results,
            metadata: StepMetadata::default(),
            summary: None,
        }
    }

    #[test]
    fn empty_results_yields_no_message() {
        assert!(render_step_message(&step(vec![])).is_none());
    }

    #[test]
    fn text_only_result_emits_text_part_only() {
        let r = ActionResult::ok("clicked login button");
        let msg = render_step_message(&step(vec![r])).expect("msg");
        let ChatMessage::User(u) = msg else {
            panic!("expected user");
        };
        assert_eq!(u.content.len(), 1);
        match &u.content[0] {
            ContentPart::Text { text } => {
                assert!(text.contains("Step 7 result:"));
                assert!(text.contains("url: https://example.com/login"));
                assert!(text.contains("clicked login button"));
            }
            other => panic!("expected text part, got {other:?}"),
        }
    }

    #[test]
    fn screenshot_result_emits_text_plus_image_part() {
        let r = ActionResult::ok("captured screenshot").with_image("AAAA");
        let msg = render_step_message(&step(vec![r])).expect("msg");
        let ChatMessage::User(u) = msg else {
            panic!("expected user");
        };
        assert_eq!(u.content.len(), 2);
        assert!(matches!(&u.content[0], ContentPart::Text { .. }));
        match &u.content[1] {
            ContentPart::ImageBase64 { media_type, data } => {
                assert_eq!(media_type, "image/png");
                assert_eq!(data, "AAAA");
            }
            other => panic!("expected image part, got {other:?}"),
        }
    }

    #[test]
    fn multiple_images_across_results_all_attached() {
        let r1 = ActionResult::ok("step a").with_image("AAAA");
        let r2 = ActionResult::ok("step b")
            .with_image("BBBB")
            .with_image("CCCC");
        let msg = render_step_message(&step(vec![r1, r2])).expect("msg");
        let ChatMessage::User(u) = msg else {
            panic!("expected user");
        };
        let images: Vec<&String> = u
            .content
            .iter()
            .filter_map(|p| match p {
                ContentPart::ImageBase64 { data, .. } => Some(data),
                _ => None,
            })
            .collect();
        assert_eq!(images, vec!["AAAA", "BBBB", "CCCC"]);
    }

    #[test]
    fn error_result_emits_error_in_text() {
        let r = ActionResult::err("element not found");
        let msg = render_step_message(&step(vec![r])).expect("msg");
        let ChatMessage::User(u) = msg else {
            panic!("expected user");
        };
        match &u.content[0] {
            ContentPart::Text { text } => assert!(text.contains("error: element not found")),
            _ => panic!("expected text"),
        }
    }
}

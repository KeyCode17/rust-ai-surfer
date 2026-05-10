use ras_errors::AppError;
use ras_llm::ChatResponse;

use crate::domain::agent_output::{ActionInvocation, AgentBrain, AgentOutput};

pub(crate) fn parse_agent_output(response: &ChatResponse) -> Result<AgentOutput, AppError> {
    if let Some(content) = &response.content {
        if let Ok(parsed) = serde_json::from_str::<AgentOutput>(content) {
            return Ok(parsed);
        }
        let cleaned = strip_code_fence(content);
        if cleaned.as_ptr() != content.as_ptr()
            && let Ok(parsed) = serde_json::from_str::<AgentOutput>(cleaned)
        {
            return Ok(parsed);
        }
    }
    Ok(AgentOutput {
        current_state: AgentBrain {
            evaluation_previous_goal: String::new(),
            memory: String::new(),
            next_goal: response.content.clone().unwrap_or_default(),
        },
        action: tool_calls_to_actions(&response.tool_calls),
        plan: None,
        current_plan_item: None,
    })
}

fn tool_calls_to_actions(calls: &[ras_llm::ToolCall]) -> Vec<ActionInvocation> {
    calls
        .iter()
        .map(|c| ActionInvocation {
            name: ras_types::ActionName(c.name.clone().into()),
            parameters: c.arguments.clone(),
        })
        .collect()
}

fn strip_code_fence(s: &str) -> &str {
    let trimmed = s.trim();
    let opens = ["```json", "```JSON", "```Json", "```"];
    let after_open = opens
        .iter()
        .find_map(|tag| trimmed.strip_prefix(tag))
        .unwrap_or(trimmed)
        .trim_start_matches('\n')
        .trim_start();
    let body = after_open
        .strip_suffix("```")
        .unwrap_or(after_open)
        .trim_end()
        .trim_end_matches('\n');
    let out = body.trim();
    if out.as_ptr() == s.as_ptr() && out.len() == s.len() {
        s
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ras_llm::{FinishReason, Usage};

    fn resp(content: &str) -> ChatResponse {
        ChatResponse {
            content: Some(content.into()),
            tool_calls: vec![],
            usage: Usage::default(),
            model: "test".into(),
            finish_reason: FinishReason::Stop,
        }
    }

    const VALID: &str = r#"{"current_state":{"evaluation_previous_goal":"","memory":"","next_goal":"go"},"action":[{"name":"navigate","parameters":{"url":"https://example.com/"}}]}"#;

    #[test]
    fn unfenced_json_parses() {
        let out = parse_agent_output(&resp(VALID)).expect("parse");
        assert_eq!(out.action.len(), 1);
    }

    #[test]
    fn fenced_with_json_tag_and_newline() {
        let body = format!("```json\n{VALID}\n```");
        let out = parse_agent_output(&resp(&body)).expect("parse");
        assert_eq!(out.action.len(), 1);
    }

    #[test]
    fn fenced_with_uppercase_json_tag() {
        let body = format!("```JSON\n{VALID}\n```");
        let out = parse_agent_output(&resp(&body)).expect("parse");
        assert_eq!(out.action.len(), 1);
    }

    #[test]
    fn fenced_with_titlecase_json_tag() {
        let body = format!("```Json\n{VALID}\n```");
        let out = parse_agent_output(&resp(&body)).expect("parse");
        assert_eq!(out.action.len(), 1);
    }

    #[test]
    fn fenced_without_language_tag() {
        let body = format!("```\n{VALID}\n```");
        let out = parse_agent_output(&resp(&body)).expect("parse");
        assert_eq!(out.action.len(), 1);
    }

    #[test]
    fn fenced_with_surrounding_whitespace() {
        let body = format!("\n\n  ```json\n{VALID}\n```  \n\n");
        let out = parse_agent_output(&resp(&body)).expect("parse");
        assert_eq!(out.action.len(), 1);
    }

    #[test]
    fn fenced_invalid_json_falls_back_to_empty_actions() {
        let body = "```json\nnot json at all\n```";
        let out = parse_agent_output(&resp(body)).expect("parse");
        assert!(out.action.is_empty());
    }

    #[test]
    fn unfenced_invalid_json_falls_back_to_empty_actions() {
        let out = parse_agent_output(&resp("totally not json")).expect("parse");
        assert!(out.action.is_empty());
    }

    #[test]
    fn null_brain_fields_parse_as_empty_strings() {
        let body = r#"{"current_state":{"evaluation_previous_goal":null,"memory":null,"next_goal":"go"},"action":[{"name":"navigate","parameters":{"url":"https://example.com/"}}]}"#;
        let out = parse_agent_output(&resp(body)).expect("parse");
        assert_eq!(
            out.action.len(),
            1,
            "action should not be lost to null brain fields"
        );
        assert_eq!(out.current_state.evaluation_previous_goal, "");
        assert_eq!(out.current_state.memory, "");
        assert_eq!(out.current_state.next_goal, "go");
    }

    #[test]
    fn missing_brain_fields_default_to_empty() {
        let body = r#"{"current_state":{"next_goal":"go"},"action":[{"name":"navigate","parameters":{"url":"https://example.com/"}}]}"#;
        let out = parse_agent_output(&resp(body)).expect("parse");
        assert_eq!(out.action.len(), 1);
        assert_eq!(out.current_state.evaluation_previous_goal, "");
        assert_eq!(out.current_state.memory, "");
    }
}

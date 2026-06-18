use crate::domain::agent_output::{ActionInvocation, AgentOutput};
use ras_tools::domain::registry::ActionRegistry;

/// Controlled fallback: when `output` has NO action, salvage a registry-VALID
/// action out of its `next_goal` and put it back in `action`. Only actions whose
/// name is a registered tool survive (the schema-validation gate); the handler
/// validates parameters at execution. No-op when `action` is already present or
/// nothing valid can be salvaged. Logs when it fires.
pub fn salvage_into(output: &mut AgentOutput, registry: &ActionRegistry) {
    if !output.action.is_empty() {
        return;
    }
    let salvaged: Vec<ActionInvocation> = salvage_actions(&output.current_state.next_goal)
        .into_iter()
        .filter(|a| registry.get(&a.name).is_some())
        .collect();
    if !salvaged.is_empty() {
        tracing::warn!(
            count = salvaged.len(),
            "salvaged schema-valid action(s) from next_goal (model misplaced the action)"
        );
        output.action = salvaged;
    }
}

/// Controlled salvage: extract schema-shaped action(s) a weak model misplaced
/// (it sometimes drops the whole action object into `next_goal`/memory while
/// leaving `action` empty). PARSE-ONLY — the caller still VALIDATES each against
/// the action registry before executing; nothing here is run, and an
/// unparseable / actionless text yields an empty vec.
#[must_use]
pub fn salvage_actions(text: &str) -> Vec<ActionInvocation> {
    let Some(json) = json_slice(text) else {
        return Vec::new();
    };
    if let Ok(out) = serde_json::from_str::<AgentOutput>(json)
        && !out.action.is_empty()
    {
        return out.action;
    }
    if let Ok(actions) = serde_json::from_str::<Vec<ActionInvocation>>(json)
        && !actions.is_empty()
    {
        return actions;
    }
    if let Ok(action) = serde_json::from_str::<ActionInvocation>(json) {
        return vec![action];
    }
    Vec::new()
}

/// The first balanced-looking JSON slice in `text` (first `{`/`[` to the last
/// `}`/`]`), so an action object embedded in prose can be parsed.
fn json_slice(text: &str) -> Option<&str> {
    let start = text.find(['{', '['])?;
    let end = text.rfind(['}', ']'])?;
    (end > start).then(|| &text[start..=end])
}

#[cfg(test)]
mod tests {
    use super::salvage_actions;

    #[test]
    fn salvages_action_from_a_nested_agent_output() {
        let ng = r#"{"current_state":{"next_goal":""},"action":[{"name":"type_text","parameters":{"index":83}}]}"#;
        let out = salvage_actions(ng);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name.0.as_str(), "type_text");
    }

    #[test]
    fn salvages_a_bare_action_object_wrapped_in_prose() {
        let out =
            salvage_actions("I will: {\"name\":\"click_element\",\"parameters\":{\"index\":5}}");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name.0.as_str(), "click_element");
    }

    #[test]
    fn no_actions_from_plain_prose() {
        assert!(salvage_actions("I will now click the search button").is_empty());
    }

    #[test]
    fn salvage_into_keeps_only_registry_valid_actions() {
        use super::salvage_into;
        use ras_tools::{ActionRegistry, register_default_actions};

        let mut registry = ActionRegistry::new();
        register_default_actions(&mut registry).expect("register default actions");

        let mut output =
            output_with_next_goal(r#"{"name":"type_text","parameters":{"index":1,"text":"x"}}"#);
        salvage_into(&mut output, &registry);
        assert_eq!(output.action.len(), 1, "a registered action is salvaged");
        assert_eq!(output.action[0].name.0.as_str(), "type_text");

        let mut unknown =
            output_with_next_goal(r#"{"name":"definitely_not_a_tool","parameters":{}}"#);
        salvage_into(&mut unknown, &registry);
        assert!(
            unknown.action.is_empty(),
            "an unregistered action is dropped"
        );
    }

    fn output_with_next_goal(next_goal: &str) -> super::AgentOutput {
        use crate::domain::agent_output::AgentBrain;
        super::AgentOutput {
            current_state: AgentBrain {
                evaluation_previous_goal: String::new(),
                memory: String::new(),
                next_goal: next_goal.into(),
            },
            action: vec![],
            plan: None,
            current_plan_item: None,
        }
    }
}

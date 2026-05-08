use crate::domain::agent_output::PlanItem;

#[must_use]
pub fn render_plan(plan: &[PlanItem], current: Option<u32>) -> String {
    if plan.is_empty() {
        return String::new();
    }
    let mut out = String::from("Plan:\n");
    for item in plan {
        let marker = if item.completed {
            "[x]"
        } else if Some(item.step) == current {
            "[>]"
        } else {
            "[ ]"
        };
        out.push_str(&format!("  {marker} {} - {}\n", item.step, item.description));
    }
    out
}

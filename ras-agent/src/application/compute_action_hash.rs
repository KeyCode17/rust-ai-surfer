use ras_types::ActionName;
use sha2::{Digest, Sha256};

use crate::domain::agent_output::ActionInvocation;

#[must_use]
pub fn compute_action_hash(action: &ActionInvocation) -> String {
    let normalized = normalize_for_hash(&action.name, &action.parameters);
    let mut h = Sha256::new();
    h.update(action.name.0.as_bytes());
    h.update(b"|");
    h.update(normalized.as_bytes());
    format!("{:x}", h.finalize())
}

fn normalize_for_hash(name: &ActionName, params: &serde_json::Value) -> String {
    let mut params = params.clone();
    if let Some(obj) = params.as_object_mut() {
        match name.0.as_str() {
            "search_page" | "find_elements" => {
                obj.remove("max_results");
                obj.remove("offset");
            }
            "navigate" => {
                if let Some(url) = obj.get("url").and_then(|v| v.as_str()) {
                    obj.insert(
                        "url".into(),
                        serde_json::Value::String(strip_query(url)),
                    );
                }
            }
            _ => {}
        }
    }
    serde_json::to_string(&params).unwrap_or_default()
}

fn strip_query(url: &str) -> String {
    url.split('?').next().unwrap_or(url).trim_end_matches('/').to_string()
}

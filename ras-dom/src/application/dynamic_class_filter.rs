use std::collections::HashSet;

const DYNAMIC_PREFIXES: &[&str] = &[
    "is-",
    "has-",
    "ng-",
    "v-",
    "css-",
    "_",
    "sc-",
    "emotion-",
    "MuiInternal-",
];

const DYNAMIC_KEYWORDS: &[&str] = &[
    "active",
    "hover",
    "focus",
    "selected",
    "open",
    "expanded",
    "current",
    "loading",
    "pending",
];

#[must_use]
pub fn filter_dynamic_classes(classes: &str) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut kept: Vec<String> = Vec::new();
    for c in classes.split_whitespace() {
        if is_dynamic(c) {
            continue;
        }
        let owned = c.to_string();
        if seen.insert(owned.clone()) {
            kept.push(owned);
        }
    }
    kept.sort();
    kept
}

fn is_dynamic(class: &str) -> bool {
    if class.is_empty() {
        return true;
    }
    if class.chars().filter(|c| c.is_ascii_digit()).count() >= 4 {
        return true;
    }
    if DYNAMIC_PREFIXES.iter().any(|p| class.starts_with(p)) {
        return true;
    }
    if DYNAMIC_KEYWORDS
        .iter()
        .any(|k| class.eq_ignore_ascii_case(k) || class.contains(&format!("--{k}")))
    {
        return true;
    }
    if class.contains("__") && class.chars().any(|c| c.is_ascii_uppercase()) {
        return true;
    }
    false
}

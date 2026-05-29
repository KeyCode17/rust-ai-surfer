pub(crate) fn derive_ax_name(attrs: &[(String, String)]) -> Option<String> {
    for key in ["aria-label", "alt", "title", "name", "placeholder"] {
        if let Some(v) = attrs.iter().find(|(k, _)| k == key)
            && !v.1.is_empty()
        {
            return Some(v.1.clone());
        }
    }
    None
}

pub(crate) fn derive_label(attrs: &[(String, String)]) -> Option<String> {
    attrs
        .iter()
        .find(|(k, _)| k == "value")
        .map(|(_, v)| v.clone())
}

pub(crate) fn role_value(attrs: &[(String, String)]) -> Option<String> {
    attrs
        .iter()
        .find(|(k, _)| k == "role")
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty())
}

/// `onclick="viewBankAccount('0')"` -> "view bank account". Verb only; args dropped.
pub(crate) fn onclick_name(attrs: &[(String, String)]) -> Option<String> {
    let handler = attrs.iter().find(|(k, _)| k == "onclick")?;
    let fname: String = handler
        .1
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    if matches!(
        fname.as_str(),
        "" | "return" | "function" | "javascript" | "window" | "this" | "void" | "if" | "event"
    ) {
        return None;
    }
    let name = humanize(&fname);
    (!name.is_empty()).then_some(name)
}

/// FontAwesome icon class -> a verb, e.g. `fa-eye` -> "view", `fa-chart-line` -> "chart line".
pub(crate) fn icon_name(attrs: &[(String, String)]) -> Option<String> {
    let class = attrs.iter().find(|(k, _)| k == "class")?;
    for tok in class.1.split_whitespace() {
        let Some(icon) = tok.strip_prefix("fa-") else {
            continue;
        };
        if icon.is_empty() || is_fa_style_token(icon) {
            continue;
        }
        let mapped = match icon {
            "eye" => "view",
            "eye-slash" => "hide",
            "plus" | "plus-circle" | "plus-square" => "add",
            "minus" | "minus-circle" => "remove",
            "trash" | "trash-alt" | "trash-can" => "delete",
            "pen" | "pencil" | "pencil-alt" | "pen-to-square" | "edit" => "edit",
            "times" | "xmark" | "close" => "close",
            "check" | "check-circle" => "confirm",
            "search" | "magnifying-glass" => "search",
            "download" => "download",
            "upload" => "upload",
            "cog" | "cogs" | "gear" => "settings",
            "copy" => "copy",
            "save" | "floppy-disk" => "save",
            "print" => "print",
            "sync" | "rotate" | "arrows-rotate" => "refresh",
            "lock" => "lock",
            "unlock" => "unlock",
            _ => return Some(humanize(icon)),
        };
        return Some(mapped.to_string());
    }
    None
}

fn is_fa_style_token(icon: &str) -> bool {
    matches!(
        icon,
        "fw" | "spin"
            | "pulse"
            | "lg"
            | "sm"
            | "xs"
            | "2x"
            | "3x"
            | "4x"
            | "5x"
            | "stack"
            | "stack-1x"
            | "stack-2x"
            | "inverse"
            | "border"
            | "pull-left"
            | "pull-right"
            | "rotate-90"
            | "rotate-180"
            | "rotate-270"
            | "flip-horizontal"
            | "flip-vertical"
    )
}

pub(crate) fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Split camelCase / snake_case / kebab-case into lowercase words.
pub(crate) fn humanize(s: &str) -> String {
    let mut out = String::new();
    let mut prev_alnum = false;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == '$' {
            if prev_alnum {
                out.push(' ');
            }
            prev_alnum = false;
            continue;
        }
        if ch.is_uppercase() && prev_alnum {
            out.push(' ');
        }
        out.extend(ch.to_lowercase());
        prev_alnum = ch.is_alphanumeric();
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(kv: &[(&str, &str)]) -> Vec<(String, String)> {
        kv.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn humanize_splits_camel_snake_kebab() {
        assert_eq!(humanize("viewBankAccount"), "view bank account");
        assert_eq!(humanize("addBankAccountModal"), "add bank account modal");
        assert_eq!(humanize("chart-line"), "chart line");
    }

    #[test]
    fn onclick_name_takes_handler_verb_only() {
        let a = pairs(&[("onclick", "viewBankAccount('0')")]);
        assert_eq!(onclick_name(&a).as_deref(), Some("view bank account"));
    }

    #[test]
    fn onclick_name_skips_js_keywords() {
        assert_eq!(onclick_name(&pairs(&[("onclick", "return false;")])), None);
        assert_eq!(onclick_name(&pairs(&[("onclick", "")])), None);
    }

    #[test]
    fn icon_name_maps_known_and_humanizes_unknown() {
        let eye = icon_name(&pairs(&[("class", "fas fa-eye text-secondary")]));
        assert_eq!(eye.as_deref(), Some("view"));
        let add = icon_name(&pairs(&[("class", "fas fa-plus-circle")]));
        assert_eq!(add.as_deref(), Some("add"));
        let chart = icon_name(&pairs(&[("class", "fas fa-chart-line")]));
        assert_eq!(chart.as_deref(), Some("chart line"));
    }

    #[test]
    fn icon_name_ignores_style_tokens() {
        assert_eq!(icon_name(&pairs(&[("class", "fa-fw fa-2x")])), None);
    }

    #[test]
    fn role_value_reads_non_empty_role() {
        assert_eq!(
            role_value(&pairs(&[("role", "button")])).as_deref(),
            Some("button")
        );
        assert_eq!(role_value(&pairs(&[("role", "")])), None);
    }

    #[test]
    fn aria_label_wins_over_handlers() {
        let a = pairs(&[("aria-label", "Close dialog"), ("onclick", "shutModal()")]);
        assert_eq!(derive_ax_name(&a).as_deref(), Some("Close dialog"));
    }

    #[test]
    fn collapse_ws_normalizes() {
        assert_eq!(collapse_ws("  Deposit \n  14 "), "Deposit 14");
    }
}

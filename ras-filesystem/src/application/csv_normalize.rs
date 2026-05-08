#[must_use]
pub fn normalize_csv(raw: &str) -> String {
    let mut out_lines: Vec<String> = Vec::new();
    let mut leading_blank = true;
    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() && leading_blank {
            continue;
        }
        leading_blank = false;
        out_lines.push(normalize_row(trimmed));
    }
    while out_lines.last().is_some_and(|l| l.is_empty()) {
        out_lines.pop();
    }
    out_lines.join("\n")
}

fn normalize_row(line: &str) -> String {
    let fields = split_respecting_quotes(line);
    fields
        .into_iter()
        .map(quote_if_needed)
        .collect::<Vec<_>>()
        .join(",")
}

fn split_respecting_quotes(line: &str) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut in_quotes = false;
    let mut field_started_quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if in_quotes => {
                if matches!(chars.peek(), Some('"')) {
                    buf.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            '"' if !field_started_quoted && buf.trim().is_empty() => {
                in_quotes = true;
                field_started_quoted = true;
                buf.clear();
            }
            '"' => buf.push('"'),
            ',' if !in_quotes => {
                fields.push(buf.trim().to_string());
                buf.clear();
                field_started_quoted = false;
            }
            _ => buf.push(c),
        }
    }
    fields.push(buf.trim().to_string());
    fields
}

fn quote_if_needed(field: String) -> String {
    let needs = field.contains(',') || field.contains('"') || field.contains('\n');
    if !needs {
        return field;
    }
    let escaped = field.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

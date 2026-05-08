use ras_filesystem::application::csv_normalize::normalize_csv;

#[test]
fn quotes_fields_with_commas() {
    let out = normalize_csv("a,b\nhello,world\nfoo,bar baz");
    assert!(out.contains("hello,world"));
}

#[test]
fn escapes_internal_quotes() {
    let out = normalize_csv("a\nhe said \"hi\"");
    assert!(out.contains("\"he said \"\"hi\"\"\""));
}

#[test]
fn preserves_already_quoted_fields() {
    let out = normalize_csv("a,b\n\"with, comma\",x");
    assert!(out.contains("\"with, comma\""));
    assert!(out.contains("x"));
}

#[test]
fn strips_leading_and_trailing_blank_lines() {
    let out = normalize_csv("\n\na,b\n1,2\n\n\n");
    assert_eq!(out, "a,b\n1,2");
}

#[test]
fn passes_through_simple_csv() {
    let out = normalize_csv("a,b\n1,2");
    assert_eq!(out, "a,b\n1,2");
}

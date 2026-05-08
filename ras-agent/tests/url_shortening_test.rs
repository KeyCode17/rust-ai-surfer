use ras_agent::application::url_shortening::UrlShortener;

#[test]
fn short_urls_are_left_alone() {
    let mut s = UrlShortener::new();
    let out = s.shorten("see http://x.io");
    assert_eq!(out, "see http://x.io");
}

#[test]
fn long_urls_are_replaced_and_restored() {
    let mut s = UrlShortener::new();
    let long = format!("https://example.com/{}", "a".repeat(120));
    let input = format!("visit {long} now");
    let shortened = s.shorten(&input);
    assert!(shortened.contains("ras://url/0"));
    assert!(!shortened.contains(&long));
    let restored = s.restore(&shortened);
    assert_eq!(restored, input);
}

#[test]
fn duplicate_urls_collapse_to_one_key() {
    let mut s = UrlShortener::new();
    let long = format!("https://example.com/{}", "x".repeat(120));
    s.shorten(&format!("a {long} b {long}"));
    let restored = s.restore("ras://url/0 ras://url/0");
    assert_eq!(restored, format!("{long} {long}"));
}

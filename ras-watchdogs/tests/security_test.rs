use ras_browser::AllowedDomains;
use ras_types::DomainPattern;
use ras_watchdogs::SecurityWatchdog;
use url::Url;

fn pattern(s: &str) -> DomainPattern {
    DomainPattern::new(s).expect("pattern")
}

#[test]
fn empty_allowlist_permits_all() {
    let w = SecurityWatchdog {
        allowed: AllowedDomains::default(),
        prohibited: Vec::new(),
        block_ip_addresses: false,
    };
    assert!(w.permits(&Url::parse("https://example.com/").expect("url")));
}

#[test]
fn allowlist_blocks_outside() {
    let w = SecurityWatchdog {
        allowed: AllowedDomains::new(vec![pattern("example.com")]),
        prohibited: Vec::new(),
        block_ip_addresses: false,
    };
    assert!(w.permits(&Url::parse("https://example.com/").expect("url")));
    assert!(!w.permits(&Url::parse("https://attacker.com/").expect("url")));
}

#[test]
fn prohibited_overrides() {
    let w = SecurityWatchdog {
        allowed: AllowedDomains::default(),
        prohibited: vec![pattern("evil.com")],
        block_ip_addresses: false,
    };
    assert!(!w.permits(&Url::parse("https://evil.com/").expect("url")));
}

#[test]
fn ip_block_when_enabled() {
    let w = SecurityWatchdog {
        allowed: AllowedDomains::default(),
        prohibited: Vec::new(),
        block_ip_addresses: true,
    };
    assert!(!w.permits(&Url::parse("http://10.0.0.1/").expect("url")));
    assert!(!w.permits(&Url::parse("http://[::1]/").expect("url")));
    assert!(w.permits(&Url::parse("http://localhost/").expect("url")));
}

#[test]
fn wildcard_pattern_allows_subdomains() {
    let w = SecurityWatchdog {
        allowed: AllowedDomains::new(vec![pattern("*.example.com")]),
        prohibited: Vec::new(),
        block_ip_addresses: false,
    };
    assert!(w.permits(&Url::parse("https://api.example.com/").expect("url")));
    assert!(w.permits(&Url::parse("https://example.com/").expect("url")));
    assert!(!w.permits(&Url::parse("https://other.com/").expect("url")));
}

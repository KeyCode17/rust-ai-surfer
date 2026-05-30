//! Unit tests for the egress policy engine.

use super::{EgressError, EgressPolicy};
use ras_types::DomainPattern;

fn default() -> EgressPolicy {
    EgressPolicy::default()
}

fn url(s: &str) -> url::Url {
    s.parse().expect("valid url")
}

#[test]
fn public_http_https_allowed() {
    let p = default();
    assert_eq!(p.check(&url("https://example.com/")), Ok(()));
    assert_eq!(p.check(&url("http://example.com/")), Ok(()));
}

#[test]
fn forbidden_schemes() {
    let p = default();
    assert_eq!(
        p.check(&url("file:///etc/passwd")),
        Err(EgressError::ForbiddenScheme("file".into()))
    );
    assert_eq!(
        p.check(&url("chrome://settings")),
        Err(EgressError::ForbiddenScheme("chrome".into()))
    );
    assert_eq!(
        p.check(&url("data:text/html,x")),
        Err(EgressError::ForbiddenScheme("data".into()))
    );
}

#[test]
fn private_ipv4() {
    let p = default();
    assert_eq!(
        p.check(&url("http://127.0.0.1/")),
        Err(EgressError::PrivateAddress("127.0.0.1".into()))
    );
    assert_eq!(
        p.check(&url("http://10.0.0.5/")),
        Err(EgressError::PrivateAddress("10.0.0.5".into()))
    );
    assert_eq!(
        p.check(&url("http://192.168.1.1/")),
        Err(EgressError::PrivateAddress("192.168.1.1".into()))
    );
    assert_eq!(
        p.check(&url("http://172.16.0.1/")),
        Err(EgressError::PrivateAddress("172.16.0.1".into()))
    );
    assert_eq!(
        p.check(&url("http://169.254.169.254/latest/meta-data/")),
        Err(EgressError::PrivateAddress("169.254.169.254".into()))
    );
}

#[test]
fn localhost_domain() {
    let p = default();
    assert_eq!(
        p.check(&url("http://localhost/")),
        Err(EgressError::PrivateAddress("localhost".into()))
    );
    assert_eq!(
        p.check(&url("http://foo.localhost/")),
        Err(EgressError::PrivateAddress("foo.localhost".into()))
    );
}

#[test]
fn private_ipv6() {
    let p = default();
    assert_eq!(
        p.check(&url("http://[::1]/")),
        Err(EgressError::PrivateAddress("::1".into()))
    );
    assert_eq!(
        p.check(&url("http://[fd00::1]/")),
        Err(EgressError::PrivateAddress("fd00::1".into()))
    );
    assert_eq!(
        p.check(&url("http://[fe80::1]/")),
        Err(EgressError::PrivateAddress("fe80::1".into()))
    );
}

#[test]
fn metadata_google_internal() {
    let p = default();
    assert_eq!(
        p.check(&url("https://metadata.google.internal/")),
        Err(EgressError::DeniedHost("metadata.google.internal".into()))
    );
}

#[test]
fn denied_ports() {
    let p = EgressPolicy::default().with_denied_ports(vec![9222]);
    assert_eq!(
        p.check(&url("http://example.com:9222/")),
        Err(EgressError::DeniedPort(9222))
    );
    assert_eq!(p.check(&url("http://example.com:8080/")), Ok(()));
}

#[test]
fn deny_list() {
    let pat = DomainPattern::new("*.internal.corp").expect("pattern");
    let p = EgressPolicy::default().with_deny(vec![pat]);
    assert_eq!(
        p.check(&url("https://x.internal.corp/")),
        Err(EgressError::DeniedHost("x.internal.corp".into()))
    );
    assert_eq!(p.check(&url("https://example.com/")), Ok(()));
}

#[test]
fn allow_list() {
    let pat = DomainPattern::new("example.com").expect("pattern");
    let p = EgressPolicy::default().with_allow(vec![pat]);
    assert_eq!(p.check(&url("https://example.com/")), Ok(()));
    assert_eq!(
        p.check(&url("https://other.com/")),
        Err(EgressError::NotAllowlisted("other.com".into()))
    );
}

#[test]
fn public_ip_allowed() {
    let p = default();
    assert_eq!(p.check(&url("http://8.8.8.8/")), Ok(()));
}

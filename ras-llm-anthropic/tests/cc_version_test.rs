use ras_llm_anthropic::domain::cc_version::{CcVersion, FALLBACK_CC_VERSION};

#[test]
fn parses_valid_semver() {
    let v = CcVersion::new("2.1.133").expect("semver");
    assert_eq!(v.as_str(), "2.1.133");
}

#[test]
fn rejects_non_semver() {
    assert!(CcVersion::new("1.2").is_err());
    assert!(CcVersion::new("v1.2.3").is_err());
    assert!(CcVersion::new("a.b.c").is_err());
}

#[test]
fn fallback_constant_is_valid() {
    assert!(CcVersion::new(FALLBACK_CC_VERSION).is_ok());
}

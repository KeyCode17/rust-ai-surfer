use ras_cosmium::CosmiumProfile;

#[test]
fn default_profile_emits_expected_flags() {
    let flags = CosmiumProfile::default().to_cli_flags();
    assert!(flags.iter().any(|f| f.starts_with("--cosmium-platform=")));
    assert!(flags.iter().any(|f| f.starts_with("--cosmium-language=")));
    assert!(flags.iter().any(|f| f.starts_with("--cosmium-screen=")));
    assert!(
        flags
            .iter()
            .any(|f| f.starts_with("--cosmium-device-memory="))
    );
}

#[test]
fn user_agent_passes_through() {
    let p = CosmiumProfile {
        user_agent: Some("Mozilla/5.0 test".into()),
        ..CosmiumProfile::default()
    };
    let flags = p.to_cli_flags();
    assert!(flags.iter().any(|f| f == "--user-agent=Mozilla/5.0 test"));
}

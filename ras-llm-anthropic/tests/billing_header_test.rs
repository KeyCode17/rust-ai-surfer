use ras_llm::{ChatMessage, SystemMessage};
use ras_llm_anthropic::application::invoke_claude_code::inject_billing_header;
use ras_llm_anthropic::domain::billing_header::BillingHeader;
use ras_llm_anthropic::domain::cc_version::CcVersion;

#[test]
fn billing_header_text_matches_python_port() {
    let v = CcVersion::new("2.1.133").expect("semver");
    let h = BillingHeader::for_cli(&v);
    assert_eq!(
        h.as_str(),
        "x-anthropic-billing-header: cc_version=2.1.133; cc_entrypoint=cli;"
    );
}

#[test]
fn inject_into_existing_system_message_prepends_billing() {
    let v = CcVersion::new("2.1.133").expect("semver");
    let billing = BillingHeader::for_cli(&v);
    let messages = vec![
        ChatMessage::System(SystemMessage { content: "you are helpful".into(), cache: false }),
        ChatMessage::user_text("hi"),
    ];
    let out = inject_billing_header(messages, &billing);
    let first = match &out[0] {
        ChatMessage::System(m) => m.content.clone(),
        _ => panic!("expected system message"),
    };
    assert!(first.starts_with(billing.as_str()));
    assert!(first.contains("you are helpful"));
}

#[test]
fn inject_when_no_system_message_inserts_new() {
    let v = CcVersion::new("2.1.133").expect("semver");
    let billing = BillingHeader::for_cli(&v);
    let messages = vec![ChatMessage::user_text("hello")];
    let out = inject_billing_header(messages, &billing);
    assert_eq!(out.len(), 2);
    match &out[0] {
        ChatMessage::System(m) => assert_eq!(m.content, billing.as_str()),
        _ => panic!("first message should be system"),
    }
}

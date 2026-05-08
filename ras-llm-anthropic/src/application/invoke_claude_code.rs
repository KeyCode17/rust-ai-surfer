use ras_llm::{ChatMessage, ContentPart, SystemMessage};

use crate::domain::billing_header::BillingHeader;

#[must_use]
pub fn inject_billing_header(
    messages: Vec<ChatMessage>,
    header: &BillingHeader,
) -> Vec<ChatMessage> {
    let billing = header.as_str().to_string();
    let mut out = messages;
    match out.first().cloned() {
        Some(ChatMessage::System(first)) => {
            let merged = format!("{billing}\n\n{}", first.content);
            out[0] = ChatMessage::System(SystemMessage {
                content: merged,
                cache: first.cache,
            });
        }
        _ => {
            out.insert(
                0,
                ChatMessage::System(SystemMessage {
                    content: billing,
                    cache: false,
                }),
            );
        }
    }
    out
}

#[must_use]
pub fn first_text(message: &ChatMessage) -> Option<&str> {
    match message {
        ChatMessage::System(m) => Some(&m.content),
        ChatMessage::User(m) => m.content.iter().find_map(|p| match p {
            ContentPart::Text { text } => Some(text.as_str()),
            _ => None,
        }),
        ChatMessage::Assistant(m) => m.content.as_deref(),
        ChatMessage::Tool(m) => Some(&m.content),
    }
}

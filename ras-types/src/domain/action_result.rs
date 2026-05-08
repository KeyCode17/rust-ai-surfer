use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionResult {
    pub extracted_content: Option<String>,
    pub include_in_memory: bool,
    pub error: Option<String>,
    pub is_done: bool,
    pub long_term_memory: Option<String>,
    pub attachments: Vec<String>,
    pub images: Vec<String>,
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl ActionResult {
    #[must_use]
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            extracted_content: Some(content.into()),
            include_in_memory: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            error: Some(message.into()),
            include_in_memory: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn done(content: impl Into<String>) -> Self {
        Self {
            extracted_content: Some(content.into()),
            include_in_memory: true,
            is_done: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_attachment(mut self, path: impl Into<String>) -> Self {
        self.attachments.push(path.into());
        self
    }

    #[must_use]
    pub fn with_image(mut self, b64: impl Into<String>) -> Self {
        self.images.push(b64.into());
        self
    }

    #[must_use]
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

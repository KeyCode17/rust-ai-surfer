use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageFingerprint(String);

impl PageFingerprint {
    #[must_use]
    pub fn new(url: &Url, title: &str, clickable_count: u32, text_chars: u32) -> Self {
        let mut h = Sha256::new();
        h.update(url.as_str().as_bytes());
        h.update(b"|");
        h.update(title.as_bytes());
        h.update(b"|");
        h.update(clickable_count.to_le_bytes());
        h.update(b"|");
        h.update(text_chars.to_le_bytes());
        Self(format!("{:x}", h.finalize()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct ActionLoopDetector {
    history: Vec<String>,
    pages: Vec<PageFingerprint>,
    pub action_threshold: u32,
    pub stagnation_threshold: u32,
}

impl ActionLoopDetector {
    #[must_use]
    pub fn new() -> Self {
        Self { history: Vec::new(), pages: Vec::new(), action_threshold: 5, stagnation_threshold: 5 }
    }

    pub fn record_action(&mut self, action_hash: impl Into<String>) {
        self.history.push(action_hash.into());
        if self.history.len() > 32 {
            self.history.remove(0);
        }
    }

    pub fn record_page(&mut self, fingerprint: PageFingerprint) {
        self.pages.push(fingerprint);
        if self.pages.len() > 32 {
            self.pages.remove(0);
        }
    }

    #[must_use]
    pub fn action_loop_detected(&self) -> bool {
        let n = self.action_threshold as usize;
        if self.history.len() < n {
            return false;
        }
        let last = &self.history[self.history.len() - 1];
        self.history.iter().rev().take(n).all(|h| h == last)
    }

    #[must_use]
    pub fn page_stagnation_detected(&self) -> bool {
        let n = self.stagnation_threshold as usize;
        if self.pages.len() < n {
            return false;
        }
        let last = &self.pages[self.pages.len() - 1];
        self.pages.iter().rev().take(n).all(|p| p == last)
    }
}

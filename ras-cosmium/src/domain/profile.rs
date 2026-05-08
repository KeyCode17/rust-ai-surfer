use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmiumProfile {
    pub identity: Identity,
    pub locale: Locale,
    pub hardware: Hardware,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub platform: Platform,
    pub vendor: String,
    pub product: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Linux,
    Windows,
    Macos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Locale {
    pub language: String,
    pub timezone: String,
    pub accept_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    pub device_memory_gb: u32,
    pub hardware_concurrency: u32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub color_depth: u32,
}

impl Default for CosmiumProfile {
    fn default() -> Self {
        Self {
            identity: Identity {
                platform: Platform::Linux,
                vendor: "Google Inc.".into(),
                product: "Gecko".into(),
            },
            locale: Locale {
                language: "en-US".into(),
                timezone: "America/New_York".into(),
                accept_language: "en-US,en;q=0.9".into(),
            },
            hardware: Hardware {
                device_memory_gb: 8,
                hardware_concurrency: 8,
                screen_width: 1920,
                screen_height: 1080,
                color_depth: 24,
            },
            user_agent: None,
        }
    }
}

impl CosmiumProfile {
    #[must_use]
    pub fn to_cli_flags(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.push(format!(
            "--cosmium-platform={}",
            platform_str(self.identity.platform)
        ));
        out.push(format!("--cosmium-vendor={}", self.identity.vendor));
        out.push(format!("--cosmium-product={}", self.identity.product));
        out.push(format!("--cosmium-language={}", self.locale.language));
        out.push(format!("--cosmium-timezone={}", self.locale.timezone));
        out.push(format!(
            "--cosmium-accept-language={}",
            self.locale.accept_language
        ));
        out.push(format!(
            "--cosmium-device-memory={}",
            self.hardware.device_memory_gb
        ));
        out.push(format!(
            "--cosmium-hardware-concurrency={}",
            self.hardware.hardware_concurrency
        ));
        out.push(format!(
            "--cosmium-screen={}x{}x{}",
            self.hardware.screen_width, self.hardware.screen_height, self.hardware.color_depth,
        ));
        if let Some(ua) = &self.user_agent {
            out.push(format!("--user-agent={ua}"));
        }
        out
    }
}

fn platform_str(p: Platform) -> &'static str {
    match p {
        Platform::Linux => "linux",
        Platform::Windows => "windows",
        Platform::Macos => "macos",
    }
}

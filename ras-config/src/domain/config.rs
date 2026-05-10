use std::env;
use std::path::PathBuf;
use std::time::Duration;

use ras_errors::AppError;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub home_dir: PathBuf,
    pub config_dir: PathBuf,
    pub extensions_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub anthropic_api_key: Option<String>,
    pub claude_code_oauth_enabled: bool,
    pub cdp_timeout: Duration,
    pub action_timeout: Duration,
    pub anonymized_telemetry: bool,
    pub cloud_sync: bool,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required env var: {0}")]
    MissingEnv(String),
    #[error("invalid value for {0}: {1}")]
    InvalidValue(String, String),
}

impl From<ConfigError> for AppError {
    fn from(e: ConfigError) -> Self {
        Self::ValidationError(e.to_string())
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let home_dir = env::var("RAS_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(dirs_home)
            .ok_or_else(|| ConfigError::MissingEnv("RAS_HOME".into()))?;
        let config_dir = env::var("RAS_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(".config").join("rust-ai-surfer"));
        let extensions_dir = env::var("RAS_EXTENSIONS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| config_dir.join("extensions"));
        let profiles_dir = env::var("RAS_PROFILES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| config_dir.join("profiles"));
        Ok(Self {
            home_dir,
            config_dir,
            extensions_dir,
            profiles_dir,
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            claude_code_oauth_enabled: env::var("RAS_CLAUDE_CODE_OAUTH").ok().as_deref()
                != Some("0"),
            cdp_timeout: parse_duration("RAS_CDP_TIMEOUT_S", 60)?,
            action_timeout: parse_duration("RAS_ACTION_TIMEOUT_S", 180)?,
            anonymized_telemetry: env::var("RAS_ANONYMIZED_TELEMETRY").ok().as_deref() != Some("0"),
            cloud_sync: env::var("RAS_CLOUD_SYNC").ok().as_deref() == Some("1"),
        })
    }
}

fn parse_duration(key: &str, default_secs: u64) -> Result<Duration, ConfigError> {
    match env::var(key) {
        Ok(v) => v
            .parse::<u64>()
            .map(Duration::from_secs)
            .map_err(|_| ConfigError::InvalidValue(key.into(), v)),
        Err(_) => Ok(Duration::from_secs(default_secs)),
    }
}

fn dirs_home() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

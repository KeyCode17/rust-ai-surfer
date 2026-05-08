use regex::Regex;

use crate::domain::cc_version::{CcVersion, FALLBACK_CC_VERSION};

pub async fn resolve_cc_version() -> CcVersion {
    let Ok(out) = tokio::process::Command::new("claude").arg("--version").output().await else {
        return CcVersion::fallback();
    };
    if !out.status.success() {
        return CcVersion::fallback();
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let Ok(re) = Regex::new(r"(\d+\.\d+\.\d+)") else {
        return CcVersion::fallback();
    };
    let Some(m) = re.find(&stdout) else {
        return CcVersion::fallback();
    };
    CcVersion::new(m.as_str()).unwrap_or_else(|_| CcVersion::new(FALLBACK_CC_VERSION).unwrap_or_default())
}

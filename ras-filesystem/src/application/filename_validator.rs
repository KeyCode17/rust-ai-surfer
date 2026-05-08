use ras_errors::AppError;
use regex::Regex;

use crate::domain::file::FileExtension;

pub fn parse_filename(name: &str) -> Result<(String, FileExtension), AppError> {
    if name.is_empty() {
        return Err(AppError::ValidationError("filename empty".into()));
    }
    let re = Regex::new(r"^[A-Za-z0-9_\-]+\.[A-Za-z0-9]+$")
        .map_err(|e| AppError::InternalError(format!("regex: {e}")))?;
    if !re.is_match(name) {
        return Err(AppError::ValidationError(format!(
            "invalid filename '{name}'; allowed: [A-Za-z0-9_-] + extension"
        )));
    }
    let mut parts = name.rsplitn(2, '.');
    let ext = parts.next().unwrap_or_default();
    let stem = parts.next().unwrap_or_default();
    let extension = FileExtension::parse(ext)?;
    Ok((stem.to_string(), extension))
}

#[must_use]
pub fn sanitize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
            out.push(c);
        } else if c == ' ' {
            out.push('_');
        }
    }
    out
}

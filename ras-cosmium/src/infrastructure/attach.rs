use ras_errors::AppError;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
struct JsonVersion {
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: String,
}

pub async fn resolve_attach_url(http_url: &Url) -> Result<Url, AppError> {
    let endpoint = http_url
        .join("json/version")
        .map_err(|e| AppError::BadRequest(format!("invalid cdp_url: {e}")))?;
    let body = reqwest::get(endpoint.as_str())
        .await
        .map_err(|e| AppError::BrowserDisconnected(format!("cdp /json/version: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::BrowserDisconnected(format!("cdp /json/version status: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::BrowserDisconnected(format!("cdp /json/version body: {e}")))?;
    let parsed: JsonVersion = serde_json::from_str(&body)
        .map_err(|e| AppError::BrowserDisconnected(format!("cdp /json/version json: {e}")))?;
    Url::parse(&parsed.web_socket_debugger_url)
        .map_err(|e| AppError::BrowserDisconnected(format!("invalid ws url: {e}")))
}

use std::path::PathBuf;

use anyhow::{Context, Result};
use ras_config::init_logger;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum Request {
    Ping,
    Status,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    ok: bool,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    init_logger();
    let socket = socket_path();
    let _ = tokio::fs::remove_file(&socket).await;
    let listener = UnixListener::bind(&socket).context("bind unix socket")?;
    tracing::info!(socket = %socket.display(), "daemon ready");
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle(stream));
    }
}

fn socket_path() -> PathBuf {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir());
    runtime.join("ras-daemon.sock")
}

async fn handle(stream: tokio::net::UnixStream) {
    let (rx, mut tx) = stream.into_split();
    let mut reader = BufReader::new(rx);
    let mut line = String::new();
    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
        return;
    }
    let response = match serde_json::from_str::<Request>(line.trim()) {
        Ok(Request::Ping) => Response { ok: true, message: "pong".into() },
        Ok(Request::Status) => Response { ok: true, message: "ready".into() },
        Ok(Request::Shutdown) => Response { ok: true, message: "shutting down".into() },
        Err(e) => Response { ok: false, message: format!("parse: {e}") },
    };
    let body = serde_json::to_string(&response).unwrap_or_else(|_| "{}".into());
    let _ = tx.write_all(body.as_bytes()).await;
    let _ = tx.write_all(b"\n").await;
    let _ = tx.shutdown().await;
}

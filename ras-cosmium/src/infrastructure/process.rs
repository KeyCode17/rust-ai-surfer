use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use ras_errors::AppError;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, info};
use url::Url;

use crate::domain::profile::CosmiumProfile;
use crate::domain::repository::{BrowserLauncher, LaunchedBrowser};

#[derive(Default)]
pub struct CosmiumProcessLauncher {
    children: Mutex<Vec<Child>>,
}

impl std::fmt::Debug for CosmiumProcessLauncher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CosmiumProcessLauncher").finish()
    }
}

#[async_trait]
impl BrowserLauncher for CosmiumProcessLauncher {
    async fn launch(
        &self,
        binary: &Path,
        profile: &CosmiumProfile,
    ) -> Result<LaunchedBrowser, AppError> {
        let port = pick_free_port()?;
        let user_data_dir = make_temp_user_data_dir().await?;
        let mut cmd = Command::new(binary);
        cmd.arg(format!("--remote-debugging-port={port}"))
            .arg(format!("--user-data-dir={}", user_data_dir.display()))
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-features=Translate")
            .args(profile.to_cli_flags())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        debug!(?cmd, "launching cosmium");
        let child = cmd
            .spawn()
            .map_err(|e| AppError::InternalError(format!("spawn cosmium: {e}")))?;
        let pid = child.id();
        self.children.lock().await.push(child);
        let cdp_url = wait_for_cdp(port).await?;
        info!(%cdp_url, ?pid, "cosmium ready");
        Ok(LaunchedBrowser {
            cdp_url,
            user_data_dir,
            pid,
        })
    }

    async fn shutdown(&self, browser: &LaunchedBrowser) -> Result<(), AppError> {
        let mut children = self.children.lock().await;
        let mut keep = Vec::new();
        for mut child in children.drain(..) {
            if browser.pid.is_some() && child.id() != browser.pid {
                keep.push(child);
                continue;
            }
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        *children = keep;
        Ok(())
    }
}

fn pick_free_port() -> Result<u16, AppError> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| AppError::InternalError(format!("bind: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| AppError::InternalError(format!("local_addr: {e}")))?
        .port();
    drop(listener);
    Ok(port)
}

async fn make_temp_user_data_dir() -> Result<PathBuf, AppError> {
    let dir = std::env::temp_dir().join(format!("ras-cosmium-{}", uuid_short()));
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::InternalError(format!("mkdir {}: {e}", dir.display())))?;
    Ok(dir)
}

fn uuid_short() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{now:x}")
}

async fn wait_for_cdp(port: u16) -> Result<Url, AppError> {
    let http = Url::parse(&format!("http://127.0.0.1:{port}/"))
        .map_err(|e| AppError::InternalError(format!("port url: {e}")))?;
    for _ in 0..60 {
        if let Ok(ws) = crate::infrastructure::attach::resolve_attach_url(&http).await {
            return Ok(ws);
        }
        sleep(Duration::from_millis(250)).await;
    }
    Err(AppError::BrowserDisconnected(format!(
        "cdp not ready on port {port}"
    )))
}

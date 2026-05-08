use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};
use ras_config::{init_logger, load_env};

#[derive(Parser, Debug)]
#[command(name = "ras", version, about = "rust-ai-surfer", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Run {
        #[arg(long)]
        task: String,
        #[arg(long, default_value = "claude-sonnet-4-5")]
        model: String,
        #[arg(long)]
        cdp_url: Option<String>,
        #[arg(long)]
        cosmium_binary: Option<PathBuf>,
        #[arg(long, default_value_t = 25)]
        max_steps: u32,
    },
    Doctor,
    Login,
    Version,
}

#[tokio::main]
async fn main() -> ExitCode {
    load_env();
    let cli = Cli::parse();
    init_logger();
    match dispatch(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "command failed");
            ExitCode::from(1)
        }
    }
}

async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Cmd::Run {
            task,
            model,
            cdp_url,
            cosmium_binary,
            max_steps,
        } => {
            tracing::info!(%task, %model, ?cdp_url, ?cosmium_binary, max_steps, "run requested");
            tracing::info!("Run subcommand wires LLM + Browser session in Phase 11 (PoC example).");
            Ok(())
        }
        Cmd::Doctor => doctor().await,
        Cmd::Login => {
            tracing::info!(
                "Run `claude` to log in via Claude Code OAuth, then unset ANTHROPIC_API_KEY."
            );
            Ok(())
        }
        Cmd::Version => {
            println!("ras {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

async fn doctor() -> Result<()> {
    let claude_ok = tokio::process::Command::new("claude")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "claude CLI:        {}",
        if claude_ok { "ok" } else { "missing" }
    );
    let api_key = std::env::var("ANTHROPIC_API_KEY").is_ok();
    println!(
        "ANTHROPIC_API_KEY: {}",
        if api_key {
            "set (would shadow OAuth)"
        } else {
            "unset (OAuth path enabled)"
        }
    );
    let creds_path = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".claude")
        .join(".credentials.json");
    println!(
        "credentials.json:  {}",
        if creds_path.exists() { "ok" } else { "missing" }
    );
    Ok(())
}

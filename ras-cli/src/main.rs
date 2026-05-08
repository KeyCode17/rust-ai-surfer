use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ras", version, about = "rust-ai-surfer", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(cli.log_level)
        .init();
    tracing::info!("ras-cli ready");
    Ok(())
}

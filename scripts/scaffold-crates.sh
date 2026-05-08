#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

emit_cargo() {
    local name="$1" desc="$2" deps_block="$3" extra_block="${4:-}"
    cat > "$name/Cargo.toml" <<EOF
[package]
name                   = "$name"
description            = "$desc"
version.workspace      = true
edition.workspace      = true
rust-version.workspace = true
license.workspace      = true
repository.workspace   = true
authors.workspace      = true

[lints]
workspace = true

[dependencies]
$deps_block
$extra_block
EOF
}

emit_lib() {
    local name="$1"
    cat > "$name/src/lib.rs" <<'EOF'
pub mod domain;
pub mod application;
pub mod infrastructure;
EOF
    : > "$name/src/domain/mod.rs"
    : > "$name/src/application/mod.rs"
    : > "$name/src/infrastructure/mod.rs"
}

emit_cargo ras-errors "Centralized AppError for rust-ai-surfer" \
"serde     = { workspace = true }
thiserror = { workspace = true }"
emit_lib ras-errors

emit_cargo ras-types "Shared types and ID newtypes" \
"ras-errors = { workspace = true }
serde      = { workspace = true }
serde_json = { workspace = true }
uuid       = { workspace = true }
url        = { workspace = true }
chrono     = { workspace = true }
smol_str   = { workspace = true }
indexmap   = { workspace = true }
schemars   = { workspace = true }
thiserror  = { workspace = true }"
emit_lib ras-types

emit_cargo ras-validation "Validated<T> extractor for tool params" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
async-trait = { workspace = true }
validator   = { workspace = true }
schemars    = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }"
emit_lib ras-validation

emit_cargo ras-config "Env and logger bootstrap" \
"ras-errors         = { workspace = true }
dotenvy            = { workspace = true }
tracing            = { workspace = true }
tracing-subscriber = { workspace = true }
serde              = { workspace = true }
serde_json         = { workspace = true }
thiserror          = { workspace = true }
once_cell          = { workspace = true }"
emit_lib ras-config

emit_cargo ras-events "Tokio broadcast event bus" \
"ras-errors      = { workspace = true }
ras-types       = { workspace = true }
async-trait     = { workspace = true }
async-broadcast = { workspace = true }
tokio           = { workspace = true }
tokio-util      = { workspace = true }
futures         = { workspace = true }
thiserror       = { workspace = true }
tracing         = { workspace = true }"
emit_lib ras-events

emit_cargo ras-cdp "Chrome DevTools Protocol adapter via chromiumoxide" \
"ras-errors    = { workspace = true }
ras-types     = { workspace = true }
ras-events    = { workspace = true }
async-trait   = { workspace = true }
chromiumoxide = { workspace = true }
tokio         = { workspace = true }
tokio-util    = { workspace = true }
futures       = { workspace = true }
serde         = { workspace = true }
serde_json    = { workspace = true }
thiserror     = { workspace = true }
tracing       = { workspace = true }
url           = { workspace = true }"
emit_lib ras-cdp

emit_cargo ras-cosmium "Cosmium binary launcher and fingerprint profile mapping" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-cdp     = { workspace = true }
async-trait = { workspace = true }
tokio       = { workspace = true }
reqwest     = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
url         = { workspace = true }"
emit_lib ras-cosmium

emit_cargo ras-llm "LLM port traits and chat message domain types" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
async-trait = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
schemars    = { workspace = true }
thiserror   = { workspace = true }
url         = { workspace = true }"
emit_lib ras-llm

emit_cargo ras-llm-anthropic "Anthropic + Claude Code OAuth LLM adapter" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-llm     = { workspace = true }
async-trait = { workspace = true }
reqwest     = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
tokio       = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
url         = { workspace = true }
keyring     = { workspace = true }
regex       = { workspace = true }
chrono      = { workspace = true }" \
"
[target.'cfg(target_os = \"macos\")'.dependencies]
security-framework = \"3\"

[features]
default     = []
claude-code = []

[dev-dependencies]
tokio    = { workspace = true }
wiremock = { workspace = true }
tempfile = { workspace = true }"
emit_lib ras-llm-anthropic

LLM_PROVIDERS=(openai google groq bedrock openrouter vercel deepseek cerebras mistral ollama oci cloud langchain)
for p in "${LLM_PROVIDERS[@]}"; do
    emit_cargo "ras-llm-$p" "$(echo "$p" | sed 's/.*/\u&/') LLM adapter" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-llm     = { workspace = true }
async-trait = { workspace = true }
reqwest     = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
tokio       = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
url         = { workspace = true }"
    emit_lib "ras-llm-$p"
done

emit_cargo ras-browser "BrowserSession orchestration and mode dispatch" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-cdp     = { workspace = true }
ras-cosmium = { workspace = true }
ras-events  = { workspace = true }
async-trait = { workspace = true }
tokio       = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
url         = { workspace = true }"
emit_lib ras-browser

emit_cargo ras-dom "DOM and AX tree extraction with clickable detection and stable hashing" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-cdp     = { workspace = true }
async-trait = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
sha2        = { workspace = true }
indexmap    = { workspace = true }
image       = { workspace = true }"
emit_lib ras-dom

emit_cargo ras-tools "Action registry and built-in browser actions" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-cdp     = { workspace = true }
ras-dom     = { workspace = true }
ras-events  = { workspace = true }
async-trait = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
schemars    = { workspace = true }
indexmap    = { workspace = true }
thiserror   = { workspace = true }
tokio       = { workspace = true }
tracing     = { workspace = true }
url         = { workspace = true }
regex       = { workspace = true }"
emit_lib ras-tools

emit_cargo ras-watchdogs "Browser watchdogs (security, downloads, popups, crash, recording)" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-cdp     = { workspace = true }
ras-events  = { workspace = true }
async-trait = { workspace = true }
tokio       = { workspace = true }
tokio-util  = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
url         = { workspace = true }"
emit_lib ras-watchdogs

emit_cargo ras-filesystem "Sandboxed file system for the agent (Csv, Docx, Pdf, Md, Json, Jsonl, Html, Txt)" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
async-trait = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
tokio       = { workspace = true }
regex       = { workspace = true }"
emit_lib ras-filesystem

emit_cargo ras-tokens "Token cost service with LiteLLM pricing fetch" \
"ras-errors = { workspace = true }
ras-types  = { workspace = true }
ras-llm    = { workspace = true }
reqwest    = { workspace = true }
tokio      = { workspace = true }
serde      = { workspace = true }
serde_json = { workspace = true }
thiserror  = { workspace = true }
tracing    = { workspace = true }
chrono     = { workspace = true }
once_cell  = { workspace = true }"
emit_lib ras-tokens

emit_cargo ras-judge "Judge eval for agent traces" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-llm     = { workspace = true }
async-trait = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
image       = { workspace = true }"
emit_lib ras-judge

emit_cargo ras-telemetry "Anonymized telemetry events" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
async-trait = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
tokio       = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }"
emit_lib ras-telemetry

emit_cargo ras-recording "Browser session recording (frames + GIF)" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-cdp     = { workspace = true }
async-trait = { workspace = true }
tokio       = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
image       = { workspace = true }"
emit_lib ras-recording

emit_cargo ras-sandbox "Sandboxed code execution" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
async-trait = { workspace = true }
tokio       = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }"
emit_lib ras-sandbox

emit_cargo ras-skills "Browser-use Skills service client" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-llm     = { workspace = true }
async-trait = { workspace = true }
reqwest     = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
schemars    = { workspace = true }
thiserror   = { workspace = true }
tokio       = { workspace = true }
tracing     = { workspace = true }"
emit_lib ras-skills

emit_cargo ras-mcp "MCP (Model Context Protocol) server and client" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
ras-tools   = { workspace = true }
async-trait = { workspace = true }
tokio       = { workspace = true }
tokio-util  = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
schemars    = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }"
emit_lib ras-mcp

emit_cargo ras-cloud "Cloud browser client and OAuth device flow" \
"ras-errors  = { workspace = true }
ras-types   = { workspace = true }
async-trait = { workspace = true }
reqwest     = { workspace = true }
tokio       = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
url         = { workspace = true }
uuid        = { workspace = true }"
emit_lib ras-cloud

emit_cargo ras-agent "Agent step loop, history, plan, rerun orchestration" \
"ras-errors      = { workspace = true }
ras-types       = { workspace = true }
ras-events      = { workspace = true }
ras-cdp         = { workspace = true }
ras-browser     = { workspace = true }
ras-dom         = { workspace = true }
ras-tools       = { workspace = true }
ras-watchdogs   = { workspace = true }
ras-filesystem  = { workspace = true }
ras-llm         = { workspace = true }
ras-tokens      = { workspace = true }
ras-judge       = { workspace = true }
ras-telemetry   = { workspace = true }
async-trait     = { workspace = true }
tokio           = { workspace = true }
tokio-util      = { workspace = true }
serde           = { workspace = true }
serde_json      = { workspace = true }
thiserror       = { workspace = true }
tracing         = { workspace = true }
url             = { workspace = true }
uuid            = { workspace = true }
sha2            = { workspace = true }
indexmap        = { workspace = true }
chrono          = { workspace = true }"
emit_lib ras-agent

cat > ras-cli/Cargo.toml <<'EOF'
[package]
name                   = "ras-cli"
description            = "rust-ai-surfer command line interface"
version.workspace      = true
edition.workspace      = true
rust-version.workspace = true
license.workspace      = true
repository.workspace   = true
authors.workspace      = true

[lints]
workspace = true

[[bin]]
name = "ras"
path = "src/main.rs"

[features]
default               = ["anthropic-claude-code", "cosmium-launcher"]
anthropic-claude-code = ["ras-llm-anthropic/claude-code"]
cosmium-launcher      = []
mcp                   = ["dep:ras-mcp"]
cloud                 = ["dep:ras-cloud"]

[dependencies]
ras-errors         = { workspace = true }
ras-types          = { workspace = true }
ras-config         = { workspace = true }
ras-validation     = { workspace = true }
ras-agent          = { workspace = true }
ras-browser        = { workspace = true }
ras-cdp            = { workspace = true }
ras-cosmium        = { workspace = true }
ras-llm            = { workspace = true }
ras-llm-anthropic  = { workspace = true }
ras-tools          = { workspace = true }
ras-mcp            = { workspace = true, optional = true }
ras-cloud          = { workspace = true, optional = true }
clap               = { workspace = true }
tokio              = { workspace = true }
serde              = { workspace = true }
serde_json         = { workspace = true }
thiserror          = { workspace = true }
anyhow             = { workspace = true }
tracing            = { workspace = true }
tracing-subscriber = { workspace = true }
dotenvy            = { workspace = true }
url                = { workspace = true }
EOF
cat > ras-cli/src/main.rs <<'EOF'
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
EOF
mkdir -p ras-cli/src/domain ras-cli/src/application ras-cli/src/infrastructure
: > ras-cli/src/domain/mod.rs
: > ras-cli/src/application/mod.rs
: > ras-cli/src/infrastructure/mod.rs

cat > ras-daemon/Cargo.toml <<'EOF'
[package]
name                   = "ras-daemon"
description            = "rust-ai-surfer long-running session daemon"
version.workspace      = true
edition.workspace      = true
rust-version.workspace = true
license.workspace      = true
repository.workspace   = true
authors.workspace      = true

[lints]
workspace = true

[[bin]]
name = "ras-daemon"
path = "src/main.rs"

[dependencies]
ras-errors         = { workspace = true }
ras-types          = { workspace = true }
ras-config         = { workspace = true }
ras-browser        = { workspace = true }
ras-cdp            = { workspace = true }
tokio              = { workspace = true }
tokio-util         = { workspace = true }
serde              = { workspace = true }
serde_json         = { workspace = true }
thiserror          = { workspace = true }
anyhow             = { workspace = true }
tracing            = { workspace = true }
tracing-subscriber = { workspace = true }
EOF
cat > ras-daemon/src/main.rs <<'EOF'
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("ras-daemon ready");
    Ok(())
}
EOF
mkdir -p ras-daemon/src/domain ras-daemon/src/application ras-daemon/src/infrastructure
: > ras-daemon/src/domain/mod.rs
: > ras-daemon/src/application/mod.rs
: > ras-daemon/src/infrastructure/mod.rs

cat > xtask/Cargo.toml <<'EOF'
[package]
name         = "xtask"
description  = "rust-ai-surfer dev automation"
version      = "0.0.0"
edition.workspace      = true
rust-version.workspace = true
license.workspace      = true
publish      = false

[lints]
workspace = true

[[bin]]
name = "xtask"
path = "src/main.rs"

[dependencies]
anyhow     = { workspace = true }
clap       = { workspace = true }
serde      = { workspace = true }
serde_json = { workspace = true }
walkdir    = { workspace = true }
EOF

echo "all crate stubs emitted"

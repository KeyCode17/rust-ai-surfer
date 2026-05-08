# rust-ai-surfer

[![ci](https://github.com/KeyCode17/rust-ai-surfer/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/KeyCode17/rust-ai-surfer/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/KeyCode17/rust-ai-surfer)](https://github.com/KeyCode17/rust-ai-surfer/releases/latest)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![rust](https://img.shields.io/badge/rust-1.85-orange)](rust-toolchain.toml)

Rust port of [browser-use](https://github.com/browser-use/browser-use), driven by [Claude Code OAuth](https://github.com/anthropics/claude-code) and [Cosmium](https://github.com/maulanasdqn/cosmium) (patched Chromium for stealth scraping inside containers).

## What's in the box

- **LLM-driven agent loop** — plan, step, multi-act, judge, rerun, fallback LLM, action-loop detector, page-stagnation guard, budget warnings, URL shortener
- **BrowserSession over CDP** via [chromiumoxide](https://crates.io/crates/chromiumoxide) with per-request timeout decorator
- **Cosmium binary launcher** — fingerprint profile JSON → `--cosmium-*` switch mapping, free-port discovery, tempdir for `--user-data-dir`, ready-poll on `/json/version`; also dual `Attach` mode for an externally launched Chromium
- **★ ChatAnthropicClaudeCode** — 4-tier OAuth credential chain (env API-key bail → macOS Keychain → `~/.claude/.credentials.json` → `~/.claude/settings.json`), `cc_version` resolution from `claude --version`, byte-parity billing header injection
- **Tools registry** — 8 built-ins (navigate, click_element, click_coordinate, type_text, scroll, screenshot, wait, done), `terminates_sequence` flag, domain filter, per-action timeout
- **DOM extraction primitives** — dynamic class filter, stable hash (Sha256 of parent xpath + tag + id + role + filtered classes + ax_name), paint-order rect union, skeleton-page detector
- **Watchdogs** — security (allowed/prohibited domains + IPv4/IPv6 block), popups, crash, downloads — backed by an `async-broadcast` event bus
- **Multi-provider LLMs** — Anthropic + Claude Code OAuth, OpenAI + Azure, Google, Groq, Bedrock, OpenRouter, Vercel, DeepSeek, Cerebras, Mistral, Ollama, OCI, Cloud, LangChain (OpenAI-compatible providers re-export `ChatOpenAICompatible`)
- **FileSystem** — `BaseFile` per type, RFC 4180 CSV normalize, regex-validated filenames, tokio::fs-backed `LocalFileSystem`
- **Token-cost tracking** — `InMemoryTokenCost` with default pricing for Sonnet 4.5, Haiku 4.5, gpt-4o, gemini-2.0-flash
- **MCP** stdio JSON-RPC primitives, **sandbox** (`sh -c` with timeout), **recording** (frame collector), **judge**, **telemetry** (NoOp by default)

## Architecture

Hexagonal vertical-slice. Each crate owns one feature, internally sliced into `domain/`, `application/`, `infrastructure/`. Layer boundaries are enforced physically — `ras-domain` cannot import `chromiumoxide` because the parent crate doesn't pull it in for the domain modules.

```
rust-ai-surfer/
├── ras-cli/                      binary entry point          (apps)
├── ras-daemon/                   long-running session daemon (apps)
├── ras-errors/                   centralized AppError        (foundation)
├── ras-types/                    shared types + ID newtypes  (foundation)
├── ras-validation/               Validated<T> extractor      (foundation)
├── ras-config/                   env + logger bootstrap      (foundation)
├── ras-events/                   tokio broadcast event bus   (platform)
├── ras-cdp/                      chromiumoxide adapter       (platform)
├── ras-cosmium/                  cosmium launcher            (platform)
├── ras-llm/                      LlmClient port              (LLM core)
├── ras-llm-anthropic/            ★ ChatAnthropicClaudeCode   (LLM ★ main course)
├── ras-llm-{openai,google,groq,bedrock,openrouter,vercel,deepseek,cerebras,mistral,ollama,oci,cloud,langchain}/
├── ras-browser/                  BrowserSession mode dispatch
├── ras-dom/                      DOM tree + clickable + hash
├── ras-tools/                    action registry + built-ins
├── ras-watchdogs/                security, popups, crash, downloads
├── ras-filesystem/               Csv/Md/Json/Jsonl/Html/Docx/Pdf/Txt
├── ras-tokens/                   token-cost service
├── ras-judge/                    judge eval port
├── ras-telemetry/                anonymized telemetry events
├── ras-recording/                session recording
├── ras-sandbox/                  sandboxed code execution
├── ras-skills/                   skills service port
├── ras-mcp/                      MCP server + client
├── ras-cloud/                    cloud browser client
├── ras-agent/                    ★ orchestrator
├── xtask/                        dev automation
├── examples/                     ★ claude_code_oauth_cosmium
├── docs/                         architecture + ADRs
└── scripts/                      lefthook helpers
```

41 workspace members (37 lib + 2 bin + xtask + examples). 50+ unit/integration tests pass.

## Quick start

Pre-requisites:

- Rust 1.85 (auto-installed on first cargo invocation via `rust-toolchain.toml`)
- `claude` CLI logged in (for the OAuth path)
- A Chromium-family binary (the cosmium binary or any plain Chromium for testing)

```bash
git clone https://github.com/KeyCode17/rust-ai-surfer
cd rust-ai-surfer
cargo install lefthook
lefthook install
cargo build --workspace
cargo test --workspace
cargo run -p ras-cli -- --help
```

End-to-end OAuth + cosmium PoC:

```bash
# 1. launch a Chromium-family browser with CDP enabled
chromium --remote-debugging-port=9222 --user-data-dir=/tmp/cdp-profile --no-first-run &

# 2. verify CDP is up
curl -s http://127.0.0.1:9222/json/version | jq .

# 3. run the example
cargo run --example claude_code_oauth_cosmium --manifest-path examples/Cargo.toml
```

Output:

```
[ok] Claude Code OAuth resolved, cc_version=2.1.133
[ok] BrowserSession attached to ws://127.0.0.1:9222/devtools/...
[done] (final result from agent)
```

Env knobs honored by the example: `RAS_MODEL` (default `claude-sonnet-4-5`), `CDP_URL` (default `http://127.0.0.1:9222`), `TASK` (default heading-extraction on example.com).

## CLI

```
ras run --task "open example.com and report the heading" --max-steps 10
ras doctor
ras login
ras version
```

`ras doctor` checks `claude` CLI presence, `ANTHROPIC_API_KEY` shadow status, and `~/.claude/.credentials.json` existence.

## Versioning + release flow

| Bump | When |
|------|------|
| Major (X) | Architectural overhaul; full ports landed |
| Minor (Y) | New phase or feature surface |
| Patch (Z) | Bug fix, CI tweak, dependency pin |

Phase map and full release history in [`CHANGELOG.md`](CHANGELOG.md). Releases on [GitHub Releases](https://github.com/KeyCode17/rust-ai-surfer/releases).

The bump-gate (`scripts/bump-gate.sh`, fired on `git push`) requires `cargo build --workspace --all-targets`, `cargo test --workspace`, and `ras --help` to pass before a tagged version goes out.

```bash
cargo run -p xtask -- bump patch       # 1.0.0 -> 1.0.1
cargo run -p xtask -- bump minor       # 1.0.0 -> 1.1.0
cargo run -p xtask -- bump major       # 1.0.0 -> 2.0.0
git push --follow-tags
```

## Contributing

`lefthook install` after clone wires up:

| Hook | Commands |
|------|----------|
| pre-commit | `cargo fmt --check`, `cargo clippy -D clippy::unwrap_used -D clippy::dbg_macro`, ≤200 LOC per file, no `//` comments (only `///` and `//!` doc comments allowed), no `.unwrap()` |
| commit-msg | Conventional commits (`feat:`, `fix:`, `chore:`, …) |
| pre-push | `cargo test --workspace --no-fail-fast`, `cargo doc --workspace --no-deps`, bump-gate (only fires on version change) |

`--no-verify` is not used. If a hook fails, fix the underlying issue.

## Architecture decisions

ADRs live in [`docs/adr/`](docs/adr/):

1. [Hexagonal vertical-slice workspace](docs/adr/0001-hexagonal-vertical-slice.md)
2. [chromiumoxide for CDP](docs/adr/0002-chromiumoxide-cdp.md)
3. [Cosmium launcher coupling](docs/adr/0003-cosmium-launcher-coupling.md)
4. [Claude Code OAuth as default LLM credential](docs/adr/0004-claude-code-oauth.md)

## Porting status

This is a feature-for-feature port of [browser-use](https://github.com/browser-use/browser-use) shipped across 11 phases. The end-to-end OAuth + cosmium path is the verified main course. See [`docs/porting-from-browser-use.md`](docs/porting-from-browser-use.md) for the phase-by-phase map and the naming map between the Python source and the Rust crate layout.

## License

MIT — see [LICENSE](LICENSE).

# rust-ai-surfer

Rust port of [browser-use](https://github.com/browser-use/browser-use), driven by [Claude Code OAuth](https://github.com/anthropics/claude-code) and [Cosmium](https://github.com/maulanasdqn/cosmium) (patched Chromium for stealth scraping).

## Scope

Full port of the browser-use Python library to idiomatic Rust:

- LLM-driven agent loop (plan, step, multi-act, judge, rerun, fallback)
- BrowserSession over CDP via [chromiumoxide](https://crates.io/crates/chromiumoxide)
- Cosmium binary launcher (profile JSON → `--cosmium-*` switch mapping)
- Tools / actions registry with built-ins (click, type, scroll, navigate, screenshot, extract, dropdown, upload, save-as-pdf, ...)
- DOM + accessibility tree extraction with stable hashing and clickable detection
- Watchdogs (security, downloads, popups, crash, recording, storage)
- Multi-provider LLM clients (Anthropic + Claude Code OAuth, OpenAI, Google, Groq, Bedrock, OpenRouter, Vercel, DeepSeek, Cerebras, Mistral, Ollama, OCI, Cloud)
- MCP server + client, sandbox, skills, telemetry, judge, recording

## Layout

Hexagonal vertical-slice. Each crate owns one feature, sliced into `domain/`, `application/`, `infrastructure/`.

```
rust-ai-surfer/
├── ras-cli/                      binary entry point
├── ras-daemon/                   long-running session daemon
├── ras-errors/                   centralized AppError
├── ras-types/                    shared types and ID newtypes
├── ras-validation/               Validated<T> extractor
├── ras-config/                   env + logger bootstrap
├── ras-events/                   tokio broadcast event bus
├── ras-cdp/                      chromiumoxide adapter
├── ras-cosmium/                  cosmium launcher + profile mapping
├── ras-llm/                      LlmClient port + ChatMessage
├── ras-llm-anthropic/            ★ Anthropic + ChatAnthropicClaudeCode
├── ras-llm-{openai,google,groq,bedrock,openrouter,vercel,deepseek,cerebras,mistral,ollama,oci,cloud,langchain}/
├── ras-browser/                  BrowserSession mode dispatch
├── ras-dom/                      DOM tree + clickable + hash + serializer
├── ras-tools/                    action registry + built-ins
├── ras-watchdogs/                security, downloads, popups, crash, ...
├── ras-filesystem/               file types + state
├── ras-tokens/                   token-cost tracking
├── ras-judge/                    judge eval
├── ras-telemetry/                anonymized telemetry events
├── ras-recording/                ffmpeg-based session recording
├── ras-sandbox/                  sandboxed code execution
├── ras-skills/                   browser-use Skills service
├── ras-mcp/                      MCP server + client
├── ras-cloud/                    CloudBrowserClient + DeviceAuth
├── ras-agent/                    ★ Agent orchestrator
├── xtask/                        dev automation (bump, check-loc, ...)
├── examples/                     runnable PoCs
├── tests/                        workspace-level e2e
└── docs/                         architecture + ADRs
```

## Versioning

| Bump | When |
|------|------|
| Major (X) | Full port complete, all phases shipped |
| Minor (Y) | One phase landed + bump-gate passes |
| Patch (Z) | Bug fix / docs / CI tweak |

Phase → version map in [`CHANGELOG.md`](CHANGELOG.md).

## Quick start

```bash
cargo build --workspace
cargo nextest run --workspace
cargo run -p ras-cli -- --help
```

End-to-end OAuth + cosmium PoC (requires `claude` CLI logged in and a cosmium binary):

```bash
chromium --remote-debugging-port=9222 --user-data-dir=/tmp/cdp-profile --no-first-run &
cargo run --example claude_code_oauth_cosmium --manifest-path examples/Cargo.toml
```

## Contributing

`lefthook install` after clone. Pre-commit enforces fmt, clippy, LOC ≤ 200, no comments, no `unwrap()`. Pre-push runs the full test matrix and bump gate.

## License

MIT

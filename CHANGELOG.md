# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [SemVer](https://semver.org/).

Versioning policy:

- **0.Y.Z** during the porting effort. `Y` bumps once per phase after the bump-gate (`cargo build --workspace && cargo nextest run --workspace`) passes.
- **1.0.0** ships when all 11 phases land and `examples/claude_code_oauth_cosmium.rs` runs end-to-end against a live cosmium binary.

## [Unreleased]

## [0.3.0] - Phase 2 — CDP + cosmium + ChatAnthropicClaudeCode

- ras-cdp infra: `ChromiumoxideAdapter` implementing `BrowserPort` (navigate, click, type, screenshot, target lifecycle), `within()` per-request timeout wrapper.
- ras-cosmium infra: `CosmiumProcessLauncher` (subprocess spawn, free-port, tempdir for `--user-data-dir`, `--cosmium-*` flag mapping, ready-poll via `/json/version`), `resolve_attach_url()` for attach mode.
- ras-llm-anthropic infra: ★ `ChatAnthropicClaudeCode` decorator over `ChatAnthropic`. 4-tier auth chain (`ResolveOauthCredentials`): `ANTHROPIC_API_KEY` bail → `MacosKeychain` (via `security` cmd) → `~/.claude/.credentials.json` → `~/.claude/settings.json`. Billing header injection (`inject_billing_header`) + byte-parity headers + `claude --version` parsing fallback.
- 8 tests pass: billing header text + injection (existing system / no system), cc_version semver guard + fallback, cosmium profile flag emission.

## [0.2.0] - Phase 1 — Domain layer

- Pure domain types and ports across all 37 lib crates.
- Claude Code OAuth domain types, `BrowserEvent`/`EventBus`, `BrowserPort`, `LlmClient` + chat messages, DOM tree + clickable, action registry, watchdog port, file system, judge, telemetry, recording, sandbox, skills, MCP, cloud, agent history + plan + step metadata + loop detector.
- `cargo check --workspace` passes for all 39 members.

## [0.1.0] - Phase 0 — workspace scaffold

- Workspace layout: 36 member crates flat at root + `xtask`.
- Hexagonal vertical-slice shape per crate.
- `lefthook.yml` for pre-commit and pre-push.
- `xtask bump {major|minor|patch}` automation.

# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [SemVer](https://semver.org/).

Versioning policy:

- **0.Y.Z** during the porting effort. `Y` bumps once per phase after the bump-gate (`cargo build --workspace && cargo nextest run --workspace`) passes.
- **1.0.0** ships when all 11 phases land and `examples/claude_code_oauth_cosmium.rs` runs end-to-end against a live cosmium binary.

## [Unreleased]
## [1.0.2] - CI release permissions + dependabot pin

- `release.yml`: add `permissions: contents: write` so the workflow can attach binaries to manually-created releases.
- `dependabot.yml`: ignore `sha2` major bumps (0.11 changed `Hasher` API), ignore `chromiumoxide` majors (0.8 has breaking API), ignore `dtolnay/rust-toolchain` action bumps (we pin via `rust-toolchain.toml`).
- Closed open dependabot PRs that broke the build.
## [1.0.1] - CI fixes

- Drop pedantic clippy from workspace lints (kept `unwrap_used = deny`, `dbg_macro = deny`).
- Drop `RUSTFLAGS=-D warnings` from CI; clippy job uses `-D clippy::unwrap_used -D clippy::dbg_macro` instead.
- Drop unstable rustfmt options (`imports_granularity`, `group_imports`) so stable toolchain doesn't warn.
- Remove unused `tokio_util::sync::CancellationToken` import in `ras-watchdogs`.
## [1.0.0] - Full port complete

All 11 phases shipped. Workspace at 39 crates, 50+ tests pass, end-to-end PoC builds. Major bump.
## [0.12.0] - Phase 11 — Examples + integration

- `examples/claude_code_oauth_cosmium.rs` (★ port of the Python PoC): `ChatAnthropicClaudeCode::new(model)` + `resolve_attach_url(cdp_url)` + `ChromiumoxideAdapter::connect` + `RunAgent::execute`. Honors `RAS_MODEL`, `CDP_URL`, `TASK` env knobs.
- Workspace builds clean; example builds clean. Total tests across the workspace: 50+ unit/integration.
## [0.11.0] - Phase 10 — MCP + sandbox + recording

- `ras-sandbox`: `ShellSandbox` runs scripts via `sh -c` with `tokio::process::Command`, env injection, hard timeout via `tokio::time::timeout`.
- `ras-recording`: `InMemoryRecorder` start/frame/stop with `RecordingState` tracking frame_count + started_at.
- `ras-mcp`: `JsonRpcRequest` / `JsonRpcResponse::{ok, err}` (codes from MCP spec), stdio-line protocol primitives.
- 7 tests pass.
## [0.10.0] - Phase 9 — CLI + daemon + cloud

- `ras` CLI with subcommands: `run` (task + model + cdp_url + cosmium_binary + max_steps), `doctor` (check claude CLI + ANTHROPIC_API_KEY shadow + ~/.claude/.credentials.json), `login` (OAuth instructions), `version`.
- `ras-daemon` Unix-socket listener at `$XDG_RUNTIME_DIR/ras-daemon.sock`; JSON-line protocol with Ping / Status / Shutdown.
- `HttpCloudClient` provisions cloud browsers via `POST /v1/browsers`, releases via `DELETE /v1/browsers/{id}`.
## [0.9.0] - Phase 8 — FileSystem

- `normalize_csv` (RFC 4180): quote fields with commas / quotes / newlines, escape internal quotes via doubling, strip leading/trailing blank lines, fields starting with `"` parsed as quoted.
- `parse_filename` regex `^[A-Za-z0-9_\-]+\.[A-Za-z0-9]+$`; rejects path separators + unsupported extensions; `sanitize` replaces spaces with `_` and drops disallowed chars.
- `LocalFileSystem` (FileSystemPort impl): tokio::fs-backed read/write/append/list/snapshot, csv-extension auto-normalize on write, list filters to known extensions only.
- 15 tests pass.
## [0.8.0] - Phase 7 — LLM providers + token cost

- ras-llm-openai: `ChatOpenAICompatible` + `OpenAiAuth::{Bearer, Header}` covering OpenAI plus all OpenAI-compatible providers (groq, cerebras, deepseek, mistral, openrouter, vercel) via re-exports.
- ras-tokens: `InMemoryTokenCost` impl with default pricing for claude-sonnet-4-5, claude-haiku-4-5, gpt-4o, gemini-2.0-flash; record/aggregate API tracks per-model usage.
- 3 token-cost tests pass.
## [0.7.0] - Phase 6 — Watchdogs + event bus

- 4 watchdog impls: `SecurityWatchdog` (allowed/prohibited domains + IP block, IPv6 bracket-aware), `PopupsWatchdog`, `CrashWatchdog`, `DownloadsWatchdog`.
- Each watchdog spawns a tokio task on attach, listens to BroadcastBus, exits on cancel token.
- 7 tests pass.
## [0.6.0] - Phase 5 — DOM extraction primitives

- `filter_dynamic_classes`: drops state classes (is-active, hover, focus, ...), hashed (css-, sc-, emotion-), long-digit-runs, BEM-modifier-with-uppercase patterns; dedup + sort.
- `stable_hash`: parent_xpath + tag + id + role + filtered_classes + ax_name -> Sha256. Stable across CSS-in-JS class hash churn.
- `remove_occluded`: paint-order rect union with reverse traversal; later-painted boxes mask earlier ones.
- `detect_skeleton`: total_elements > 20 AND text_chars < total_elements * 5.
- 13 tests pass.
## [0.5.0] - Phase 4 — Tools registry + built-ins

- 8 default actions: navigate, click_element, click_coordinate, type_text, scroll, screenshot, wait, done.
- `register_default_actions` populates `ActionRegistry`; `terminates_sequence` flag set on navigate + done.
- click variant pair (index/coordinate) for OAuth/anthropic-style coord clicking opt-in.
- screenshot encodes PNG to base64 via inline encoder (no extra dep).
- 5 registry tests pass.
## [0.4.0] - Phase 3 — Agent step loop

- `RunAgent` orchestrator + `RunStep` use case (LLM call + parse + history append + loop-detector update + step-interval tracking).
- `compute_action_hash` with normalization rules (search strips `max_results`/`offset`, navigate strips query string).
- `ActionLoopDetector` + `PageFingerprint` integrated; nudges built via `build_loop_nudge` and `build_budget_warning` (75% / 90% escalation).
- `should_switch_to_fallback`: classifies `LlmRateLimited`/`LlmAuthExpired`/`LlmProviderError`/`CdpTimeout`/`BrowserDisconnected` for fallback LLM swap inside `RunStep::invoke_with_fallback`.
- `UrlShortener` with map-backed shorten/restore (>= 80 char threshold, dedup).
- `render_plan` produces `[x]/[>]/[ ]` markers for plan items.
- 12 tests pass.

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

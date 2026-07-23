# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [SemVer](https://semver.org/).

Versioning policy:

- **0.Y.Z** during the porting effort. `Y` bumps once per phase after the bump-gate (`cargo build --workspace && cargo nextest run --workspace`) passes.
- **1.0.0** ships when all 11 phases land and `examples/claude_code_oauth_cosmium.rs` runs end-to-end against a live cosmium binary.

## [Unreleased]
## [4.3.0] - Per-step screenshots + push-gate hardening

### Added

- **Per-step screenshot sink.** `RunAgent::with_screenshot_sink(...)` captures one screenshot per step once its actions settle. The destination is a port — `ras_agent::StepScreenshotSink` — so a host project can persist to a database or object storage without this workspace taking on a storage dependency. `FolderScreenshotSink` ships as the local-directory default, writing `{root}/{agent_id}/step-{n:04}.png` with step numbers zero-padded so lexicographic order matches run order.
- `StepRecord.screenshot` carries the stored location rather than inline bytes, keeping history JSON small. The field is `#[serde(default)]`, so history written by earlier versions still deserializes.
- `SpawnParams.screenshot_sink` threads the sink into sessions, so each spawned agent writes under its own directory.

### Changed

- `RunStep::new` takes a `RunStepDeps` parameter struct instead of seven positional arguments.
- Screenshot capture and save failures log at `warn` and yield no artifact; a full disk or a dead CDP connection degrades the record, never the run.
- `scripts/bump-gate.sh` derives the required bump from the commits a branch adds to `origin/main` and fails on a missing, backwards, or wrong-sized change. It previously returned success when the version was unchanged, so pushes landed with no bump at all — the root version had drifted to `2.7.0` while `v4.1.0` was tagged.
- `scripts/check-commit-msg.sh` rejects `chore(scope)`, `docs(scope)`, the invented `release:` type, capitalized and period-terminated subjects, vague subjects, and AI attribution trailers.
- Docs sorted into `docs/guides/` and `docs/rfcs/`; README pointed at a specs file that never existed.

### Removed

- `clear_context_cookies` in `ras-cdp`, which had no callers and carried an `allow(dead_code)` to stay quiet. `BrowserPort::clear_cookies` is per-target and backed by `cdp_clear_cookies`.

> Entries for the `3.x` line and `4.0.0`–`4.2.0` are missing from this file; see the [GitHub Releases](https://github.com/KeyCode17/rust-ai-surfer/releases) page for those.

## [2.1.0] - Dependency patches + publish workflow polish

- Dependabot retargeted at `develop` (matches branch flow); `keyring` major bumps added to ignore list.
- Patch bumps: `smol_str` 0.3.2 → 0.3.6, `image` 0.25.9 → 0.25.10.
- `publish.yml`: `set +e -o pipefail` so cargo-search misses don't abort; treat `already exists on crates.io` as success; 420s sleep on 429 rate-limit; failed crates aggregated and reported at end.
## [2.0.0] - Major bump + main branch + crates.io publish

- Pin Rust toolchain to **1.95.0** (latest stable as of 2026-04-16); CI workflows + audit + workspace `rust-version` all aligned. Profile `minimal` to skip `rust-docs`.
- `main` becomes the default branch; `develop` remains the integration branch.
- crates.io publish workflow uses precompiled `cargo-workspaces` via `taiki-e/install-action` (avoids MSRV friction).
- Publish gate: minor or major bump AND tag reachable from `origin/main`.
## [2.0.0] - Major bump + main branch + first crates.io publish

- Branch policy: `main` becomes the default branch; `develop` is the integration branch.
- crates.io: this tag fires the publish workflow (minor-or-major + tagged on `origin/main`). All publishable crates land on crates.io as `2.0.0`.
- No source-level breaking changes vs 1.1.0; the major bump marks the publishing contract going public.
## [1.1.0] - First minor bump

- All `1.0.x` patches consolidated into a stable minor release line.
- crates.io publish workflow active but still gated to `origin/main` (not yet created).
- Next: `2.0.0` will create `main` branch, set as default, and trigger the first crates.io publish.
## [1.0.5] - crates.io publish prep

- Add `version = "1.0.5"` to all internal `[workspace.dependencies]` so `cargo publish` accepts the path deps.
- Mark `ras-cli` and `ras-daemon` as `publish = false` (binaries; `cargo install` from git instead).
- `publish.yml`: gate publish on (a) tagged commit reachable from `origin/main` AND (b) minor or major bump vs prior tag. Patch bumps and tags off non-main branches always skip crates.io.
## [1.0.4] - README refresh

- Update `README.md`: add CI / release / license / Rust badges, expand "What's in the box" to reflect shipped capabilities, document CLI subcommands, link ADRs, document lefthook hook matrix, mention bump-gate flow.
## [1.0.3] - Pin remaining dependabot conflicts

- Add `schemars` (semver-major), `validator`, `wiremock` to dependabot ignore list. Closed PRs #4, #5, #8.
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

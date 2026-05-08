# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [SemVer](https://semver.org/).

Versioning policy:

- **0.Y.Z** during the porting effort. `Y` bumps once per phase after the bump-gate (`cargo build --workspace && cargo nextest run --workspace`) passes.
- **1.0.0** ships when all 11 phases land and `examples/claude_code_oauth_cosmium.rs` runs end-to-end against a live cosmium binary.

## [Unreleased]
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

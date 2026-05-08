# ADR-0004: Claude Code OAuth as default LLM credential

## Status

Accepted - 2026-05-08

## Context

`browser_use/llm/anthropic/claude_code.py` reuses the local Claude Code CLI's OAuth login so agent runs bill against a Claude subscription instead of API credits. The Python implementation:

- Refuses to run if `ANTHROPIC_API_KEY` is set.
- Reads OAuth tokens from (in order) macOS Keychain (`security find-generic-password -s "Claude Code-credentials" -w`), `~/.claude/.credentials.json`, `~/.claude/settings.json`.
- Validates `expiresAt` against wall clock.
- Sends a coordinated set of headers plus a body-level billing line.
- Tracks the installed `claude` CLI version via `claude --version`, with a pinned fallback constant.

## Decision

Port verbatim into `ras-llm-anthropic` as `ChatAnthropicClaudeCode`, structured as a decorator over `ChatAnthropic`. The 4-tier auth chain becomes the `ResolveOauthCredentials` use case in `application/`. Each repository (env, keychain, credentials file, settings file) is a separate adapter in `infrastructure/`.

The `claude-code` Cargo feature gates the optional dependencies (keyring, security-framework on macOS).

## Consequences

- Header byte-parity matters; a regression test pins the exact set.
- The decorator pattern means `ChatAnthropic` (plain API key) and `ChatAnthropicClaudeCode` share the request/response code path.
- macOS-only keychain access via `security-framework` plus `keyring` for cross-platform parity later.

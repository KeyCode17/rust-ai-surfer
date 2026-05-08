# ADR-0002: chromiumoxide for CDP

## Status

Accepted - 2026-05-08

## Context

browser-use uses `cdp-use`, a thin Python typed wrapper around CDP. Rust has several options:

1. `chromiumoxide` - mature, async, Playwright-style high-level API plus raw CDP escape hatch.
2. Hand-rolled atop `chromiumoxide_cdp` typed protocol crate.
3. `headless_chrome` - older, blocking-style.
4. `fantoccini` - WebDriver, not CDP.

Cosmium's README explicitly mentions chromiumoxide as a supported client.

## Decision

chromiumoxide as the CDP adapter, wrapped in our own `BrowserPort` trait inside `ras-cdp`. We also own `TimeoutWrappedCdpClient` (per-request timeout) and `CdpSessionManager` (target/session lifecycle), since chromiumoxide's defaults do not match the semantics browser-use established.

## Consequences

- Single-source CDP dependency.
- Adapter pattern means swapping libraries later is a one-crate change.
- `ras-cdp` is the only crate that knows chromiumoxide types exist.

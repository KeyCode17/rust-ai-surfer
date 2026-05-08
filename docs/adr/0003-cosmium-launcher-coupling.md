# ADR-0003: Cosmium launcher coupling

## Status

Accepted - 2026-05-08

## Context

Cosmium is a sibling project (patched Chromium + Rust workspace). It exposes:

- A patched chrome binary at a known path with `--cosmium-*` switches.
- A Rust `Profile` domain type that maps to those switches.

We need to decide whether to depend on cosmium as a Cargo dependency or stay loosely coupled.

## Decision

`ras-cosmium` depends on cosmium via path dependency when it lives next to us in the same workspace, and via git dependency otherwise. The dependency is scoped to the `Profile` value object only; we re-export a minimal subset.

`BrowserSessionMode::Launch` in `ras-browser` accepts either a path to the cosmium binary (we spawn) or an existing `cdp_url` (we attach), matching the dual-mode behaviour of browser-use's `BrowserSession`.

## Consequences

- Tight type sharing for `Profile`; minimal version churn risk.
- The launcher logic (free-port discovery, tempdir for `--user-data-dir`, subprocess lifecycle, kill on `Drop`, graceful shutdown via `Browser.close` then `SIGTERM`) lives in `ras-cosmium`, not in cosmium itself.

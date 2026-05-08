# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [SemVer](https://semver.org/).

Versioning policy:

- **0.Y.Z** during the porting effort. `Y` bumps once per phase after the bump-gate (`cargo build --workspace && cargo nextest run --workspace`) passes.
- **1.0.0** ships when all 11 phases land and `examples/claude_code_oauth_cosmium.rs` runs end-to-end against a live cosmium binary.

## [Unreleased]

## [0.1.0] - Phase 0 — workspace scaffold

- Workspace layout: 36 member crates flat at root + `xtask`.
- Hexagonal vertical-slice shape per crate (`domain/`, `application/`, `infrastructure/`).
- `lefthook.yml` for pre-commit (fmt + clippy + LOC + no-comments + no-unwrap + deny) and pre-push (test + doc + audit + bump-gate).
- `xtask bump {major|minor|patch}` automation.

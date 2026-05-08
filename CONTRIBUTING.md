# Contributing

## Setup

```bash
git clone https://github.com/KeyCode17/rust-ai-surfer
cd rust-ai-surfer
cargo install lefthook cargo-nextest cargo-deny
lefthook install
cargo build --workspace
```

The toolchain pins itself to 1.85 via `rust-toolchain.toml` on first cargo invocation.

## Workflow

- One feature crate per concern (`ras-{feature}`).
- One use case per file inside `src/application/`.
- `domain/` modules import only `ras-errors`, `ras-types`, and the crate's own `domain`.
- ≤200 LOC per file, no comments, no `unwrap()` outside `tests/` or `examples/`.
- Use `Arc<dyn Trait>` for dependency injection between layers.

## Pre-commit hooks

`lefthook` enforces:

- `cargo fmt --check`
- `cargo clippy -D warnings`
- LOC ≤ 200
- No `//` comments (only `///` and `//!` doc comments allowed)
- No `.unwrap()`

## Pre-push hooks

- `cargo nextest run --workspace`
- `cargo doc --workspace`
- Bump gate (only fires when the workspace `version` changed since the last tag)

## Phase + version

| Phase | Subject | Version target |
|-------|---------|----------------|
| 0 | Scaffold | 0.1.0 |
| 1 | Domain layer | 0.2.0 |
| 2 | CDP + cosmium + ChatAnthropicClaudeCode | 0.3.0 |
| ... | ... | ... |
| 11 | Examples + integration tests | 0.12.0 |
| Release | Full port | 1.0.0 |

To bump:

```bash
cargo run -p xtask -- bump minor 1
git push --follow-tags
```

`xtask bump` runs `cargo build --workspace && cargo test --workspace` first, then commits and tags.

## Conventional commits

`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:`, `perf:`, `build:`, `ci:`, `style:`, `revert:`. Optional scope: `feat(llm):`, `fix(cdp):`, ...

## Architecture decisions

ADRs live in [`docs/adr/`](docs/adr/). New significant decisions get a new ADR file.

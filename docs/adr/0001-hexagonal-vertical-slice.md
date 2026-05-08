# ADR-0001: Hexagonal vertical-slice workspace

## Status

Accepted - 2026-05-08

## Context

The Python source (browser-use) is 432 files across many concerns. A naive port into a single Rust crate would compile slowly, blur layer boundaries, and make the LLM-provider fan-out hard to feature-flag.

## Decision

One Cargo crate per feature. Each crate carries the full hexagonal triple internally:

- `domain/` - entities, value objects, repository ports
- `application/` - one use case per file
- `infrastructure/` - adapters implementing the ports

Layer boundaries are enforced physically: a crate's `domain/` cannot import third-party SDKs because the parent `Cargo.toml` does not pull them in for the domain modules. Direction is checked by `tests/arch.rs`.

## Consequences

- Compilation parallelizes: editing `ras-llm-anthropic` does not touch `ras-llm-openai`.
- Layer purity is harder to violate by accident.
- More `Cargo.toml` files to maintain, mitigated by workspace inheritance.
- Public API is the umbrella crate (or feature-flagged re-exports from `ras-cli`).

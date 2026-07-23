# Multi-Tenant Phase 2 — Egress / SSRF Policy (AR-5)

**Goal:** Stop a tenant's agent from reaching forbidden destinations (cloud metadata, loopback incl. the CDP port, private ranges, `file:`/`chrome:`/`data:`), via a pure `EgressPolicy` engine plus a navigation gate.

**Tech:** Rust, `url`, `std::net`, `ras_types::DomainPattern`.

**Repo rules:** ≤200 LOC/file, no `//` comments, no `.unwrap()` in non-test code, conventional commits.

## Tasks
1. **EgressPolicy engine** — `ras-validation/src/domain/egress.rs`: pure `check(&Url) -> Result<(), EgressError>`. Scheme allowlist (default http/https), block loopback/private/link-local/metadata IPs + `localhost`, denied ports, consumer allow/deny via `DomainPattern`. Comprehensive offline unit tests.
2. **Navigate gate** — `ras-tools/src/infrastructure/builtin/navigate.rs`: run `EgressPolicy::default().check(&url)` before navigating; reject with `ValidationError`.

**Scope note:** The CDP-layer `Fetch.requestPaused` enforcement (covers 3xx redirects + JS `window.location`, per AR-5) is **Phase 2b**, tracked separately — this phase ships the policy engine + the direct-navigation gate (closes the primary SSRF vector: an agent steered straight to metadata/localhost/file). DNS-rebinding (host resolves to a private IP at fetch time) is also deferred to 2b with the request-layer hook.

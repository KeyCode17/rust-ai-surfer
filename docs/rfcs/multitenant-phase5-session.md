# Multi-Tenant Phase 5 — `ras-session` crate (capstone)

**Goal:** Optional convenience layer: `SessionManager` maps a tenant `Owner` → an isolated session (its own `BrowserContext` + tab + per-session `EventBus` + bound agent). Bounded sessions, idle eviction, one-active-task-per-session. Isolation strategy behind `BrowserProvider` (ships context-per-session; consumer can swap process-per-tenant).

**Repo rules:** ≤200 LOC/file, no `//` comments, no `.unwrap()` in non-test code, conventional commits.

## File structure (each ≤200 LOC)
- `ras-session/src/lib.rs` — re-exports.
- `ras-session/src/config.rs` — `AgentSessionId`, `SessionConfig`, `OnFull`, `SessionError`.
- `ras-session/src/provider.rs` — `BrowserProvider` trait + `SharedBrowserProvider`.
- `ras-session/src/spawn_params.rs` — `SpawnParams` (llm, registry, dom_extractor, max_steps).
- `ras-session/src/manager.rs` — `SessionManager<Owner>`: `spawn`/`get`/`list` + idle reaper.
- `ras-session/src/handle.rs` — `SessionHandle`: `id`/`events`/`run`/`close`.

## Tasks
1. **Scaffold + config + provider** — add crate to workspace members + `[workspace.dependencies]`; `config.rs` types; `BrowserProvider` trait + `SharedBrowserProvider` (acquire = `browser.create_context()`; release = `browser.close_context()`). Unit tests for `SharedBrowserProvider` with a mock `BrowserPort`.
2. **Manager + handle** — `SessionManager<Owner: Eq+Hash+Clone>`: `spawn` (enforce `max_sessions`/`on_full` Reject|EvictOldest; one-per-owner reuse unless `allow_multi_per_owner`; `provider.acquire()` → `new_target_in` → per-session `BroadcastBus` → `attach_events`), `get`, `list`; background idle reaper closes non-running sessions past `idle_timeout`. `SessionHandle`: `events()`=bus.subscribe; `run(task)` CAS a running flag (else `SessionError::Busy`), builds `RunAgent::new(..).with_target(tab)[.with_dom_extractor]`, `execute()`; `close(self)`=`provider.release(ctx)`. Lifecycle unit tests with a mock `BrowserProvider`/`BrowserPort` (no live browser/LLM): max+on_full, one-per-owner, idle eviction, busy guard, close releases context.

**Scope note:** Abandoned sessions are reclaimed by the idle reaper (Rust has no async `Drop`, so cleanup is explicit `close()` + reaper, not RAII-on-drop). A full live "two sessions, two agents, cookie isolation" run needs LLM credentials → documented as a manual integration test, not in CI.

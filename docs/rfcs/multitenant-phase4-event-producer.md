# Multi-Tenant Phase 4 — Context-Tagged Event Producer

**Goal:** Build the missing CDP→`BrowserEvent` producer (audit found events were drained and discarded). Per-tab listeners publish to a supplied `EventBus`, so a session that gives its own bus gets ONLY its own tab's events — structural per-session isolation, no global firehose.

**Repo rules:** ≤200 LOC/file, no `//` comments, no `.unwrap()` in non-test code, conventional commits.

## Tasks
1. **Adapter split** — `chromiumoxide_adapter.rs` is at the 200-LOC cap. Move the inherent `impl ChromiumoxideAdapter` (`connect`, `browser_arc`) and the `Debug` impl into a new `chromiumoxide_adapter_setup.rs` (inherent impls may live in a sibling file). Leaves room in the main file for the new trait method. Build green, both files ≤200.
2. **Event pump + `attach_events`** — `ras-cdp/src/infrastructure/event_pump.rs`: given a `chromiumoxide::Page`, a `TargetId`, and `Arc<dyn EventBus>`, spawn listener task(s):
   - `EventFrameNavigated` (main frame only) → `BrowserEvent::NavigationCompleted { target, url }`.
   - (if straightforward) `EventJavascriptDialogOpening` → `BrowserEvent::DialogOpened { kind, message }`.
   Add `BrowserPort::attach_events(&self, target: &TargetId, bus: Arc<dyn EventBus>) -> Result<(), AppError>` (default-error impl; adapter resolves the `Page` and calls the pump).
3. **Live e2e** — `#[ignore]`+`CDP_URL`: create context+tab, `attach_events` with a `BroadcastBus`, subscribe, navigate, assert a `NavigationCompleted` arrives.

**Scope note:** Phase 4 establishes the producer architecture + navigation/dialog events. Broader event coverage (downloads, network, target lifecycle) is incremental and can be added later without changing the per-tab→bus model.

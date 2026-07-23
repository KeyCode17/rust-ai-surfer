# Multi-Tenant Phase 3 — Agent Target Binding (AR-3)

**Goal:** Agents drive an explicitly bound tab, not "the last focused tab" — removing the cross-session race. Builtins read the target from `ToolContext`; `RunAgent` can bind a target; single-user keeps the `focused_target` fallback (only in `run_step`, never in builtins).

**Repo rules:** ≤200 LOC/file, no `//` comments, no `.unwrap()` in non-test code, conventional commits.

## One cohesive change (interdependent — implement together)
- `ras-tools/src/domain/registry.rs`: add `pub target: Option<TargetId>` to `ToolContext`.
- 7 builtins (navigate, click ×2, type_text, scroll, screenshot, press_and_hold ×2): replace `let target = ctx.browser.focused_target().await?;` with `let target = ctx.target.clone().ok_or_else(|| AppError::NotFound("no active target".into()))?;`.
- `ras-agent/src/application/run_agent.rs`: add `bound_target: Option<TargetId>` field (default None) + `with_target(mut self, t: TargetId) -> Self`.
- `ras-agent/src/application/run_step.rs`: source target from the bound target first, else `focused_target().ok()`; pass it into `ToolContext.target`.
- Fix the 2 test `ToolContext` construction sites to set `target`.

**Scope note:** AR-3's "required target / no fallback" is satisfied at the builtin layer (no builtin calls `focused_target`). The single-user `focused_target` fallback survives only in `run_step`; the multi-tenant `SessionManager` (Phase 5) always binds, so MT never hits the fallback.

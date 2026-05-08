## What

<!-- one paragraph describing the change -->

## Phase

<!-- which porting phase this lands; remove if patch / fix only -->

## Bump gate

- [ ] `cargo build --workspace --all-targets` passes
- [ ] `cargo nextest run --workspace --no-fail-fast` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --all -- --check` clean
- [ ] LOC, no-comments, no-unwrap checks pass
- [ ] Workspace `version` bumped (if shipping a phase)
- [ ] CHANGELOG entry added

## Risk

<!-- what could break, what was reviewed manually, what could not be tested -->

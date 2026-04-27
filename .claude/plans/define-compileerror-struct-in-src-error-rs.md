Now I have enough context to write the plan.

# Plan: define-compileerror-struct-in-src-error-rs

## Goal
Confirm the existing `CompileError` struct in `vertex_stage0/src/error/mod.rs` matches the spec'd shape, and add the `error::tests::compile_error_builder_chains` unit test that exercises `new` plus the `with_suggestion` / `with_note` builder chain.

## Steps
1. Open `vertex_stage0/src/error/mod.rs`. The struct, fields, and three required methods (`new`, `with_suggestion`, `with_note`) already exist exactly as the sub-steps describe (the struct also carries an extra `labels: Vec<Label>` field that other items rely on — leave it). No edits to the struct or methods are required.
2. Append a `#[cfg(test)] mod tests { ... }` block at the bottom of `vertex_stage0/src/error/mod.rs` containing a single test named `compile_error_builder_chains`. The test should:
   - Construct a `Span` via `Span::new(FileId(0), 0, 1)`.
   - Build a `CompileError` with `CompileError::new(ErrorCode::E0001, ErrorKind::Lexical, span, "msg")`.
   - Chain `.with_suggestion(Suggestion { message, replacement: None, span })` and `.with_note("note text")` (twice each, to confirm Vec push semantics).
   - Assert `code`, `kind`, `span`, `message`, and that `suggestions.len() == 2` and `notes == vec!["note text", ...]`, plus the chained call returns `Self` (binds back into a `CompileError` value).
3. Run `cargo fmt` and `cargo test --lib error::tests::compile_error_builder_chains` from the workspace root to confirm the new test passes.

## Files
- `vertex_stage0/src/error/mod.rs` — append `#[cfg(test)] mod tests` with `compile_error_builder_chains`. No changes to the existing `CompileError` definition.

## Risks
- The slug names `src/error.rs`, but the actual path is `vertex_stage0/src/error/mod.rs` (module-with-submodule layout for `render`). Editing the slug-named path would create a duplicate module. Edit the existing file.
- The struct already has more fields than the sub-step lists (`labels: Vec<Label>`). Removing `labels` would break `add-multi-label-support-to-renderer` and the existing renderer in `error/render.rs`. Leave it in place.
- The verify command is path-qualified to `error::tests::compile_error_builder_chains`. Putting the test in any other module or under any other name (e.g. inline `#[test]` outside `mod tests`) would make verify fail.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib -p vertex_stage0 error::tests::compile_error_builder_chains
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

## Assumptions
- The todo's "src/error.rs" refers to the existing `vertex_stage0/src/error/mod.rs`, since the workspace has only one crate (`vertex_stage0`) and that file already defines `CompileError` with the exact fields and methods the sub-steps require.
- The struct shape and method signatures already match the spec, so this item reduces to adding the verify test. I will not refactor or remove the extra `labels` field — other planned items (`add-multi-label-support-to-renderer`) and the existing renderer depend on it.
- `pub fn new(code, kind, span, msg)` is the existing `pub fn new(code: ErrorCode, kind: ErrorKind, span: Span, message: impl Into<String>) -> Self`. The `impl Into<String>` flexibility is preserved.
- The test will live in `mod tests` inside `error/mod.rs` (so its path is `error::tests::compile_error_builder_chains`), gated by `#[cfg(test)]`, importing `super::*` and `crate::span::{FileId, Span}`.
- `cargo fmt --check` and `cargo clippy -D warnings` are run as part of the existing CI gate; including them in verify catches formatting/lint regressions before the runner moves on.

## Blockers
Blockers: none

## Summary
Adds the `compile_error_builder_chains` unit test that locks in the already-implemented `CompileError::new` / `with_suggestion` / `with_note` builder chain.

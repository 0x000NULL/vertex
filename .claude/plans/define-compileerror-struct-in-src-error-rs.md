# Plan: define-compileerror-struct-in-src-error-rs

## Goal
Confirm the existing `CompileError` struct, fields, and builder methods in `vertex_stage0/src/error/mod.rs` already satisfy the sub-steps, and add the `error::tests::compile_error_builder_chains` unit test the verify line requires.

## Steps
1. Inspect `vertex_stage0/src/error/mod.rs`. The struct already has the required `code: ErrorCode`, `kind: ErrorKind`, `span: Span`, `message: String`, `suggestions: Vec<Suggestion>`, `notes: Vec<String>` fields (plus an extra `labels: Vec<Label>` that other items already depend on — leave it). The methods `pub fn new(code, kind, span, message)`, `pub fn with_suggestion(self, s) -> Self`, `pub fn with_note(self, n) -> Self` are already defined with the right shapes. No edits to the struct/impl block.
2. Add a new `#[test]` named `compile_error_builder_chains` inside the existing `#[cfg(test)] mod tests { ... }` block at the bottom of `vertex_stage0/src/error/mod.rs`. The test should:
   - Build a `Span` via `Span::new(FileId(0), 0, 1)`.
   - Construct `CompileError::new(ErrorCode::E0001, ErrorKind::Lexical, span, "boom")`.
   - Chain `.with_suggestion(Suggestion { message: "try this".into(), replacement: None, span })` and `.with_note("aside")` (call each twice to prove the chain pushes onto the `Vec` rather than overwriting).
   - Bind the result back into a `CompileError` to prove `with_*` returns `Self` by value.
   - Assert `code == ErrorCode::E0001`, `kind == ErrorKind::Lexical`, `span` round-trips, `message == "boom"`, `suggestions.len() == 2`, `notes == vec!["aside", "aside"]`.
3. Run `cargo fmt`, then run the verify commands.

## Files
- `vertex_stage0/src/error/mod.rs` — append the `compile_error_builder_chains` test inside the existing `mod tests` block. The struct, the `new` / `with_suggestion` / `with_note` methods, and the surrounding `Suggestion` / `ErrorCode` / `ErrorKind` types are unchanged.

## Risks
- The slug says `src/error.rs`, but the workspace has a single crate `vertex_stage0` whose error module lives at `vertex_stage0/src/error/mod.rs` (module-with-`render`-submodule layout). Creating a new top-level `src/error.rs` would either be unreachable from the crate or shadow the existing module. The right move is to edit the existing file.
- The verify path is `error::tests::compile_error_builder_chains`. Placing the test in any other module path (e.g. a new file or a sibling test module) would silently miss the verify filter.
- Removing the existing `labels: Vec<Label>` field to match the sub-step list verbatim would break `error::render` and any planned multi-label work. Treat the sub-steps as a minimum-required field set, not an exhaustive one.
- The existing `mod tests` already has `accumulator_caps_at_100` and `accumulator_dedupes`. Take care to add the new test alongside them (still under `mod tests`), not a duplicate sibling module.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib -p vertex_stage0 error::tests::compile_error_builder_chains
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

## Assumptions
- "src/error.rs" in the todo refers to `vertex_stage0/src/error/mod.rs` since that is the only crate in the workspace and the file already defines `CompileError` with the spec'd fields and methods.
- The struct already matches the spec, so this item reduces to writing the verify test. I will not delete the extra `labels` field; the existing renderer (`error/render.rs`) and downstream multi-label work depend on it.
- `pub fn new(code, kind, span, msg)` is already implemented as `pub fn new(code: ErrorCode, kind: ErrorKind, span: Span, message: impl Into<String>) -> Self`; I will not narrow `impl Into<String>` to a plain `String`.
- The test imports `super::*` and `crate::span::{FileId, Span}` (mirroring the existing tests) so it exercises the public builder surface.
- Including `cargo fmt --check` and `cargo clippy -D warnings` in verify catches lint/format regressions early; both are no-ops if the change is clean.

## Blockers
Blockers: none

## Summary
Locks in the already-implemented `CompileError::new` / `with_suggestion` / `with_note` builder chain by adding the `compile_error_builder_chains` unit test the verify line requires; no struct or method changes needed.

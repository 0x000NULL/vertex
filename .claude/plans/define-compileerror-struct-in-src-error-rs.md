# Plan: define-compileerror-struct-in-src-error-rs

## Goal
Add a `CompileError` diagnostic struct (with builder-style `with_suggestion` / `with_note` methods) and a unit test `error::tests::compile_error_builder_chains` to `vertex_stage0/src/error.rs`.

## Steps
1. In `vertex_stage0/src/error.rs`, keep the existing `Suggestion` struct and add `use crate::error::{ErrorCode, ErrorKind};`-equivalent imports (they live in the same module, so just rely on local items defined by the prior todo).
2. Define `pub struct CompileError` with fields `code: ErrorCode`, `kind: ErrorKind`, `span: Span`, `message: String`, `suggestions: Vec<Suggestion>`, `notes: Vec<String>`. Derive `Debug, Clone` to match `Suggestion`.
3. Add an `impl CompileError` block with three methods:
   - `pub fn new(code: ErrorCode, kind: ErrorKind, span: Span, message: impl Into<String>) -> Self` — constructs with empty `suggestions` and `notes` and `message: message.into()`.
   - `pub fn with_suggestion(mut self, s: Suggestion) -> Self` — pushes `s` onto `self.suggestions`, returns `self`.
   - `pub fn with_note(mut self, n: impl Into<String>) -> Self` — pushes `n.into()` onto `self.notes`, returns `self`.
4. Add a `#[cfg(test)] mod tests { ... }` block with `compile_error_builder_chains`:
   - Build a dummy `Span` (e.g. `Span::new(FileId(0), 0, 1)`).
   - Pick any concrete `ErrorCode` value (e.g. `ErrorCode(1)` from the prior todo's newtype) and any `ErrorKind` variant (e.g. `ErrorKind::Other`).
   - Call `CompileError::new(...)` then chain `.with_suggestion(Suggestion { message: "s".into(), replacement: None, span })` then `.with_note("n")`.
   - Assert `err.code`, `err.kind`, `err.span`, `err.message`, `err.suggestions.len() == 1`, `err.notes == vec!["n".to_string()]`.
5. Run the verify commands.

## Files
- `vertex_stage0/src/error.rs` — append `CompileError` struct, `impl CompileError` with `new` / `with_suggestion` / `with_note`, and a `#[cfg(test)] mod tests` module containing `compile_error_builder_chains`. The existing `Suggestion` struct and the `Span` import stay.

## Risks
- `ErrorCode` and `ErrorKind` are defined by the immediately prior todo (`define-errorcode-and-errorkind-in-src-error-rs`). If the runner executes this item out of order, the file will not compile. The execute step should detect this (compile failure with "cannot find type ErrorCode") and surface it rather than silently inventing stubs.
- The verify command in the todo (`cargo test --lib error::tests::compile_error_builder_chains`) does not include `--manifest-path`. Since this repo has no workspace `Cargo.toml` at the root, the command must be run with `--manifest-path vertex_stage0/Cargo.toml` to find the crate.
- Test-only construction of a `Suggestion`/`Span`/`ErrorCode`/`ErrorKind` value relies on those types being constructible from outside their defining module — `Suggestion` already has all-public fields, `Span::new` is public, `ErrorCode(pub u32)` is a tuple-struct with a public field per the prior plan, and `ErrorKind` is a unit-variant enum per the prior plan. All good.

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::tests::compile_error_builder_chains
cargo build --manifest-path vertex_stage0/Cargo.toml
grep -q 'pub struct CompileError' vertex_stage0/src/error.rs
grep -q 'fn with_suggestion' vertex_stage0/src/error.rs
grep -q 'fn with_note' vertex_stage0/src/error.rs
```

## Assumptions
- The prior todo (`define-errorcode-and-errorkind-in-src-error-rs`) has already been executed, so `ErrorCode` (tuple struct with a `pub u32` field) and `ErrorKind` (unit-variant enum including at least `Other`) exist in the same `error` module. No stubs are added by this item.
- `CompileError` derives `Debug, Clone` (same derives the existing `Suggestion` uses); no `PartialEq` is required because the test asserts on individual fields, not whole-value equality.
- `new`'s `message` parameter is `impl Into<String>` for ergonomic call sites, even though the todo writes it as `msg`. This matches `with_note`'s signature and is a strict superset of `String`.
- `with_suggestion` / `with_note` consume `self` and return `Self` (per the todo signature), enabling chained-builder usage. They use `mut self` internally.
- The test lives in `vertex_stage0/src/error.rs` under `#[cfg(test)] mod tests`, so its full path is `vertex_stage0::error::tests::compile_error_builder_chains`; the verify filter `error::tests::compile_error_builder_chains` is a substring match that cargo test accepts.
- The verify `cargo test --lib` is augmented with `--manifest-path vertex_stage0/Cargo.toml` because there is no workspace root `Cargo.toml`. This preserves the spirit of the todo's verify line while making it actually executable from the repo root.
- The path string in the todo (`src/error.rs`) refers to `vertex_stage0/src/error.rs`, the only `error.rs` in the repo.

## Blockers
Blockers: none

## Summary
Adds the `CompileError` diagnostic struct with `new` / `with_suggestion` / `with_note` builder methods and a chaining unit test, completing the third piece of the error-reporting foundation in `vertex_stage0/src/error.rs`.

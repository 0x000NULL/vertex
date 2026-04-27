# Plan: implement-erroraccumulator-in-src-error-rs

## Goal
Add an `ErrorAccumulator` to the error module that collects `CompileError`s with deduping by `(code, file_id, start)` and a hard cap of 100, and converts to a `Result`.

## Steps
1. In `vertex_stage0/src/error/mod.rs` (the actual location of the error module — see Assumptions), add a new `ErrorAccumulator` struct holding:
   - `errors: Vec<CompileError>`
   - `seen: HashSet<(ErrorCode, FileId, u32)>` for dedupe lookup
   - `dropped: u32` counter for errors silently dropped after the cap
   - A `MAX_ERRORS: usize = 100` associated const (private or `pub const`).
2. Implement `pub fn new() -> Self` returning an empty accumulator (use `Default` impl if convenient, but expose `new()`).
3. Implement `pub fn push(&mut self, e: CompileError)`:
   - Compute key `(e.code, e.span.file_id, e.span.start)`.
   - If key is already in `seen`, return without doing anything (dedupe — does NOT increment `dropped`).
   - Else if `errors.len() >= MAX_ERRORS`, increment `dropped` and return (silent drop).
   - Else insert key into `seen` and push `e` into `errors`.
4. Implement `pub fn into_result<T>(self, ok: T) -> Result<T, Vec<CompileError>>`:
   - If `errors.is_empty()`, return `Ok(ok)`.
   - Else return `Err(self.errors)`.
5. Add accessors needed by tests (`pub fn dropped(&self) -> u32`, `pub fn len(&self) -> usize`, `pub fn is_empty(&self) -> bool`) — small surface, used by `accumulator_caps_at_100`.
6. Implement `Default for ErrorAccumulator` matching `new()`.
7. In a `#[cfg(test)] mod tests` block within the same file, add:
   - `accumulator_caps_at_100`: build a `FileId(0)`; push 150 errors with distinct `start` offsets (so dedupe doesn't kick in); assert `into_result(())` returns `Err(v)` with `v.len() == 100`. Before consuming, also check `dropped() == 50` via a separate accumulator instance (two short bodies, or store the count before `into_result`).
   - `accumulator_dedupes`: push the same `CompileError` (same `code`, `span.file_id`, `span.start`) 5 times; push a second variant differing only in `code`; push a third variant differing only in `start`; assert `into_result(())` is `Err(v)` with `v.len() == 3`.
8. Run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` to satisfy the existing CI gate.

## Files
- `vertex_stage0/src/error/mod.rs` -- add `use std::collections::HashSet;` and `use crate::span::FileId;` (FileId is referenced by the dedupe key type — import only if Rust complains; otherwise the inferred tuple type is fine and we can skip the import). Add `ErrorAccumulator` struct, its impl block, `Default` impl, and the two `#[cfg(test)]` tests inside the existing or a new `mod tests` block.

## Risks
- Test path mismatch: the verify uses `error::tests::accumulator_caps_at_100`. Because `mod.rs` is the module root, the inner module path is `error::tests::*` — correct. No nested path needed.
- If a different pending item also adds a `mod tests` block in `error/mod.rs`, future merges may collide; we keep the test module local and additive.
- Dedupe semantics: the spec says dedupe by `(code, span.file_id, span.start)` — duplicates are silently skipped and *do not* count toward the dropped counter. The plan codifies that ordering (dedupe first, then cap). If reviewer wanted dropped to count dedupes too, the test `accumulator_caps_at_100` would still pass (since it uses distinct starts), so the choice is locally safe but documented as an Assumption.
- `into_result` consumes `self`, so the test must read `dropped()` before calling it. Plan reflects that.

## Prereqs
- define-errorcode-and-errorkind-in-src-error-rs
- define-compileerror-struct-in-src-error-rs
- implement-span-struct-in-src-span-rs

## Verify
```
cargo test --lib error::tests::accumulator_caps_at_100
cargo test --lib error::tests::accumulator_dedupes
cargo build
cargo clippy --all-targets -- -D warnings
```

## Assumptions
- The TODO references `src/error.rs`, but the workspace member layout is `vertex_stage0/src/error/mod.rs` (already containing `ErrorCode`, `ErrorKind`, `CompileError`). I plan to edit `vertex_stage0/src/error/mod.rs`. Cargo workspace has a single member (`vertex_stage0`), so `cargo test --lib` from the repo root resolves to the same crate.
- `ErrorCode` derives `Hash` is NOT yet present (current code has `Copy, Clone, PartialEq, Eq, Debug`). For the `HashSet` key I will derive/add `Hash` to `ErrorCode` (a one-line addition that's compatible with its current `pub u32` representation and consistent with the `Hash` derive already on `FileId`/`Span`). This is a tiny, additive change to existing types and falls inside the bundled commit for this item.
- Dedupe skips do not increment the `dropped` counter; only post-cap drops do. The spec phrasing ("silent drop after [the cap], but increment a counter") binds the counter to the cap, not to dedupe.
- Push order is preserved in the resulting `Vec<CompileError>` (insertion order), mirroring how rustc surfaces diagnostics. No sorting is performed in `into_result`.
- Tests use `crate::span::{FileId, Span}` and `crate::error::{CompileError, ErrorAccumulator, ErrorCode, ErrorKind}` — all types now exist after the prereq items.
- `MAX_ERRORS` is a `const` on the impl (not a feature-flag or env-driven knob) — matches "Cap at 100" literally.

## Blockers
Blockers: none

## Summary
Adds `ErrorAccumulator` (push/into_result with 100-cap drop counter and `(code, file_id, start)` dedupe) plus the two named tests to `vertex_stage0/src/error/mod.rs`.

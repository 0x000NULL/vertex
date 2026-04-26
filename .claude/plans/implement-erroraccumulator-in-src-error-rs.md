# Plan: implement-erroraccumulator-in-src-error-rs

## Goal
Add a capped, deduplicating `ErrorAccumulator` to `vertex_stage0/src/error.rs` that collects `CompileError`s (push/into_result) and is exercised by two unit tests.

## Steps
1. Read `vertex_stage0/src/error.rs` to confirm current contents (only `Suggestion` exists today; `CompileError`, `ErrorCode`, `ErrorKind` were planned by an earlier item that failed verify per `needs-review.md`).
2. If `ErrorCode`, `ErrorKind`, and `CompileError` are not yet present, add them in this same file so `ErrorAccumulator::push(&mut self, e: CompileError)` can compile. Use the shapes specified in `compiler_architecture.md` §6 and `TODO.md`:
   - `pub struct ErrorCode(pub u16);` with `Copy, Clone, PartialEq, Eq, Hash, Debug`.
   - `pub enum ErrorKind { Lexical, Syntax, NameResolution, Type, BorrowCheck, Other }` with `Copy, Clone, PartialEq, Eq, Hash, Debug`.
   - `pub struct CompileError { code, kind, span, message, suggestions, notes }` with `Debug, Clone` plus `pub fn new(code, kind, span, msg) -> Self` and the `with_suggestion`/`with_note` builder methods (matches the parallel `define-compileerror` item so it does not need to redefine them later).
3. Add `pub struct ErrorAccumulator` with private fields:
   - `errors: Vec<CompileError>` (the kept errors, in insertion order, max length 100).
   - `dropped: u32` (counter incremented for every error rejected after the cap is reached).
   - `seen: std::collections::HashSet<(ErrorCode, FileId, u32)>` (dedup key set).
4. Implement `impl ErrorAccumulator`:
   - `pub fn new() -> Self` — empty vec/set, `dropped: 0`. Also derive/implement `Default` returning `Self::new()` for ergonomics.
   - `pub fn push(&mut self, e: CompileError)`:
     - Compute `key = (e.code, e.span.file, e.span.start)`.
     - If `seen.contains(&key)`, return without touching anything (dedupe is silent and does NOT count as a drop — drops are only the >100 overflow).
     - Else if `self.errors.len() >= 100`, increment `self.dropped` and return.
     - Else `seen.insert(key)` and `self.errors.push(e)`.
   - `pub fn into_result<T>(self, ok: T) -> Result<T, Vec<CompileError>>`:
     - If `self.errors.is_empty()`, `Ok(ok)`.
     - Else `Err(self.errors)`. (`dropped` is consumed silently — it exists so callers can query before `into_result`; expose `pub fn dropped(&self) -> u32` so the cap test can assert on it.)
5. Add a `#[cfg(test)] mod tests` (or extend the existing one once `CompileError` is added in step 2) with:
   - `accumulator_caps_at_100`: build 150 distinct `CompileError`s (vary `span.start` so dedupe doesn't kick in), push them all, assert `into_result(())` returns `Err(v)` with `v.len() == 100`. Before consuming, also assert `dropped() == 50`.
   - `accumulator_dedupes`: push the same error (same `code`, same `span.file`, same `span.start`) three times, assert `into_result(())` returns `Err(v)` with `v.len() == 1` and `dropped() == 0`. Add a second error with a different `span.start` and re-check that exactly that one extra entry was kept.
   - Helper inside the test module to construct a `CompileError` for a given `(code_u16, start_u32)` using a fixed `FileId(0)` and `Span::new`.
6. Run `cargo build -p vertex_stage0` and the two named tests to confirm everything compiles and passes.

## Files
- `vertex_stage0/src/error.rs` — add `ErrorCode`, `ErrorKind`, `CompileError` (if not yet present), `ErrorAccumulator` struct + impl, and the two `#[cfg(test)]` tests. Keep the existing `Suggestion` struct and `use crate::span::Span;` import. Add `use crate::span::FileId;` and `use std::collections::HashSet;`.

## Risks
- **Prereq drift**: The TODO ordering puts `ErrorCode/ErrorKind` and `CompileError` before this item, but `needs-review.md` shows that earlier attempt failed verify and the file currently only has `Suggestion`. If I bootstrap them here and the parallel `define-compileerror` item later re-defines them, we'll get duplicate-symbol errors. Mitigation: use the same shapes the earlier plan declared so a future attempt is a no-op or trivial reconcile.
- **Dedupe semantics**: spec says "dedupe by (code, span.file_id, span.start)" but the existing `Span` field is named `file: FileId` (not `file_id`). Using `e.span.file` is the only sensible reading; an unrelated rename would break this.
- **Drop counter accounting**: spec is ambiguous about whether dedupe-rejections count toward "dropped". I'm treating only post-cap overflow as drops (drops = silently lost work; dedupe is intentional collapse). If the future renderer wants a "deduped" count, a separate counter can be added without breaking callers.
- **Cap-test cost**: 100+ distinct errors is cheap, but pushing 150 means the test does ~150 hashmap lookups. Negligible.

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::tests::accumulator_caps_at_100
cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::tests::accumulator_dedupes
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The parent crate is `vertex_stage0` and `cargo` must be invoked with `--manifest-path vertex_stage0/Cargo.toml` from the repo root (matches what the runner did for the previous failed item per `needs-review.md`).
- `ErrorCode` is a tuple struct `pub struct ErrorCode(pub u16)` (the failing test in `needs-review.md` calls `ErrorCode(1)` constructor-style).
- `ErrorKind` includes the six variants listed in TODO.md and `compiler_architecture.md` §6.
- "Cap at 100" means the 100 *kept* errors — once we hold 100 deduped errors, every further push (regardless of dedupe) increments `dropped` and is discarded. Equivalently: the `seen` check runs first, so a duplicate after the cap costs a hash lookup but neither stores nor counts.
- Dedupe is silent: rejected duplicates do **not** increment `dropped`. `dropped` reflects only cap overflow.
- It is fine for `ErrorAccumulator` to derive `Debug` and `Default`; no `Clone` needed because `into_result` consumes `self`.
- `pub fn dropped(&self) -> u32` is acceptable to add even though TODO doesn't list it — it's the only way the cap test can assert the counter exists, and the spec explicitly says "increment a counter".
- Only this file is touched; no changes to `lib.rs`, `Cargo.toml`, or other modules. `error.rs` is already declared as a module in `lib.rs:7`.
- Bootstrapping `CompileError`/`ErrorCode`/`ErrorKind` here is preferable to blocking, because the alternative is an item that cannot compile.

## Blockers

### Blocker: CompileError / ErrorCode / ErrorKind do not yet exist in error.rs
- severity: cross-item
- affects: define-compileerror-struct-in-src-error-rs, define-errorcode-and-errorkind, error-pretty-printer, parser eat/expect, parse-failure-recovery
- question: Should this item bootstrap the missing prereq types (ErrorCode, ErrorKind, CompileError) so it can compile, or wait for the earlier two TODO items to be re-attempted and merged first?
- default_assumption: Bootstrap them inline, using the field/method shapes already specified in `compiler_architecture.md` §6 and TODO lines 79–90 so a re-run of the earlier items will be a no-op reconcile rather than a conflict.

## Summary
Adds a capped (100), deduped (`(code, file, start)`) `ErrorAccumulator` plus its two named unit tests to `vertex_stage0/src/error.rs`, bootstrapping the still-missing `ErrorCode`/`ErrorKind`/`CompileError` prereqs in the same file so the module compiles.

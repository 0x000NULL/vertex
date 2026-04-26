# Plan: implement-span-struct-in-src-span-rs

## Goal
Add a `Span` struct to `vertex_stage0/src/span.rs` with file/start/end fields, basic methods, standard derives, and a unit test verifying the `merge` semantics.

## Steps
1. In `vertex_stage0/src/span.rs`, below the existing `FileId` newtype, define `pub struct Span { pub file_id: FileId, pub start: u32, pub end: u32 }` with derives `#[derive(Copy, Clone, PartialEq, Eq, Debug)]`.
2. Implement an `impl Span` block with:
   - `pub fn new(file_id: FileId, start: u32, end: u32) -> Self` — direct field constructor.
   - `pub fn len(&self) -> u32` — returns `self.end - self.start` (saturating subtraction to avoid panics if `end < start`).
   - `pub fn merge(&self, other: &Span) -> Span` — returns a `Span` with `file_id` from `self`, `start = min(self.start, other.start)`, `end = max(self.end, other.end)` (i.e. the outer/enclosing bounds across both spans).
3. Add a `#[cfg(test)] mod tests { ... }` submodule inside `span.rs` containing at least the test `span_merge_takes_outer_bounds` that constructs two overlapping/adjacent spans and asserts the merged span uses the lower start and higher end. Add a couple of small companion assertions for `len` and ordering invariants while we're there.
4. No changes needed to `lib.rs` (`pub mod span;` is already wired).

## Files
- `vertex_stage0/src/span.rs` -- add `Span` struct, `impl Span` (with `new`, `len`, `merge`), and `#[cfg(test)] mod tests` containing `span_merge_takes_outer_bounds`.

## Risks
- `merge` across different `FileId`s is semantically dubious; we silently pick `self.file_id` rather than panic. If later code expects a debug-assert here, this will need revisiting.
- `len` on a malformed span (`end < start`) returning `0` via saturating subtraction may hide bugs; alternative is to panic in debug. Choosing saturating to keep the type infallible at this stage.
- The required test name `span::tests::span_merge_takes_outer_bounds` constrains the module path; the test must live in `mod tests` inside `span.rs` (not in `tests/`).

## Verify
```
cargo test -p vertex_stage0 --lib span::tests::span_merge_takes_outer_bounds
cargo build -p vertex_stage0
```

## Assumptions
- "Outer bounds" for `merge` means `min(start)` and `max(end)` across the two spans (the enclosing range), not just appending one after the other.
- `merge` keeps `self.file_id`; cross-file merges are not validated at this stage.
- `len` uses saturating subtraction so it cannot panic on malformed spans.
- The unit test lives in an inline `#[cfg(test)] mod tests` inside `span.rs`, so the path `span::tests::span_merge_takes_outer_bounds` resolves under `--lib`.
- `Hash` is intentionally NOT derived on `Span` (the spec lists only `Copy, Clone, PartialEq, Eq, Debug`), even though `FileId` derives `Hash`.
- All fields are `pub` so other modules can pattern-match without accessors; matches the existing `FileId(pub u32)` style.
- The crate is `vertex_stage0` (per `Cargo.toml` layout shown in recent commits), so verify uses `-p vertex_stage0`.

## Blockers
Blockers: none

## Summary
Introduces the `Span { file_id, start, end }` value type with `new`/`len`/`merge` and a unit test pinning down `merge`'s outer-bounds behavior, giving downstream lexer/parser work a stable source-range primitive.

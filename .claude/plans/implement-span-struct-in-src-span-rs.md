# Plan: implement-span-struct-in-src-span-rs

## Goal
Bring `Span` in `vertex_stage0/src/span.rs` up to spec by renaming its file field to `file_id`, adding `len`/`merge` methods, and adding the `span_merge_takes_outer_bounds` unit test.

## Steps
1. In `vertex_stage0/src/span.rs`, rename the `Span` field `file: FileId` to `file_id: FileId`. Update the `pub fn new` parameter name and struct literal. Keep derives `Copy, Clone, PartialEq, Eq, Hash, Debug` (Hash is preexisting and not excluded by the spec; removing it would be unrelated churn).
2. Add `pub fn len(&self) -> u32 { self.end - self.start }`.
3. Add `pub fn merge(&self, other: &Span) -> Span { Span { file_id: self.file_id, start: self.start.min(other.start), end: self.end.max(other.end) } }` — assumes both spans share the same `file_id`; takes outer bounds (min start, max end).
4. Update `SourceMap::snippet` (uses `span.file`) to use `span.file_id`.
5. In the existing `tests` module of `span.rs`, every `Span { file, .. }` / `span.file` reference is via `Span::new(...)` so nothing breaks; verify `source_map_round_trip_ascii_and_utf8` and `line_col_handles_multibyte` still compile.
6. Add a new test `span_merge_takes_outer_bounds` in `span::tests`: build two `Span`s on the same `FileId` with overlapping/disjoint ranges, assert `a.merge(&b)` returns a span with `start = min(a.start, b.start)`, `end = max(a.end, b.end)`, and the same `file_id`. Also assert `merge` is symmetric (`a.merge(&b) == b.merge(&a)`) and that `len()` equals `end - start` on the result.
7. In `vertex_stage0/src/error/render.rs`, update field accesses `primary_span.file`, `label.span.file` to `.file_id` (4 occurrences on lines 18, 19, 54, 55).
8. Run `cargo fmt`, `cargo build`, `cargo test --lib` and the targeted verify command.

## Files
- `vertex_stage0/src/span.rs` — rename `file` field → `file_id`, add `len`, add `merge`, add `span_merge_takes_outer_bounds` test, update `SourceMap::snippet` field access.
- `vertex_stage0/src/error/render.rs` — update 4 field accesses from `.file` to `.file_id`.

## Risks
- Renaming `file` → `file_id` touches `error/render.rs`; if any other in-tree code (or future-pending items in this run) already references `.file`, this rename is a small breaking change. Mitigated by grep across the crate first; only `render.rs` references it today.
- `merge` across different `file_id`s is undefined here — choosing `self.file_id` silently could mask bugs. Acceptable for stage0 since callers are expected to merge within one file; not asserting because the spec says nothing about cross-file behavior.
- Keeping the `Hash` derive technically deviates from the spec's listed derives (`Copy, Clone, PartialEq, Eq, Debug`); dropping it would break any `HashMap<Span, _>` consumers. Today there are none, but I'm preserving Hash to match the existing trait surface and avoid a hidden regression. See Assumptions.

## Prereqs
Prereqs: none

## Verify
```
cargo build -p vertex_stage0
cargo test -p vertex_stage0 --lib span::tests::span_merge_takes_outer_bounds
cargo test -p vertex_stage0 --lib
grep -q "pub fn merge" vertex_stage0/src/span.rs
grep -q "pub fn len" vertex_stage0/src/span.rs
grep -q "file_id: FileId" vertex_stage0/src/span.rs
```

## Assumptions
- The existing `Hash` derive is kept on `Span` even though the todo's listed derives omit it; removing it would be an unrelated, breaking change with no benefit.
- `merge` uses `self.file_id` and does not assert that both spans are in the same file. This matches typical compiler `Span::to` semantics; no spec text constrains the cross-file case.
- The existing field name `file` is to be renamed to `file_id` to match the spec exactly. Touching `error/render.rs` (4 field accesses) is in-scope as part of "implement Span struct" because the rename can't be partial.
- `len` returns `u32` (matching the field types) rather than `usize`; safe since `end >= start` by construction in this codebase.
- The test file location is the existing inline `mod tests` block at the bottom of `src/span.rs` (no separate test file), since the verify path is `span::tests::span_merge_takes_outer_bounds`.
- The crate path for cargo is the workspace member `vertex_stage0`; verify uses `-p vertex_stage0` so it works from the workspace root.

## Blockers
Blockers: none

## Summary
Renames `Span.file` to `file_id`, adds `len`/`merge`, adds the required merge test, and updates the only consumer (`error/render.rs`).

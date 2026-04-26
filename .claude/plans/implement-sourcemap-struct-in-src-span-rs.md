# Plan: implement-sourcemap-struct-in-src-span-rs

## Goal
Add a `SourceMap` to `vertex_stage0/src/span.rs` that owns added source files and exposes byte-span snippet lookup and `(line, col)` conversion, plus a unit test covering ASCII and UTF-8 round-trips.

## Steps
1. In `vertex_stage0/src/span.rs`, add `use std::path::{Path, PathBuf};` and keep the existing `FileId(pub u32)`.
2. Introduce a minimal `Span { file: FileId, start: u32, end: u32 }` (Copy/Clone/PartialEq/Eq/Hash/Debug) with a `pub fn new(file, start, end) -> Span` constructor — required because `SourceMap::snippet` takes a span and no `Span` type exists yet.
3. Define `pub struct SourceFile { pub id: FileId, pub name: PathBuf, pub content: String, pub line_starts: Vec<u32> }` (Debug, Clone). Add a private helper `compute_line_starts(content: &str) -> Vec<u32>` that emits `0` plus the byte offset of every char immediately following a `\n`.
4. Define `pub struct SourceMap { files: Vec<SourceFile> }` with `Default`/`Debug` derives and `pub fn new() -> Self` returning an empty map.
5. Implement `pub fn add_file(&mut self, name: impl Into<PathBuf>, content: impl Into<String>) -> FileId`: assigns `FileId(self.files.len() as u32)`, computes `line_starts`, pushes the `SourceFile`, returns the id.
6. Implement `pub fn file(&self, id: FileId) -> &SourceFile` (panics on out-of-range — internal-only API, validation belongs at the boundary).
7. Implement `pub fn snippet(&self, span: Span) -> &str`: indexes `self.file(span.file).content[span.start as usize .. span.end as usize]`.
8. Implement `pub fn line_col(&self, file: FileId, byte_offset: u32) -> (u32, u32)`: binary-search `line_starts` (`partition_point`) to find the 1-based line, then compute the 1-based column as the UTF-8 *character* count from the line start to `byte_offset` within `content`. Using char count (not byte delta) is what makes the UTF-8 test meaningful.
9. Add `#[cfg(test)] mod tests` containing `source_map_round_trip_ascii_and_utf8`:
   - Create `SourceMap::new()`.
   - Add an ASCII file `"a.vx"` with `"abc\ndef\nghi"`; assert `FileId(0)`, snippet `(0, 3)` is `"abc"`, snippet `(4, 7)` is `"def"`, `line_col` at offsets `0`, `4`, `5`, `8` give `(1,1)`, `(2,1)`, `(2,2)`, `(3,1)`.
   - Add a UTF-8 file `"u.vx"` with `"αβ\nγδε"` (each Greek letter = 2 bytes); assert `FileId(1)`, snippet over the first line is `"αβ"`, and `line_col` after `α` (byte offset 2) returns `(1,2)`, after `\n` (offset 5) returns `(2,1)`, after `γδ` (offset 9) returns `(2,3)`.

## Files
- `vertex_stage0/src/span.rs` — add `Span`, `SourceFile`, `SourceMap` with `add_file`/`snippet`/`line_col`/`file`, the `compute_line_starts` helper, and the `tests` module with `source_map_round_trip_ascii_and_utf8`.

## Risks
- Conflating byte offsets and char columns would break the UTF-8 case; mitigated by using char count for the column and bytes for the line lookup.
- `line_starts` must include leading `0` and the position after each `\n` (not the `\n` itself) so the binary search yields the correct line; off-by-one here is the most likely bug.
- `partition_point` requires Rust ≥ 1.52 (edition 2021 in `Cargo.toml` implies a modern toolchain — safe).
- Future `Span` users elsewhere may want different semantics (e.g. `NonZeroU32` ids, half-open vs. inclusive end). Keeping it minimal here is intentional; expansion is a later item.

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib span::tests::source_map_round_trip_ascii_and_utf8
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- A `Span` type is required because `snippet` takes a span and none exists yet; the minimal `{ file, start, end: u32 }` shape is defined here. If a richer `Span` is planned in a sibling todo, this one stays compatible (fields are `pub`).
- Span ranges are byte offsets, half-open `[start, end)`, matching Rust string slicing.
- `line_col` returns 1-based `(line, column)` where column is a UTF-8 character count from line start — the convention compilers report to humans.
- `add_file` accepts `impl Into<PathBuf>` / `impl Into<String>` for ergonomics; the spec's looser signature (`name, content`) is silent on types.
- Out-of-range `FileId` or span byte ranges are programmer errors and may panic; this is an internal API with no untrusted input.
- `compute_line_starts` treats only `\n` as a line break (not `\r` alone); `\r\n` lines still split correctly because the offset-after-`\n` rule lands past the `\r\n` pair.
- The test is placed inside `span.rs` as `mod tests` so the verify path `span::tests::source_map_round_trip_ascii_and_utf8` resolves via `cargo test --lib`.
- Existing `FileId(pub u32)` and its derives are kept verbatim.
- No new dependencies; only `std::path::PathBuf` is added.

## Blockers
Blockers: none

## Summary
Adds `Span`, `SourceFile`, and `SourceMap` to `span.rs` with file registration, snippet slicing, and 1-based UTF-8-aware line/column lookup, validated by an ASCII+UTF-8 round-trip unit test.

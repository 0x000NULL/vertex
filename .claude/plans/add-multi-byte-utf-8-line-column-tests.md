# Plan: add-multi-byte-utf-8-line-column-tests

## Goal
Add a `#[test]` named `line_col_handles_multibyte` in `vertex_stage0/src/span.rs` that exercises `SourceMap::line_col` with em-dash (3-byte UTF-8) and emoji (4-byte UTF-8) inputs.

## Steps
1. Open `vertex_stage0/src/span.rs` and locate the existing `#[cfg(test)] mod tests` block.
2. Append a new `#[test] fn line_col_handles_multibyte()` after `source_map_round_trip_ascii_and_utf8`.
3. Inside the test:
   - Construct a `SourceMap`, register a file whose content contains an em-dash `—` (U+2014, 3 bytes) and an emoji such as `😀` (U+1F600, 4 bytes), e.g. content `"a—b\n😀c"`.
   - Compute byte offsets manually (em-dash at bytes 1..4, `b` at byte 4, newline at 5, emoji at bytes 6..10, `c` at byte 10).
   - Assert `line_col` returns 1-based line and *character* (not byte) column for: start of em-dash, byte right after em-dash, start of new line, start of emoji, and byte right after emoji.
   - Also assert `snippet` returns the em-dash and emoji slices correctly to anchor the byte offsets.
4. Run `cargo test --lib span::tests::line_col_handles_multibyte` to confirm.

## Files
- `vertex_stage0/src/span.rs` -- add a new `line_col_handles_multibyte` test inside the existing `tests` module; no production code changes.

## Risks
- Mis-counting UTF-8 byte boundaries for `—` (3 bytes) or `😀` (4 bytes) would yield assertions that don't match the implementation, masking real bugs. Mitigate by sanity-checking with `snippet` over the same span.
- Editor/source encoding: file is already UTF-8 (existing test uses Greek letters), so no BOM/encoding risk on Windows.
- Picking an emoji that is part of a grapheme cluster (e.g., flag, ZWJ sequence) would conflate scalar count with grapheme count. Mitigate by using a single-codepoint emoji like `😀` so `chars().count()` matches the user-visible expectation.

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml span::tests::line_col_handles_multibyte
```

## Assumptions
- The expected column semantics in `SourceMap::line_col` are 1-based **character** (Unicode scalar) counts, as implied by `chars().count() as u32 + 1`. The new test asserts character columns, not byte columns or graphemes.
- Emoji choice is `😀` (U+1F600, 4-byte UTF-8, single scalar). This avoids ZWJ/variation-selector complications.
- Test content shape `"a—b\n😀c"` is acceptable; no requirement to match a specific fixture.
- The test belongs in the existing `mod tests` in `span.rs` (matches the verify path `span::tests::line_col_handles_multibyte`), rather than as an integration test under `tests/`.
- `cargo test --lib` is run from the workspace root; passing `--manifest-path vertex_stage0/Cargo.toml` ensures the right crate is targeted regardless of workspace layout.
- No new dependencies, helpers, or refactors are needed; this is a pure test addition in a single commit.

## Blockers
Blockers: none

## Summary
Adds a focused unit test proving `SourceMap::line_col` reports correct 1-based character columns across em-dash and emoji byte boundaries.

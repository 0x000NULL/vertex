# Plan: implement-line-and-block-comment-scanning

## Goal
Add `Scanner::skip_comments` to `vertex_stage0/src/lexer/scan.rs` that consumes a single non-doc `//`-line or `/* … */` block comment (with proper nesting) and rewinds on an unterminated block, plus the `nested_block_comments` unit test required by the verify step.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, add a public method `pub fn skip_comments(&mut self) -> bool` on `Scanner<'a>`. Returns `true` if it consumed a comment, `false` otherwise (so the eventual `next_token` driver can loop). Does **not** touch whitespace — that is a separate concern.
2. Inside `skip_comments`:
   - Save `start = self.pos`.
   - If next two bytes are `//`:
     - If the next byte after that is `/` (i.e. `///`) **or** `!` (i.e. `//!`), it is a doc-comment — return `false` without advancing (doc-comment scanning is a separate item).
     - Otherwise, advance past `//`, then `eat_while(|b| b != b'\n')`. Do **not** consume the trailing `\n` — leave it for whitespace handling. Return `true`.
   - Else if next two bytes are `/*`:
     - If the next byte after that is `*` followed by anything other than `/` (i.e. `/**` doc) **or** is `!` (i.e. `/*!` doc), return `false` without advancing. (Conservative: treat `/**` and `/*!` as doc forms reserved for the doc-comment item; plain `/* */` and `/**/` empty block stays handled here. To keep this simple and match the verify test, treat any `/*` start as a normal block comment — see Assumptions.)
     - Advance past `/*`, set `depth = 1`.
     - Loop: on `/*` increment depth and advance 2; on `*/` decrement depth and advance 2, returning `true` once `depth == 0`; on EOF before close, set `self.pos = start` and return `false` (rewind-on-failure, consistent with the rest of the scanner); else advance one byte using `bytes` (block comments are byte-transparent except for the `/*` / `*/` markers, so a single-byte step is fine and won't mis-split UTF-8 inside the comment because we only ever match ASCII tokens).
   - Else return `false`.
3. Add `#[test] fn nested_block_comments()` inside `mod tests` covering:
   - Plain line comment: `"// hello\n"` → consumed, pos at the `\n` (length − 1).
   - Plain block comment: `"/* hi */rest"` → consumed, pos at start of `rest`.
   - Nested block comment: `"/* a /* b */ c */tail"` → single call consumes the whole outer block; pos at start of `tail`.
   - Deeply nested: `"/*/*/*x*/*/*/Z"` → consumed, pos at `Z`.
   - Empty block: `"/**/X"` → consumed, pos at `X`.
   - Unterminated block (rewind): `"/* never ends"` → returns `false`, `pos == 0`.
   - Unterminated nested (rewind): `"/* a /* b */ c"` → returns `false`, `pos == 0`.
   - Doc-comment forms left alone: `"/// doc\n"` and `"//! doc\n"` → return `false`, `pos == 0`.
   - Non-comment input: `"abc"` → returns `false`, `pos == 0`.

## Files
- `vertex_stage0/src/lexer/scan.rs` -- add `skip_comments` method on `Scanner<'a>` (placed near `eat_while`, before the literal scanners) and a `nested_block_comments` `#[test]` inside the existing `mod tests`. No other files change.

## Risks
- Wrong handling of EOF inside a nested block could leave `pos` mid-stream; mitigated by the rewind-on-failure pattern already used by `scan_string` / `scan_raw_string`.
- Off-by-one when matching `*/` could double-count (e.g. `*/*` could be misread). Advancing exactly 2 bytes on each `/*` and `*/` match avoids that.
- A future `next_token` driver expects to loop over `skip_whitespace` + `skip_comments`; returning `bool` keeps it composable. If the eventual driver expects `()` instead, a trivial wrapper is enough — low cost.
- Treating `/**` like a normal block comment (rather than reserving it for doc-comments) may need revisiting when `implement-doc-comment-scanning` lands; see Assumptions.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml lexer::scan::tests::nested_block_comments
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate lives at `vertex_stage0/`, so `cargo` invocations need `--manifest-path vertex_stage0/Cargo.toml` (matches the existing layout — `vertex_stage0/src/lexer/scan.rs`, `vertex_stage0/Cargo.toml`).
- Block comments **nest**, even though the spec EBNF (`block_comment = "/*" { any_char } "*/"`) does not say so explicitly. The todo item explicitly requires "proper nesting depth counter," and Rust-style nesting matches every other Rust-shaped construct already in this scanner.
- `//!` and `///` are *doc* comments and belong to the separate `implement-doc-comment-scanning` item; `skip_comments` must therefore leave them untouched (return `false`, no advance) so the doc-comment scanner can claim them.
- `/**` and `/*!` block-doc forms: treated as **normal** block comments here. The spec's doc-comment EBNF only lists `///` and `//!` line forms, so there is no separate block-doc form to defer to. The doc-comment item can still introduce one later if needed; until then, `/**…*/` is consumed as a regular block comment.
- `skip_comments` does *not* consume surrounding whitespace and does *not* consume the newline that terminates a `//` line comment. Whitespace skipping is a separate concern handled by the eventual `next_token` driver (`wire-all-scanners-into-scanner-next-token-driver`).
- Returns `bool` (consumed / not consumed) so the future driver can loop `while skip_whitespace() | skip_comments() {}`. No `Span` is returned because comments are not tokens in this lexer.
- Inside a block-comment body, advancing one byte at a time is safe: we only match ASCII delimiters (`/*`, `*/`), and Rust's UTF-8 multibyte continuation bytes never collide with `/` (0x2F) or `*` (0x2A), so we cannot accidentally start a marker mid-codepoint. No need to use `self.src.chars()` here.
- On unterminated block comment we rewind `self.pos` to `start` and return `false`, matching the rewind-on-failure convention of the existing literal scanners. Error reporting (`unterminated block comment`) belongs to a later recovery item, not this one.
- The verify test path `lexer::scan::tests::nested_block_comments` matches the existing `#[cfg(test)] mod tests` block in `scan.rs`; no new test module is created.

## Blockers
Blockers: none

## Summary
Adds `Scanner::skip_comments` for line and (nestable) block comments with rewind on unterminated input, plus the `nested_block_comments` unit test the verify command runs.

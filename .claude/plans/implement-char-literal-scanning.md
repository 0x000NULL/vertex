# Plan: implement-char-literal-scanning

## Goal
Add `Scanner::scan_char` that recognizes a single-quoted character literal (with the spec's escape set), rewinds on rejection, and is covered by a unit test named `char_literal_escapes`.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, add `pub fn scan_char(&mut self) -> Option<(char, Span)>` on `Scanner<'a>`. The caller is expected to be looking at `'`; the method itself records `start = self.pos`, then `bump()`s the opening `'`. If the leading byte is not `'`, return `None` without moving.
2. After the opening quote, branch on the next byte:
   - `\\` (backslash) → call a new private helper `scan_char_escape(&mut self) -> Option<char>` that consumes the `\` and the rest of the escape and returns the decoded codepoint. Supports `n → '\n'`, `t → '\t'`, `r → '\r'`, `\\ → '\\'`, `' → '\''`, `" → '"'`, `0 → '\0'`, `xNN → char` (exactly two hex digits, value must be ≤ 0x7F per the spec range used elsewhere — see Assumptions), `u{NNNN}` (1–6 hex digits inside braces, must be a valid `char::from_u32`). Any other escape character, missing closing brace, non-hex digit, out-of-range scalar, or surrogate is rejected.
   - `'` (immediate close, i.e. empty `''`) → reject.
   - `\n` (raw newline) → reject (an unterminated literal).
   - EOF → reject (unterminated).
   - Otherwise → decode one full UTF-8 scalar starting at `self.pos`. Use `self.src[self.pos..].chars().next()` to get the codepoint and its byte length, advance `self.pos` by that length.
3. After the content, require the next byte to be `'`. If not (e.g. `'ab'`, `'a`, `'\xFFx'`), reject. On any rejection along the path, restore `self.pos = start` and return `None`. On success, `bump()` the closing `'` and return `Some((ch, Span::new(self.file_id, start as u32, self.pos as u32)))`.
4. Add `#[test] fn char_literal_escapes()` in the existing `mod tests`. Cover, at minimum:
   - Happy path: `'a'`, `' '`, an ASCII printable, a non-ASCII single codepoint like `'é'` (verifies multi-byte UTF-8 single-codepoint handling), and the full escape set: `'\n'`, `'\t'`, `'\r'`, `'\\'`, `'\''`, `'\"'`, `'\0'`, `'\x7F'`, `'\u{1F600}'`. Each asserts the returned `char`, full-input span, and `pos == input.len()`.
   - Rejections (each must return `None` with `pos == 0`): empty content `''`, multi-codepoint `'ab'`, unterminated `'a`, bare `'`, raw newline inside `'\n` (literal newline byte, not the escape), unknown escape `'\q'`, malformed hex escape `'\xZZ'`, malformed unicode escape `'\u{}'`, `'\u{D800}'` (surrogate), `'\u{110000}'` (out of range).

## Files
- `vertex_stage0/src/lexer/scan.rs` — add `scan_char` (public) and `scan_char_escape` (private) methods on `Scanner<'a>`; add the `char_literal_escapes` unit test in the existing `mod tests`.

## Risks
- UTF-8 byte-vs-char accounting: the rest of the scanner is byte-indexed, but a char literal contains a Unicode scalar. Using `self.src[self.pos..].chars().next()` keeps the position byte-indexed and aligned with the rest of the file.
- "Multi-codepoint" rejection includes things like grapheme clusters (`'é'` written as `e` + combining acute = two scalars). The spec talks about codepoints, so we reject that — easy to get wrong by using grapheme width instead.
- `\xNN` range: the spec line lists `\xNN` without an upper bound, but the conventional Rust meaning restricts byte escapes in chars to `\x00..=\x7F` (anything higher needs `\u{}`). See Assumptions.
- Forgetting to rewind on every failure path would leave the scanner mid-literal and corrupt later tokens. Use a single `start` capture and one rewind point.

## Prereqs
- implement-scanner-struct-in-src-lexer-scan-rs
- define-token-struct
- add-literal-variants-to-tokenkind
- implement-span-struct-in-src-span-rs

## Verify
```
cargo test -p vertex_stage0 --lib lexer::scan::tests::char_literal_escapes
cargo build -p vertex_stage0
```

## Assumptions
- Return type is `Option<(char, Span)>` to match the rewind-on-failure pattern used by `scan_int_hex` / `scan_int_bin` / `scan_float`, and because `TokenKind::CharLiteral(char)` already takes a single `char`.
- The method assumes the caller positions it on the opening `'`; if not, it returns `None` without moving (defensive, mirrors `scan_int_hex`'s `0x` precondition).
- `\xNN` is restricted to `0x00..=0x7F` (ASCII range). Higher byte values would not form a valid single Unicode scalar and the spec already provides `\u{}` for arbitrary codepoints — this matches Rust's behavior, which is the closest reference language.
- `\u{...}` accepts 1–6 hex digits with no internal `_` separators (the spec line shows `\u{NNNN}` literally; underscores can be added later if the full lexer spec wants them — out of scope here).
- Surrogate codepoints (`U+D800..=U+DFFF`) and codepoints `> U+10FFFF` are rejected since `char::from_u32` returns `None`.
- A literal newline byte inside the quotes (`'\n` where `\n` is a real LF) is treated as unterminated, not as an embedded newline char. Embedding a newline still requires the `'\n'` escape.
- No driver wiring into `next_token` — that is the separate `wire-all-scanners-into-scanner-next-token-driver` item.
- Test module path is `lexer::scan::tests::char_literal_escapes` (the existing test module in `scan.rs` is `mod tests`, and the file is `crate::lexer::scan`), matching the verify command stated by the task.
- The crate is `vertex_stage0` (from the existing file path); `cargo test --lib` uses `-p vertex_stage0` to disambiguate in case other workspace members exist.

## Blockers
Blockers: none

## Summary
Add `Scanner::scan_char` with full escape support, multi-codepoint and unterminated rejection (with rewind), and a `char_literal_escapes` unit test covering happy paths, escapes, and every rejection mode.

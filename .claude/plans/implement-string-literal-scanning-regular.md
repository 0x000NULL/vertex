# Plan: implement-string-literal-scanning-regular

## Goal
Add `Scanner::scan_string` to `vertex_stage0/src/lexer/scan.rs` that recognizes `"..."` string literals (with the same escape set as `scan_char` and embedded newlines allowed), returning the decoded `String` and a `Span`, with rewind on failure, plus a `string_literal_escapes` unit test.

## Steps
1. Refactor the inner escape body of `scan_char_escape` into a shared helper that, given a scanner already positioned at the `\`, advances past the escape sequence and returns `Option<char>`. Reuse it from both `scan_char` and `scan_string` so the escape set stays identical (`\n`, `\t`, `\r`, `\\`, `\'`, `\"`, `\0`, `\xHH` ≤ 0x7F, `\u{...}` 1–6 hex digits, valid `char::from_u32`). Keep the existing private API or restructure to a single helper named like `scan_escape_char` — internal change, no external callers yet.
2. Add `pub fn scan_string(&mut self) -> Option<(String, Span)>`:
   - If `peek() != Some(b'"')` return `None` without changing `pos`.
   - Save `start = self.pos`, bump past the opening `"`.
   - Loop accumulating into a `String` buf:
     - On `None` (EOF before close): rewind `pos = start`, return `None`.
     - On `b'"'`: bump past it, build span `[start, pos)`, return `Some((buf, span))`.
     - On `b'\\'`: call the shared escape helper; if it returns `None`, rewind to `start` and return `None`. Otherwise push the decoded `char`.
     - On any other byte: decode the next UTF-8 `char` from `&self.src[self.pos..]` (mirroring how `scan_char` handles non-ASCII), push it, advance `pos` by `c.len_utf8()`. This naturally allows embedded raw `\n`, `\r`, and arbitrary Unicode scalars.
   - Always restore `self.pos = start` when returning `None` (unterminated, bad escape).
3. Add a `#[test] fn string_literal_escapes` in the existing `mod tests` covering:
   - Happy paths: empty `"\"\""`, ASCII `"\"hello\""`, all single-char escapes (`\n \t \r \\ \" \' \0`), `\xHH`, `\u{...}` (BMP + supplementary), Unicode source char (e.g. `"é"`), and a literal embedded newline (input `"\"a\nb\""` → value `"a\nb"`). For each, assert decoded value, `span.file_id`, `span.start == 0`, `span.end == input.len()`, and `s.pos == input.len()`.
   - Rejections (each must return `None` and leave `pos == 0`): unterminated `"abc`, lone backslash before EOF `"\\`, bad escape `"\\q"`, bad hex `"\\xZZ"`, out-of-range `\\xFF` (>0x7F), bad unicode `"\\u{}"`, surrogate `"\\u{D800}"`, out-of-range `"\\u{110000}"`, missing close before EOF after content `"\"abc"` (typed as `"\"abc"` with no closing quote — i.e. `"\"abc`).
   - Also assert that input not starting with `"` returns `None` with `pos == 0`.

## Files
- `vertex_stage0/src/lexer/scan.rs` — add `scan_string`, factor out the shared escape helper used by both `scan_char` and `scan_string`, and add the `string_literal_escapes` unit test in the existing `mod tests`.

## Risks
- Refactoring the escape helper out of `scan_char_escape` could break the existing `char_literal_escapes` test if the byte-position bookkeeping (the leading `\`) is handled differently. Mitigation: keep `scan_char` calling the new helper at exactly the same position the old code did, and re-run `char_literal_escapes` as part of verify.
- UTF-8 decoding inside the string body must advance by `c.len_utf8()`, not 1 byte, or non-ASCII chars will desync `pos`. The `scan_char` non-escape arm is the precedent; mirror it.
- Allowing embedded newlines means we don't reject on `\n` mid-string — only EOF terminates the search. Make sure the unterminated-recovery test in this run (separate item) still has room to do its own work; this plan only handles the successful and locally-rejected cases via rewind.
- `String` allocation per scan is fine for stage-0 but means `scan_string` is not zero-alloc. Acceptable given `TokenKind::StringLiteral(String)` already owns the data.

## Prereqs
Prereqs: none

## Verify
```
cargo test -p vertex_stage0 --lib lexer::scan::tests::string_literal_escapes
cargo test -p vertex_stage0 --lib lexer::scan::tests::char_literal_escapes
cargo build -p vertex_stage0
```

## Assumptions
- The crate is `vertex_stage0` (matches the module path on existing tests like `lexer::scan::tests::char_literal_escapes`); using `-p vertex_stage0` keeps verify scoped even if the workspace grows.
- `scan_string` returns `Option<(String, Span)>` — no `StringSuffix`, since none of `TokenKind::StringLiteral` carries one.
- "Same escape set as chars" means literally the set in `scan_char_escape`: `n t r \ ' " 0 x u`. No `\a`, `\b`, `\f`, `\v`, no line-continuation `\<newline>`, no `\<digit>` octal — those would have to be added to `scan_char` too and are out of scope for this item.
- "Allow embedded newlines" means a raw `\n` byte inside the literal is part of the string value (no rejection, no normalization). `\r\n` is preserved as-is (not collapsed).
- `\xHH` keeps the same ≤ 0x7F constraint as `scan_char`. Strings do not get a wider range — that would diverge from "same escape set as chars".
- On any failure during scanning (unterminated, bad escape, etc.) the scanner rewinds `pos` to the start, mirroring the convention established by `scan_char` / `scan_int_hex` / `scan_float`. The driver (a later item) will re-attempt with a recovery path.
- Refactoring `scan_char_escape` into a shared helper is in scope here; if minimizing churn is preferred, an alternative is to inline a near-duplicate escape decoder inside `scan_string` instead. Going with the shared helper to avoid drift between the two escape sets.
- The test goes in the existing `mod tests` block in `scan.rs`; no new file. Test name is exactly `string_literal_escapes` to match the verify path.
- No `TokenKind` integration in this item — the driver wiring is `wire-all-scanners-into-scanner-next-token-driver`. This plan only adds the standalone `scan_string` method and its unit test.

## Blockers
Blockers: none

## Summary
Adds `Scanner::scan_string` (rewind-on-failure, shared escape decoder with `scan_char`, embedded newlines allowed) plus the `string_literal_escapes` unit test the spec verifies against.

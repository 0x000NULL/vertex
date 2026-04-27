# Plan: unterminated-string-recovery

## Goal
Make the scanner emit a single `TokenKind::Error` token spanning the open-quote through EOF when a string literal has no closing quote, then continue at EOF (yielding `Eof` next).

## Steps
1. Read `vertex_stage0/src/lexer/scan.rs` and re-confirm the current behavior of `scan_string` (resets `pos` to the open-quote on any failure, returns `None`) and `next_token`'s `b'"'` arm (currently bumps one byte and emits `Error("\"")`).
2. Refactor `scan_string` so it can distinguish "unterminated" (reached `None` at the top of the body loop) from "malformed escape / bad UTF-8". Concrete shape:
   - Define a small private `enum ScanStringOutcome { Ok(String, Span), Unterminated(Span), Failed }` (or equivalent) inside `scan.rs`.
   - On the `None => { self.pos = start; return None; }` branch at the top of the body loop (line 367-370), instead advance `self.pos` to `self.bytes.len()`, build `Span::new(file_id, start as u32, self.pos as u32)`, and return `Unterminated(span)`. Do NOT consume past the buffer end.
   - All other failure branches (bad escape, char-decode fail) keep the existing reset-to-start semantics and return `Failed`.
   - Happy path returns `Ok(buf, span)`.
3. Update `next_token`'s `b'"'` arm (lines 744-753) to match the three outcomes:
   - `Ok` → emit `TokenKind::StringLiteral(s)`.
   - `Unterminated(span)` → emit `Token::new(TokenKind::Error("unterminated string literal".to_string()), span)`. `self.pos` is already at EOF, so the next call returns `Eof`.
   - `Failed` → preserve the current single-byte-advance behavior (`self.pos += 1`; emit `Error("\"")` with a 1-byte span).
4. Update the existing `string_literal_escapes` rejection list (lines 1093-1112). The two unterminated cases — `"\"abc"` (the literal `"abc`) and `"\"\\"` (the literal `"\`) — now go down the `Unterminated` path, so calling `scan_string` directly will not return `Some(...)` either, but `pos` will end up at `bytes.len()` rather than `0`. Switch those two inputs to assert the new `ScanStringOutcome::Unterminated` shape, and leave the other malformed-escape rejections asserting `Failed` with `pos == 0`. Use whatever predicate matches the new return type.
5. Add the new `unterminated_string_recovers` test in `mod tests`. Cover at minimum:
   - Source `"abc` (no close quote, EOF mid-content): drive `next_token` in a loop until `Eof`. Expect exactly two tokens: `Error("unterminated string literal")` with span `0..4` and `file_id` matching, then `Eof` with empty span at offset 4.
   - Source `"abc\n` (newline allowed inside string per existing test, still no close quote): same shape — Error spans the whole 5-byte input, then Eof.
   - Source `prefix "abc` (preceded by an ident) to confirm the error span starts at the open-quote, not byte 0. Expect `Ident("prefix")`, `Error("unterminated string literal")` spanning open-quote → EOF, then `Eof`.
   - Assert all error tokens carry `TokenKind::Error(_)`, span `file_id` matches, `span.end as usize == src.len()`, and the final `Eof` sits at `src.len()` with `start == end`.
6. Run `cargo test --lib` once locally (mentally — no shell here) to verify nothing else regresses; rely on the broader `cargo test` in verify to catch wider breakage.

## Files
- `vertex_stage0/src/lexer/scan.rs` — refactor `scan_string` return type to a 3-variant outcome, update `next_token`'s `b'"'` arm to handle the unterminated case by emitting a single `Error` token spanning open-quote → EOF, adjust two entries in the `string_literal_escapes` rejection table to match the new return type, and add the new `unterminated_string_recovers` test inside `mod tests`.

## Risks
- The two unterminated entries already in `string_literal_escapes` rejections (`"\"abc"`, `"\"\\"`) will assert against the old `Option<...>` shape; if they're not updated alongside the signature change, that pre-existing test fails. Step 4 covers this.
- Choosing to change `scan_string`'s return type (vs. layering a new method on top) is a small public-surface change inside the crate — but `scan_string` is only called from `next_token` (and the unit tests), so blast radius is contained. Verified via the file's contents (only `next_token` and one test invoke it).
- Error message wording (`"unterminated string literal"`) is a guess at the project's tone; existing errors use lowercase descriptive strings like `"invalid character: $"`, so this matches.
- `scan_raw_string` has the same unterminated-EOF pattern but is explicitly out of scope for this todo (its slug is `implement-raw-string-literal-scanning`, already done; a separate recovery todo is not in the pending list). Leave it untouched.
- The existing `scan_string` `None` branch on internal char-decode failure is unreachable for valid UTF-8 `&str` input; keeping it as `Failed` is harmless.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib lexer::scan::tests::unterminated_string_recovers --manifest-path vertex_stage0/Cargo.toml
cargo test --lib --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate is `vertex_stage0` at `vertex_stage0/Cargo.toml`; `cargo test --lib` from the workspace root needs `--manifest-path` to target it. (Verified by the file layout: only `vertex_stage0/src/...` exists.)
- The new error message string is `"unterminated string literal"` — descriptive, lowercase, follows the precedent of `"invalid character: $"`. Not yet hooked to a structured `ErrorCode`/`ErrorKind` because those types are still pending (`define-errorcode-and-errorkind-in-src-error-rs`); the existing scanner emits raw strings via `TokenKind::Error(String)`, so we stay consistent with that.
- The error span covers `[open_quote_offset, src.len())` — i.e., includes the open quote and every byte up to (but not past) EOF. After emitting it, `self.pos == self.bytes.len()`, so the next `next_token` call returns `TokenKind::Eof` with an empty span at `src.len()`. This matches the verify-step phrasing "continue at EOF".
- Newlines inside strings remain allowed (the existing happy-path test `string_literal_escapes` accepts `"\"a\nb\""`), so an unterminated string is defined purely by reaching `None` (EOF) inside the body loop, not by hitting a newline.
- Bad-escape cases (`"\\q"`, `"\\xZZ"`, etc.) keep their current 1-byte-bump recovery for now; tightening that behavior is a separate todo (not in the pending list, and not implied by this slug).
- I'll introduce a private `enum ScanStringOutcome` (or equivalent) inside `scan.rs` rather than reusing `Result`/exposing a new public type, keeping the API surface small.
- The test asserts the exact error string `"unterminated string literal"`; if reviewers prefer a different message, only the test and one literal need updating.

## Blockers
Blockers: none

## Summary
Refactor `scan_string` to signal unterminated-EOF as a distinct outcome and have `next_token` emit one `Error` token spanning open-quote → EOF before continuing at EOF, pinned by a new `unterminated_string_recovers` test.

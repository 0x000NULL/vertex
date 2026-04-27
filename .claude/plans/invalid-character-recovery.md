# Plan: invalid-character-recovery

## Goal
Make the scanner's catch-all path emit a descriptive `TokenKind::Error("invalid character: <ch>")`, advance exactly one Unicode codepoint, and continue scanning — covered by a regression test pinning recovery semantics.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, locate the catch-all tail of `Scanner::next_token` (the block after `scan_operator` returns `None` at lines 817–821) which currently emits `TokenKind::Error(ch.to_string())` after advancing one codepoint.
2. Change the emitted message from `ch.to_string()` to `format!("invalid character: {}", ch)`. Keep the codepoint-width advance (`self.pos += ch.len_utf8()`) and the span (`start..self.pos`) — those are already correct for the "advance one codepoint; continue" requirement.
3. Do **not** touch the two earlier early-return paths that emit `TokenKind::Error("\"")` and `TokenKind::Error("'")` for unterminated string / char literals — those belong to the `unterminated-string-recovery` and (implicitly) char-recovery items, and changing them now would step on that work.
4. Add a `#[test] fn invalid_char_recovers()` to the `tests` mod inside `src/lexer/scan.rs` that drives the full `Scanner::next_token` loop on input that mixes valid tokens with invalid characters (e.g., `"a $ b @ c"` plus a multibyte invalid like `"€"` or an emoji), and asserts:
   - The invalid-character tokens are `TokenKind::Error("invalid character: $".to_string())`, `TokenKind::Error("invalid character: @".to_string())`, etc.
   - Each error span covers exactly one codepoint (so `span.end - span.start == ch.len_utf8() as u32`, and for the multibyte case the width is >1).
   - Scanning continues past the invalid char — the surrounding `Ident("a")`, `Ident("b")`, `Ident("c")` tokens are produced before `Eof`.
   - Final token is `Eof` with empty span at `src.len()`.
5. Run the verify commands below to confirm the new test passes and the existing scanner tests (especially `all_tokens_have_nonzero_span`, which already feeds a `$` to the catch-all) still pass.

## Files
- `vertex_stage0/src/lexer/scan.rs` — change the catch-all `Error(ch.to_string())` at the tail of `next_token` to `Error(format!("invalid character: {}", ch))`; add `invalid_char_recovers` test in the existing `#[cfg(test)] mod tests` block.

## Risks
- The existing `all_tokens_have_nonzero_span` test (line 1549) already feeds a `$` to this catch-all but only asserts `t.span.start < t.span.end`; it does not pin the error string, so it will keep passing after the message change. Verified by inspection — no other test asserts the literal Error string for the catch-all path.
- The string/char fall-throughs (lines 750, 761) intentionally emit a bare `"\""` / `"'"` Error and are out of scope; touching them would conflict with `unterminated-string-recovery`. Plan keeps them untouched.
- Multibyte codepoint advance is already implemented via `ch.len_utf8()`; no UTF-8 boundary risk introduced.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib lexer::scan::tests::invalid_char_recovers
cargo test --manifest-path vertex_stage0/Cargo.toml --lib lexer::scan
```

## Assumptions
- The crate manifest lives at `vertex_stage0/Cargo.toml` (confirmed by `vertex_stage0/src/...` layout); the verify line specified by the todo (`cargo test --lib lexer::scan::tests::invalid_char_recovers`) is run from the workspace root, so I pass `--manifest-path` to make it work regardless of cwd.
- The required error message format is the literal Rust string `"invalid character: <ch>"` where `<ch>` is the character's `Display` form (e.g., `"invalid character: $"`, `"invalid character: €"`). This matches the todo wording verbatim and is the natural `format!("invalid character: {}", ch)` rendering.
- "Advance one codepoint" means `ch.len_utf8()` bytes, which is what the current code already does — no change needed there.
- The new test belongs in `src/lexer/scan.rs`'s existing `tests` module (matches the verify path `lexer::scan::tests::invalid_char_recovers`); no new file is needed.
- I will not modify the unterminated-string `Error("\"")` or unterminated-char `Error("'")` branches even though they share the `Error` constructor — those are the responsibility of the `unterminated-string-recovery` item and a future char-recovery item.
- The `is_ascii_alphabetic` ident path at line 789 cannot fall through to the catch-all (every ASCII alpha is consumed by `scan_ident_or_keyword`), so the new branch only fires for genuinely non-token bytes/codepoints — consistent with "invalid character" framing.

## Blockers
Blockers: none

## Summary
Replace the scanner's catch-all `Error(ch)` with a descriptive `Error("invalid character: <ch>")` while preserving single-codepoint advance, and pin the behavior with an `invalid_char_recovers` regression test.

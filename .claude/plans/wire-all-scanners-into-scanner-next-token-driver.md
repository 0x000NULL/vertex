# Plan: wire-all-scanners-into-scanner-next-token-driver

## Goal
Add a `Scanner::next_token` driver in `vertex_stage0/src/lexer/scan.rs` that skips whitespace/comments, dispatches on the first byte to the existing sub-scanners, and emits `Token { kind, span }` (including `TokenKind::Eof`), then add the `tokenizes_full_program` unit test that exercises it end-to-end.

## Steps
1. Add a private `is_whitespace` helper (matches ` `, `\t`, `\n`, `\r`) and a `skip_whitespace_and_comments` helper that loops calling `eat_while(is_whitespace)` and `skip_comments`, terminating only when neither makes progress. Doc comments must NOT be eaten here — `skip_comments` already returns `false` on `///` / `//!`, so the loop naturally stops on a doc-comment start and lets the dispatcher emit a `DocComment` token.
2. Add `pub fn next_token(&mut self) -> Token`. Body:
   - call the skip helper.
   - if `peek().is_none()` → return `Token { kind: TokenKind::Eof, span: Span::new(file_id, pos, pos) }`.
   - record `start = self.pos` for fallback / error spans.
   - dispatch on `peek().unwrap()`:
     - `b'/'` with second byte `b'/'` or `b'!'` (i.e. `///` or `//!`) → `scan_doc_comment` → `TokenKind::DocComment(body, style)`. (Plain `//` / `/*` were already skipped above; reaching `/` here means it's an operator or doc.) Otherwise fall through to operator dispatch for `/` and `/=`.
     - `b'r'` with next byte `b'#'` or `b'"'` → `scan_raw_string` → `TokenKind::RawStringLiteral(s)`. If `scan_raw_string` returns `None` (defensive), fall through to identifier scanning.
     - `b'"'` → `scan_string` → `TokenKind::StringLiteral(s)`; on `None`, emit a one-byte `TokenKind::Error` token covering the `"` and advance past it (this is just the dispatcher's last-resort behaviour; the dedicated unterminated-string-recovery item refines it later).
     - `b'\''` → `scan_char` → `TokenKind::CharLiteral(c)`; on `None` same one-byte error fallback.
     - byte is ASCII digit:
        - if `b'0'` and `peek_at(1) == Some(b'x' | b'X')` → `scan_int_hex` → `TokenKind::IntLiteral(v, suf)`; on `None` fall through to decimal so we still consume.
        - if `b'0'` and `peek_at(1) == Some(b'b' | b'B')` → `scan_int_bin` similarly.
        - else try `scan_float` first (it rewinds on failure since it needs `digit '.' digit`); if `Some` → `TokenKind::FloatLiteral`; if `None` → `scan_int_decimal` → `TokenKind::IntLiteral`.
     - byte is ASCII alphabetic → `scan_ident_or_keyword` (already returns the right `TokenKind`).
     - byte is `b'_'`:
        - if `peek_at(1)` is alphanumeric or `_`, treat as identifier: consume run, build `TokenKind::Ident(lex.to_string())`.
        - else emit `TokenKind::Underscore` for the single byte.
     - otherwise → `scan_operator`; if `Some` use it, if `None` advance one UTF-8 char and emit `TokenKind { kind: TokenKind::Error(<that char as String>), span }` — the dedicated `invalid-character-recovery` item will refine the error payload later, but the dispatcher must always make progress so the test doesn't infinite-loop.
3. Make sure every returned `Token` has a span starting at the pre-skip dispatch `start` and ending at the current `self.pos` (sub-scanners already produce correct spans; just reuse those). For the Eof token, `start == end == self.pos`.
4. Add `#[test] fn tokenizes_full_program` in the existing `tests` module. Feed a single source string that exercises every dispatcher branch — keywords (`fn let if`), identifiers, `_`, decimal/hex/bin int literals with suffixes, a float literal, a char literal, a regular string, a raw string with hashes, a doc comment (`///`), a regular line comment + block comment (skipped), every operator class touched by the dispatch (`+ - * / % == != <= >= << >> .. ..= -> => :: ; , ( ) { } [ ] ? & | ^ ~ . : =`), and trailing whitespace. Loop calling `next_token` until `TokenKind::Eof`, collect into a `Vec<TokenKind>` (drop spans for the equality assert but separately assert that no token's span is `is_empty()` except `Eof`, and that successive token spans are non-decreasing and contiguous-or-separated-only-by-whitespace/comments). Compare the kinds vector against the hand-written expected vector.

## Files
- `vertex_stage0/src/lexer/scan.rs` — add the `is_whitespace` / `skip_whitespace_and_comments` private helpers, the new `pub fn next_token(&mut self) -> Token` method, and the `tokenizes_full_program` test in the existing `#[cfg(test)] mod tests`. No changes outside this file.

## Risks
- Float-vs-int dispatch: calling `scan_float` first then falling back to `scan_int_decimal` works only because `scan_float` rewinds on failure. Confirmed by reading `scan_float` (rewinds to `start` on every failure path) — relying on that contract.
- Hex/bin fallback: `scan_int_hex`/`scan_int_bin` already rewind on `0x`/`0b` with no following digits, so falling through to `scan_int_decimal` (which will then eat just the leading `0`) is safe; the test should not include such malformed inputs since this item is the dispatcher, not the recovery item.
- `scan_doc_comment` after `skip_whitespace_and_comments` — must not be double-skipped. `skip_comments` returns `false` for `///` and `//!`, so the loop stops naturally; verified in `nested_block_comments` test (lines 1076–1082).
- `r` identifier vs raw string: must check `peek_at(1)` is `#` or `"` before calling `scan_raw_string`, otherwise plain identifier `r` (or `rest`) would get mis-dispatched and rewound, wasting work but not breaking correctness. Guarding the dispatch is cheaper than relying on rewind.
- Infinite loop on unrecognized byte: must always advance at least one UTF-8 char on the error fallback path, even if it costs us the "perfect" span. Without this, malformed input hangs `next_token`.
- Test brittleness: keep the test source string short and all-ASCII so its `expected` vector is reviewable.

## Prereqs
Prereqs: none

(All sub-scanners and `TokenKind` variants this driver references already exist on disk per the recent commits and `vertex_stage0/src/lexer/{scan.rs,token.rs}`. The dedicated recovery items — `invalid-character-recovery`, `unterminated-string-recovery`, `invalid-numeric-literal-recovery` — refine the `TokenKind::Error` paths *after* this driver lands, so they depend on this item, not the other way around.)

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib lexer::scan::tests::tokenizes_full_program
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate manifest lives at `vertex_stage0/Cargo.toml` (confirmed by the `vertex_stage0/src/lexer/scan.rs` location); the verify command therefore needs `--manifest-path`. If a workspace `Cargo.toml` exists at repo root, plain `cargo test --lib lexer::scan::tests::tokenizes_full_program` would also work, but `--manifest-path` is robust to both layouts.
- Whitespace is ASCII only (` `, `\t`, `\n`, `\r`). Unicode whitespace is out of scope per the existing scanners (e.g. `eat_while` is byte-oriented).
- `_` alone (not followed by an identifier-continuation byte) emits `TokenKind::Underscore`; `_foo` is a `TokenKind::Ident("_foo")`. This matches `TokenKind::Underscore` already existing in `token.rs` and the rejection of `_`-led identifiers in `scan_ident_or_keyword` (line 570 — only `is_ascii_alphabetic` start), so the dispatcher must own this case.
- Unknown byte / unrecognized punctuation (`#`, `@`, `$`, lone `!`, etc.) emits `TokenKind::Error(String)` carrying the offending UTF-8 char, advancing exactly that char's `len_utf8()` bytes. This is a placeholder; the `invalid-character-recovery` item will harden it.
- Doc comments come out as their own token, not merged with surrounding tokens, and are not silently skipped (per the existing `scan_doc_comment` contract and the `doc_comments_preserved` test).
- `Token` is constructible via `Token::new(kind, span)` (confirmed at `token.rs:122-125`).
- The `tokenizes_full_program` test compares `Vec<TokenKind>` (cloning `kind` out of each token) — `TokenKind` already derives `PartialEq` (token.rs:31), so direct `assert_eq!` works.
- Float literal in the test will be `1.5` (or similar `digit '.' digit`) so `scan_float` accepts it; `scan_float` rejects `1.` and `.5`, which matches the established contract.
- `r`-prefixed raw string in the test will use a bare-quote form like `r"abc"` so it's unambiguous; we will not try to test `r#"..."#` parsing-vs-attribute boundary here (covered already by `raw_string_arbitrary_hashes`).

## Blockers
Blockers: none

## Summary
Adds `Scanner::next_token` plus a whitespace/comment skipper that fans every input byte into the right existing sub-scanner, emits `TokenKind::Eof` at end of input, and proves the wiring with a single end-to-end `tokenizes_full_program` test.

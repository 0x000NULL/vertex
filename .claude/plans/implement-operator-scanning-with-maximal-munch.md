# Plan: implement-operator-scanning-with-maximal-munch

## Goal
Add a `Scanner::scan_operator` method that recognizes Vertex's punctuation/operator tokens using maximal munch (longest-match-wins), with a unit test asserting prefix-conflict pairs/triples resolve to the longest valid token.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, add a public method `scan_operator(&mut self) -> Option<(TokenKind, Span)>` on `Scanner<'a>`. Add `use crate::lexer::token::TokenKind;` to the file's imports.
2. Implement `scan_operator` by peeking the first byte, then dispatching per-glyph and checking subsequent bytes for the longest match. Concretely:
   - `.`: try `..=` → `DotDotEq`, else `..` → `DotDot`, else `.` → `Dot`.
   - `<`: try `<=` → `Le`, else `<<` → `Shl`, else `<` → `Lt`. (No `<<=` in `TokenKind`; longest available wins.)
   - `>`: try `>=` → `Ge`, else `>>` → `Shr`, else `>` → `Gt`.
   - `=`: try `==` → `EqEq`, else `=>` → `FatArrow`, else `=` → `Eq`.
   - `-`: try `-=` → `MinusEq`, else `->` → `Arrow`, else `-` → `Minus`.
   - `+`: try `+=` → `PlusEq`, else `+` → `Plus`.
   - `*`: try `*=` → `StarEq`, else `*` → `Star`.
   - `/`: try `/=` → `SlashEq`, else `/` → `Slash`. (Comment / doc-comment dispatch lives upstream in `next_token`; `scan_operator` returns `Slash` for a lone `/`.)
   - `%`: try `%=` → `PercentEq`, else `%` → `Percent`.
   - `!`: only `!=` → `BangEq`. Lone `!` returns `None` (no `Bang` variant in `TokenKind` yet; caller handles).
   - `:`: try `::` → `ColonColon`, else `:` → `Colon`.
   - `&` → `Amp`, `|` → `Pipe`, `^` → `Caret`, `~` → `Tilde`.
   - `(` → `LParen`, `)` → `RParen`, `[` → `LBracket`, `]` → `RBracket`, `{` → `LBrace`, `}` → `RBrace`.
   - `?` → `Question`, `;` → `Semi`, `,` → `Comma`.
   - Anything else: return `None` (do not consume). `Underscore`, identifiers, keywords, numerics, strings, doc comments, and `#`/`@`/`$` are out of scope here.
3. Each match captures `start = self.pos as u32` before advancing, advances `self.pos` by the matched byte length, and returns `Some((kind, Span::new(self.file_id, start, self.pos as u32)))`. On `None`, `self.pos` is unchanged.
4. Add a `#[test] fn operator_maximal_munch()` in the existing `mod tests` block. Drive a table of `(input, expected_kind, expected_consumed_len)` covering:
   - Triple/double/single conflict roots: `..=`, `..`, `.`; `<=`, `<<`, `<`; `>=`, `>>`, `>`; `==`, `=>`, `=`; `-=`, `->`, `-`; `::`, `:`; `!=`.
   - Plain assignment/op pairs: `+=`/`+`, `*=`/`*`, `/=`/`/`, `%=`/`%`.
   - Single-byte punctuation: `&`, `|`, `^`, `~`, `?`, `;`, `,`, `(`, `)`, `[`, `]`, `{`, `}`.
   - Boundary case: input `..=x` consumes 3 bytes and produces `DotDotEq`; input `..x` consumes 2 bytes and produces `DotDot`; input `.x` consumes 1.
   - Negative cases that must return `None` and leave `pos == 0`: `"a"`, `"_"`, `"#"`, `"@"`, `"$"`, `""`, `"!"` (lone `!`).
   For each happy case the test asserts the returned `TokenKind`, `span.start == 0`, `span.end as usize == expected_consumed_len`, `s.pos == expected_consumed_len`, and `span.file_id` matches the constructor argument.
5. Do not wire `scan_operator` into a `next_token` driver; that is a separate todo item (`wire-all-scanners-into-scanner-next-token-driver`).

## Files
- `vertex_stage0/src/lexer/scan.rs` — add `use crate::lexer::token::TokenKind;`, add `pub fn scan_operator` method on `Scanner`, add `#[test] fn operator_maximal_munch` inside the existing `mod tests`.

## Risks
- Prefix-conflict ordering bugs: e.g., checking `..` before `..=` would shadow the longer match. Each glyph's branches must check the longest candidate first.
- Forgetting to rewind on `None`: the contract elsewhere in this file is "if `Some`, `pos` advanced; if `None`, `pos` unchanged." `scan_operator` only advances after committing to a match, so this is naturally upheld — but easy to break if a future maintainer pre-advances `self.pos`.
- Lone `!` returning `None` may surprise a future driver author who expects `scan_operator` to produce *something* for `!`. Documented via the assumptions section; a `Bang` variant can be added later when macro recognition lands.
- `/` returning `Slash` here means a future `next_token` driver must call `skip_comments`/`scan_doc_comment` *before* `scan_operator` to avoid consuming `//`/`/*`/`///`/`//!` as a `Slash`. Matches the existing module's design.
- Multi-byte UTF-8 leading bytes (e.g. an em-dash) would have their first byte fail every match arm and hit the default `None`, leaving `pos` at 0. Safe.

## Prereqs
Prereqs: none

(Token variants `DotDotEq`, `DotDot`, `Dot`, `Le`, `Shl`, `Lt`, `Ge`, `Shr`, `Gt`, `EqEq`, `FatArrow`, `Eq`, `MinusEq`, `Arrow`, `Minus`, `PlusEq`, `Plus`, `StarEq`, `Star`, `SlashEq`, `Slash`, `PercentEq`, `Percent`, `BangEq`, `ColonColon`, `Colon`, `Amp`, `Pipe`, `Caret`, `Tilde`, `LParen`, `RParen`, `LBracket`, `RBracket`, `LBrace`, `RBrace`, `Question`, `Semi`, `Comma` are already defined in `vertex_stage0/src/lexer/token.rs`, so no token-enum work is required.)

## Verify
```
cargo test --lib -p vertex_stage0 lexer::scan::tests::operator_maximal_munch
cargo build -p vertex_stage0
```

## Assumptions
- Method signature is `pub fn scan_operator(&mut self) -> Option<(TokenKind, Span)>` to match the existing `scan_char`/`scan_string` shape (Option of value+span). The bundled todo says "Method `Scanner::scan_operator`" without specifying a return type; this shape is consistent with the rest of the module.
- Lone `!` returns `None` rather than producing an error token, because `TokenKind` has no `Bang` variant. A future macro-recognition todo can add `Bang` and update this method; the unit test pins this behavior so the change is intentional.
- `/` returns `Slash` even though `//`/`/*` start comments. The existing `skip_comments`/`scan_doc_comment` already own that dispatch; `scan_operator` is one of several mutually-exclusive scanners the future `next_token` driver will sequence (per `wire-all-scanners-into-scanner-next-token-driver`).
- `_` is not handled by `scan_operator`; it is consumed by identifier scanning (it can be an identifier prefix). The negative test asserts `scan_operator("_")` returns `None`.
- `#`, `@`, `$` are not produced as tokens here. `#` is part of attributes (parsed at item level), `@` is a binding pattern operator (handled when patterns land), `$` is unused. They all return `None` from `scan_operator` so the future driver can decide how to surface them (likely `Error` token or attribute parsing).
- `<<=` and `>>=` are not produced. The bundled spec uses `<<=` only as an *example* of the maximal-munch ordering principle; the language's actual assignment operators per spec §22 are `=` `+=` `-=` `*=` `/=` `%=` only, and `TokenKind` reflects this. The maximal-munch ordering for `<` and `>` therefore stops at the 2-char level (`<=`/`<<`, `>=`/`>>`).
- Verify uses `-p vertex_stage0` because the workspace root has no library of its own; running `cargo test --lib` at the workspace root would otherwise miss the package filter.

## Blockers
Blockers: none

## Summary
Adds maximal-munch operator/punctuation scanning to `Scanner` and a table-driven unit test that pins prefix-conflict resolution for `.`/`..`/`..=`, `<`/`<=`/`<<`, `>`/`>=`/`>>`, `=`/`==`/`=>`, `-`/`-=`/`->`, `:`/`::`, and `!=`.

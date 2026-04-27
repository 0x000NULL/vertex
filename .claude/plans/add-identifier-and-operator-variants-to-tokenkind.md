# Plan: add-identifier-and-operator-variants-to-tokenkind

## Goal
Extend the existing `TokenKind` enum in `vertex_stage0/src/lexer/token.rs` with identifier, operator, punctuation, and special variants so the lexer has a complete vocabulary for subsequent scanning steps.

## Steps
1. Open `vertex_stage0/src/lexer/token.rs` and locate the existing `TokenKind` enum (currently has keyword + literal variants).
2. Append the `Ident(String)` variant (carrying the source identifier text).
3. Append the operator variants: `Plus, Minus, Star, Slash, Percent, EqEq, BangEq, Lt, Gt, Le, Ge, Amp, Pipe, Caret, Tilde, Shl, Shr, Eq, PlusEq, MinusEq, StarEq, SlashEq, PercentEq`.
4. Append the punctuation variants: `Dot, ColonColon, LBracket, RBracket, LParen, RParen, LBrace, RBrace, Question, DotDot, DotDotEq, Arrow, FatArrow, Semi, Comma, Colon, Underscore`.
5. Append the special variants: `Eof` and `Error(String)` (carrying a diagnostic-friendly message).
6. Keep the existing `#[derive(Debug, Clone, PartialEq)]` on `TokenKind` (already present; the new payload-carrying `String` variants are compatible with `Clone + PartialEq`, and unit variants are trivially fine).
7. Run `cargo build -p vertex_stage0` to confirm the enum still compiles and the crate continues to build.

## Files
- `vertex_stage0/src/lexer/token.rs` -- append the new `Ident`, operator, punctuation, and special variants to the existing `TokenKind` enum (no other items added; no changes to `IntSuffix`/`FloatSuffix`).

## Risks
- Naming collision risk is low since these variants are new, but `Not`, `And`, `Or` already exist as keyword variants — must NOT add operator variants under those same names. The spec uses `Bang`-style names is avoided here per the explicit list (logical `!`, `&&`, `||` are not requested in this todo's operator set, and bitwise versions use `Amp`, `Pipe`); confirmed no overlap.
- Future lexer code will need to construct `Error(String)` and `Ident(String)` — heap allocation per token is acceptable for stage0 but could be revisited later (out of scope).
- Adding many variants without `#[non_exhaustive]` means downstream `match` exhaustiveness will fail until follow-up parser/scanner code handles them; acceptable since no consumers exist yet.

## Prereqs
- define-tokenkind-enum-in-src-lexer-token-rs-keyword-variants
- add-literal-variants-to-tokenkind

## Verify
```
cargo build -p vertex_stage0
grep -q "Ident(String)" vertex_stage0/src/lexer/token.rs
grep -q "DotDotEq" vertex_stage0/src/lexer/token.rs
grep -q "FatArrow" vertex_stage0/src/lexer/token.rs
grep -q "Error(String)" vertex_stage0/src/lexer/token.rs
grep -q "Eof" vertex_stage0/src/lexer/token.rs
```

## Assumptions
- The existing `TokenKind` enum in `vertex_stage0/src/lexer/token.rs` (which already contains keyword + literal variants per recent commits) is the correct extension point — no second enum is created.
- Existing derives `#[derive(Debug, Clone, PartialEq)]` on `TokenKind` are kept; `Eq`/`Hash` are NOT added because `FloatLiteral(f64, ...)` already prevents `Eq`/`Hash`. New variants don't change that.
- `Ident(String)` and `Error(String)` use owned `String` rather than `&str`/interned ID, consistent with the existing `StringLiteral(String)` / `RawStringLiteral(String)` choices in this enum.
- `Underscore` is included as a token, with the keyword-vs-identifier disambiguation step (later) responsible for emitting it for the bare `_` lexeme.
- Operator names like `Shl`/`Shr` correspond to `<<`/`>>` (left/right shift) per the explicit spec list; no separate `LtLt`/`GtGt` aliases.
- No tests are added in this item — verification is `cargo build` plus grep presence checks for the most distinctive new variants. Behavioral testing happens in later scanner items.
- Variant ordering: append after existing literal variants. No re-ordering of existing variants (which would churn diffs and risk breaking earlier commits' references).

## Blockers
Blockers: none

## Summary
Adds identifier, operator, punctuation, and special (`Eof`, `Error`) variants to `TokenKind`, completing the token vocabulary needed by upcoming scanner sub-tasks.

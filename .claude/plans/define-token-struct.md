Now I have a complete picture. The `Token` struct needs to live in `vertex_stage0/src/lexer/token.rs` alongside the existing `TokenKind`, `IntSuffix`, and `FloatSuffix` enums. `Span` is already defined in `vertex_stage0/src/span.rs`.

# Plan: define-token-struct

## Goal
Add a public `Token` struct with `kind: TokenKind` and `span: Span` fields to `vertex_stage0/src/lexer/token.rs`, giving downstream lexer work a concrete value type to emit.

## Steps
1. In `vertex_stage0/src/lexer/token.rs`, add a `use crate::span::Span;` import at the top of the file.
2. Append a public `Token` struct with two fields: `pub kind: TokenKind` and `pub span: Span`.
3. Derive `Debug` and `Clone` on `Token` (matching `TokenKind`'s derives — `PartialEq` is omitted since `TokenKind::FloatLiteral(f64, ...)` does not implement `Eq` and would only constrain consumers; the existing `TokenKind` already has `PartialEq` so callers can still compare kinds directly).
4. Add a small `impl Token { pub fn new(kind: TokenKind, span: Span) -> Self }` constructor for ergonomic construction by the upcoming scanner.
5. Run `cargo build` to confirm the crate still compiles cleanly.

## Files
- `vertex_stage0/src/lexer/token.rs` — add `use crate::span::Span;` import; append `pub struct Token { pub kind: TokenKind, pub span: Span }` with `#[derive(Debug, Clone)]` and a `new` constructor.

## Risks
- Deriving `PartialEq` on `Token` would fail because `TokenKind::FloatLiteral` carries `f64` which is not `Eq`. The existing `TokenKind` only derives `PartialEq` (not `Eq`); we should match that or omit it on `Token`. Mitigation: derive only `Debug, Clone` to keep parity with downstream needs and avoid trait-bound surprises.
- Future tests/snapshot helpers may want `PartialEq` to compare tokens. Mitigation: easy to add later when those items materialize; keeping the surface minimal now follows the "don't add until needed" guideline.

## Prereqs
Prereqs: none

(`Span` already exists in `vertex_stage0/src/span.rs` and `TokenKind` is already complete in `vertex_stage0/src/lexer/token.rs` per recent commits — no other pending items in this run are upstream of this change.)

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
grep -q 'pub struct Token' vertex_stage0/src/lexer/token.rs
grep -q 'pub kind: TokenKind' vertex_stage0/src/lexer/token.rs
grep -q 'pub span: Span' vertex_stage0/src/lexer/token.rs
```

## Assumptions
- The crate root is `vertex_stage0/` (Cargo manifest there), based on the actual file layout — the todo says `src/lexer/token.rs` but the real path is `vertex_stage0/src/lexer/token.rs`. Verify commands use the real path.
- `Token` belongs in the same file as `TokenKind` (`lexer/token.rs`), not a separate file. The todo's verify hint (`grep -q 'pub struct Token' src/lexer/token.rs`) confirms this colocation.
- Derive set: `Debug, Clone` only. `PartialEq`/`Eq` are skipped because `TokenKind` cannot derive `Eq` (carries `f64`); matching `TokenKind`'s existing derives is the conservative choice.
- A `new(kind, span)` constructor is included for ergonomics; this is a tiny convenience that the scanner will use heavily and does not constitute over-engineering.
- No edits to `lexer/mod.rs` are needed — `token` is already declared as a public submodule, so `Token` will be reachable as `crate::lexer::token::Token` automatically.
- No re-export of `Token` from `lexer/mod.rs` is added; the spec doesn't ask for one and downstream items can reference the full path or add a re-export when they actually need it.

## Blockers
Blockers: none

## Summary
Adds a 2-field `Token { kind, span }` struct to `vertex_stage0/src/lexer/token.rs` so the scanner has a concrete token value type to emit.

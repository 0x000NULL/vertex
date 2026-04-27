# Plan: parse-parenthesized-tuple-unit

## Goal
Add `Parser::parse_paren_or_tuple` to `src/parser/expr.rs` that disambiguates `()`, `(expr)`, `(a, b, ...)`, and `(x,)` into the existing `Expr::TupleLit` (or unwrapped inner `Expr`), backed by a temporary `parse_primary_for_paren` literal stub and a `paren_tuple_unit` unit test.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, add a private helper `fn parse_primary_for_paren(&mut self) -> Result<Expr, CompileError>` that peeks and dispatches to the existing literal parsers — `IntLiteral`/`FloatLiteral`/`CharLiteral`/`StringLiteral`/`RawStringLiteral`/`True`/`False` map to `parse_int_lit`/`parse_float_lit`/`parse_char_lit`/`parse_str_lit`/`parse_bool_lit` respectively. Any other head returns `self.unexpected_token_error("expression")` without advancing. Comment it as a temporary stub to be replaced by `parse_primary` in item 49.
2. Add `pub fn parse_paren_or_tuple(&mut self) -> Result<Expr, CompileError>` on the existing `impl Parser` block.
3. Validate the head: if `peek() != TokenKind::LParen`, return `self.unexpected_token_error("`(`")` without advancing.
4. Bump `(`, save `lparen_span = tok.span`.
5. Empty case `()`: if `peek() == RParen`, bump it (`rparen_span`), allocate a `NodeId`, build `Expr::TupleLit(TupleLit { id, span: lparen_span.merge(&rparen_span), elems: vec![] })`, return.
6. Otherwise parse first inner expression via `self.parse_primary_for_paren()?`, store as `first`.
7. Branch on `peek()`:
   - `RParen`: bump it; return `Ok(first)` unwrapped — no `Paren` wrapper node (per resolved blocker default).
   - `Comma`: bump it. Promote to tuple. Initialize `elems = vec![first]`. Loop: while `peek() != RParen` and `peek() != Eof`, parse an element with `self.parse_primary_for_paren()?`, push it; if `peek() == Comma`, bump it and continue (trailing comma allowed); else break out so the `RParen` check runs. After the loop, `expect(&TokenKind::RParen)?`. Allocate a `NodeId`, build `Expr::TupleLit(TupleLit { id, span: lparen_span.merge(&rparen_span), elems })`, return. The single-comma form `(x,)` falls out of this loop with `elems = vec![first]`.
   - Anything else: return `self.unexpected_token_error("`,` or `)`")` (does not advance, matches existing error style).
8. Add a `#[test] fn paren_tuple_unit` to the existing `tests` mod in `parser/expr.rs`. Drive each scenario with hand-built `Token` vectors ending in `Eof`, asserting `pos` lands on the `Eof` token, `errors.is_empty()`, and the AST shape:
   - `( )` → `Expr::TupleLit` with `elems.len() == 0`.
   - `( 1i32 )` → `Expr::IntLit(IntLit { value: 1, .. })` (inner unwrapped).
   - `( 1i32 , )` → `Expr::TupleLit` with `elems.len() == 1`, elem `IntLit(1)`.
   - `( 1i32 , 2i32 )` → `Expr::TupleLit` with `elems.len() == 2`.
   - `( 1i32 , 2i32 , )` → `Expr::TupleLit` with `elems.len() == 2` (trailing comma tolerated).
   - Wrong head: `+` followed by `Eof` → `parse_paren_or_tuple` returns `Err` and `pos` stays at `0`, `errors.is_empty()`.

## Files
- `vertex_stage0/src/parser/expr.rs` — add the `parse_primary_for_paren` private stub, the `parse_paren_or_tuple` method on `impl Parser`, and the `paren_tuple_unit` test in `mod tests`. No edits to AST, lexer, or `parser/mod.rs`.

## Risks
- The `parse_primary_for_paren` stub only knows about literals; it will be deleted/replaced when `parse_primary` lands (item 49). Tests below stay within literal heads, so this is acceptable until then.
- A `( x ; y )` mid-stream (junk between elems) routes to the `Comma`-vs-`RParen` branch's error path; no resync is performed inside this method — callers using `recover_to_sync` handle that. Matches the existing literal-parser error contract.
- Span merging assumes the `LParen`/`RParen` token spans bound the literal — which they do here because the test tokens carry trivial spans.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::expr::tests::paren_tuple_unit
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- Per the resolved blocker, `(expr)` is unwrapped to the inner `Expr` — no new `Expr::Paren` variant is added (none exists today).
- `()` produces `Expr::TupleLit { elems: [] }`; there is no separate `Expr::Unit`/`UnitLit` variant in `ast::expr`, so the empty tuple literal models the unit value.
- `(x,)` (single trailing comma) is a 1-tuple, matching Rust's disambiguation; without the trailing comma `(x)` is a parenthesized expression.
- Per the resolved blocker, adding the temporary `parse_primary_for_paren` stub is expected; it will be unified with the future Pratt-driver primary parser in item 49.
- The verify test name `paren_tuple_unit` lives in `parser::expr::tests`, matching the literal-tests pattern already established there.
- Cargo manifest lives at `vertex_stage0/Cargo.toml` (only crate in the workspace), so verify commands target it explicitly — same convention used by the parse-path-expressions plan.
- A trailing comma inside a tuple is tolerated; `(,)` (leading comma) is not — it would fall out as the second `parse_primary_for_paren` call hitting `Comma` and erroring.
- Recovery on malformed paren bodies is left to the caller via `recover_to_sync`; this method does not push to `self.errors`, it returns `Err`.

## Blockers
Blockers: none

## Summary
Implements `Parser::parse_paren_or_tuple` to disambiguate `()`, `(expr)`, `(x,)`, and `(a, b, ...)` into existing AST nodes (unwrapped `Expr` or `TupleLit`), backed by a temporary literal-only primary stub and the unit test the verify line requires.

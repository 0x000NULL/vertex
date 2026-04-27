# Plan: block-trailing-expression-as-value-semantics

## Goal
Lock in the existing `parse_block` semantic — last expression without `;` becomes the block's tail value, otherwise the block is unit — by adding a `parser::stmt::tests::block_value_semantics` unit test in `src/parser/stmt.rs`.

## Steps
1. Open `vertex_stage0/src/parser/stmt.rs` and extend the existing `#[cfg(test)] mod tests`.
2. Add `use crate::ast::expr::{Block, Expr};` to the test module's imports (alongside the existing `Token`/`TokenKind`/`Span` imports) so the test can pattern-match `Expr::Block(Block { stmts, tail, .. })`.
3. Add a small helper `fn as_block(e: Expr) -> Block` that unwraps `Expr::Block(b) => b` and panics otherwise (mirrors `int_value` style already in the file).
4. Add `#[test] fn block_value_semantics` that constructs token streams via the existing `int_tok` / `tok` helpers and calls `Parser::parse_block` for each case below. After each case assert `p.errors.is_empty()` and that `p.peek()` is at `TokenKind::Eof`:
   - **Tail only — block-typed:** `{ 1i32 }` → `stmts.is_empty()`, `tail.is_some()`, inner `IntLit.value == 1`.
   - **Trailing semi — unit-typed:** `{ 1i32 ; }` → `stmts.len() == 1`, the lone stmt is `Stmt::Expr { has_semi: true }`, `tail.is_none()`.
   - **Empty — unit-typed:** `{ }` → `stmts.is_empty()`, `tail.is_none()`.
   - **Stmt then tail:** `{ 1i32 ; 2i32 }` → one `Stmt::Expr { has_semi: true, value 1 }`, `tail = Some(IntLit(2))`.
   - **All semi'd — unit-typed:** `{ 1i32 ; 2i32 ; }` → two `Stmt::Expr { has_semi: true }`, `tail.is_none()`.
5. Run `cargo test --lib parser::stmt::tests::block_value_semantics` from the workspace root and confirm it passes — no production-code changes are expected because `parse_block` (`vertex_stage0/src/parser/expr.rs:528-560`) already implements these semantics.

## Files
- `vertex_stage0/src/parser/stmt.rs` -- extend `mod tests` with `as_block` helper, additional imports for `Expr`/`Block`, and `#[test] fn block_value_semantics` covering the five cases above. No production code changes.

## Risks
- The verify path `parser::stmt::tests::block_value_semantics` is in `parser/stmt.rs`, but the function under test (`parse_block`) lives in `parser/expr.rs`. This is intentional and matches how `parse-let-statements` plans `parser::stmt::tests::let_forms` for code that exercises a sibling module — the test name's module path is what the spec dictates, not the location of the SUT. No risk of collision.
- `parse_block` currently treats `{ expr expr }` (two exprs with no `;` between them) as `Stmt::Expr { has_semi: false }` followed by the second as `tail`. The plan does **not** lock that quirk in — I avoid asserting on it because the spec line "last statement without `;` is the block's value; otherwise unit" is silent on the inter-stmt-no-semi case, and a future statement-dispatch refactor (e.g. `parse-let-statements`, `parse-item-statements-nested-fn-struct-inside-a-block`) may legitimately tighten this into an error.
- The five cases use only `IntLit` because `parse_primary_for_paren` doesn't yet accept identifiers — same constraint already accepted by `parser::expr::tests::block_trailing_expr` and `parser::stmt::tests::semicolon_significance`.

## Prereqs
Prereqs: none

(`parse_block`, `Block`, and `Stmt::Expr { has_semi }` all already exist; the `define-stmt-enum-in-src-ast-stmt-rs` item is bookkeeping for a later AST tidy-up and does not block this test, since `Stmt::Expr { expr, has_semi }` is the shape this test asserts on today.)

## Verify
```
cargo test --lib parser::stmt::tests::block_value_semantics
cargo test --lib parser::expr::tests::block_trailing_expr
cargo test --lib parser::stmt::tests::semicolon_significance
```

## Assumptions
- The spec's verify command `cargo test --lib parser::stmt::tests::block_value_semantics` runs from the workspace root and resolves into `vertex_stage0` automatically (matches how `semicolon_significance` was added — it's also reachable from the root since `vertex_stage0` is the only workspace member).
- "Otherwise unit" in the spec means `tail = None`. The block AST (`vertex_stage0/src/ast/expr.rs:278-283`) has no explicit unit flag — absence of a tail is the unit signal — so the test asserts on `tail.is_none()` rather than constructing a `()` expression.
- The five cases above (empty, tail-only, semi-only, stmt+tail, two stmts both semi'd) exhaustively pin down the spec's wording. I do not add a sixth case for `{ expr expr }` (see Risks).
- Test setup uses the existing `int_tok` / `tok` helpers. No new helper is needed beyond `as_block`.
- Adding `use crate::ast::expr::{Block, Expr};` inside the test module is sufficient — the production module already imports `Expr` at the top of the file. Tests' `use super::*;` also re-pulls `Stmt`.
- This item produces a new test only; the `parse_block` body and `Stmt::Expr` shape both already match the spec, so a single coherent commit can land this test without touching production code.

## Blockers
Blockers: none

## Summary
Adds a `parser::stmt::tests::block_value_semantics` unit test that pins the existing `parse_block` semantics — tail expression on no trailing `;`, unit (no tail) otherwise — across empty, tail-only, semi-only, stmt+tail, and two-semi'd-stmts cases.

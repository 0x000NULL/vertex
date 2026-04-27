# Plan: parse-block-expressions

## Goal
Replace the literal-only `parse_block_stub` with a real `parse_block` that parses `{ stmts; tail_expr_optional }`, distinguishing tail-typed blocks (last expr without `;`) from unit-typed blocks (last expr with `;` or empty).

## Steps
1. In `src/parser/expr.rs`, add a `pub fn parse_block(&mut self) -> Result<Expr, CompileError>` that:
   - Expects `{`, capturing `lbrace_span`.
   - Loops while `peek() != RBrace && peek() != Eof`:
     - Calls `self.parse_expr()` to get an expression.
     - If next token is `Semi`: bump it, push `Stmt::Expr { expr, has_semi: true }`.
     - Else if next token is `RBrace`: this is the tail; record `tail = Some(Box::new(expr))` and break.
     - Else: push `Stmt::Expr { expr, has_semi: false }` and continue (covers an expression-without-semi followed by another stmt, deferred semantics; the dedicated `block-trailing-expression-as-value-semantics` item will refine).
   - Expects `}`, capturing `rbrace_span`.
   - Returns `Expr::Block(Block { id, span: lbrace_span.merge(&rbrace_span), stmts, tail })`.
2. Delete `parse_block_stub` and rewire its sole caller in `parse_closure` (line ~270) to call `parse_block` instead. Keep the closure's `LBrace`-branch decision logic unchanged.
3. Add an `LBrace` arm in `parse_primary_for_paren` (line ~628) that delegates to `parse_block`, so a `{ ... }` is now a valid primary expression head (needed for the test, and unblocks `if`/`while`/`loop` body parsing in later items).
4. Add unit test `block_trailing_expr` in the `tests` mod of `src/parser/expr.rs` covering at minimum:
   - `{}` → `Block { stmts: [], tail: None }` (unit).
   - `{ 1i32 }` → `tail = Some(IntLit)`, `stmts.len() == 0` (block-typed).
   - `{ 1i32; }` → `stmts = [Stmt::Expr { has_semi: true }]`, `tail = None` (unit).
   - `{ 1i32; 2i32 }` → one `Stmt::Expr { has_semi: true }`, `tail = Some(IntLit(2))`.
   - `{ 1i32; 2i32; }` → two `Stmt::Expr { has_semi: true }`, `tail = None`.
   - Missing-`}`: `{ 1i32` → `Err`.
5. Confirm `cargo build` and `cargo test --lib parser::expr::tests::block_trailing_expr` both pass; closure tests continue to pass after the rewire.

## Files
- `vertex_stage0/src/parser/expr.rs` -- add `parse_block`, delete `parse_block_stub`, update `parse_closure` call site, add `LBrace` arm to `parse_primary_for_paren`, add `block_trailing_expr` test.

## Risks
- Stmt enum still only has `Let`, `Expr { has_semi }`, `Item` — `parse_let` and item parsing aren't wired in yet, so a block containing `let` or a nested item will currently fail when `parse_expr` rejects those head tokens. Acceptable: the dedicated items `parse-let-statements` and `parse-item-statements-nested-fn-struct-inside-a-block` extend this same loop later.
- Recovery: `parse_block` propagates errors via `?` rather than using `recover_to_sync` inside the block. The dedicated `insert-placeholder-expr-error-nodeid-span-and-continue` item will retrofit recovery — out of scope here.
- `parse_primary_for_paren` is named for a transitional role; adding `LBrace` to it is a minor scope creep, but matches the same pattern already taken for `LBracket` (array literal) and is necessary to make blocks reachable from `parse_expr`.
- The `{ expr expr }` case (two exprs with no `;` between them) is treated by step 1 as `Stmt::Expr { has_semi: false }` followed by the second as tail. The block-trailing-expression item refines this; for now no test asserts that shape.

## Prereqs
Prereqs: none

(`Stmt`, `Block`, and `CompileError` already exist in `src/ast/stmt.rs`, `src/ast/expr.rs`, and `src/error/mod.rs` respectively; the listed pending `define-*` items are bookkeeping for later refinements but not blockers for this work.)

## Verify
```
cargo test --lib -p vertex_stage0 parser::expr::tests::block_trailing_expr
cargo test --lib -p vertex_stage0 parser::expr::tests::closure_forms
cargo build -p vertex_stage0
```

## Assumptions
- The verify test path `parser::expr::tests::block_trailing_expr` keeps the test inside the existing `mod tests` in `src/parser/expr.rs` (consistent with `closure_forms`, `array_literal_and_repeat`, etc.). I will not introduce a new `parser::stmt::tests` module — that belongs to later items.
- `parse_block` should be `pub` on `Parser` so the upcoming `parse_if`/`parse_loop`/`parse_while`/`parse_for`/`parse_match` items (which require a block body) can call it without re-implementing.
- `parse_block` continues to return `Expr` (as the AST defines `Block` as an `Expr` variant), not a separate `Block` value — matches how `parse_block_stub` works today and how `parse_closure` uses it.
- Test bodies use literal-only expressions (`IntLit`) since `parse_primary_for_paren` does not yet accept identifiers/paths; that is sufficient for the spec's three sub-steps.
- The Stmt::Expr variant currently has no `id` or `span` fields — block stmts therefore don't get their own NodeId yet; tail and stmt expressions retain their inner `Expr`'s id/span.
- Hard-error on unexpected EOF inside a block (propagate the `expect(&RBrace)` error) — error-recovery hooks come in later items.
- The `Pipe` head check at the top of `parse_expr` means `parse_block`'s first inner-expr call will not mis-trigger closure parsing for a stray `|`; nothing extra is needed.

## Blockers
Blockers: none

## Summary
Implements real block-expression parsing in `parse_block`, retires `parse_block_stub`, exposes `{...}` as a primary expression head, and adds a `block_trailing_expr` unit test covering empty / tail-only / stmt-only / stmt+tail / two-stmt / missing-`}` cases.

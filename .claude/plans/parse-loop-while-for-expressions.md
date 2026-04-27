# Plan: parse-loop-while-for-expressions

## Goal
Add `parse_loop`, `parse_while`, and `parse_for` to the expression parser so `loop { body }`, `while cond { body }`, and `for pat in iter { body }` produce `Expr::Loop` / `Expr::While` / `Expr::For` nodes, with one bundled unit test `loop_while_for`.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, extend the `crate::ast::expr::{...}` import list to bring in `For`, `Loop`, `Pat`, `While` (the `Loop`/`While`/`For` structs and the `Pat` placeholder enum).
2. Add `fn parse_loop(&mut self) -> Result<Expr, CompileError>` modeled on `parse_if` (line 307): expect `TokenKind::Loop`, capture its span as start, call `self.parse_block()?` for the body, merge spans, allocate a `NodeId`, and return `Expr::Loop(Loop { id, span, body: Box::new(body) })`.
3. Add `fn parse_while(&mut self) -> Result<Expr, CompileError>`: expect `TokenKind::While`, parse the cond via `self.parse_expr()?` (matches how `parse_if` reads its cond — safe because struct literals aren't implemented yet, so `{` will not be eaten by the cond parser), parse the body via `self.parse_block()?`, merge spans, return `Expr::While(While { id, span, cond: Box::new(cond), body: Box::new(body) })`.
4. Add `fn parse_for(&mut self) -> Result<Expr, CompileError>`: expect `TokenKind::For`, parse a stub pattern by consuming a single `TokenKind::Ident(_)` token (mirroring the closure-param stub at lines 287-305 — produces `Pat::Placeholder`; richer patterns land in `parse-ident-patterns-mut-sub-binding` etc.), `expect(&TokenKind::In)`, parse the iterator with `self.parse_expr()?`, parse the body with `self.parse_block()?`, merge spans, return `Expr::For(For { id, span, pat: Pat::Placeholder, iter: Box::new(iter), body: Box::new(body) })`.
5. Extend the head dispatch in `parse_primary_for_paren` (lines 683-695) with arms `TokenKind::Loop => self.parse_loop()`, `TokenKind::While => self.parse_while()`, `TokenKind::For => self.parse_for()` so these forms are valid primary expressions (and so postfix/binary chains like `loop { ... }.foo()` work consistently with how `if` and `{` are wired).
6. Add a `#[test] fn loop_while_for()` in the `tests` module covering: (a) `loop { 1i32 }` → `Expr::Loop` whose body is a `Block` with tail `IntLit(1)`; (b) `while true { 1i32 }` → `Expr::While` with `BoolLit(true)` cond and `Block` body; (c) `for x in 1i32..10i32 { 2i32 }` → `Expr::For` with `Pat::Placeholder`, `Range` iter, `Block` body; (d) negative `for x 1i32 { 2i32 }` (missing `in`) → `Err` with `ErrorCode::E0100`; (e) negative `while true 1i32` (non-block body) → `Err` with `ErrorCode::E0100`.

## Files
- `vertex_stage0/src/parser/expr.rs` — add `Loop`, `While`, `For`, `Pat` to the AST import list; add three new private parse functions; add three head arms in `parse_primary_for_paren`; add a `loop_while_for` unit test in the existing `tests` module.

## Risks
- `while cond { body }` and `for pat in iter { body }` reuse `parse_expr` for the cond/iter, which today parses ranges, binary ops, and blocks but not struct literals. Once struct-literal heads land, `while p { ... }` could ambiguously parse the brace as a struct-literal body — out of scope for this item, but a follow-up will need a "no-struct-literal" expression context. Documented as TODO in the new functions.
- The `for` pattern slot only accepts a single bare identifier as a stub (matching the existing closure-param stub). Real pattern parsing (`mut x`, tuple patterns, ref patterns) arrives in later items; the stub is intentionally narrow so tests are deterministic.
- `Loop`/`While`/`For` head tokens are not yet listed in `range_rhs_starts_here`, so `a.. loop { ... }` would fail to parse as a range with RHS. This matches the existing TODO comment above `range_rhs_starts_here` (lines 714-716) and is deferred to a future expansion of that allow-list — not part of this item.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::expr::tests::loop_while_for
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- `parse_loop`/`parse_while`/`parse_for` are private (`fn`, not `pub fn`) and reached only through `parse_primary_for_paren`, matching the existing convention for `parse_if`, `parse_block`, `parse_closure`, `parse_array_literal`.
- The `for` pattern slot is intentionally limited to a single bare `Ident` token producing `Pat::Placeholder`; `mut`-bindings, tuple patterns, etc. arrive in the later `parse-*-patterns` items. This mirrors how closure params already stub their pattern (`parse_closure_param`).
- `parse_expr` (not `parse_binary` or `parse_range` directly) is the right entry point for `while` cond and `for` iter — same choice `parse_if` makes for its cond.
- The body of all three forms must be a `{ ... }` block — `parse_block` enforces `expect(LBrace)`, so a non-block body produces an `E0100` "expected `{`" error naturally.
- The new unit test lives in the existing `mod tests` block at the bottom of `parser/expr.rs` (alongside `if_else_chain`, `block_trailing_expr`), so the verify path `parser::expr::tests::loop_while_for` resolves.
- The `--manifest-path vertex_stage0/Cargo.toml` flag is needed because `cargo test` is run from the repo root `C:\Users\Ethan\vertex` where the workspace member lives in the `vertex_stage0/` subdirectory.
- `Expr::Loop`/`While`/`For` already exist in `ast/expr.rs` (lines 295-320) with the exact field shapes used here, so no AST changes are required.

## Blockers
Blockers: none

## Summary
Wires `loop`/`while`/`for` keyword heads into the expression parser as `Expr::Loop`/`While`/`For`, gated by a single bundled `loop_while_for` unit test.

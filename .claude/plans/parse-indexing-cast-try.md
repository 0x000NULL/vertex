# Plan: parse-indexing-cast-try

## Goal
Extend the existing expression parser with three forms — `x[i]` (indexing), `x as T` (cast), and `x?` (try) — by adding `[…]`/`?` to the existing postfix loop and inserting a new `parse_cast` layer between `parse_binary` and `parse_unary`, plus a unit test named `index_cast_try`.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, extend `parse_postfix`'s loop with two new branches:
   - On `TokenKind::LBracket`: bump `[`, call `parse_expr()` for the index, `expect(&TokenKind::RBracket)` for `]`, build `Expr::Index(Index { id, span, receiver: Box::new(expr), idx: Box::new(idx) })`.
   - On `TokenKind::Question`: bump `?`, build `Expr::Try(Try { id, span, expr: Box::new(expr) })`.
   These chain naturally with the existing `()`/`.`/`.0` arms.
2. Add a new method `parse_cast(&mut self) -> Result<Expr, CompileError>` that:
   - Calls `parse_unary()` to get the LHS.
   - Loops while `peek()` matches `TokenKind::Ident(s)` with `s == "as"`. For each iteration: bump the `as` ident, then consume one identifier-like token (`Ident` or `SelfUpper`) as a temporary cast-type stub (returns `CastTy::Placeholder`; mirrors the `parse_primary_for_paren` stub pattern, with a `// TODO: replace when type parser lands` comment). Build `Expr::Cast(Cast { id, span, expr: Box::new(lhs), ty: Box::new(CastTy::Placeholder) })`.
3. In `parse_binary` (line 152 lhs and line 185 recursive rhs), replace the two `self.parse_unary()` calls with `self.parse_cast()` so `as` sits between unary and the lowest pratt level (matches Rust's `unary > as > * / %` precedence).
4. Add `Cast`, `Index`, `Try`, `CastTy` to the `use crate::ast::expr::{...}` import at the top of `expr.rs`.
5. Add unit test `index_cast_try` in `parser::expr::tests` covering:
   - `42[1i32]` → `Expr::Index { receiver=IntLit(42), idx=IntLit(1) }`, `pos == 4`.
   - `42?` → `Expr::Try { expr=IntLit(42) }`, `pos == 2`.
   - `42 as i32` (via `parse_expr`) → `Expr::Cast { expr=IntLit(42), ty=Placeholder }`, `pos == 3`.
   - `- x as i32` → `Cast(Unary(Neg, …), Placeholder)` (i.e. `(-x) as T`, not `-(x as T)`), confirming the cast-vs-unary precedence.
   - `42[1i32]?` → `Try(Index(IntLit, IntLit))` to verify chaining.
   - `42[1` (missing `]`) → `Err` with `ErrorCode::E0100`.

## Files
- `vertex_stage0/src/parser/expr.rs` — add `parse_cast`; add `LBracket`/`Question` arms to `parse_postfix`; swap two `parse_unary()` calls in `parse_binary` to `parse_cast()`; extend the `ast::expr` import; add the `index_cast_try` test.

## Risks
- The `as` keyword is not in `TokenKind` (lexer keyword table in `scan.rs:585-617` confirms). Parser must string-compare `Ident("as")`. If a later item adds `TokenKind::As`, this match guard must be updated.
- The cast-type stub only handles `Ident`/`SelfUpper` RHS; complex types (`&i32`, `[i32; N]`, `Vec<u8>`) won't parse until the type-parser items land. The verify only needs `i32`-style cases.
- Inserting a new `parse_cast` layer must not regress existing tests (`operator_precedence`, `comparison_non_associative_rejected`, etc.); `parse_cast` is a no-op when the next token isn't `Ident("as")`, so it should be transparent.
- `parse_postfix` already chains `()`, `.field`, `.method()`, `.0`; mixing in `[]` and `?` keeps left-associativity.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::expr::tests::index_cast_try
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- `as` tokenizes as `TokenKind::Ident("as")` (confirmed: `scan.rs:585-617` keyword table excludes it).
- `CastTy::Placeholder` is the correct shape to emit; a real cast-type parser is deferred. The stub may consume any single `Ident` or `SelfUpper` token.
- Indexing uses full `parse_expr()` inside `[...]` (so `x[a+b]` works).
- `?` is a single-token postfix.
- Precedence: `?` binds tighter than unary (so `-x?` = `-(x?)`), `as` binds looser than unary (so `-x as T` = `(-x) as T`) — achieved by placing `?`/`[]` in `parse_postfix` and `as` in `parse_cast` above `parse_unary`.
- The literal verify command in the todo lacks `--manifest-path`; using the explicit form because the workspace root sits one directory above the crate (`vertex_stage0/`), and the runner shells out via `bash -c` from the repo root.
- No new error code needed — reuse `ErrorCode::E0100` for missing-`]` and missing-cast-type-RHS, matching the existing convention in `parse_postfix`.

## Blockers
Blockers: none

## Summary
Adds `parse_cast` (loops on `Ident("as")` after `parse_unary`) plus `LBracket`/`Question` arms in `parse_postfix`, then lifts `parse_binary`'s sub-calls to `parse_cast`, with a unit test `index_cast_try` proving each form and the unary-vs-cast precedence.

Now I have enough context to draft the plan.

# Plan: parse-expression-statements-with-semicolon-significance

## Goal
Extract expression-statement parsing into a new `parser::stmt` module so a trailing `;` yields `Stmt::Expr { expr, has_semi: true }` and an absent `;` (when terminator follows) yields `Stmt::Expr { expr, has_semi: false }`, locked in by a `semicolon_significance` unit test.

## Steps
1. Create `vertex_stage0/src/parser/stmt.rs` with a `pub fn parse_expr_stmt(&mut self) -> Result<Stmt, CompileError>` impl on `Parser`. Its behavior: parse one `Expr` via `parse_expr`, then if `peek()` is `Semi` bump it and return `Stmt::Expr { expr, has_semi: true }`; otherwise return `Stmt::Expr { expr, has_semi: false }`. Decision about whether the no-semi branch becomes a tail or a stmt remains in the caller (`parse_block`); this routine just classifies based on what it consumed.
2. Wire `pub mod stmt;` into `vertex_stage0/src/parser/mod.rs` (add the module declaration alongside `pub mod expr;`).
3. Refactor `parse_block` (currently in `parser/expr.rs` ~line 528) so the inline `match self.peek() { Semi => ... | RBrace => tail | _ => no-semi stmt }` calls into the new `parse_expr_stmt` for the `Semi` and `_` (no-terminator) branches; the `RBrace` branch still steals the just-parsed expr as `tail`. To preserve current tail-vs-stmt semantics without re-parsing, keep the `parse_expr` call in `parse_block` and set `has_semi` based on whether `Semi` was eaten — i.e., factor out a small `expr_stmt_from(expr) -> Stmt` helper rather than restructuring `parse_block`. Alternative: have `parse_expr_stmt` itself look ahead to `RBrace` and return `Stmt` only when it isn't a tail; the caller checks. Pick the helper approach to minimize churn.
4. Add `#[cfg(test)] mod tests` in `parser/stmt.rs` containing a `#[test] fn semicolon_significance()` that constructs `Parser` directly with token vectors `[1i32, ;, EOF]` and `[1i32, EOF]`, calls `parse_expr_stmt`, and asserts `Stmt::Expr { has_semi: true/false }` plus the inner `IntLit` value. Mirror the `int_tok`/`int_value` helpers from `parser/expr.rs` (or copy minimal versions) so the test is self-contained.
5. Confirm nothing else (e.g. integration smoke, existing `block_trailing_expr` test) regressed; the existing `Stmt::Expr { has_semi }` assertions in `parser/expr.rs` continue to pass because the on-the-wire shape is unchanged.

## Files
- `vertex_stage0/src/parser/stmt.rs` -- NEW: `parse_expr_stmt` impl + `tests::semicolon_significance`.
- `vertex_stage0/src/parser/mod.rs` -- add `pub mod stmt;` next to `pub mod expr;`.
- `vertex_stage0/src/parser/expr.rs` -- replace inline `Stmt::Expr { ... has_semi: true/false }` constructors in `parse_block` (~lines 537-554) with calls into the new `parse_expr_stmt` helper (or a tiny `expr_stmt_from(expr, has_semi)` helper) so the canonical construction lives in `parser::stmt`.

## Risks
- Refactoring `parse_block` could subtly change tail-vs-stmt classification if the no-semi branch is moved into `parse_expr_stmt` without preserving the `RBrace` lookahead in the caller. Mitigation: keep the `RBrace` check in `parse_block` and only delegate the `Semi`/`else` classification.
- The verify path is `parser::stmt::tests::semicolon_significance`. This requires the test to live in `parser/stmt.rs` (not `parser/expr.rs`). Make sure the file is wired via `pub mod stmt;` so cargo treats it as `parser::stmt`.
- `Stmt::Expr` is currently struct-style (`{ expr, has_semi }`), not tuple-style as the todo prose suggests. Test must use struct-pattern destructuring; do not change the variant shape.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::stmt::tests::semicolon_significance
cargo build --manifest-path vertex_stage0/Cargo.toml
test -f vertex_stage0/src/parser/stmt.rs
```

## Assumptions
- The crate root is `vertex_stage0/` (the only Cargo.toml in the tree based on file layout). Verify commands therefore use `--manifest-path vertex_stage0/Cargo.toml`. If a workspace root exists, cargo will still resolve `--lib` correctly because `vertex_stage0` is the only library crate.
- `Stmt::Expr` keeps its current struct-variant shape (`{ expr, has_semi }`); the todo's `Stmt::Expr(expr, true)` is descriptive shorthand, not a request to restructure the enum.
- The `parse_expr_stmt` helper does not consume a trailing terminator beyond `;` (no eating of `RBrace`/`Eof`); those stay in `parse_block`'s control. This matches the existing semantics (block tail expr is captured before `RBrace`).
- The new module is plain `parser/stmt.rs` (not a `parser/stmt/mod.rs` directory). Future stmt-related code (let-stmts, item-stmts) can grow into the same file.
- Test uses raw `Parser::new(vec![...])` with hand-built tokens, matching the existing test style in `parser/expr.rs` (no fixture/snapshot helper assumed).
- No other pending items (e.g. `parse-let-statements`) need to land first; expression statements are independent of let/item statement forms.
- The existing `block_trailing_expr` tests in `parser/expr.rs` continue to validate the same `Stmt::Expr { has_semi }` shape and will keep passing — this task is additive (extracting helper + new dedicated test).

## Blockers
Blockers: none

## Summary
Extract expression-statement classification into `parser::stmt::parse_expr_stmt`, preserving `has_semi` semantics, and pin behavior with a `semicolon_significance` unit test.

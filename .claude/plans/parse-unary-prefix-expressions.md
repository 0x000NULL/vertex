# Plan: parse-unary-prefix-expressions

## Goal
Add `Parser::parse_unary` that parses the five spec-listed prefix operators (`-`, `not`, `*`, `&`, `&mut`) into `Expr::Unary`, layered on top of the existing literal-only primary stub.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, add `pub fn parse_unary(&mut self) -> Result<Expr, CompileError>` on `impl Parser`:
   - Capture `op_span = self.tokens[self.pos].span` (or `current_span`-equivalent) before consuming.
   - Match `self.peek()`:
     - `TokenKind::Minus` → bump, op = `UnaryOp::Neg`.
     - `TokenKind::Not` → bump, op = `UnaryOp::Not`.
     - `TokenKind::Star` → bump, op = `UnaryOp::Deref`.
     - `TokenKind::Amp` → bump; if `self.peek()` is now `TokenKind::Mut`, bump again and use `UnaryOp::RefMut`, else `UnaryOp::Ref`.
     - Otherwise: tail-call `self.parse_primary_for_paren()` and return its result (no `Unary` node).
   - Recurse: `let operand = self.parse_unary()?;` so chained prefixes like `- - 7` and `& * x` work right-associatively.
   - Build `Expr::Unary(Unary { id, span: op_span.merge(&operand.span()), op, operand: Box::new(operand) })`.
2. Do NOT parse `~` (`TokenKind::Tilde`) as a prefix — spec lists only the five ops above; `UnaryOp::BitNot` stays as a defined-but-unused variant.
3. Do NOT add a new `parse_primary` stub; reuse the existing `parse_primary_for_paren` as the non-prefix fallback (matches Q1 default).
4. Add a `#[test] fn unary_prefix()` in the existing `tests` module of `parser/expr.rs` covering:
   - `-7i32` → `Unary { op: Neg, operand: IntLit(7) }`, `pos == 2`.
   - `not true` → `Unary { op: Not, operand: BoolLit(true) }`.
   - `*1i32` → `Unary { op: Deref, operand: IntLit(1) }`.
   - `&1i32` → `Unary { op: Ref, operand: IntLit(1) }`, `pos == 2`.
   - `&mut 1i32` → `Unary { op: RefMut, operand: IntLit(1) }`, `pos == 3`.
   - Chained: `- - 7i32` → `Unary(Neg, Unary(Neg, IntLit(7)))`, depth 2, `pos == 3`.
   - Chained mixed: `& * 1i32` → `Unary(Ref, Unary(Deref, IntLit(1)))`.
   - Pass-through: when peek is a literal (e.g. `42i32`), `parse_unary` returns `Expr::IntLit` directly (no `Unary` wrapper) and consumes one token.
   - Wrong head: token stream `[Plus, Eof]` → `parse_unary().is_err()` and `pos == 0` (delegated error from primary stub).
   - Each successful case asserts `p.errors.is_empty()`.
5. Confirm the file still compiles cleanly with `cargo check --lib` and that the new test name matches the verify line exactly: `parser::expr::tests::unary_prefix`.

## Files
- `vertex_stage0/src/parser/expr.rs` — add `parse_unary` method on `impl Parser` and a `unary_prefix` unit test in the existing `tests` module. No other files change; `Unary`/`UnaryOp` already exist in `ast::expr`, and `Minus`/`Not`/`Star`/`Amp`/`Mut` already exist in `lexer::token`.

## Risks
- `&mut` ambiguity: must `bump()` `Amp` first, then check `peek() == Mut` to decide `Ref` vs `RefMut`. Forgetting to bump the `Mut` token would cause downstream reparse failures.
- Span merge: `Span::merge` exists and is symmetric — but if `op_span` is captured *after* `bump()`, it would be the operand's span; capture it before bumping.
- Recursion vs delegation: recursing into `parse_unary` (rather than `parse_primary_for_paren`) for the operand is required so `--7` parses; must not accidentally call the primary stub directly.
- Test naming: the verify line targets `parser::expr::tests::unary_prefix` literally. A typo (`unary_prefixes`, `unary_prefix_expr`) breaks verification even if the code is correct.
- `parse_primary_for_paren` is currently `fn` (private) — `parse_unary` lives in the same `impl Parser` block in the same file, so the existing visibility is fine; nothing to change.
- Stack depth on pathological `------...x` is bounded by token count; no special handling needed for stage 0.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::expr::tests::unary_prefix
```

## Assumptions
- "Accept default" on Q1 means: do not introduce a new `parse_primary` symbol; the existing literal-only `parse_primary_for_paren` stub is the fallback that `parse_unary` delegates to when no prefix operator is present. A real `parse_primary` arrives with the Pratt driver item.
- "Accept default" on Q2 means: do **not** parse `~` as a prefix here, because the spec's `prefix_op` rule lists only `- not & &mut *`. `UnaryOp::BitNot` remains an unused AST variant (consistent with `#[allow(dead_code)]` on the enum).
- `parse_unary` recurses into itself for the operand so chained prefixes parse right-associatively (`- - 7`, `& * x`). This matches the spec's BNF `prefix_expr = prefix_op expression`.
- The `Unary` node's span is `op_span.merge(&operand.span())`; node id is allocated via the existing `self.new_node_id()` after parsing the operand (id-allocation order is not observable to current tests).
- We do **not** change `parse_paren_or_tuple` to thread unary into tuple elements — that's the Pratt-parser item's job (`pratt-parser-for-binary-operators` or a successor). Keeping `parse_primary_for_paren` calls as-is.
- `parse_unary` is added as `pub` so the upcoming Pratt driver and statement parsers can call it without re-exporting.
- The `unary_prefix` test uses the same `tok(...)` helper and `Parser::new` already used by `literal_expressions` and `paren_tuple_unit`; no test fixture changes.
- Running the test through the workspace `Cargo.toml` requires `--manifest-path vertex_stage0/Cargo.toml` because the crate lives in a subdirectory; confirmed by the existing `vertex_stage0/Cargo.toml` layout.

## Blockers
Blockers: none

## Summary
Adds a right-recursive `Parser::parse_unary` covering `-`, `not`, `*`, `&`, `&mut` plus a `unary_prefix` unit test, wired on top of the existing literal primary stub.

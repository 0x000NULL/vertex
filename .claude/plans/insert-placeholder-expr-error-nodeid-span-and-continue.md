# Plan: insert-placeholder-expr-error-nodeid-span-and-continue

## Goal
Add an `Expr::Error` placeholder variant and a recovery path inside `parse_block` so a failed expression parse pushes a `CompileError`, returns the placeholder, and resyncs to the next statement boundary instead of aborting the whole block.

## Steps
1. In `vertex_stage0/src/ast/expr.rs`:
   - Add a new variant `Error(NodeId, Span)` to the `pub enum Expr` (tuple form, matching the todo title verbatim).
   - Extend `Expr::id()` with `Expr::Error(id, _) => *id` and `Expr::span()` with `Expr::Error(_, span) => *span`.
2. In `vertex_stage0/src/parser/expr.rs`:
   - Add a helper `pub(crate) fn parse_expr_or_recover(&mut self) -> Expr` that calls `self.parse_expr()`. On `Err(e)`: capture the span (use `e.span` so the error variant points at the offending token), call `self.errors.push(e)`, call `self.recover_to_sync()`, mint a fresh `NodeId`, and return `Expr::Error(id, span)`.
   - Modify `parse_block` (at `expr.rs:528`) so the inner loop calls `parse_expr_or_recover()` instead of `self.parse_expr()?`. Keep the existing post-expr branch (Semi → push as `expr_stmt_from(.., true)`; RBrace → set `tail`; otherwise → push as `expr_stmt_from(.., false)`). Note: `recover_to_sync` already eats a trailing `;`, so after recovery the loop falls into the "no trailing semi" branch — that's fine for the placeholder (it becomes a `Stmt::Expr { has_semi: false }` containing `Expr::Error`). Do not also branch on `Semi` after a recovered expr because the sync ate it.
   - Leave `parse_expr`, the deeper helpers (`parse_binary`, `parse_postfix`, `parse_primary_for_paren`, etc.), and the public `parse_expr_stmt` returning `Result` — recovery is introduced only at the block boundary so other call sites keep their current short-circuit behavior. (Other items like `end-to-end-recovery-test` will exercise wider recovery later.)
3. In `vertex_stage0/src/parser/mod.rs`, add a `#[test] fn error_node_recovery()` inside the existing `mod tests` block that:
   - Builds tokens for `{ @ ; 1i32 }` analogue: `LBrace`, an unrecognized expression head (e.g., `TokenKind::Comma` or `TokenKind::RParen` — anything `parse_primary_for_paren` rejects), `Semi`, `IntLiteral(1, IntSuffix::I32)`, `RBrace`, `Eof`.
   - Calls `p.parse_block()`, asserts `Ok`, asserts the block has 1 stmt whose expr matches `Expr::Error(_, _)` (via `matches!`), and the tail is `Some(IntLit { value: 1, .. })`.
   - Asserts `p.errors.len() == 1` and the lone error has `code == ErrorCode::E0100` and `kind == ErrorKind::Syntax`.
   - Asserts `p.peek() == &TokenKind::Eof` (block consumed cleanly).

## Files
- `vertex_stage0/src/ast/expr.rs` — add `Expr::Error(NodeId, Span)` variant; add arms to `Expr::id()` and `Expr::span()`.
- `vertex_stage0/src/parser/expr.rs` — add `parse_expr_or_recover`; switch `parse_block`'s inner expression call to it; import `NodeId`/`Span` if not already in scope (they are reachable through `crate::ast::NodeId` and `crate::span::Span`).
- `vertex_stage0/src/parser/mod.rs` — add `error_node_recovery` test in the existing `mod tests`.

## Risks
- `Expr::id()` / `Expr::span()` are exhaustive `match`es; missing the new arm becomes a compile error elsewhere — easy to spot but must not be skipped.
- Existing `block_value_semantics` test in `parser/stmt.rs` exercises `parse_block`'s happy paths; we must not change that behavior. The helper only intervenes when `parse_expr` returns `Err`, so successful paths are unchanged.
- `recover_to_sync` consumes `Semi` when present. That means after a recovered error the loop doesn't see the `Semi` again — the placeholder is pushed via the no-semi branch with `has_semi: false`. Fine for now, but worth noting in case a future "missing semi after stmt" diagnostic is added; tracked implicitly by the `parse-expression-statements-with-semicolon-significance` item.
- `recover_to_sync` syncs on `RBrace`/`Eof` *without* consuming them, so the surrounding `while !RBrace|Eof` loop terminates correctly and the trailing `expect(RBrace)` still fires.
- `Expr::Error` carries no inner expression, so other passes (resolve/typecheck) must learn to treat it as a poison value. None of those passes are wired up yet (Cargo dep tree shows only the parser/AST in active use), so no immediate cross-cutting fallout — but the `document-phase-1-5-boundary` item should mention it.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::tests::error_node_recovery
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::stmt::tests
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The todo title "`Expr::Error(NodeId, Span)`" is taken literally: a tuple variant on the `Expr` enum, not a `struct ErrorExpr { id, span }` payload like the other variants. Going with the literal spec because it's explicit and the `id()`/`span()` arms can read tuple fields just as easily.
- `parse-let-statements`, `parse-item-statements-...` (let/item statement heads in blocks) aren't yet plumbed into `parse_block` — the block currently only contains expression-statements. So the recovery hook only needs to live on the expression branch in `parse_block`. Future items adding `let`/item parsing will add their own recovery branches.
- The verify path `parser::tests::error_node_recovery` refers to the `mod tests` already present in `src/parser/mod.rs`, not a new test file.
- It is acceptable for the recovered statement to land as `Stmt::Expr { expr: Expr::Error(..), has_semi: false }`, because `recover_to_sync` already swallows the `;`. The test asserts on that shape and on the count of accumulated errors rather than on `has_semi`.
- `ErrorAccumulator::push` deduplicates by `(code, file_id, span.start)`; the test uses a single error so dedup is not exercised.
- `Expr::Error` is `#[allow(dead_code)]`-compatible because the surrounding enum already carries that attribute; no extra suppressions needed.
- No public-API shim (e.g., constructor helper) is added — direct `Expr::Error(id, span)` construction is fine and keeps the surface minimal.

## Blockers
Blockers: none

## Summary
Adds an `Expr::Error(NodeId, Span)` placeholder and uses it at the block boundary so a single bad expression accumulates one diagnostic and recovers to the next statement instead of aborting the whole block.

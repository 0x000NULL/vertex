# Plan: end-to-end-recovery-test

## Goal
Add the `parser::tests::recovery_let_garbage_let_valid` unit test that drives the lexer+parser end-to-end on `let x = ; let y = 10;` and asserts exactly one diagnostic plus a valid `Stmt::Let` for `let y = 10`.

## Steps
1. In `vertex_stage0/src/parser/mod.rs`'s `mod tests`, add a small helper that runs `lexer::scan::Scanner::new(input, FileId(0))` until `Eof`, collecting `Token`s, and feeds them into `Parser::new(...)`. (Mirrors the `lex_eq!` driver in `lexer/test_util.rs`.)
2. Add `#[test] fn recovery_let_garbage_let_valid()` that:
   - Lexes the source string `let x = ; let y = 10;`.
   - Wraps the token stream in synthetic `LBrace` / `RBrace` (or appends them around the lexer output) so `parse_block` is the entry, since `parse_block` is the only existing block-of-stmts parsing entry. (See assumption.)
   - Calls `p.parse_block()` and unwraps the resulting `Expr::Block`.
3. Assert `p.errors.len() == 1` and that the single error is `ErrorCode::E0100` / `ErrorKind::Syntax`.
4. Assert the block contains a `Stmt::Let { pattern: Pattern(y), init: Some(Expr::IntLit { value: 10, .. }), .. }` for `let y = 10;` somewhere in `block.stmts`. Do not over-specify the shape of the recovered first statement (see assumption).
5. Assert the parser is positioned at `Eof` after the block close brace.

## Files
- `vertex_stage0/src/parser/mod.rs` — extend the existing `#[cfg(test)] mod tests` with the lex-helper and the new `recovery_let_garbage_let_valid` test. No production code changes in this plan.

## Risks
- Exact `Stmt::Let` field names / nested `Pattern` shape depend on `parse-let-statements`; the test must match whatever ident-pattern variant that item produces. The current `ast::stmt::Stmt::Let { pattern, ty, init, span, id }` is the assumed shape.
- `parse_block` currently dispatches every non-`RBrace` token through `parse_expr_or_recover`; it must learn to route a leading `Let` token to `parse_let` (or `parse_let_or_recover`). That wiring is the responsibility of the `parse-let-statements` prereq, not this plan.
- `is_sync_point` does not currently include `Let`. Recovery from `let x = ;` consumes the trailing `;` (already a sync point that `recover_to_sync` eats) and resumes on `let`, which the block loop sees on its next iteration — so this should work without adding `Let` to the sync set, provided the prereq emits the diagnostic from inside `parse_let` and lets the loop re-enter naturally.
- If `parse-let-statements` chooses to also accept `let x;` (no initializer) as legal, the diagnostic count for `let x = ;` could differ — the test pins "exactly one" and would surface that ambiguity.

## Prereqs
parse-let-statements

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::tests::recovery_let_garbage_let_valid
```

## Assumptions
- `parse-let-statements` (prereq) lands `Parser::parse_let` returning `Stmt::Let { pattern, ty, init, span, id }`, plus the dispatch in `parse_block` so a leading `Let` token routes there, plus internal recovery so a missing initializer pushes exactly one `E0100`/`Syntax` diagnostic and `recover_to_sync` consumes the trailing `;`.
- The test uses the real lexer (`lexer::scan::Scanner`) rather than hand-constructed tokens, since the item is described as "end-to-end". This is a deliberate departure from neighboring parser unit tests that build tokens directly.
- `parse_block` is the only block-of-stmts entry today, so the test wraps the source's tokens in synthetic `LBrace`/`RBrace` to invoke it. If `parse-let-statements` introduces a top-level `parse_stmts` (or similar) entry, the test should switch to that; until then, the wrap approach keeps this test self-contained.
- The test asserts `errors.len() == 1` and the *valid* `let y = 10` statement; it does not over-specify whether the recovered first statement is `Stmt::Let { init: None, .. }`, `Stmt::Let { init: Some(Expr::Error(..)), .. }`, or absent — that's the prereq's call.
- The integer literal `10` is matched on `Expr::IntLit { value: 10, .. }` (the `IntLit` variant already exists in `ast::expr`, used by sibling tests).

## Blockers

### Blocker: Recovered representation of `let x = ;`
- severity: cross-item
- affects: parse-let-statements, end-to-end-recovery-test
- question: After diagnosing the missing initializer, should the recovered first statement be `Stmt::Let { init: None }`, `Stmt::Let { init: Some(Expr::Error(..)) }`, or skipped entirely (no statement pushed)?
- default_assumption: Don't pin the shape. Test asserts `errors.len() == 1` plus presence of a valid `Stmt::Let` with `pattern==y` and `init==Some(IntLit(10))`, so any of those three recovery shapes passes.

### Blocker: Block-loop re-entry after `let` recovery
- severity: cross-item
- affects: parser::recover_to_sync, parse_block, parse-let-statements
- question: Does the parse_block loop re-enter cleanly on `Let` after `recover_to_sync` consumes the trailing `;`, or does `Let` need to be added to `is_sync_point` so recovery stops *at* `let` rather than past it?
- default_assumption: Current `recover_to_sync` stops at the `;` then consumes it, leaving `Let` as the next peek; the block loop sees `Let != RBrace/Eof` and dispatches again. No change to `is_sync_point` required from this plan; if the prereq disagrees, this test will fail loudly.

## Summary
Adds a parser-level end-to-end unit test proving a malformed `let` recovers with exactly one diagnostic and a valid AST for the following `let`.

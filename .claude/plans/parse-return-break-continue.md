# Plan: parse-return-break-continue

## Goal
Wire `return`, `break`, and `continue` keyword heads into the expression parser as `Expr::Return` / `Expr::Break` / `Expr::Continue`, with optional trailing values where the grammar allows, gated by a single bundled `return_break_continue` unit test.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, extend the AST import line at the top (currently `use crate::ast::expr::{ ... };`) to also bring in `Break, Continue, Return`.
2. Add three new `Parser` methods alongside `parse_if`/`parse_loop`/`parse_while`/`parse_for`/`parse_match`:
   - `parse_return(&mut self) -> Result<Expr, CompileError>`: `expect(&TokenKind::Return)`, capture `start_span`, then peek to decide whether a value follows. If the next token does NOT terminate the expression (i.e. is not one of `Semi | Comma | RParen | RBrace | RBracket | FatArrow | Eof`), parse a full `self.parse_expr()?` and use it as `Some(Box::new(_))`; otherwise `None`. Span = `start_span` merged with the value's span when present, else `start_span` alone. Emit `Expr::Return(Return { id, span, value })`.
   - `parse_break(&mut self) -> Result<Expr, CompileError>`: same shape as `parse_return`, but produces `Expr::Break(Break { id, span, label: None, value })`. Leave a `// TODO:` comment beside `label: None` noting that `'label` is not yet representable: the lexer (`vertex_stage0/src/lexer/token.rs`) has no lifetime/label token kind today (only `CharLiteral` for `'x'`), so label support is deferred until that token lands. This mirrors the existing closure-param/`for`-pattern stub strategy.
   - `parse_continue(&mut self) -> Result<Expr, CompileError>`: `expect(&TokenKind::Continue)`, capture `start_span`, never parse a trailing value. Emit `Expr::Continue(Continue { id, span: start_span, label: None })` with the same `// TODO:` label comment.
3. Add three head arms in `parse_primary_for_paren` (just below the `Match` arm):
   ```
   TokenKind::Return => self.parse_return(),
   TokenKind::Break => self.parse_break(),
   TokenKind::Continue => self.parse_continue(),
   ```
4. Add a `#[test] fn return_break_continue()` in the existing `mod tests` block in `vertex_stage0/src/parser/expr.rs`. It drives `parse_expr` (not the helpers directly, to also exercise the primary-head wiring) and covers:
   - `return` followed by `Eof` → `Expr::Return` with `value: None`, `pos == 1`.
   - `return 1i32` → `Expr::Return` with `value: Some(IntLit { 1 })`.
   - `return ;` (Semi terminator) → `Expr::Return` with `value: None`, parser stopped at the `Semi`.
   - `break` followed by `Eof` → `Expr::Break` with `value: None`, `label: None`.
   - `break 7i32` → `Expr::Break` with `value: Some(IntLit { 7 })`, `label: None`.
   - `continue` followed by `Eof` → `Expr::Continue` with `label: None`.
   - `continue ;` (Semi terminator) → `Expr::Continue`, parser stopped at the `Semi` (Continue must NOT consume a following expression).
   - Negative shape: `return + 1i32` (Plus is not a valid value head once the parser steps into `parse_expr`) propagates an `Err` from `parse_unary`/`parse_primary_for_paren`. The wider goal is just to lock in that `return` recurses through `parse_expr` and surfaces real errors, not to assert a particular message.
   - Block integration: parse a `{ return; }` block via `parse_block`/`parse_expr` and assert the block's `stmts` contains a single `Stmt::Expr { expr: Expr::Return(_), has_semi: true }` and `tail` is `None`. This verifies the new heads compose with `parse_block`.
5. Run `cargo fmt` and `cargo test --lib parser::expr::tests::return_break_continue` to confirm shape is locked in.

## Files
- `vertex_stage0/src/parser/expr.rs` — extend the AST import set with `Break, Continue, Return`; add `parse_return` / `parse_break` / `parse_continue` `impl Parser` methods; add three head arms in `parse_primary_for_paren`; add `#[test] fn return_break_continue` covering the cases above. No other source files change.

## Risks
- Optional-value detection for `return`/`break` uses a *terminator denylist* (`Semi | Comma | RParen | RBrace | RBracket | FatArrow | Eof`). If a token outside that set appears that isn't actually a valid expression head (e.g. `Else`), `parse_expr` will be called and return an error. That's acceptable, but it does mean the error message points at the failed sub-parse rather than at `return` itself. This matches how the rest of the parser already behaves (e.g. `if x else { ... }` would already misreport).
- `label: None` is a hard stub: any user program that writes `break 'outer` cannot be expressed yet because the lexer emits `Error("'")` for a bare `'outer`. Documented inline as a TODO; nothing else in the planned items list (`parse-loop-while-for-expressions` is already done, no `add-labeled-loops` item) blocks on it within phase 1.5.
- `return`/`break` greedily call `self.parse_expr()`, so `return a = b` parses as `Return(Assign(a, b))`, matching Rust. No assignment-vs-statement ambiguity is introduced because the keyword heads sit at primary level and the recursion goes through the full Pratt cascade.
- Putting `Return`/`Break`/`Continue` at the primary level means `1 + return 2` parses as `Add(1, Return(2))`. That's a wart shared with Rust ("never type"-typed expressions in operand position) and is correct for a syntax-only parser. Type checking will reject it later.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::expr::tests::return_break_continue
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate root is `vertex_stage0/` (verified: `vertex_stage0/Cargo.toml` exists, `src/parser/expr.rs` is the live parser file, sibling items like `parse_match`/`parse_loop` were added there). The `--manifest-path vertex_stage0/Cargo.toml` flag is used so the verify works regardless of which directory `bash -c` lands in.
- `Return`, `Break`, `Continue` AST structs (with `value: Option<Box<Expr>>`, `label: Option<String>` as appropriate) are already defined and re-exported by `crate::ast::expr` — confirmed in `vertex_stage0/src/ast/expr.rs:343-364` and the public `pub enum Expr { ..., Return(Return), Break(Break), Continue(Continue) }` at lines 396–398. No AST module changes are needed.
- `TokenKind::Return`, `TokenKind::Break`, `TokenKind::Continue` already exist as keyword tokens (confirmed in `vertex_stage0/src/lexer/token.rs:33,35,53`), so no lexer changes are needed.
- Label parsing is *intentionally* not implemented in this item. The lexer has no `Lifetime`/`Label` token (verified via grep; `'a'` always tokenizes as `CharLiteral`, and `'foo` produces `TokenKind::Error`). Adding such a token is out of scope for this item and is not in the pending-items list, so labels stay `None` with a TODO. This is the same stub strategy `parse_for` and `parse_closure` use for patterns.
- Optional-value detection uses a terminator denylist rather than an expression-start allowlist because the existing primary-head set is wide and growing (adding new heads later would otherwise require updating two places). The denylist `{Semi, Comma, RParen, RBrace, RBracket, FatArrow, Eof}` is the minimal set seen at the immediate call sites of expressions: statement terminators, list/tuple separators, all close-brackets, and the match-arm fat arrow body terminator (technically `,` covers most match-arm uses, but `FatArrow` is included defensively).
- The `return_break_continue` test name matches the verify command exactly (`parser::expr::tests::return_break_continue`).
- The test is added to the existing `#[cfg(test)] mod tests` block in `parser/expr.rs` (where `match_basic`, `loop_while_for`, `if_else_chain`, etc. live), not a new module.

## Blockers
Blockers: none

## Summary
Adds `parse_return`/`parse_break`/`parse_continue` (with terminator-aware optional-value handling and `label: None` stubs noted with TODOs since the lexer has no label token), wires their heads into `parse_primary_for_paren`, and locks in shape with a `return_break_continue` unit test covering value-present, value-absent, semi-terminated, and block-statement cases.

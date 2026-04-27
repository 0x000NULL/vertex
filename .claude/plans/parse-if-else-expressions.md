# Plan: parse-if-else-expressions

## Goal
Add an `if`/`else if`/`else` expression parser that produces `Expr::If` and requires every branch to be a brace-block.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, add a private helper `parse_if(&mut self) -> Result<Expr, CompileError>` on `impl Parser` that:
   - Records the start span from the current `if` token, then consumes it via `self.expect(&TokenKind::If)?`.
   - Parses the condition with `self.parse_expr()` (struct-literal ambiguity is not yet a concern because struct literals aren't recognized in `parse_primary_for_paren` yet; revisit when item `parse-struct-literal-expressions` lands).
   - Requires `{` next: calls `self.parse_block()?` (errors via the existing `expect(&LBrace)` if the user wrote a non-block branch like `if c 1`), capturing it as the `then` branch (an `Expr::Block`).
   - If the next token is `TokenKind::Else`, bumps it and:
     - If the next token is `TokenKind::If`, recursively calls `self.parse_if()` and uses the result as the `else_branch`.
     - Else if next is `TokenKind::LBrace`, calls `self.parse_block()?` as the `else_branch`.
     - Otherwise returns `Err(self.unexpected_token_error("`{` or `if`"))` — this enforces "non-block branches not allowed" and rejects `else 1`.
   - Builds `Expr::If(If { id, span: start.merge(&last_branch.span()), cond: Box::new(cond), then: Box::new(then), else_branch })`.
2. Add `If` to the imports at the top of `parser/expr.rs` from `crate::ast::expr`.
3. Wire `if` into the expression head: in `parse_primary_for_paren`, add a `TokenKind::If => self.parse_if(),` arm so `if` is recognized as a primary expression (chosen over `parse_expr` because `parse_postfix`'s `expr.method()` calls flow back through this dispatcher and an `if` block can also be a postfix receiver per Rust-style semantics; matches how `LBrace`/`LBracket` are wired).
4. Extend `range_rhs_starts_here` in `parser/expr.rs` to include `TokenKind::If` (the existing TODO comment in that function explicitly calls this out) so `0..if c {1} else {2}` parses cleanly. Leave the other "future heads" alone — they'll be added by their own items.
5. Add a `#[test] fn if_else_chain()` in the `mod tests` block in `parser/expr.rs` exercising:
   - Plain `if c { 1 }` → `Expr::If` with no else.
   - `if c { 1 } else { 2 }` → `Expr::If` with `else_branch = Some(Expr::Block(_))`.
   - `if c { 1 } else if c2 { 2 } else { 3 }` → outer `If` whose `else_branch` is another `Expr::If` whose own `else_branch` is `Expr::Block`.
   - Error: `if c 1` (non-block then) returns `Err`.
   - Error: `if c { 1 } else 2` (non-block else) returns `Err`.
   For the condition tokens, use a literal like `true`/`false` rather than identifiers so no path/ident-head plumbing is required. For block bodies, use a single integer literal as the trailing expression (the existing `parse_block` already handles tail-only blocks, per `block_trailing_expr`).

## Files
- `vertex_stage0/src/parser/expr.rs` — add `parse_if`, add `If` to imports, dispatch from `parse_primary_for_paren`, extend `range_rhs_starts_here`, add `if_else_chain` unit test.

## Risks
- Condition uses `self.parse_expr()`, which calls `parse_range`. A struct-literal RHS (e.g. `if Foo { x: 1 } { ... }`) would be ambiguous, but struct-literal parsing isn't a recognized head yet, so this can't actually fire. Note as an assumption to revisit when `parse-struct-literal-expressions` lands.
- Routing `if` through `parse_primary_for_paren` rather than directly from `parse_expr` means `if c {1} else {2}.foo` would parse as field access on the if-expression. That matches Rust semantics and is the desired behavior; flagged here in case the spec disagrees.
- The "block-only branch" rule depends on `parse_block` rejecting non-`{` heads via its leading `expect(&LBrace)` — verified in source at `parser/expr.rs:308`.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::expr::tests::if_else_chain
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate `Cargo.toml` lives at `vertex_stage0/Cargo.toml` (confirmed by file layout: `vertex_stage0/src/...`); test module path is `parser::expr::tests::if_else_chain` matching the existing `block_trailing_expr` / `closure_forms` style.
- `Expr::If` already exists in `src/ast/expr.rs` (confirmed at lines 286–293) with fields `id, span, cond, then, else_branch: Option<Box<Expr>>`; no AST changes needed.
- `TokenKind::If` and `TokenKind::Else` already exist in the lexer (confirmed via `describe()` arms in `parser/mod.rs:140` and `:146`) and don't need to be added.
- Wiring `if` into `parse_primary_for_paren` (rather than only `parse_expr`) is preferred so postfix operations like `(if c {a} else {b}).field` and `if c {a} else {b}.method()` flow naturally; matches the placement of `LBrace`/`LBracket`.
- The `then` field on `If` is typed `Box<Expr>` (not `Box<Block>`), so storing the `Expr::Block` returned by `parse_block` is fine.
- Condition parser is `self.parse_expr()`. Until struct-literal parsing exists, no ambiguity arises; once it does, that item will need to introduce a "no-struct-literal" expression mode and update this call.
- The test does not need to assert spans, only AST shape — keeps it robust to minor span-merging tweaks.
- Adding `TokenKind::If` to `range_rhs_starts_here` is in-scope for this item because the TODO in that function explicitly calls it out and the if-expression head would otherwise misparse `..if c {1} else {2}`.

## Blockers
Blockers: none

## Summary
Adds `parse_if` producing `Expr::If` for `if`/`else if`/`else` chains, requires `{...}` branches, wires `if` into the primary-expression head, and adds an `if_else_chain` unit test.

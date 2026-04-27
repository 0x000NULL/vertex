# Plan: parse-closure-expressions

## Goal
Add a closure-expression parser that produces `Expr::Closure` for the three spec forms `|params| body`, `move |params| body`, and `|x: i32| -> i32 { body }`, gated at the top of `parse_expr` so it cannot collide with the existing `Pipe` binary operator.

## Steps
1. In `src/parser/expr.rs`, modify `parse_expr` to detect a closure head before delegating to `parse_range`. The head is either `TokenKind::Pipe`, **or** `TokenKind::Ident("move")` followed (via `peek_at(1)`) by `TokenKind::Pipe`. On a hit, dispatch to a new `parse_closure` method. This is safe because `|` cannot legally start a non-closure expression (no prefix `|` operator exists; `BitOr` is only encountered after `parse_binary` already has an LHS).
2. Implement `fn parse_closure(&mut self) -> Result<Expr, CompileError>`:
   - Capture the start span. If `peek() == Ident("move")`, bump it and set `move_kw = true`; otherwise `false`.
   - Expect `TokenKind::Pipe` (open params).
   - Loop until `TokenKind::Pipe`: parse one param, then accept an optional trailing `Comma`. Each param is parsed by a small helper that consumes an `Ident` (the binding name) and, if the next token is `Colon`, also consumes a single `Ident` / `SelfUpper` token as a type-stub (mirroring the `parse_cast` placeholder approach for `as <ty>`). Push `ClosureParam::Placeholder` for each param — `ClosureParam` is currently a placeholder enum (`src/ast/expr.rs:80`), so we cannot yet store name/type, but the Pat-aware variant lands with the patterns work.
   - Expect closing `TokenKind::Pipe`.
   - If the next token is `TokenKind::Arrow`, bump it and consume one `Ident` / `SelfUpper` token as a return-type stub. The current `Closure` AST struct (`src/ast/expr.rs:233`) has no `return_ty` field, so the return type is consumed and discarded for now (leave an inline `// TODO:` referencing the eventual type-parser hook).
   - Parse the body:
     - If `peek() == TokenKind::LBrace`, call a private `parse_block_stub` helper that bumps `{`, parses one inner expression via `parse_expr`, expects `}`, and returns `Expr::Block { stmts: vec![], tail: Some(inner), .. }`. Mark the helper with a `// TODO: replaced by parse-block-expressions` comment so the real block parser supersedes it cleanly.
     - Otherwise, call `parse_expr` directly so `|x| x + 1` keeps the body as a full expression (Rust-style: closure body extends as far as the expression goes).
   - Build `Expr::Closure(Closure { id, span: start.merge(&body.span()), params, body: Box::new(body), move_kw })`.
3. Add a `#[test] fn closure_forms()` to the existing `tests` module at the bottom of `src/parser/expr.rs`. It must exercise:
   - `|| 1i32` → `Expr::Closure { params: [], move_kw: false, body: IntLit(1) }`.
   - `|x| x` (well, `|x|` followed by a literal stand-in like `1i32` since path-expression parsing is a separate pending item — body is a literal so we don't depend on `parse-path-expressions`) → 1 param, `move_kw: false`.
   - `move || 1i32` → `move_kw: true`, no params.
   - `|x: i32| 1i32` → 1 param with type-stub consumed.
   - `|x: i32| -> i32 { 1i32 }` → block body, return-type-stub consumed; assert body is `Expr::Block` with `tail = Some(IntLit(1))` and `stmts.is_empty()`.
   - One negative case: `| 1i32` (missing closing `|`) → `Err(E0100)`.
4. Run `cargo test --lib parser::expr::tests::closure_forms` to confirm green.

## Files
- `src/parser/expr.rs` — add `parse_closure`, the `parse_block_stub` helper, the `parse_expr` dispatch guard, and the `closure_forms` unit test. No other file changes; `Expr::Closure`, `Closure`, `ClosureParam`, and `Block` are already defined in `src/ast/expr.rs`.

## Risks
- **`|` ambiguity with `BitOr`.** Mitigated by only checking for `Pipe` at the entry of `parse_expr`, before any LHS exists. `parse_binary` continues to see `Pipe` as `BitOr` only when an LHS is already in hand. Sub-expressions reached via recursive `parse_expr` calls (e.g. inside `(...)`, `f(...)`, `a[...]`) also legitimately allow a closure as the first token, which matches Rust semantics.
- **`move` is an `Ident`, not a keyword token.** The spec lexer has no `Move` variant, so the detection uses the same `Ident("move")` pattern that `parse_cast` already uses for `as`. If the lexer ever promotes `move` to a keyword, this guard plus the `bump` need to switch to that variant — kept localized to one match arm.
- **Param/return type stubs.** Types are not parseable yet; we consume a single `Ident`/`SelfUpper` token as a placeholder, identical to `parse_cast`'s strategy. Multi-token types like `&mut i32` would fail today — same limitation as `as`. Acceptable per current parser stage.
- **Block-body stub overlaps with `parse-block-expressions`.** The minimal `parse_block_stub` only handles `{ <single expr> }` and is explicitly marked with a `TODO` so the real block parser replaces it without duplication. Using a stub here avoids waiting for that item to land.
- **`Closure.return_ty` field missing.** We silently discard the parsed `-> Ty` stub. This is correct for the current AST shape; when the type system lands, the field will be added and this site will read the parsed type instead of dropping it. Leave a TODO at the discard site.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::expr::tests::closure_forms
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate root is `vertex_stage0/` (confirmed by file layout — `vertex_stage0/src/parser/expr.rs` is the only parser-expr source). The `--manifest-path` is required because the working directory is the parent `vertex` repo, not the crate.
- Closure body precedence matches Rust: the body extends as far as a full `parse_expr` will go, so `|x| 1 + 2` parses as `|x| (1 + 2)` (no need to stop at any operator boundary).
- Empty-param closure `||` is two `Pipe` tokens (confirmed: lexer never composes `||`, see `src/lexer/scan.rs:555`); it parses by entering the param loop and immediately seeing the closing `Pipe` on the first iteration.
- Param type-stub and return type-stub each consume exactly one `Ident` / `SelfUpper` token, mirroring the `as <ty>` stub in `parse_cast` (`src/parser/expr.rs:262`). Anything richer is out of scope until the type parser lands.
- The block-body stub builds `Expr::Block { stmts: vec![], tail: Some(<parsed expr>) }` rather than returning the inner expression directly, so the AST node type matches what `parse-block-expressions` will eventually produce. This lets the test assert on `Expr::Block` immediately.
- `Pat`/typed-param info is intentionally lost into `ClosureParam::Placeholder` because `ClosureParam` is still a placeholder enum (`src/ast/expr.rs:80`). The patterns work will replace this enum and the call sites simultaneously.
- `parse_expr` is the right level to dispatch from (rather than `parse_range` or `parse_unary`): in Rust the closure expression sits below range in precedence, but since closures cannot occur as a binary operand in any of our existing tests (range RHS, binary RHS, etc. all funnel through `parse_binary`/`parse_cast`/`parse_unary`/`parse_postfix`, none of which would ever see a leading `|`), gating at `parse_expr` is both correct for top-level use and reachable from any sub-expression that calls `parse_expr` recursively (e.g. call args, index expr, paren contents).

## Blockers
Blockers: none

## Summary
Adds `parse_closure` (plus a tiny `{ expr }` block stub) wired into the head of `parse_expr`, producing `Expr::Closure` for the three spec forms with a `closure_forms` unit test covering each.

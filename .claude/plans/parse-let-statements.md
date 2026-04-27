Now I have enough context. Let me write the plan.

# Plan: parse-let-statements

## Goal
Add a real `parse_let` that produces `Stmt::Let { pattern, ty, init, span, id }` for `let pat: Type = expr;`, `let pat = expr;`, and `let pat: Type;`, wire it into `parse_block`'s statement dispatch, and lock in shape with a `parser::stmt::tests::let_forms` unit test.

## Steps
1. Create new file `vertex_stage0/src/parser/stmt.rs` and register it via `pub mod stmt;` in `vertex_stage0/src/parser/mod.rs` (alongside the existing `pub mod expr;`).
2. In `src/parser/stmt.rs`, add an `impl Parser` block with:
   - `pub fn parse_let(&mut self) -> Result<Stmt, CompileError>` that expects `TokenKind::Let`, captures `let_span`, then:
     - Calls a private `parse_let_pattern_stub(&mut self) -> Result<Pattern, CompileError>` that handles three head cases (mirroring the closure-param / `for` / match-arm stubs already in the codebase):
       - `Mut` then `Ident(name)` → `Pattern::Ident { name, mutable: true, sub: None }`
       - `Ident(name)` → `Pattern::Ident { name, mutable: false, sub: None }`
       - `Underscore` → `Pattern::Wild`
       - Anything else → `Err(self.unexpected_token_error("pattern"))`
       - Add a `// TODO: replace stub when pattern parser lands (parse-ident-patterns-mut-sub-binding / parse-or-patterns-and-wildcard).` comment, mirroring the existing stub comments in `parse_for` / `parse_closure_param` / `parse_match_arm`.
     - If next token is `Colon`: bump it, then call a private `parse_let_type_stub` that consumes a single `Ident(_)` or `SelfUpper` token (mirroring the `parse_closure_param` and `parse_cast` stubs) and returns `Type::Infer` (the lightest existing stub variant). On any other lookahead, return `Err(self.unexpected_token_error("type after `:`"))`. Add `// TODO: replace stub when type parser lands (parse-path-types-with-generic-args / parse-infer-placeholder).`
     - If next token is `Eq`: bump it, call `self.parse_expr()` for the initializer.
     - Validate the form: at least one of `ty` or `init` must be present (i.e. reject the bare `let pat;` form, which is not in spec). If both are absent, return `Err(self.unexpected_token_error("`:` or `=`"))` before consuming the semicolon.
     - Expect `Semi`, capture `semi_span`, build `span = let_span.merge(&semi_span)`, allocate `id = self.new_node_id()`, and return `Ok(Stmt::Let { pattern, ty, init, span, id })`.
3. In `src/parser/expr.rs`, extend the `parse_block` head loop (currently lines 528–566) to dispatch when `self.peek() == TokenKind::Let`: call `self.parse_let()?` and push it as a `Stmt::Let { ... }` (the value returned by `parse_let` is already a `Stmt`, so just `stmts.push(parse_let()?)`). Keep the existing expression / semi / tail logic for non-`let` heads unchanged.
4. Add a `#[cfg(test)] mod tests` to `src/parser/stmt.rs` containing a single `#[test] fn let_forms()` that builds token streams by hand (using the existing pattern: `Token::new(kind, Span::new(FileId(0), 0, 0))`) and asserts each of:
   - `let x: T = 1i32;` → `Stmt::Let { pattern: Pattern::Ident { name: "x", mutable: false, sub: None }, ty: Some(Type::Infer), init: Some(IntLit(1)), .. }`, `p.pos` advanced to EOF, `errors.is_empty()`.
   - `let x = 1i32;` → `ty: None`, `init: Some(IntLit(1))`.
   - `let x: T;` → `ty: Some(Type::Infer)`, `init: None`.
   - `let mut x = 1i32;` → `pattern.mutable == true`.
   - `let _ = 1i32;` → `pattern == Pattern::Wild`.
   - Two error cases that recovery-budget-free assert `Err`: missing `;` (`let x = 1i32` + EOF) and forbidden bare form (`let x;` returns `Err`, no advance past `;`).
5. Run `cargo build -p vertex_stage0` and the verify command to confirm both compile and pass.

## Files
- `vertex_stage0/src/parser/stmt.rs` -- new file: `parse_let`, the two private stubs (pattern, type), and the `let_forms` test module.
- `vertex_stage0/src/parser/mod.rs` -- add `pub mod stmt;` next to existing `pub mod expr;`.
- `vertex_stage0/src/parser/expr.rs` -- extend `parse_block`'s loop with a `TokenKind::Let` head arm that calls `self.parse_let()?` and pushes the resulting `Stmt::Let`.

## Risks
- **Pattern/type stubs diverge from real parsers later.** Mitigation: the stubs follow the exact same pattern (and TODO style) as the in-tree `parse_for` / `parse_closure_param` / `parse_match_arm` / `parse_cast` stubs, so the eventual swap point is identical and no tests will pin AST shapes the real parsers can't reproduce.
- **`Type::Infer` overloads two meanings.** Using `Type::Infer` as the stub for an annotated `let x: T` means `Some(Type::Infer)` does not actually represent `T` — it just records that the type slot was filled. That's fine for shape-only tests today, but the dedicated `parse-infer-placeholder` and `parse-path-types-with-generic-args` items must replace this stub before any later pass interprets `Type::Infer` semantically. Documented in a TODO comment.
- **`parse_expr` could consume `=` as `Assign`.** The Pratt parser treats `=` as a low-precedence binary; this matters only inside the initializer (after `Let`'s `=` has been bumped), so `let x = a = 1;` would parse the RHS as an assignment chain. That matches Rust's behavior and is acceptable; no test pins it.
- **Bare `let pat;` rejection** is a judgment call (the spec only enumerates the three forms with at least one of `:`/`=`). I reject it with an error to keep `Stmt::Let` always carrying meaningful info; if later items need bare `let pat;`, this single check is trivial to relax.
- **Block dispatch ordering.** Adding the `Let` arm before the `parse_expr()` call in `parse_block` is necessary because `parse_expr` would otherwise hit `Let` and return `Err`. Verified by re-reading the existing loop (`src/parser/expr.rs:534`).

## Prereqs
Prereqs: none

(The `Stmt::Let` variant in `src/ast/stmt.rs:7-13`, the `Pattern::Ident`/`Pattern::Wild` variants in `src/ast/pat.rs:23-29`, the `Type::Infer` variant in `src/ast/ty.rs:30`, and `CompileError` in `src/error/mod.rs` all already exist. The `define-stmt-enum-*`, `define-compileerror-struct-*`, and `define-generics-and-whereclause-*` slugs are bookkeeping items whose deliverables are already in tree, matching the precedent set in `.claude/plans/parse-block-expressions.md:36-39`.)

## Verify
```
cargo test --lib -p vertex_stage0 parser::stmt::tests::let_forms
cargo test --lib -p vertex_stage0 parser::expr::tests::block_trailing_expr
cargo build -p vertex_stage0
```

## Assumptions
- The verify path `parser::stmt::tests::let_forms` requires creating `src/parser/stmt.rs` as a new module containing its own `#[cfg(test)] mod tests` — a sibling to the existing `src/parser/expr.rs::tests` module rather than reusing it. This follows the test path explicitly named in the spec.
- `parse_let` returns `Result<Stmt, CompileError>` (not `Result<Expr, CompileError>`) because `Stmt::Let` is a `Stmt` variant and blocks store `Stmt`. The existing `parse_block` loop already pushes into a `Vec<Stmt>`.
- The pattern stub accepts `Ident`, `Mut Ident`, and `Underscore` only — enough to cover all five test cases without overreaching into territory owned by the real pattern-parser items.
- The type stub accepts `Ident(_)` or `SelfUpper` and returns `Type::Infer`, mirroring `parse_closure_param`'s strategy. Tests assert `Some(Type::Infer)` rather than the source-level type name; the real type parser will replace this and tests will be updated by that item.
- Bare `let pat;` (no type, no init) is rejected as a syntax error in this item. None of the three spec sub-steps require it; relaxing later is one-line.
- The new file gets `#![allow(dead_code)]`-style `#[allow(dead_code)]` only where existing patterns require — `parse_let` itself is `pub` and called from `parse_block`, so it should not need an allow.
- `parse_let` is `pub` (callable as `self.parse_let()` from within the `Parser` impl across modules) — same visibility convention as `parse_block`, `parse_expr`, etc.
- The five-form positive test set + two-form negative test set fits cleanly inside one `#[test] fn let_forms()` (the test bundles cases the way `block_trailing_expr`, `closure_forms`, `return_break_continue`, etc. already do).
- I will not (yet) update the `is_sync_point` set or `describe` table in `parser/mod.rs` — `Let` is already listed in `describe` (line 149). `Let` is not a sync-point head, which is the correct call until the error-recovery item lands.

## Blockers
Blockers: none

## Summary
Adds `parse_let` (with stub pattern + type slots that mirror the existing closure/for/match-arm stub strategy) producing `Stmt::Let` for the three spec forms, dispatches `Let` from `parse_block`'s statement loop, and locks in shape with a `parser::stmt::tests::let_forms` test covering type-and-init / init-only / type-only / `mut` / wildcard plus two error cases.

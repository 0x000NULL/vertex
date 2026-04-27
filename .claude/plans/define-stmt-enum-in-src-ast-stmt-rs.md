# Plan: define-stmt-enum-in-src-ast-stmt-rs

## Goal
Replace the placeholder `Stmt` enum in `ast::expr` with a real `Stmt` enum in a new `ast::stmt` module that carries `Let`, `Expr`, and `Item` statement variants, matching the spec's `statement = let_stmt | expression_stmt | item` rule.

## Steps
1. Create `vertex_stage0/src/ast/stmt.rs` with `pub enum Stmt` containing three variants:
   - `Let { pattern: Pattern, ty: Option<Type>, init: Option<Expr>, span: Span, id: NodeId }` — `ty` and `init` are `Option` because the spec marks `[":" type]` and `["=" expression]` as optional.
   - `Expr { expr: Expr, has_semi: bool }` — struct variant (the source todo's `Expr(Expr, has_semi: bool)` is not valid Rust as a tuple variant; converted to a struct variant so the `has_semi` flag stays named).
   - `Item(Item)` — wraps a nested item statement (`fn`/`struct`/etc. inside a block).
   - Add the `#[allow(dead_code)]` and `#[derive(Debug, Clone)]` attributes used uniformly across `ast::*` types.
   - Imports: `crate::ast::{Expr, Item, NodeId, Pattern, Type}` and `crate::span::Span`.
2. Update `vertex_stage0/src/ast/mod.rs`: add `pub mod stmt;` (alphabetical position between `pat` and `ty`) and `pub use stmt::Stmt;` (alphabetical position between `Pattern` and `Type`).
3. In `vertex_stage0/src/ast/expr.rs`:
   - Delete the placeholder block at lines 84–89 (`// TODO: replaced by define-stmt-enum-in-src-ast-stmt-rs` + `pub enum Stmt { Placeholder }`).
   - Add `use crate::ast::Stmt;` near the top so the existing `Block.stmts: Vec<Stmt>` field continues to resolve to the real enum re-exported from `ast::mod`.
4. Run `cargo build` from the repo root to confirm the workspace compiles.

## Files
- `vertex_stage0/src/ast/stmt.rs` — new file; defines `pub enum Stmt` with the three variants above.
- `vertex_stage0/src/ast/mod.rs` — register `pub mod stmt;` and re-export `Stmt`.
- `vertex_stage0/src/ast/expr.rs` — drop placeholder `Stmt` enum + its TODO comment; add `use crate::ast::Stmt;` so `Block.stmts: Vec<Stmt>` still resolves.

## Risks
- `Block` in `expr.rs` already references `Stmt` by bare name; if the `use` import is omitted after deletion, compile breaks. Mitigation: add the `use` in step 3.
- The todo wording `Expr(Expr, has_semi: bool)` is not legal Rust syntax (named field in tuple position). Using a struct variant preserves the named-flag intent without divergence from the spec; downstream parser items (`parse-expression-statements-with-semicolon-significance`) will need to construct `Stmt::Expr { expr, has_semi }` rather than the tuple form. Documented in Assumptions.
- No other source file references `Stmt::Placeholder` (verified via grep), so removing it is safe.

## Prereqs
Prereqs: none

## Verify
```
test -f vertex_stage0/src/ast/stmt.rs
grep -q 'pub enum Stmt' vertex_stage0/src/ast/stmt.rs
cargo build
```

## Assumptions
- The todo's path `src/ast/stmt.rs` is relative to the `vertex_stage0` crate root (the only workspace member with `src/ast/`); the verify `grep` therefore targets `vertex_stage0/src/ast/stmt.rs` even though `cargo build` is run from the repo root.
- `Expr(Expr, has_semi: bool)` in the source todo is interpreted as a struct variant `Expr { expr: Expr, has_semi: bool }` because Rust tuple variants cannot have named fields. The struct-variant form preserves the explicit `has_semi` name (which the placement-significance items rely on) and matches the existing `ast::*` style (e.g. `Pattern::Range { start, end, inclusive }`, `Type::Ref { mutable, ty, span, id }`).
- `Let.ty` and `Let.init` are `Option`, since the spec's `let_stmt` marks `[":" type]` and `["=" expression]` as optional; storing them as `Option<Type>` / `Option<Expr>` (not boxed) follows the same pattern as `If.else_branch` / `Return.value` for cheap, owned children — boxing isn't needed because `Stmt` is held behind `Vec<Stmt>` in `Block`, breaking any recursion-size cycle.
- `Item` and `Expr` variants do not duplicate `id`/`span` fields; their inner enums already expose `id()` and `span()` accessors, so `Stmt` can derive its own accessors later if needed without schema churn.
- No `impl Stmt { fn id() / fn span() }` is added in this commit — Pattern was shipped in item 40 without one and the verify line only requires the enum to exist; subsequent users can add accessors if needed.
- The placeholder `pub enum Pat { Placeholder }` and the unrelated `ClosureParam`/`CastTy`/`GenericArg` placeholders in `expr.rs` are intentionally left alone — they belong to other pending items (`define-pattern-enum-…`, `parse-closure-expressions`, `parse-indexing-cast-try`, generics item) and removing them is out of scope here.
- `#[allow(dead_code)]` is applied to `Stmt` and its variants, matching the convention every other `ast::*` enum uses while the parser is still landing.

## Blockers
Blockers: none

## Summary
Introduces `ast::stmt::Stmt` with `Let`/`Expr`/`Item` variants, removes the `Stmt` placeholder from `ast::expr`, and rewires `Block` to the real enum so subsequent statement-parsing items have a typed target.

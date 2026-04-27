# Plan: add-aggregate-literal-construction-variants-to-expr

## Goal
Extend `vertex_stage0::ast::expr::Expr` with seven aggregate / construction / closure / range / block variants, following the existing per-variant-struct + id/span dispatch pattern, using local placeholder types where dependent AST nodes (`Pat`, `Stmt`) do not yet exist.

## Steps
1. Open `vertex_stage0/src/ast/expr.rs` and locate the existing per-variant struct block (after `Try`) and the `Expr` enum + `impl Expr { id, span }` block.
2. Add two local placeholder enums next to the existing `CastTy::Placeholder` / `GenericArg::Placeholder` so the new variants do not require `ast::stmt` or `ast::pat` to land first:
   - `pub enum ClosureParam { Placeholder }` with TODO comment pointing to `define-pattern-enum-in-src-ast-pat-rs`.
   - `pub enum Stmt { Placeholder }` with TODO comment pointing to `define-stmt-enum-in-src-ast-stmt-rs`.
3. Add a small `StructLitField { name: String, value: Expr }` struct (needed by `StructLit.fields`); no `id`/`span` to keep churn minimal -- it is a sub-node, matching how `PathSegment` is modeled.
4. Add seven new per-variant structs (each `#[allow(dead_code)] #[derive(Debug, Clone)]`, each starting with `pub id: NodeId, pub span: Span,`):
   - `Range { start: Option<Box<Expr>>, end: Option<Box<Expr>>, inclusive: bool }`
   - `Closure { params: Vec<ClosureParam>, body: Box<Expr>, move_kw: bool }`
   - `StructLit { path: Path, fields: Vec<StructLitField>, base: Option<Box<Expr>> }`
   - `TupleLit { elems: Vec<Expr> }`
   - `ArrayLit { elems: Vec<Expr> }`
   - `ArrayRepeat { value: Box<Expr>, count: Box<Expr> }`
   - `Block { stmts: Vec<Stmt>, tail: Option<Box<Expr>> }`
5. Append the seven new variants to the `Expr` enum in this order: `Range(Range), Closure(Closure), StructLit(StructLit), TupleLit(TupleLit), ArrayLit(ArrayLit), ArrayRepeat(ArrayRepeat), Block(Block)`.
6. Extend the `id()` and `span()` match arms in `impl Expr` with one arm per new variant, returning `e.id` / `e.span`.
7. Run `cargo build -p vertex_stage0` to confirm everything compiles cleanly with the existing `#[allow(dead_code)]` policy.

## Files
- `vertex_stage0/src/ast/expr.rs` -- add `ClosureParam`, `Stmt` (local placeholders), `StructLitField` sub-node, seven new per-variant structs (`Range`, `Closure`, `StructLit`, `TupleLit`, `ArrayLit`, `ArrayRepeat`, `Block`), seven new `Expr` enum variants, and seven new arms each in `Expr::id()` and `Expr::span()`.

## Risks
- Naming collision: the local `Stmt` placeholder enum will be shadowed/replaced once `define-stmt-enum-in-src-ast-stmt-rs` lands; downstream users of `Block.stmts` will need a one-line type swap. Mitigated with a TODO comment.
- Same risk for `ClosureParam` once `define-pattern-enum-in-src-ast-pat-rs` lands.
- `Range.inclusive: bool` collapses `..` / `..=` into one variant; matches Rust's own AST convention but worth noting if the parser wants to distinguish exclusive-from-half-open syntactically -- not in scope here.
- No risk to id/span dispatch correctness: pattern is mechanical and mirrored across all existing variants.

## Prereqs
Prereqs: none

## Verify
```
cargo build -p vertex_stage0
grep -q "Range(Range)" vertex_stage0/src/ast/expr.rs
grep -q "Closure(Closure)" vertex_stage0/src/ast/expr.rs
grep -q "StructLit(StructLit)" vertex_stage0/src/ast/expr.rs
grep -q "TupleLit(TupleLit)" vertex_stage0/src/ast/expr.rs
grep -q "ArrayLit(ArrayLit)" vertex_stage0/src/ast/expr.rs
grep -q "ArrayRepeat(ArrayRepeat)" vertex_stage0/src/ast/expr.rs
grep -q "Block(Block)" vertex_stage0/src/ast/expr.rs
```

## Assumptions
- `Range.start`/`Range.end` are `Option<Box<Expr>>` so the variant can model `..`, `..end`, `start..`, `start..end`, `start..=end` uniformly; `inclusive: bool` distinguishes `..` from `..=`. Matches `rustc_ast::ExprKind::Range` shape.
- `Closure.params` uses a local `ClosureParam::Placeholder` enum (mirroring the existing `CastTy::Placeholder` / `GenericArg::Placeholder` precedent) so this item does not block on `define-pattern-enum-in-src-ast-pat-rs`.
- `Block.stmts` uses a local `Stmt::Placeholder` enum for the same reason; will be replaced when `define-stmt-enum-in-src-ast-stmt-rs` lands.
- `StructLit.path` reuses the existing `Path` struct (already defined in this file) rather than wrapping in `Box`, since `Path` is a small struct of (id, span, segments) and the surrounding code does not box `Path` elsewhere.
- `StructLitField` is modeled as a plain sub-node (no `NodeId`/`Span`) matching the existing `PathSegment` precedent; if individual fields later need diagnostic spans, that is a follow-up refactor.
- `ArrayRepeat.count` is `Box<Expr>` rather than a const-evaluated value, since stage0 has no const-eval and the value is whatever the parser produces.
- `Closure.move_kw: bool` records whether the `move` keyword was present; no type annotations on params or return type yet -- those depend on `define-type-enum-in-src-ast-ty-rs`.
- All new structs and the new placeholder enums get `#[allow(dead_code)] #[derive(Debug, Clone)]` to match every existing item in the file.
- New variants are appended to the end of the `Expr` enum (preserving source-order grouping: literals, path, operators, access, range, closure, aggregates, block), with matching arm order in `id()`/`span()`.
- Verify uses `cargo build -p vertex_stage0` (workspace-scoped, fast) plus deliverable-presence greps; no clippy/test pressure since these types are still `#[allow(dead_code)]` and have no behavior to test yet.

## Blockers
Blockers: none

## Summary
Adds `Range`, `Closure`, `StructLit`, `TupleLit`, `ArrayLit`, `ArrayRepeat`, and `Block` variants to the stage0 `Expr` AST so downstream parser items have a typed surface for aggregate literals, ranges, closures, and blocks.

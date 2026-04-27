Now I have enough context. Producing the plan.

# Plan: add-control-flow-variants-to-expr

## Goal
Extend `vertex_stage0::ast::expr::Expr` with control-flow variants (`If`, `Loop`, `While`, `For`, `Match`, `Return`, `Break`, `Continue`) and a `MatchArm` struct, following the existing per-variant-struct + id/span-dispatch pattern.

## Steps
1. In `vertex_stage0/src/ast/expr.rs`, add eight new per-variant structs, each with `id: NodeId`, `span: Span`, plus the variant-specific fields:
   - `If { id, span, cond: Box<Expr>, then: Box<Expr>, else_branch: Option<Box<Expr>> }` (using `Box<Expr>` for `then` so the branch can be any expression form, though in practice it will be a `Block`).
   - `Loop { id, span, body: Box<Expr> }`.
   - `While { id, span, cond: Box<Expr>, body: Box<Expr> }`.
   - `For { id, span, pat: Pat, iter: Box<Expr>, body: Box<Expr> }` — placeholder `Pat` type defined locally (parallel to the existing `ClosureParam`/`Stmt`/`CastTy` placeholders) with a `// TODO: replaced by define-pattern-enum-in-src-ast-pat-rs` note.
   - `Match { id, span, scrutinee: Box<Expr>, arms: Vec<MatchArm> }`.
   - `Return { id, span, value: Option<Box<Expr>> }`.
   - `Break { id, span, label: Option<String>, value: Option<Box<Expr>> }`.
   - `Continue { id, span, label: Option<String> }`.
2. Add a `MatchArm { id: NodeId, span: Span, pattern: Pat, guard: Option<Box<Expr>>, body: Box<Expr> }` struct (same `Debug, Clone, #[allow(dead_code)]` decorations).
3. Add a placeholder `Pat` enum next to the existing placeholder enums (`ClosureParam`, `Stmt`, `CastTy`) with a single `Placeholder` variant and a `// TODO: replaced by define-pattern-enum-in-src-ast-pat-rs` marker, so this item does not block on the still-pending `Pat` enum.
4. Extend the `Expr` enum with the 8 new variants in the existing order convention (literals, path, ops/access, range/closure, aggregates, block, then control flow).
5. Extend `Expr::id(&self)` and `Expr::span(&self)` match arms to cover the 8 new variants.
6. Run `cargo build` to confirm the crate still compiles.

## Files
- `vertex_stage0/src/ast/expr.rs` -- add 8 new variant structs, `MatchArm` struct, `Pat` placeholder enum, 8 new `Expr` variants, and update `id()`/`span()` dispatch arms.

## Risks
- Naming collision: there is no existing `Pat` symbol in this file, but the future `define-pattern-enum-in-src-ast-pat-rs` item will introduce one — placeholder must be local-only and clearly marked TODO so it gets replaced cleanly.
- `else_branch` field name (rather than `else`) is required because `else` is a Rust keyword; the spec already uses `else_branch` so this is fine.
- The spec says `If { cond, then, else_branch }` without typing — using `Box<Expr>` for both branches keeps the AST flexible (the parser can wrap `Block` exprs); committing to `Box<Block>` would force every branch to be a block at the AST level, which is more restrictive than necessary at this stage.
- `For { pat, iter, body }` in the spec lists `pat` as a field — using a placeholder `Pat` keeps the field name and type stable so the later pattern-enum item just swaps the placeholder out.
- `Break`/`Continue` labels are stored as `Option<String>`; matches Rust convention. Could later become a `Label` newtype but that's out of scope for this item.

## Prereqs
Prereqs: none

## Verify
```
cargo build -p vertex_stage0
```

## Assumptions
- The placeholder `Pat` enum belongs in `expr.rs` next to the other local placeholder enums (`ClosureParam`, `Stmt`, `CastTy`), not in a new `pat.rs` module — that is the responsibility of the separate `define-pattern-enum-in-src-ast-pat-rs` item.
- `MatchArm` carries its own `id` and `span` even though the spec lists fields as `{ pattern, guard, body, span, id }` — interpreting `span, id` literally means the struct has both fields, matching every other AST struct in this file.
- `If.then` and `If.else_branch` are typed `Box<Expr>` rather than `Box<Block>`; the parser will normally place a `Block` expression there but enforcing block-only at the AST level is a stricter invariant that should be considered separately.
- `Break.value` is included so `break <expr>` (loop-as-expression) works once the type checker lands; the spec's field list `{ label, value }` already implies this.
- `For.body`, `Loop.body`, `While.body` are `Box<Expr>` rather than `Box<Block>` for the same reason as `If` — keeps the AST permissive; the parser can decide to only emit `Block`s here.
- Labels are `Option<String>`; matches the lightweight identifier representation used elsewhere in this file (e.g. `PathSegment.ident: String`).
- Existing `#[allow(dead_code)] #[derive(Debug, Clone)]` decoration is applied to every new struct/enum to match the file's convention.
- No exports from `ast/mod.rs` need to change — `Expr` is already re-exported and the new variants are reached through it.

## Blockers
Blockers: none

## Summary
Adds 8 control-flow `Expr` variants plus `MatchArm` and a placeholder `Pat` to the stage0 AST so subsequent parser items can construct `if`/`loop`/`while`/`for`/`match`/`return`/`break`/`continue` nodes.

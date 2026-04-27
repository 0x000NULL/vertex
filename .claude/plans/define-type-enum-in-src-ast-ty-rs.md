# Plan: define-type-enum-in-src-ast-ty-rs

## Goal
Introduce a new `ast::ty` module containing a `Type` enum with the eight specified variants, structurally aligned with `ast::item` / `ast::expr`, so downstream parser/type-related items have a typed surface to construct.

## Steps
1. Create `vertex_stage0/src/ast/ty.rs` with:
   - `use crate::ast::{NodeId, expr::Path}; use crate::span::Span;`
   - A `#[derive(Debug, Clone)] #[allow(dead_code)] pub enum Type` with these variants exactly as specified:
     - `Path(Path)`
     - `Ref { mutable: bool, ty: Box<Type>, span: Span, id: NodeId }`
     - `Ptr { mutable: bool, ty: Box<Type> }`
     - `Array { elem: Box<Type>, len: Box<crate::ast::expr::Expr> }`
     - `Slice { elem: Box<Type> }`
     - `Tuple(Vec<Type>)`
     - `Fn { params: Vec<Type>, ret: Box<Type> }`
     - `Infer`
2. Wire the new module into `vertex_stage0/src/ast/mod.rs`: add `pub mod ty;` and `pub use ty::Type;`.
3. Leave the existing `CastTy` placeholder in `expr.rs` untouched — replacing its usages is the responsibility of later parser items, not this one.
4. `cargo build` to confirm the workspace still compiles.

## Files
- `vertex_stage0/src/ast/ty.rs` -- new file; defines `pub enum Type` with the eight variants from the spec, importing `NodeId` from `crate::ast`, `Path` from `crate::ast::expr`, and `Span` from `crate::span`.
- `vertex_stage0/src/ast/mod.rs` -- add `pub mod ty;` and `pub use ty::Type;` next to the existing `expr`/`item` lines.

## Risks
- `Path` re-export collision: `crate::ast` does not currently re-export `Path`, so referencing it as `crate::ast::expr::Path` avoids ambiguity. Re-exporting `Type` at the `ast` root could clash with `std::any::type` etc. — keep it scoped to `ast::Type` only.
- Variant-shape inconsistency in the spec: `Ref` carries `id`/`span` while `Ptr`/`Array`/`Slice`/`Tuple`/`Fn` do not. Following the spec literally means parent structures (e.g. a future `TypeNode` wrapper or call sites) must attach span/id contextually. Documented as an assumption rather than "fixed" by this item.
- `Array { len: Box<Expr> }` introduces a `Type → Expr` dep; harmless because `Expr` already exists, but it locks in the convention that array lengths are general const-expressions parsed into `Expr` rather than a smaller `ConstExpr` enum.
- The placeholder `CastTy` and `GenericArg` enums in `expr.rs` are not migrated here; downstream items must do that swap. Avoiding it now keeps this commit small and within the stated todo scope.

## Prereqs
Prereqs: none

(`Path` already exists in `expr.rs` from a prior commit; `NodeId` and `Span` are already in place.)

## Verify
```
cargo build -p vertex_stage0
grep -q 'pub enum Type' vertex_stage0/src/ast/ty.rs
test -f vertex_stage0/src/ast/ty.rs
```

## Assumptions
- The intended file path under the workspace is `vertex_stage0/src/ast/ty.rs` (the todo's `src/ast/ty.rs` is relative to the crate, mirroring how prior items used `src/ast/expr.rs`/`item.rs`). Verify uses the workspace-relative path so `bash -c` from the repo root succeeds.
- The enum is named `Type` (not `Ty`), per the verify substring `'pub enum Type'`.
- `Path(Path)` re-uses the existing `crate::ast::expr::Path` rather than introducing a new path-in-type AST node.
- `mutable` is `bool` for both `Ref` and `Ptr`.
- `Ref.ty` / `Ptr.ty` / `Array.elem` / `Slice.elem` / `Fn.ret` are `Box<Type>` to break the recursion; `Tuple` and `Fn.params` use `Vec<Type>` directly.
- `Array.len` is `Box<crate::ast::expr::Expr>` (general const-expression), the simplest representation that lets the parser populate it without inventing a new `ConstExpr` enum yet.
- No id/span dispatch helper is added on `Type` (unlike `Item::id()`/`Expr::span()`) because most variants intentionally lack `id`/`span` fields per the spec; adding such helpers would force decisions outside this item's scope.
- `#[derive(Debug, Clone)]` and `#[allow(dead_code)]` mirror the convention already in `item.rs` / `expr.rs`.
- The new `Type` is re-exported from `ast::mod` as `pub use ty::Type;` to match the existing `pub use expr::Expr;` / `pub use item::Item;` pattern.
- Existing `CastTy` and `GenericArg::Placeholder` in `expr.rs` are NOT replaced here; sweeping those is the work of later parser items that introduce real cast/generic-arg parsing.

## Blockers
Blockers: none

## Summary
Adds a new `ast::ty` module with a `Type` enum carrying the eight spec-mandated variants, wired into `ast::mod`, giving subsequent parser/type items a typed surface without touching the existing `CastTy`/`GenericArg` placeholders.

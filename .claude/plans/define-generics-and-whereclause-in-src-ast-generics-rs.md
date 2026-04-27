# Plan: define-generics-and-whereclause-in-src-ast-generics-rs

## Goal
Add a new `ast::generics` module that defines `Generics`, `TypeParam`, `WhereClause`, `WherePred`, and `TraitBound` so downstream parser items for fn/struct/enum/trait/impl have a typed surface for generic parameters and where clauses.

## Steps
1. Create `vertex_stage0/src/ast/generics.rs` with these public types (each `#[allow(dead_code)] #[derive(Debug, Clone)]` to match the existing `ast` style):
   - `pub struct Generics { pub params: Vec<TypeParam>, pub where_clause: Option<WhereClause> }`
   - `pub struct TypeParam { pub name: String, pub bounds: Vec<TraitBound> }`
   - `pub struct WhereClause { pub predicates: Vec<WherePred> }`
   - `pub struct WherePred { pub ty: Type, pub bounds: Vec<TraitBound> }` (minimal shape — a constrained type plus its bounds; see Assumptions)
   - `pub struct TraitBound { pub path: Path, pub generic_args: Vec<GenericArg> }`
2. Import `Type` from `crate::ast::ty::Type` and `Path`/`GenericArg` from `crate::ast::expr` to reuse existing surfaces (matches how `ast::ty` already imports `Path`).
3. Register the new module in `vertex_stage0/src/ast/mod.rs`: add `pub mod generics;` next to the other module declarations and add `pub use generics::{Generics, TypeParam, WhereClause, WherePred, TraitBound};` next to the other re-exports.
4. Leave the existing `expr::GenericArg` placeholder enum untouched — its inline TODO already names this slug, but replacing it is the job of the generic-arg parsing item, not this struct-definition item.
5. Run `cargo build` from the workspace root to confirm compilation; the new types are dead-code-allowed so unused-warning is fine.

## Files
- `vertex_stage0/src/ast/generics.rs` — new file containing the five struct definitions and the `use` lines for `Type`, `Path`, `GenericArg`.
- `vertex_stage0/src/ast/mod.rs` — add `pub mod generics;` plus a `pub use generics::{...};` re-export line.

## Risks
- The todo spec says "src/ast/generics.rs" but the workspace puts the crate under `vertex_stage0/`; using the wrong path would fail verify. Mitigated by using `vertex_stage0/src/ast/generics.rs` consistently in both Files and Verify.
- `WherePred`'s exact shape is unspecified by the bullet list (it only mentions the field name `predicates: Vec<WherePred>`). A future parser item may want a richer variant set (lifetime preds, equality preds). Mitigated by documenting the chosen minimal shape in Assumptions; future items can extend the struct without breaking callers since nothing constructs it yet.
- Adding `pub use generics::TypeParam` could collide if a future item also defines a `TypeParam` elsewhere; current tree has no such symbol, so this is theoretical.

## Prereqs
Prereqs: none

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
test -f vertex_stage0/src/ast/generics.rs
grep -q 'pub struct Generics' vertex_stage0/src/ast/generics.rs
grep -q 'pub struct TypeParam' vertex_stage0/src/ast/generics.rs
grep -q 'pub struct WhereClause' vertex_stage0/src/ast/generics.rs
grep -q 'pub struct TraitBound' vertex_stage0/src/ast/generics.rs
grep -q 'pub mod generics' vertex_stage0/src/ast/mod.rs
```

## Assumptions
- The crate lives at `vertex_stage0/`, so `src/ast/generics.rs` in the spec means `vertex_stage0/src/ast/generics.rs`. Verify paths use the workspace-relative path.
- `TypeParam.name` is `String` (matches how `PathSegment.ident`, `FieldAccess.name`, `MethodCall.method`, `Pattern::Ident.name` are all `String` in the existing AST — no interner yet).
- `TraitBound.path` reuses the existing `crate::ast::expr::Path`, and `TraitBound.generic_args` reuses the existing `crate::ast::expr::GenericArg` placeholder (same pattern as `PathSegment.generic_args` and `MethodCall.generic_args`). When `GenericArg` is fleshed out by a later item, this struct picks up the change for free.
- `WherePred` is given a minimal `{ ty: Type, bounds: Vec<TraitBound> }` shape since the spec only names the field `predicates: Vec<WherePred>` without prescribing internals; this matches Rust's most common where-predicate form (`T: Bound1 + Bound2`). Lifetime/equality predicates can be added later as additional fields or by promoting `WherePred` to an enum.
- No ID/Span fields on these structs — the spec bullets explicitly list only the fields named above, and `Generics`/`WhereClause` aren't standalone AST nodes that need to be looked up by `NodeId`. Matches the choice in `ast::ty::Type` where most variants also lack `id`/`span`.
- All five types derive `Debug, Clone` and carry `#[allow(dead_code)]`, mirroring every other type in `ast::expr`, `ast::ty`, `ast::pat`, and `ast::item`.
- The existing `expr::GenericArg::Placeholder` stays in place; this item adds new types and does not refactor placeholders. The TODO comment on `GenericArg` referencing this slug is aspirational — the real `GenericArg` rework belongs to a parser-side item, not this struct-only item.

## Blockers

### Blocker: WherePred shape unspecified
- severity: local
- affects: where-clause parsing, generics parsing, future trait/impl items
- question: Should `WherePred` be a struct `{ ty: Type, bounds: Vec<TraitBound> }`, an enum covering `Type: Bounds` / `'a: 'b` / `T = U`, or include a `span`/`id`?
- default_assumption: Define `WherePred` as a single struct `{ ty: Type, bounds: Vec<TraitBound> }` with no id/span; later items can extend it (extra fields) or promote it to an enum once lifetime and equality predicates are needed. This is the smallest shape consistent with the spec's `predicates: Vec<WherePred>` bullet.

## Summary
Introduces the `ast::generics` module with `Generics`, `TypeParam`, `WhereClause`, `WherePred`, and `TraitBound` so downstream parsers can attach generic parameters and where clauses to function, struct, enum, trait, and impl items.

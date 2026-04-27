# Plan: define-generics-and-whereclause-in-src-ast-generics-rs

## Goal
Introduce a new `ast::generics` module that defines `Generics`, `TypeParam`, `WhereClause`, `WherePred`, and `TraitBound` so later parser items (functions, structs, enums, traits, impls, type aliases) have a concrete target for `<...>` and `where ...` syntax.

## Steps
1. Create `vertex_stage0/src/ast/generics.rs` with five `#[derive(Debug, Clone)]` types (each `#[allow(dead_code)]` to match the convention used by sibling AST modules):
   - `pub struct Generics { pub id: NodeId, pub span: Span, pub params: Vec<TypeParam>, pub where_clause: Option<WhereClause> }`
   - `pub struct TypeParam { pub id: NodeId, pub span: Span, pub name: String, pub bounds: Vec<TraitBound> }`
   - `pub struct WhereClause { pub id: NodeId, pub span: Span, pub predicates: Vec<WherePred> }`
   - `pub struct WherePred { pub id: NodeId, pub span: Span, pub ty: Type, pub bounds: Vec<TraitBound> }` — accepting the default per the resolved blocker (struct, not enum).
   - `pub struct TraitBound { pub id: NodeId, pub span: Span, pub path: Path, pub generic_args: Vec<GenericArg> }`
2. At the top of the new file, import the existing AST building blocks: `crate::ast::NodeId`, `crate::ast::Type`, `crate::ast::expr::{Path, GenericArg}`, and `crate::span::Span`.
3. Register the new module in `vertex_stage0/src/ast/mod.rs`: add `pub mod generics;` next to the other `pub mod …` lines and re-export the entry types with `pub use generics::{Generics, TypeParam, WhereClause, WherePred, TraitBound};` to mirror the existing re-export pattern (`pub use expr::Expr;`, etc.).
4. Do not touch `Item`/`FnDef`/`StructDef` yet — wiring `Generics` into items is the responsibility of the later `add-generics-and-where-clauses-to-function-items` and per-item parsers; this item only defines the types.
5. Run `cargo build` from `vertex_stage0/` to confirm the new module compiles cleanly with the rest of the crate (no unused-import warnings since each imported type appears in at least one struct field).

## Files
- `vertex_stage0/src/ast/generics.rs` -- new file containing the five struct definitions described above.
- `vertex_stage0/src/ast/mod.rs` -- add `pub mod generics;` and a `pub use generics::{Generics, TypeParam, WhereClause, WherePred, TraitBound};` re-export line.

## Risks
- The TODO verify line uses the path `src/ast/generics.rs`, but the actual crate lives under `vertex_stage0/src/`. Running `grep` from the workspace root with the literal path would fail; the verify section below uses the real path. Same caveat for the build command, which targets the workspace member's manifest.
- `TraitBound` carries both a `path: Path` and a `generic_args: Vec<GenericArg>` even though `Path` already nests generic arguments inside each `PathSegment`. Keeping the explicit `generic_args` field matches the task spec literally; the parser item that produces `TraitBound`s will decide how the two relate (likely keeping segment-level args in `Path` and leaving the trailing `generic_args` field empty/unused for now).
- `WherePred` defaulting to `{ ty: Type, bounds: Vec<TraitBound> }` excludes lifetime predicates (`'a: 'b`) and equality predicates (`T = U`); both are out of scope for the v1 spec grammar (`where_predicate = type ":" bounds`), so this matches §spec 3253–3254.
- `Type` already references `crate::ast::expr::Path`/`Expr`, so importing `Type` here does not create a new cycle but it does deepen the dependency graph; the modules are fine as long as everything stays within `crate::ast`.

## Prereqs
Prereqs: none

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
test -f vertex_stage0/src/ast/generics.rs
grep -q 'pub struct Generics' vertex_stage0/src/ast/generics.rs
grep -q 'pub struct TypeParam' vertex_stage0/src/ast/generics.rs
grep -q 'pub struct WhereClause' vertex_stage0/src/ast/generics.rs
grep -q 'pub struct WherePred' vertex_stage0/src/ast/generics.rs
grep -q 'pub struct TraitBound' vertex_stage0/src/ast/generics.rs
grep -q 'pub mod generics' vertex_stage0/src/ast/mod.rs
```

## Assumptions
- Crate location: the `src/ast/...` paths in TODO.md map to `vertex_stage0/src/ast/...` (the only workspace member, matching every previously-completed AST item like `ast/ty.rs`, `ast/pat.rs`, `ast/stmt.rs`).
- Each struct gets `id: NodeId` and `span: Span` fields, matching the convention used by every other AST node in `ast/expr.rs`, `ast/item.rs`, and `ast/ty.rs`. The TODO field list is treated as a non-exhaustive minimum, not a closed schema.
- `TypeParam.name` is a plain `String` (mirroring `PathSegment.ident: String` and `FieldAccess.name: String`), not an interned symbol — the codebase has no interner yet.
- `TraitBound.path` reuses `crate::ast::expr::Path` (the same `Path` already used by `Type::Path` and the `Pattern::Struct` variant); we do not introduce a separate `TraitPath` type.
- `TraitBound.generic_args` reuses the existing `crate::ast::expr::GenericArg` placeholder enum; refining `GenericArg` is the responsibility of a later item and stays untouched here.
- `WherePred` is a struct, not an enum, per the resolved blocker note ("Accept default") — the v1 grammar only specifies `type ":" bounds`, so an enum is unnecessary.
- All five types derive `Debug, Clone` and carry `#[allow(dead_code)]` attributes, matching the rest of the AST modules where consumers (parser, resolver, typecheck) do not yet exist.
- The new module is added to `ast/mod.rs` with both `pub mod generics;` and a `pub use generics::{...};` re-export, consistent with `pub use expr::Expr;` and friends, so callers can write `crate::ast::Generics`.
- This item adds the *types only*. It does not add `Generics` fields to `FnDef`, `StructDef`, etc.; that wiring belongs to the per-item parser tasks (`add-generics-and-where-clauses-to-function-items`, etc.) listed later in the run.
- No unit tests are added — the sibling AST modules (`ty.rs`, `pat.rs`, `item.rs`) contain no tests of their own; verification is via `cargo build`, matching the verify line in TODO.md.

## Blockers
Blockers: none

## Summary
Adds an `ast::generics` module with `Generics`, `TypeParam`, `WhereClause`, `WherePred`, and `TraitBound` so subsequent parser items have a typed home for `<...>` and `where ...` syntax.

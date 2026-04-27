# Plan: define-pattern-enum-in-src-ast-pat-rs

## Goal
Create a new `ast::pat` module containing a `Pattern` enum with all nine spec-mandated variants, wired into `ast::mod`, so downstream pattern-parser items have a typed surface to populate.

## Steps
1. Create `vertex_stage0/src/ast/pat.rs` with:
   - A small local `Lit` enum re-using the existing per-kind literal structs from `ast::expr` (`IntLit`, `FloatLit`, `CharLit`, `StrLit`, `BoolLit`) — the spec lists `Literal(Lit)` but no unified `Lit` exists yet, and prior items (`Type`, `CastTy`) show the pattern of defining locally-scoped helpers without cross-cutting refactors.
   - A `StructPatField { name: String, pattern: Pattern }` helper struct for `Pattern::Struct`'s `fields`.
   - The `Pattern` enum with the nine spec-listed variants:
     - `Wild`
     - `Ident { name: String, mutable: bool, sub: Option<Box<Pattern>> }`
     - `Literal(Lit)`
     - `Range { start: Box<Pattern>, end: Box<Pattern>, inclusive: bool }`
     - `Tuple(Vec<Pattern>)`
     - `Struct { path: Path, fields: Vec<StructPatField>, rest: bool }`
     - `TupleStruct { path: Path, elems: Vec<Pattern> }`
     - `Ref { mutable: bool, pattern: Box<Pattern> }`
     - `Or(Vec<Pattern>)`
   - Apply `#[allow(dead_code)]` and `#[derive(Debug, Clone)]` to match sibling files.
2. Register the module in `vertex_stage0/src/ast/mod.rs`: add `pub mod pat;` and `pub use pat::Pattern;` next to the existing `expr`/`item`/`ty` lines.
3. Leave the placeholder `pub enum Pat { Placeholder }` in `expr.rs` untouched — this matches the pattern set by `define-type-enum-in-src-ast-ty-rs` (which also left `CastTy::Placeholder` in place); migrating `For`/`MatchArm` to `Pattern` is a downstream parser concern.
4. Run `cargo build` to confirm everything compiles.

## Files
- `vertex_stage0/src/ast/pat.rs` — new file: `Lit` enum, `StructPatField` struct, `Pattern` enum with nine variants.
- `vertex_stage0/src/ast/mod.rs` — add `pub mod pat;` and `pub use pat::Pattern;`.

## Risks
- Variant field choices (e.g., `Box<Pattern>` vs `Lit` for `Range` bounds) are not pinned by the task spec; downstream parser items may need to refine them. Choosing `Box<Pattern>` is the most flexible and least likely to require breaking changes.
- Adding a local `Lit` enum means a future unification of literal representations (e.g., a top-level `ast::Lit`) will need to migrate this site — acceptable, mirrors how `CastTy` is locally scoped today.

## Prereqs
Prereqs: none

## Verify
```
cargo build
test -f vertex_stage0/src/ast/pat.rs
grep -q 'pub enum Pattern' vertex_stage0/src/ast/pat.rs
```

## Assumptions
- "src/ast/pat.rs" in the task refers to `vertex_stage0/src/ast/pat.rs`, since the workspace lives there and prior AST items used the same path convention.
- The unified `Lit` referenced by `Literal(Lit)` doesn't exist yet, so I'll define a local `Lit` enum inside `pat.rs` wrapping the existing per-kind literal structs from `ast::expr`.
- `Range`'s `start`/`end` are typed as `Box<Pattern>` (not `Lit`) for maximum flexibility — the parser can decide what's valid; this is the smallest-commitment choice.
- `Struct`'s `fields` use a new `StructPatField { name, pattern }` helper to keep field-name + sub-pattern bound together, matching how `expr.rs` defines `StructLitField`.
- The `path` field in `Struct`/`TupleStruct` reuses the existing `ast::expr::Path` (same approach taken by `ast::ty::Type::Path`).
- Derive `Debug, Clone` and `#[allow(dead_code)]`, matching every other AST file in the crate.
- The existing `Pat::Placeholder` enum in `expr.rs` and its references from `For`/`MatchArm` stay as-is for this item; migration is downstream parser work.
- No `id`/`span` fields per variant or accessor methods are required — the task spec lists pure data variants and the verify only checks for `pub enum Pattern`. Sibling `Type` enum (the most recently completed analog) also omits per-variant id/span dispatch.

## Blockers
Blockers: none

## Summary
Adds a new `ast::pat` module with a nine-variant `Pattern` enum and a local `Lit` helper, wired into `ast::mod`, giving downstream pattern parsers a typed surface without disturbing existing placeholders.

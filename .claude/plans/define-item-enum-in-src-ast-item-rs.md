# Plan: define-item-enum-in-src-ast-item-rs

## Goal
Create `src/ast/item.rs` defining the `Item` enum with 10 stub variants (one per top-level item kind in the spec), each wrapping a per-kind struct that carries `id: NodeId` and `span: Span`.

## Steps
1. Create `vertex_stage0/src/ast/item.rs`. Add `use crate::ast::NodeId;` and `use crate::span::Span;`.
2. Define ten `pub struct` stubs — `FnDef`, `StructDef`, `EnumDef`, `ImplDef`, `TraitDef`, `ModDef`, `UseDef`, `ConstDef`, `StaticDef`, `TypeAliasDef` — each with exactly two `pub` fields: `id: NodeId` and `span: Span`. Derive `Debug, Clone` on each (matches the `Debug` bound `NodeId` and `Span` already provide). Mark each with `#[allow(dead_code)]` so the unused fields don't break a future `-D warnings` build (the bare types are needed now; richer fields are added in `parse-plain-function-items`, `parse-normal-struct-items`, etc.).
3. Define `pub enum Item` with exactly the ten variants required: `Fn(FnDef), Struct(StructDef), Enum(EnumDef), Impl(ImplDef), Trait(TraitDef), Mod(ModDef), Use(UseDef), ConstDef(ConstDef)` — wait, the spec mandates the variant names `Const(ConstDef), Static(StaticDef), TypeAlias(TypeAliasDef)`. Final list is exactly: `Fn(FnDef), Struct(StructDef), Enum(EnumDef), Impl(ImplDef), Trait(TraitDef), Mod(ModDef), Use(UseDef), Const(ConstDef), Static(StaticDef), TypeAlias(TypeAliasDef)`. Derive `Debug, Clone`. Add `#[allow(dead_code)]` on the enum.
4. Add a small inherent `impl Item` with one helper `pub fn span(&self) -> Span` that pattern-matches each variant and returns the inner struct's `span`. This justifies the `span` field's existence and silences `dead_code` for it without needing `#[allow(dead_code)]` per field. Same trick for `pub fn id(&self) -> NodeId`.
5. Register the new module: in `vertex_stage0/src/ast/mod.rs`, append `pub mod item;` and `pub use item::Item;`.
6. Run `cargo build -p vertex_stage0` locally (mentally) to verify no errors. No tests required by this item — later parser items will exercise the variants.

## Files
- `vertex_stage0/src/ast/item.rs` -- new file: `Item` enum + 10 stub structs (`FnDef`, `StructDef`, `EnumDef`, `ImplDef`, `TraitDef`, `ModDef`, `UseDef`, `ConstDef`, `StaticDef`, `TypeAliasDef`), each with `id: NodeId, span: Span`; plus `Item::id` / `Item::span` accessors.
- `vertex_stage0/src/ast/mod.rs` -- add `pub mod item;` and `pub use item::Item;` so the module compiles and downstream code can reference `crate::ast::Item`.

## Risks
- Dead-code warnings on unused variants/structs could be promoted to errors by a future `-D warnings` CI step (`set-ci-fmt-clippy-gate-to-deny-warnings`). The `id`/`span` accessors and `#[allow(dead_code)]` on the enum mitigate this; if a stricter clippy lint trips, the next planned item should remove the allow once a variant is constructed.
- Naming collision: spec calls one variant `Const(ConstDef)`. Rust treats `Const` as a contextual keyword in some positions but it's a valid variant identifier — no conflict.
- Ten empty structs with identical shape look like premature scaffolding, but the spec for this item explicitly says "Each variant references a struct stub (fields can be added later items)," so per-variant structs are required even though a single shared `ItemHeader` would be terser today.

## Prereqs
- define-nodeid-newtype-in-src-ast-mod-rs
- implement-span-struct-in-src-span-rs

(Both types already exist in the tree at `src/ast/mod.rs` and `src/span.rs`, so this plan is unblocked in practice — the prereqs are listed only because their slugs remain in the pending set.)

## Verify
```
cargo build -p vertex_stage0
test -f vertex_stage0/src/ast/item.rs
grep -q 'pub enum Item' vertex_stage0/src/ast/item.rs
grep -q 'Fn(FnDef)' vertex_stage0/src/ast/item.rs
grep -q 'Struct(StructDef)' vertex_stage0/src/ast/item.rs
grep -q 'Enum(EnumDef)' vertex_stage0/src/ast/item.rs
grep -q 'Impl(ImplDef)' vertex_stage0/src/ast/item.rs
grep -q 'Trait(TraitDef)' vertex_stage0/src/ast/item.rs
grep -q 'Mod(ModDef)' vertex_stage0/src/ast/item.rs
grep -q 'Use(UseDef)' vertex_stage0/src/ast/item.rs
grep -q 'Const(ConstDef)' vertex_stage0/src/ast/item.rs
grep -q 'Static(StaticDef)' vertex_stage0/src/ast/item.rs
grep -q 'TypeAlias(TypeAliasDef)' vertex_stage0/src/ast/item.rs
```

## Assumptions
- The crate to modify is `vertex_stage0` (the only crate in the workspace with an `src/ast/` directory).
- `NodeId` is `crate::ast::NodeId` (already defined in `src/ast/mod.rs`) and `Span` is `crate::span::Span` (already defined in `src/span.rs`); no new imports are needed elsewhere.
- The stub structs each get `#[derive(Debug, Clone)]` to match the rest of the AST surface and let future test snapshots format them. `Copy` is intentionally NOT derived because later items will add owning fields (e.g. `Vec<Param>`, `Box<Block>`).
- An `impl Item { pub fn id(&self) -> NodeId; pub fn span(&self) -> Span; }` is added so `id`/`span` are reachable and don't trigger `dead_code` warnings — matches what every later parser item will need anyway.
- Variant names follow the spec wording exactly: `Const`, `Static`, `TypeAlias` (not `ConstItem` etc.).
- `pub use item::Item;` is added at the `ast` module root so downstream code can write `crate::ast::Item`. Per-struct re-exports are deferred — callers that need `FnDef` etc. will use `crate::ast::item::FnDef` until the parser items decide on the surface.

## Blockers
Blockers: none

## Summary
Adds a stub `Item` enum and 10 placeholder per-kind structs in a new `src/ast/item.rs`, wired into `ast/mod.rs`, giving downstream parser/AST work a typed surface to fill in.

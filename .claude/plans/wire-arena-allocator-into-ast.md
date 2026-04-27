# Plan: wire-arena-allocator-into-ast

## Goal
Pull `typed-arena` into the workspace and introduce a stub `Arena` type in `vertex_stage0::ast` that later AST node enums will hang their per-type allocators off of.

## Steps
1. Add a `[dependencies]` section to `vertex_stage0/Cargo.toml` with `typed-arena = "2"`. (The root `Cargo.toml` is a virtual workspace manifest with no `[dependencies]`, so the dep belongs on the crate.)
2. Create `vertex_stage0/src/ast/arena.rs` defining `pub struct Arena` that wraps `typed_arena::Arena` instances. Since none of the per-node-type enums (`Item`, `Stmt`, `Expr`, `Ty`, `Pat`) exist yet — they're each pending in their own items — give the struct a single `#[allow(dead_code)]` placeholder field of type `typed_arena::Arena<()>` plus a `pub fn new() -> Self` and `Default` impl. Document that per-node-type fields will be added as node enums come online.
3. Re-export the new module from `vertex_stage0/src/ast/mod.rs` (`pub mod arena;` and `pub use arena::Arena;`) so downstream consumers can write `vertex_stage0::ast::Arena`.
4. Run `cargo build -p vertex_stage0` to confirm the dep resolves and the new struct compiles.

## Files
- `vertex_stage0/Cargo.toml` -- add `[dependencies]` table containing `typed-arena = "2"`.
- `vertex_stage0/src/ast/arena.rs` -- new file: `pub struct Arena` wrapping `typed_arena::Arena`, with `new()`/`Default`.
- `vertex_stage0/src/ast/mod.rs` -- declare `pub mod arena;` and re-export `Arena`. Existing `NodeId` definition stays untouched.

## Risks
- The verify `grep -q '^typed-arena' Cargo.toml` is ambiguous about which Cargo.toml; the workspace root has no deps section, so the dep MUST land in `vertex_stage0/Cargo.toml`. The verify line below grep's that file explicitly to avoid the ambiguity.
- A completely empty `Arena {}` would let `typed-arena` sit as an unused dependency and waste compile time. The placeholder field keeps the dep load-bearing now and avoids the warning churn of adding/removing it later.
- Per-node-type fields cannot be added in this commit because the node enums don't exist yet; subsequent `define-*-enum-*` items will edit `arena.rs` to add `items: TypedArena<Item>`, etc. The placeholder field is intentionally transient.

## Prereqs
Prereqs: none

## Verify
```
cargo build -p vertex_stage0 --manifest-path vertex_stage0/Cargo.toml
grep -q '^typed-arena' vertex_stage0/Cargo.toml
test -f vertex_stage0/src/ast/arena.rs
grep -q 'pub struct Arena' vertex_stage0/src/ast/arena.rs
```

## Assumptions
- The dep belongs on `vertex_stage0/Cargo.toml`, not the workspace root, because the workspace root is a virtual manifest (`[workspace]` only, no `[package]`) and the consuming code lives in the crate.
- "Wrapping per-node-type arenas" is implemented as a `typed_arena::Arena<T>` per node enum, but since `Item`/`Stmt`/`Expr`/`Ty`/`Pat` are each scheduled as their own future items, this commit ships a struct skeleton with one placeholder `typed_arena::Arena<()>` field. Later items will add concrete fields as their enums land.
- `Arena` is re-exported at `vertex_stage0::ast::Arena` (via `pub use`) so downstream call sites don't have to write `ast::arena::Arena`.
- A `Default` impl plus `new()` are in scope; no allocation API (e.g. `alloc_expr`) is added yet — that is each enum's own item to wire.
- No `cargo run` / binary changes are needed; the bin already builds and we add no `use ast::Arena` anywhere yet.

## Blockers
Blockers: none

## Summary
Add `typed-arena = "2"` to the `vertex_stage0` crate and introduce a placeholder `ast::Arena` struct that subsequent AST enum items will extend with per-node-type arenas.

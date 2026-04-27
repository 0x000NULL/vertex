# Plan: define-nodeid-newtype-in-src-ast-mod-rs

## Goal
Create the `vertex_stage0::ast` module with a `NodeId(u32)` newtype that other AST nodes will use as a stable per-node identifier.

## Steps
1. Create `vertex_stage0/src/ast/mod.rs` with `pub struct NodeId(pub u32);` deriving `Copy, Clone, PartialEq, Eq, Hash, Debug`.
2. Register the module in `vertex_stage0/src/lib.rs` by adding `pub mod ast;` (alphabetically between `error` and `explain`... actually between `ast` and `codegen`, i.e. before `codegen`).
3. Run `cargo build -p vertex_stage0` locally to confirm it compiles cleanly with no new warnings.

## Files
- `vertex_stage0/src/ast/mod.rs` -- new file containing the `NodeId` newtype with the required derives. No other items added (later todos will add the arena allocator, `Item`, `Stmt`, `Expr`, etc.).
- `vertex_stage0/src/lib.rs` -- add `pub mod ast;` so the module is wired into the crate root.

## Risks
- The todo text says `src/ast/mod.rs` but the workspace puts the crate under `vertex_stage0/`; using the literal path would fail. The plan uses `vertex_stage0/src/ast/mod.rs` to match the actual layout, and the verify `grep` line is adjusted to that path.
- Adding the module without using it produces a `dead_code` warning on `NodeId.0`. Marking the field `pub` (as the spec requires) and making the struct `pub` prevents `dead_code`, since the field is reachable from outside the crate.
- Future todos (`define-item-enum-in-src-ast-item-rs`, `wire-arena-allocator-into-ast`, etc.) will extend this module; keeping `mod.rs` minimal here avoids merge churn with those items.

## Prereqs
Prereqs: none

## Verify
```
cargo build -p vertex_stage0
test -f vertex_stage0/src/ast/mod.rs
grep -q 'pub struct NodeId' vertex_stage0/src/ast/mod.rs
grep -q 'pub mod ast' vertex_stage0/src/lib.rs
```

## Assumptions
- The todo's path `src/ast/mod.rs` is relative to the `vertex_stage0` crate, so the real on-disk path is `vertex_stage0/src/ast/mod.rs` (the workspace root only contains a `[workspace]` `Cargo.toml`).
- `NodeId` wraps `u32` (matches the spec line literally) -- not `usize` or `NonZeroU32`. Reservation of id 0 / niche-optimization is deferred until `wire-arena-allocator-into-ast` decides on an allocation scheme.
- The struct field is left `pub` (per spec) rather than wrapped behind a constructor; the arena todo can later restrict construction if desired.
- Only the five required derives plus `Debug` are added. `Ord`/`PartialOrd`/`Default` are not added because the spec did not list them; later items can extend.
- No `impl NodeId` block (no `DUMMY`, `new`, `as_u32`, etc.) is added in this commit -- those belong with the arena/error-recovery work that consumes them.
- The module is added to `lib.rs` even though nothing uses it yet; otherwise the file would not compile-check. A `#[allow(dead_code)]` is *not* added because `pub` visibility on both the type and field already suppresses unused warnings in a library crate.

## Blockers
Blockers: none

## Summary
Introduce the `vertex_stage0::ast` module containing the `NodeId(u32)` newtype that subsequent AST and parser items will reference.

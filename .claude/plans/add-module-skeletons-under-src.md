# Plan: add-module-skeletons-under-src

## Goal
Create empty module skeletons (`lexer`, `parser`, `resolve`, `typecheck`, `mir`, `codegen`, plus `error.rs`, `span.rs`, `util.rs`) under the crate's `src/` and wire them into `lib.rs` so the crate still builds.

## Steps
1. Create six module directories each containing an empty `mod.rs`: `vertex_stage0/src/lexer/mod.rs`, `vertex_stage0/src/parser/mod.rs`, `vertex_stage0/src/resolve/mod.rs`, `vertex_stage0/src/typecheck/mod.rs`, `vertex_stage0/src/mir/mod.rs`, `vertex_stage0/src/codegen/mod.rs`.
2. Create three empty leaf module files: `vertex_stage0/src/error.rs`, `vertex_stage0/src/span.rs`, `vertex_stage0/src/util.rs`.
3. Update `vertex_stage0/src/lib.rs` to declare each module via `pub mod <name>;` while preserving the existing `pub fn run() {}` entry point used by `main.rs`.
4. Run `cargo build` from the crate directory to confirm everything still compiles.

## Files
- `vertex_stage0/src/lexer/mod.rs` -- new, empty
- `vertex_stage0/src/parser/mod.rs` -- new, empty
- `vertex_stage0/src/resolve/mod.rs` -- new, empty
- `vertex_stage0/src/typecheck/mod.rs` -- new, empty
- `vertex_stage0/src/mir/mod.rs` -- new, empty
- `vertex_stage0/src/codegen/mod.rs` -- new, empty
- `vertex_stage0/src/error.rs` -- new, empty
- `vertex_stage0/src/span.rs` -- new, empty
- `vertex_stage0/src/util.rs` -- new, empty
- `vertex_stage0/src/lib.rs` -- add nine `pub mod` declarations; keep `pub fn run() {}`

## Risks
- The todo refers to `src/` at the repo root, but the only crate lives at `vertex_stage0/`. Creating a stray top-level `src/` would not be picked up by Cargo and would be dead weight; placing modules inside the existing crate is the only interpretation that makes `cargo build` meaningful.
- Empty `.rs` files are valid Rust (zero items) and will not break the build. No `#![deny(missing_docs)]` or similar lints exist in `lib.rs` today, so empty modules compile cleanly.
- `main.rs` calls `vertex_stage0::run()`; keeping `run` in `lib.rs` preserves the binary.

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
test -f vertex_stage0/src/lexer/mod.rs
test -f vertex_stage0/src/parser/mod.rs
test -f vertex_stage0/src/resolve/mod.rs
test -f vertex_stage0/src/typecheck/mod.rs
test -f vertex_stage0/src/mir/mod.rs
test -f vertex_stage0/src/codegen/mod.rs
test -f vertex_stage0/src/error.rs
test -f vertex_stage0/src/span.rs
test -f vertex_stage0/src/util.rs
grep -q "pub mod lexer" vertex_stage0/src/lib.rs
grep -q "pub mod parser" vertex_stage0/src/lib.rs
grep -q "pub mod resolve" vertex_stage0/src/lib.rs
grep -q "pub mod typecheck" vertex_stage0/src/lib.rs
grep -q "pub mod mir" vertex_stage0/src/lib.rs
grep -q "pub mod codegen" vertex_stage0/src/lib.rs
grep -q "pub mod error" vertex_stage0/src/lib.rs
grep -q "pub mod span" vertex_stage0/src/lib.rs
grep -q "pub mod util" vertex_stage0/src/lib.rs
```

## Assumptions
- "src/" in the todo means the crate's source directory, i.e. `vertex_stage0/src/`, since that is the only Rust source tree in the repo (commit `8a342dc` scaffolded `vertex_stage0` as the crate root).
- "Empty" means zero-byte (or near-zero) files with no content; lints/warnings on unused empty modules will not fail `cargo build` because the existing crate has no `deny(warnings)` or `deny(dead_code)` configuration.
- `pub fn run() {}` must remain in `lib.rs` because `main.rs` calls it; the new `pub mod` lines are added alongside it, not in place of it.
- Module directories use the `mod.rs` convention (as the sub-step explicitly specifies), not the Rust 2018 `lexer.rs` + `lexer/` style.
- Sub-modules are declared `pub mod` (matching the sub-step wording) so they remain externally accessible from the binary and any future tests.

## Blockers
Blockers: none

## Summary
Lay down the empty module tree (`lexer`, `parser`, `resolve`, `typecheck`, `mir`, `codegen`, `error`, `span`, `util`) inside `vertex_stage0/src/` and wire it into `lib.rs` so the compiler skeleton has a place to grow.

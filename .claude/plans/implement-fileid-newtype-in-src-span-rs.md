# Plan: implement-fileid-newtype-in-src-span-rs

## Goal
Add a `FileId(pub u32)` newtype with the standard derive set in the stage-0 compiler's `span` module so later items can index source files.

## Steps
1. Open `vertex_stage0/src/span.rs` (currently empty) and add `pub struct FileId(pub u32);` with derives `Copy, Clone, PartialEq, Eq, Hash, Debug`.
2. Confirm the crate still builds via `cargo build`.

## Files
- `vertex_stage0/src/span.rs` -- add the `FileId` newtype declaration with the required derives. No other items added; module already wired in `vertex_stage0/src/lib.rs:8`.

## Risks
- The todo references the path `src/span.rs`, but the actual file lives under `vertex_stage0/src/span.rs` (the only Cargo crate in the workspace). Editing the wrong path would no-op. Mitigated by editing the real file and adjusting the verify path accordingly.
- `Hash` derive ordering vs other derives doesn't affect correctness, but the spec lists a specific order; I'll preserve the listed order.

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
grep -q 'pub struct FileId' vertex_stage0/src/span.rs
```

## Assumptions
- "src/span.rs" in the todo refers to `vertex_stage0/src/span.rs` since that is the only crate in the repo and the only existing `span.rs`.
- Field is a single tuple field `pub u32` (matches the spec literal `pub struct FileId(pub u32);`).
- Derive list is exactly `#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]` in the order given.
- No additional impls (constructors, `Display`, etc.) are required for this item; later items will add them as needed.
- No new dependencies in `Cargo.toml`; `u32` and the listed derives are core/std.

## Blockers
Blockers: none

## Summary
Introduce the `FileId(u32)` newtype with standard derives in `vertex_stage0/src/span.rs` so future span/source-map work has a stable file handle.

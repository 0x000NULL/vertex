Here's the plan.

# Plan: define-errorcode-and-errorkind-in-src-error-rs

## Goal
Populate the empty `vertex_stage0/src/error.rs` with an `ErrorCode(pub u32)` newtype carrying associated-const range markers (`E0001..E1999` partitioned across lex/syntax/resolve/type/borrow/other) and an `ErrorKind` enum covering those same six diagnostic categories.

## Steps
1. Open `vertex_stage0/src/error.rs` (currently a 1-line empty file) and write the full module body.
2. Define `pub struct ErrorCode(pub u32)` with derives `Copy, Clone, PartialEq, Eq, Hash, Debug` (matching `FileId` style in `span.rs`).
3. On `impl ErrorCode`, declare `pub const` range markers spanning E0001..E1999. Use a six-band partition (each band = 333 codes, last band absorbs remainder) so every `ErrorKind` has a contiguous `_START`/`_END` pair:
   - `LEXICAL_START = ErrorCode(1)`, `LEXICAL_END = ErrorCode(333)`
   - `SYNTAX_START = ErrorCode(334)`, `SYNTAX_END = ErrorCode(666)`
   - `NAME_RESOLUTION_START = ErrorCode(667)`, `NAME_RESOLUTION_END = ErrorCode(999)`
   - `TYPE_START = ErrorCode(1000)`, `TYPE_END = ErrorCode(1332)`
   - `BORROW_CHECK_START = ErrorCode(1333)`, `BORROW_CHECK_END = ErrorCode(1665)`
   - `OTHER_START = ErrorCode(1666)`, `OTHER_END = ErrorCode(1999)`
4. Define `pub enum ErrorKind { Lexical, Syntax, NameResolution, Type, BorrowCheck, Other }` with derives `Copy, Clone, PartialEq, Eq, Hash, Debug`.
5. Run `cargo build -p vertex_stage0` to confirm the new module compiles cleanly with no warnings introduced.

## Files
- `vertex_stage0/src/error.rs` -- replace empty contents with `ErrorCode` newtype, six pairs of range constants spanning E0001..E1999, and the `ErrorKind` enum.

## Risks
- The todo string `src/error.rs` is shorthand; the actual path is `vertex_stage0/src/error.rs` (single-crate workspace). Editing the wrong path would silently no-op.
- `cargo build` for a library item with only type definitions will emit `dead_code` warnings if the crate's lint config is strict (`-D warnings`). Mitigation: no `#![deny(warnings)]` is currently set in `lib.rs`, so plain `cargo build` should pass; the verify step uses `cargo build` (not `cargo build -- -D warnings`).
- Choosing exact numeric boundaries for the six bands is somewhat arbitrary; downstream items that allocate concrete codes (e.g., `E0101`) must sit inside the band declared here. An even six-way split keeps every band non-empty and leaves room.

## Verify
```
cargo build -p vertex_stage0
grep -q 'pub struct ErrorCode' vertex_stage0/src/error.rs
grep -q 'pub enum ErrorKind' vertex_stage0/src/error.rs
```

## Assumptions
- "src/error.rs" in the todo refers to `vertex_stage0/src/error.rs` (the only crate in this workspace; verified by `Cargo.toml` and the existing empty `error.rs`).
- The `E0001..E1999` range is inclusive of both endpoints, partitioned into six bands; an even split (≈333 codes/band, last band gets the remainder up to 1999) is acceptable since the todo only specifies the overall range and the six categories, not per-band sizes.
- Range markers are exposed as associated `pub const` items on `ErrorCode` (e.g., `ErrorCode::LEXICAL_START`) rather than free constants -- this matches Rust idiom and keeps the namespace tidy.
- `ErrorKind` and `ErrorCode` derive the standard small-value derives (`Copy, Clone, PartialEq, Eq, Hash, Debug`) consistent with `FileId` in `span.rs`. No `serde`, `Display`, or `thiserror` integration is added -- not requested by this todo.
- No `pub use` re-export is added to `lib.rs`; `error` is already declared as `pub mod error` so `vertex_stage0::error::ErrorCode` is reachable.
- No conversion/helper methods (e.g., `ErrorCode::kind() -> ErrorKind`) are added; the todo lists only the type definitions and the verify greps only assert the type declarations exist. Helpers belong to a follow-up item if needed.

## Blockers
Blockers: none

## Summary
Adds the `ErrorCode` newtype with six range constants covering E0001..E1999 and the six-variant `ErrorKind` enum to the previously empty `vertex_stage0/src/error.rs`, giving the diagnostics subsystem its foundational types.

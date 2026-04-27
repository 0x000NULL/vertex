# Plan: define-errorcode-and-errorkind-in-src-error-rs

## Goal
Flesh out the `ErrorCode` newtype with category-range associated constants (E0001–E1999) and confirm the `ErrorKind` enum covers all six categories in `vertex_stage0/src/error/mod.rs`.

## Steps
1. Open `vertex_stage0/src/error/mod.rs` and confirm the existing `pub struct ErrorCode(pub u32);` newtype and `pub enum ErrorKind { Lexical, Syntax, NameResolution, Type, BorrowCheck, Other }` enum (already present from prior work).
2. Expand the `impl ErrorCode` block to add representative associated constants spanning all six category ranges per `compiler_architecture.md` §"Error Code System":
   - Lexical (E0001–E0099): `E0001` (invalid character), `E0002` (unterminated string), `E0003` (invalid numeric literal).
   - Syntax (E0100–E0299): `E0100` (unexpected token), `E0101` (unclosed delimiter), `E0102` (missing semicolon).
   - Name resolution (E0300–E0499): `E0425` (unresolved name), `E0433` (failed to resolve import); keep `E0308` co-located with type errors below since it's a type code per the architecture doc.
   - Type (E0500–E0799): retain `E0308` (type mismatch — note: lives outside the 500–799 band but is the canonical rustc-style code; keep as-is), add `E0369` (binop not supported), `E0277` (trait bound), `E0599` (method not found), `E0608` (string index).
   - Borrow check (E0800–E0999): retain `E0502`; add `E0382` (moved value), `E0499` (mut-borrow twice), `E0503`, `E0505`.
   - Other (E1000–E1999): add `E1000` (placeholder/internal), `E1001` (const eval failed), `E1002` (unsafe in const).
3. Add a short doc comment above each constant indicating its meaning (the `--explain` subcommand, planned later, will key off these). Keep names matching the rustc-style `E####` literal exactly so the `explain E0xxx` subcommand can do a string match.
4. Leave `ErrorKind` untouched (already complete: six variants).
5. Run `cargo build` and `cargo fmt` to confirm the additions compile and stay tidy. The existing `render.rs` consumers already reference `ErrorCode::E0308` / `E0502` and must continue to compile unchanged.

## Files
- `vertex_stage0/src/error/mod.rs` -- add associated `ErrorCode` constants for the lex/syntax/resolve/type/borrow/other ranges; preserve the already-present `ErrorKind` enum and `CompileError` struct.

## Risks
- The todo description says `src/error.rs`, but the actual file lives at `vertex_stage0/src/error/mod.rs` (a directory module). Editing `mod.rs` is the correct location; don't create a duplicate `src/error.rs` file or it'll shadow/conflict with the directory module.
- The `E0308` / `E0502` constants already exist; re-declaring them would be a compile error. Add only NEW constants, leaving the existing two intact.
- Vertex's architecture doc places `E0308` (type mismatch) and `E0277` (trait bound) outside their nominal ranges to mirror rustc; resist the urge to renumber them.
- `pub u32` newtype is already specified — don't switch to `u16` even though `compiler_architecture.md` shows `u16` in pseudo-code; the existing field type and `with_label` callers depend on `u32`.

## Prereqs
Prereqs: none

(The companion item `define-compileerror-struct-in-src-error-rs` consumes `ErrorCode`/`ErrorKind`, so this plan is its prerequisite — not the other way around. The `CompileError` struct already happens to coexist in the same file, but the todo run will treat that as a separate item; nothing to depend on here.)

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
cargo fmt --manifest-path vertex_stage0/Cargo.toml -- --check
grep -q 'pub struct ErrorCode' vertex_stage0/src/error/mod.rs
grep -q 'pub enum ErrorKind' vertex_stage0/src/error/mod.rs
grep -q 'E0001' vertex_stage0/src/error/mod.rs
grep -q 'E0100' vertex_stage0/src/error/mod.rs
grep -q 'E0382' vertex_stage0/src/error/mod.rs
grep -q 'E1000' vertex_stage0/src/error/mod.rs
grep -q 'Lexical' vertex_stage0/src/error/mod.rs
grep -q 'BorrowCheck' vertex_stage0/src/error/mod.rs
```

## Assumptions
- "in `src/error.rs`" in the todo refers to the canonical error module, which currently lives at `vertex_stage0/src/error/mod.rs`. Editing `mod.rs` (rather than collapsing the dir back to a single file) preserves the existing `error::render` submodule and its tests.
- The "associated consts E0001..E1999 ranges" sub-step asks for *representative* constants across ranges, not all 1999 codes. I'm picking ~3–5 per range, matching codes already enumerated in `compiler_architecture.md` so future error-emission code has well-known names to reach for.
- Keep the existing `E0308` and `E0502` constants exactly as-is so `error/render.rs` tests keep passing without touching them.
- Constants stay on `ErrorCode` (associated `pub const`) rather than turning into a separate enum; the existing newtype shape is the contract referenced by `CompileError::new(code, …)` and its callers.
- No need to add `Display`/`Debug` impls beyond the existing `derive`s; the `--explain` subcommand item will own that wiring later.
- No `ErrorKind` changes required — all six variants (`Lexical, Syntax, NameResolution, Type, BorrowCheck, Other`) already exist verbatim from the prior commit.

## Blockers
Blockers: none

## Summary
Expand `ErrorCode` with category-range associated constants spanning lex/syntax/resolve/type/borrow/other; `ErrorKind` already lists all six variants and stays as-is.

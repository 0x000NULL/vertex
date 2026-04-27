# Plan: add-literal-variants-to-tokenkind

## Goal
Extend `TokenKind` with the five literal variants (`IntLiteral`, `FloatLiteral`, `CharLiteral`, `StringLiteral`, `RawStringLiteral`) and define the supporting `IntSuffix` / `FloatSuffix` enums in the same file.

## Steps
1. In `vertex_stage0/src/lexer/token.rs`, add `IntSuffix` enum with variants `I8, I16, I32, I64, ISize, U8, U16, U32, U64, USize, Unsuffixed`, deriving `Debug, Clone, Copy, PartialEq, Eq, Hash`.
2. Add `FloatSuffix` enum with variants `F32, F64, Unsuffixed`, deriving `Debug, Clone, Copy, PartialEq, Eq, Hash`.
3. Adjust `TokenKind`'s derives: drop `Copy`, `Eq`, and `Hash` (because `String` is not `Copy` and `f64` is neither `Eq` nor `Hash`); keep `Debug, Clone, PartialEq`.
4. Append the five literal variants to `TokenKind`: `IntLiteral(u64, IntSuffix)`, `FloatLiteral(f64, FloatSuffix)`, `CharLiteral(char)`, `StringLiteral(String)`, `RawStringLiteral(String)`.
5. Run `cargo build -p vertex_stage0` to confirm everything compiles (no downstream code references the dropped `Copy`/`Eq`/`Hash` derives yet — `lib.rs` only exposes the module tree).

## Files
- `vertex_stage0/src/lexer/token.rs` — add `IntSuffix` and `FloatSuffix` enums; adjust `TokenKind` derives; add five literal variants.

## Risks
- Adding `String` and `f64` payloads forces dropping `Copy`/`Eq`/`Hash` from `TokenKind`. If any later (not-yet-written) code assumes `TokenKind: Copy` or `TokenKind: Eq`, it will need updating — but no current code references `TokenKind` besides the module declaration, so this is safe today.
- `f64` payload means `PartialEq` only (no `Eq`); downstream tests that want set/hash semantics will need to wrap or compare carefully.

## Prereqs
- define-tokenkind-enum-in-src-lexer-token-rs-keyword-variants

## Verify
```
cargo build -p vertex_stage0
grep -q 'IntLiteral' vertex_stage0/src/lexer/token.rs
grep -q 'FloatLiteral' vertex_stage0/src/lexer/token.rs
grep -q 'CharLiteral' vertex_stage0/src/lexer/token.rs
grep -q 'StringLiteral' vertex_stage0/src/lexer/token.rs
grep -q 'RawStringLiteral' vertex_stage0/src/lexer/token.rs
grep -q 'enum IntSuffix' vertex_stage0/src/lexer/token.rs
grep -q 'enum FloatSuffix' vertex_stage0/src/lexer/token.rs
```

## Assumptions
- The verify path in the spec (`src/lexer/token.rs`) is shorthand; the real file lives at `vertex_stage0/src/lexer/token.rs` per the existing crate layout, so verify uses the actual path.
- `IntLiteral` carries `u64` as the unsigned bit-pattern of the parsed integer; sign handling belongs to a unary-minus expression, not the token (matches how Rust's lexer works).
- `IntSuffix`/`FloatSuffix` `Unsuffixed` represents literals written without a type suffix (e.g., `42` vs `42i32`); the literal scanner will later set this for plain numeric literals.
- It is acceptable to drop `Copy`, `Eq`, and `Hash` from `TokenKind`'s derive list because no code currently relies on them (the only reference to the type is `pub mod token;` from `mod.rs`); `Debug, Clone, PartialEq` is the maximal compatible set given the new payloads.
- `StringLiteral`/`RawStringLiteral` hold the *unescaped* content as `String` (raw keeps bytes verbatim); deciding the exact representation is the scanner's job, but the type is `String` either way.
- `CharLiteral` holds a `char` (a single Unicode scalar value), matching Rust semantics.

## Blockers
Blockers: none

## Summary
Lays down the literal half of `TokenKind` plus the two suffix enums so subsequent literal-scanning steps have concrete variants to emit.

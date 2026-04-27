# Plan: define-tokenkind-enum-in-src-lexer-token-rs-keyword-variants

## Goal
Create `vertex_stage0/src/lexer/token.rs` defining a `pub enum TokenKind` with the 32 keyword variants enumerated in the todo, and wire the new module into `lexer/mod.rs`.

## Steps
1. Create `vertex_stage0/src/lexer/token.rs` with `pub enum TokenKind` containing exactly the variants listed in the todo: `Break, Const, Continue, Else, Enum, Extern, False, Fn, For, If, Impl, In, Let, Loop, Match, Mod, Mut, Not, Or, Pub, Return, SelfLower, SelfUpper, Struct, Trait, True, Type, Unsafe, Use, Where, While, And`.
2. Derive `Debug, Clone, Copy, PartialEq, Eq, Hash` on the enum so downstream lexer/parser code can compare and pattern-match efficiently.
3. Add `pub mod token;` to `vertex_stage0/src/lexer/mod.rs` (currently empty) so the new file is reachable from the crate root.
4. Build the workspace to confirm the enum compiles cleanly with no warnings.

## Files
- `vertex_stage0/src/lexer/token.rs` -- new file; declares `pub enum TokenKind { ... }` with the 32 listed keyword variants and standard derives.
- `vertex_stage0/src/lexer/mod.rs` -- add `pub mod token;` so the new submodule is exposed.

## Risks
- Variant naming: spec uses lowercase `self`/`Self` but Rust enum variants must be `UpperCamelCase`; the todo already prescribes `SelfLower`/`SelfUpper`, which I will use verbatim.
- The todo description says "29 keyword variants" but actually lists 32 names (including `Not`, `Or`, `And` which the spec classifies as logical operators). I will follow the explicit variant list, not the count in the prose.
- `defer` and `static` appear in the spec's grammar but are *not* in the todo's enumeration; omitting them now means later items must add them. Following the todo's literal list keeps this commit scoped.
- Adding only keyword variants (no literals/idents/operators) means the enum cannot yet be used to tokenize a real source file — that is intentional; later sibling items add the remaining variants.

## Prereqs
Prereqs: none

## Verify
```
cargo build -p vertex_stage0
grep -q 'pub enum TokenKind' vertex_stage0/src/lexer/token.rs
test -f vertex_stage0/src/lexer/token.rs
grep -q 'pub mod token' vertex_stage0/src/lexer/mod.rs
```

## Assumptions
- The deliverable path `src/lexer/token.rs` in the todo refers to `vertex_stage0/src/lexer/token.rs` (the only crate in the workspace; `vertex_stage0/src/lexer/` already exists with an empty `mod.rs`).
- I must follow the explicit variant list in the todo rather than the numerically-stated count or the spec's slightly different keyword set; later items will reconcile any gaps.
- Standard derives (`Debug, Clone, Copy, PartialEq, Eq, Hash`) are appropriate for a fieldless enum and will not conflict with later additions of literal/identifier variants (which can add per-variant data and may force dropping `Copy` later — but that's a future item's concern).
- No new dependencies are required; this is plain Rust code in an existing crate.
- `cargo build -p vertex_stage0` is the right verify because the todo's spec line said `cargo build`; scoping to the package keeps it fast and avoids touching unrelated workspace members.

## Blockers
Blockers: none

## Summary
Lay down the keyword-only first slice of `TokenKind` in a new `lexer/token.rs`, wired into `lexer/mod.rs`, so subsequent lexer items can extend it with literals, identifiers, and operators.

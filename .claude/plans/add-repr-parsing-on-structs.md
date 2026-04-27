# Plan: add-repr-parsing-on-structs

## Goal
Recognize `#[repr(C)]` and `#[repr(transparent)]` preceding a struct item, and surface the repr name on `StructDef` (AST only, no validation).

## Steps
1. Extend `StructDef` (in `src/ast/item.rs`) with a new field `repr: Option<String>`, defaulting to `None`. Update any `StructDef { .. }` construction site so the new field is populated (currently only in `parser::item::parse_struct`).
2. In `src/parser/item.rs`, teach `parse_struct` to consume zero or more leading attribute groups of shape `# [ Ident ( Ident ) ]` *before* the `struct` keyword (not before any pre-existing visibility/modifier handling — there is none on structs today). The attribute loop should:
   - peek for `Pound` (the lexer token added by the `add-attribute-parsing` prereq);
   - consume `Pound LBracket Ident LParen Ident RParen RBracket`;
   - if the outer ident is `"repr"`, capture the inner ident as the struct's `repr` (last-write-wins if multiple `repr` attributes appear);
   - silently ignore any non-`repr` attribute name (general attribute storage belongs to the `add-attribute-parsing` item — this plan only mirrors the `repr` value into `StructDef.repr`);
   - track the leading attribute span so the final `StructDef.span` covers from the first `#` to the existing `end_span`.
3. Initialize `repr: None` on the unit/tuple/record paths and overwrite from the captured value at the end of `parse_struct` before constructing the `StructDef`.
4. Add a `parser::item::tests::struct_repr` unit test in the existing `tests` module of `src/parser/item.rs` covering at least:
   - `#[repr(C)] struct Foo { x: i32 }` → `s.repr == Some("C")`, `kind == Record`, one field;
   - `#[repr(transparent)] struct Bar(i32);` → `s.repr == Some("transparent")`, `kind == Tuple`, one field;
   - a control case (`struct Baz;`) confirming `repr` defaults to `None`;
   - assert `p.errors.is_empty()` and the parser is positioned at `Eof` after each.

## Files
- `vertex_stage0/src/ast/item.rs` — add `pub repr: Option<String>` field on `StructDef`.
- `vertex_stage0/src/parser/item.rs` — recognize `#[repr(<ident>)]` attributes before `struct`, store on `StructDef.repr`, and add the `struct_repr` unit test.

## Risks
- The lexer currently emits `Error("invalid character: #")` for `#` (see `scan.rs:2374`), so without a `Pound` token in `TokenKind`, this plan cannot be tested with raw source. Tests build `Token` vectors directly, so they will use the new `Pound` token name added by the `add-attribute-parsing` prereq; if that prereq spells the token differently (e.g. `Hash`), the test and parser code need to follow suit.
- Modifying `StructDef`'s shape touches every match arm and constructor; today the only construction site is `parse_struct`, but any future MIR/typecheck stubs that destructure `StructDef` will need updating. The `#[allow(dead_code)]` on `StructDef` already accommodates the unused field.
- Span merging if multiple attributes precede the struct — pick the first attribute's `Pound` span as the leading anchor and merge through `end_span`.

## Prereqs
- add-attribute-parsing

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::struct_repr
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The `add-attribute-parsing` prereq will introduce a `TokenKind::Pound` (or equivalent `#`) variant in `src/lexer/token.rs` and update `src/lexer/scan.rs` so `#` lexes as that token rather than the current `Error` variant. This plan does *not* duplicate that lexer work.
- `repr` is stored as a plain `Option<String>` (not a typed enum) because the spec sub-step says "AST only — no semantic checks"; semantic validation of which `repr` names are legal lives in a later semantic pass.
- Only one inner ident is supported (`#[repr(C)]`, `#[repr(transparent)]`); forms like `#[repr(C, packed)]` or `#[repr(packed(2))]` are out of scope for this item — they are not in the spec sub-step's enumerated list.
- Last-write-wins on duplicate `#[repr(...)]` attributes; no diagnostic emitted (this matches the "AST only" framing). If a stricter rule is needed, it lands with `add-attribute-parsing` or the future validate pass.
- Non-`repr` attributes appearing before a struct are silently consumed by this item's attribute loop. The general attribute storage (e.g. an `attrs: Vec<Attribute>` on items) is owned by the `add-attribute-parsing` item — once that lands, this plan's loop should be merged into the shared attribute helper rather than doing its own consumption. This plan keeps the recognition local for now to avoid speculating on the prereq's API surface.
- The new test follows the existing pattern in `parser::item::tests` of constructing `Token` vectors directly via the `tok` / `ident_tok` helpers, so it does not depend on any source-string lexer behavior.

## Blockers

### Blocker: prereq not landed
- severity: cross-item
- affects: add-attribute-parsing, lexer Pound token, attribute AST shape
- question: Has `add-attribute-parsing` already added a `Pound` (or `Hash`) `TokenKind` and a basic `Attribute` AST node, or is this item expected to add the lexer support itself?
- default_assumption: Proceed assuming `add-attribute-parsing` runs first and provides `TokenKind::Pound`. If it has not, the execute phase will additionally (a) add `TokenKind::Pound` to `src/lexer/token.rs`, (b) emit it from `src/lexer/scan.rs` for `#` (replacing the current `Error("invalid character: #")` path), and (c) update the `builtin_derive_attr` lexer test to expect the new token. Tests in this plan use the `Pound` token directly via `tok(TokenKind::Pound)` regardless.

### Blocker: scope of attribute consumption
- severity: local
- affects: parse_struct, future general attribute parsing
- question: Should `parse_struct` also consume non-`repr` attributes (silently dropping them) so it can survive code with mixed attributes, or should it bail/error on any non-`repr` attribute?
- default_assumption: Silently consume any leading `#[Ident(Ident)]` attribute, only mirroring the value when the outer ident is `repr`. This keeps the parser tolerant and avoids tripping on `#[derive(...)]` etc. that other items will later add proper handling for.

## Summary
Adds `StructDef.repr: Option<String>` and teaches `parse_struct` to recognize `#[repr(C)]` / `#[repr(transparent)]` preceding the struct, locked in by a new `parser::item::tests::struct_repr` unit test.

# Plan: parse-normal-struct-items

## Goal
Add `Parser::parse_struct` that recognizes `struct Name<T> { field: Ty, pub field2: Ty }`, recording field-level `pub` visibility and optional generics on a widened `StructDef`, and pin it with a `struct_normal` unit test.

## Steps
1. In `vertex_stage0/src/ast/item.rs`, define a new `Field { id: NodeId, span: Span, name: String, ty: Type, is_pub: bool }` struct (Debug + Clone, behind `#[allow(dead_code)]` like the others) and extend `StructDef` to carry `name: String`, `generics: Option<Generics>`, `fields: Vec<Field>` alongside the existing `id`/`span`. The `Item::Struct(i) => i.id`/`.span` arms keep working since they bind by name.
2. In `vertex_stage0/src/parser/item.rs`, add `pub fn parse_struct(&mut self) -> Result<Item, CompileError>`:
   - expect `Struct`; expect `Ident` for the name (capturing start span from the `struct` kw).
   - if `peek() == Lt`, call existing `parse_generics_params` and remember the list span.
   - expect `LBrace`.
   - field loop: stop at `RBrace`; otherwise read optional `Pub` (record `is_pub`), expect `Ident`, expect `Colon`, call existing `parse_type`, push `Field`; consume optional `Comma`; if no comma, break (then expect `RBrace`).
   - expect `RBrace`; capture end span.
   - build `Option<Generics>` only when the generics list span exists (no `where` clause for normal structs in this plan); merge spans for the outer `StructDef.span`.
3. Add a `struct_normal` test mirroring the in-memory token approach of `plain_fn`/`fn_modifiers`: tokens for `struct Name < T > { field : Ty , pub field2 : Ty }`, then assert `def.name == "Name"`, generics has one `T` param with empty bounds, `fields.len() == 2`, `fields[0]` = `field: Ty` with `!is_pub`, `fields[1]` = `field2: Ty` with `is_pub`, and `p.errors.is_empty()` with `peek() == Eof`.

## Files
- `vertex_stage0/src/ast/item.rs` -- add `Field`; widen `StructDef` with `name`, `generics`, `fields`.
- `vertex_stage0/src/parser/item.rs` -- add `parse_struct` and `parser::item::tests::struct_normal`.

## Risks
- The stopgap `parse_type` here only accepts a single bare ident path — fields like `Vec<T>` or `&T` won't parse. Acceptable: the spec snippet uses bare `Ty`, and richer types are deferred to `parse-path-types-with-generic-args`.
- Field visibility is stored as `is_pub: bool`; `add-visibility-pub-pub-crate-pub-super-pub-in-path` will need to migrate it to a `Visibility` enum (and probably also expand to top-level item visibility).
- `parse_generics_params` shares the `>>`-as-`Shr` limitation noted in `parse_fn`; same limitation applies here and lifts when path-types land.
- Widening `StructDef` is a tiny ABI change inside the workspace; `Item::Struct(_)` consumers only access `id`/`span` today and continue to compile.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib parser::item::tests::struct_normal
```

## Assumptions
- Field visibility is recorded as `is_pub: bool` to mirror the existing `is_const`/`is_unsafe` style on `FnDef`; the future visibility plan will replace it with a `Visibility` enum.
- No struct-level `where` clause, no top-level `pub`, no attributes (`#[derive(...)]`/`#[repr(...)]`) handled here — each is its own pending item.
- Trailing commas inside the field list are accepted (standard form, matches how `parse_fn` accepts trailing commas in params).
- The test uses synthetic in-memory tokens (no lexer round-trip), matching the file's existing test helpers (`tok`, `ident_tok`).
- The new `Field` struct does not need to participate in `Item::id()`/`Item::span()`; only `StructDef`'s own `id`/`span` are exposed there.
- `parse_struct` is added but not wired into a top-level item dispatcher in this plan (no such dispatcher exists yet — `parse_fn` is also called directly from tests).

## Blockers
Blockers: none

## Summary
Extend `StructDef` with `name`/`generics`/`fields` (plus a new `Field` AST node carrying `is_pub`) and add a minimal `parse_struct` so normal record-style structs with optional generics and per-field `pub` round-trip into AST, locked in by a `struct_normal` unit test.

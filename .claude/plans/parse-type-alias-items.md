# Plan: parse-type-alias-items

## Goal
Promote `TypeAliasDef` to a real AST node and add `parse_type_alias` so `type Alias<T> = ConcreteTy;` round-trips into the AST, locked in by a `type_alias` unit test.

## Steps
1. In `vertex_stage0/src/ast/item.rs`, promote `TypeAliasDef` to carry real payload: keep `id` and `span`, add `name: String`, `generics: Option<Generics>`, and `ty: Type` (the aliased right-hand-side type). Keep the `#[allow(dead_code)]` and `#[derive(Debug, Clone)]` attributes consistent with sibling defs (`ConstDef`, `StaticDef`).
2. In `vertex_stage0/src/parser/item.rs`, add `pub fn parse_type_alias(&mut self) -> Result<Item, CompileError>` modeled on `parse_const` / `parse_static`:
   - `expect(TokenKind::Type)` to consume `type`; remember its span as the start.
   - `expect` an identifier for the alias name.
   - If `peek() == Lt`, call the existing `parse_generics_params` and build a `Generics` (no where-clause for now — same scope choice already made for `parse_struct` / `parse_enum`).
   - `expect(Eq)`, then call the stopgap `parse_type` to parse the right-hand-side type.
   - `expect(Semi)` and merge spans from `start_span` to the semicolon span.
   - Allocate a `NodeId` and return `Item::TypeAlias(TypeAliasDef { id, span, name, generics, ty })`.
3. Update the imports at the top of `parser/item.rs` to bring `TypeAliasDef` into scope alongside the other item types.
4. Add a `#[test] fn type_alias()` in the existing `mod tests` of `parser/item.rs` that:
   - Drives `parse_type_alias` on tokens for `type Alias = i32;` and asserts `name == "Alias"`, `generics.is_none()`, the RHS type ident is `"i32"`, no errors, and parser is at `Eof`.
   - Drives `parse_type_alias` on tokens for `type Alias<T> = T;` and asserts `name == "Alias"`, `generics.is_some()` with one type param named `"T"`, RHS type ident is `"T"`, no errors, parser at `Eof`.
   - Mirrors helper conventions already used by `const_item` / `static_item` (e.g. an `as_type_alias` extractor).

## Files
- `vertex_stage0/src/ast/item.rs` -- expand `TypeAliasDef` fields (`name`, `generics`, `ty`); imports of `Generics` / `Type` are already in scope.
- `vertex_stage0/src/parser/item.rs` -- add `parse_type_alias` method; extend the `use crate::ast::item::{...}` list with `TypeAliasDef`; add `type_alias` test plus an `as_type_alias` helper inside the existing `tests` module.

## Risks
- The `TokenKind::Type` keyword is already a token (used by `TraitItemType` in `parse_trait`); adding a top-level `parse_type_alias` does not collide because dispatch into top-level items is handled by the eventual file-level item parser, not yet wired here. The unit test calls `parse_type_alias` directly, so no dispatcher change is required.
- The stopgap `parse_type` only accepts a single bare ident (with no generic args); `type Alias<T> = Vec<T>;` would not parse. The sub-step's example uses `ConcreteTy` (a bare ident), and the test cases above stay within stopgap territory, so this is fine for now and will be subsumed by `parse-path-types-with-generic-args`.
- The stopgap `parse_generics_params` handles only `<T>`-style params with optional bounds and shares the `>>` lexing limitation with sibling items; consistent with how `parse_struct` and `parse_enum` behave today.
- `TypeAliasDef`'s `ty` field is `Type`, not `Option<Type>`. Rust permits associated-type style `type Alias;` only inside traits — that path is already handled by `TraitItemType`, so the standalone item form is required to have `= <type>`.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::type_alias
```

## Assumptions
- `TypeAliasDef` is promoted to `{ id, span, name, generics: Option<Generics>, ty: Type }`, paralleling how `ConstDef` and `StaticDef` were promoted in the two preceding commits.
- Where-clauses on type aliases are deferred (matches how struct/enum currently skip them); only the `<T>`-style param list is recognized.
- No dispatcher wiring (e.g. into `parse_mod_inline_item`) is part of this item — the spec is "parse type-alias items" plus the unit test, both of which can be exercised by calling `parse_type_alias` directly. Dispatcher updates can ride later items if needed.
- The `cargo test --lib parser::item::tests::type_alias` command was given as `cargo test --lib ...`; I'm running it via `--manifest-path vertex_stage0/Cargo.toml` because the crate sits in the `vertex_stage0/` subdirectory rather than the repo root.
- The new test lives inline in `src/parser/item.rs`'s existing `mod tests`, consistent with `const_item` and `static_item` (rather than in `tests/`).

## Blockers
Blockers: none

## Summary
Promote `TypeAliasDef` to a real AST node and add a stopgap `parse_type_alias` covering `type Alias = T;` and `type Alias<T> = T;`, locked in by a single `type_alias` unit test.

# Plan: add-generics-and-where-clauses-to-function-items

## Goal
Extend `parse_fn` to accept an optional `<T, U[: bounds]>` generic-parameter list after the function name and an optional `where T: Bound + Bound` clause between the return type and the body, recording both on `FnDef`.

## Steps
1. Give `FnDef` an `Option<Generics>` field. In `vertex_stage0/src/ast/item.rs`, add `pub generics: Option<Generics>` to `FnDef` (importing `crate::ast::generics::Generics`). Update every `Item::Fn(FnDef { ... })` construction site to set the new field — currently only `parser::item::parse_fn` builds `FnDef`.
2. In `vertex_stage0/src/parser/item.rs`, add a private helper `parse_trait_bound(&mut self) -> Result<TraitBound, CompileError>` that consumes a single identifier as the bound path (single-segment, no generic args) using the same single-ident pattern as the local stopgap `parse_type`. Note in a comment that it is a stopgap to be widened by `parse-path-types-with-generic-args`.
3. Add a private helper `parse_bounds(&mut self) -> Result<Vec<TraitBound>, CompileError>` that parses `bound { "+" bound }` by calling `parse_trait_bound`, then while `self.eat(&TokenKind::Plus)` keep appending. Bounds list always contains ≥1 bound.
4. Add a private helper `parse_generics_params(&mut self) -> Result<(Vec<TypeParam>, Span), CompileError>`. Caller decides whether to enter; this helper assumes the next token is `Lt`. It bumps `<`, then loops reading `Ident` (the param name), optional `: bounds`, terminator `,` or `>`. Use `expect_one_of(&[Comma, Gt])` for trailing-comma support. Returns the list and the span of the closing `>`.
5. Add a private helper `parse_where_clause(&mut self) -> Result<WhereClause, CompileError>` that assumes the next token is `Where`. It bumps `where`, then loops reading `where_predicate = type ":" bounds`, separated by `,`. Stop when the next token is `LBrace` (start of body) or `Semi`. Build each `WherePred` with a fresh node id; build the `WhereClause` with a fresh node id and a span from `where` to the last predicate.
6. In `parse_fn`, after the name token, if `peek()` is `Lt`, call `parse_generics_params`. Save the params list and the `>` span.
7. After the existing `ret_ty` block and before `parse_block`, if `peek()` is `Where`, call `parse_where_clause` and save the result.
8. After parsing the body, build the `Generics` struct as follows: if either generic params or a where clause is present, populate `Some(Generics { id: new_node_id, span: <merge of generic-list span and where-clause span if either is present, falling back to the present one>, params, where_clause })`. If neither is present, `None`. Set `generics` on `FnDef`.
9. Reuse imports — add `crate::ast::generics::{Generics, TraitBound, TypeParam, WhereClause, WherePred}` to the top of `parser/item.rs`. The `Generics`/`TraitBound`/etc. structs are already defined and re-exported from `ast::mod`, so this is just a use.
10. Drop the `#[allow(dead_code)]` attribute on `Generics` (and any of its sub-types) once they have a real consumer through `FnDef`. Leave it on the others.
11. Add a `fn_generics_and_where` unit test in the existing `mod tests` block of `parser/item.rs` that builds the token stream for `fn foo<T, U>(x: T) -> U where T: Clone + Debug { }` and asserts: `f.name == "foo"`, `f.generics.is_some()`, the generics has 2 params named `T` and `U` (each with `bounds.is_empty()`), the where clause has 1 predicate whose `ty` is the path `T` and whose `bounds.len() == 2` with paths `Clone` and `Debug`, and `f.params` has the single `x: T` entry. Verify `p.errors.is_empty()` and the parser reaches `Eof`. Token order: `Fn, Ident("foo"), Lt, Ident("T"), Comma, Ident("U"), Gt, LParen, Ident("x"), Colon, Ident("T"), RParen, Arrow, Ident("U"), Where, Ident("T"), Colon, Ident("Clone"), Plus, Ident("Debug"), LBrace, RBrace, Eof`.

## Files
- `vertex_stage0/src/ast/item.rs` — add `generics: Option<Generics>` field to `FnDef`; import `Generics`.
- `vertex_stage0/src/parser/item.rs` — add `parse_trait_bound`, `parse_bounds`, `parse_generics_params`, `parse_where_clause` helpers; extend `parse_fn` to consume optional generics list (after name) and optional where clause (after return type, before body); update the `Ok(Item::Fn(FnDef { ... }))` construction to set `generics`. Add the `fn_generics_and_where` unit test inside the existing `mod tests`.
- `vertex_stage0/src/ast/generics.rs` — drop `#[allow(dead_code)]` from `Generics` (others kept until consumed).

## Risks
- `>>` shift token: nested generics like `Vec<Vec<T>>` would produce a `Shr` token rather than two `Gt`s. This plan does NOT handle that — the verify case uses only the flat `<T, U>` form, and full type parsing (with the `>>` split heuristic) is the responsibility of the upcoming `parse-path-types-with-generic-args` slug. Document this limitation in a comment near `parse_generics_params`.
- The stopgap `parse_type` only accepts a single ident, so where-predicate types must currently be a bare ident (e.g. `T`). The test stays inside this constraint. When `parse-path-types-with-generic-args` lands, the where parser will gain support for richer types automatically because `parse_type` is shared.
- `TraitBound` currently has both `path` and `generic_args` fields. We populate `path` with a single-segment path (no generic args) and leave `generic_args` empty — consistent with how `parse_type` already handles paths.
- Adding a field to `FnDef` is a breaking change for any other constructor; an `rg "Item::Fn\\(" -t rust` over `vertex_stage0/src` should turn up only `parser/item.rs` today. Any future construction site added in a parallel branch will get a clear "missing field `generics`" compile error.

## Prereqs
Prereqs: none

(`define-generics-and-whereclause-in-src-ast-generics-rs` would be a logical prereq, but the AST types `Generics`/`TypeParam`/`WhereClause`/`WherePred`/`TraitBound` are already defined in `vertex_stage0/src/ast/generics.rs` and re-exported from `ast/mod.rs`, so this slug can land first without blocking.)

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::item::tests::fn_generics_and_where
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::item::tests
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The Rust crate lives at `vertex_stage0/`, so `cargo` invocations need `--manifest-path vertex_stage0/Cargo.toml`. `cargo test --lib` filters by module path; the spec-given filter `parser::item::tests::fn_generics_and_where` resolves correctly under this manifest.
- `FnDef.generics` is `Option<Generics>` rather than always-present `Generics`. `None` ↔ no `<...>` and no `where`. This mirrors the spec's BNF (both are bracketed) and avoids constructing an empty `Generics` struct in the common case. Downstream code that wants to iterate type params can `.iter().flat_map(...)`.
- `Generics::span` covers from the opening `<` (or the `where` keyword if no `<...>` is present) through the close of the where clause (or the closing `>` if no where clause). Construction merges whichever spans are present.
- A bound `+` separator uses the existing `TokenKind::Plus`. A trailing `+` is rejected (i.e., `parse_bounds` requires ≥1 bound and bumps `+` only when followed by another bound — implemented by an unconditional first parse, then a `while self.eat(Plus) { parse_trait_bound; }` loop). This matches the spec ABNF.
- `parse_generics_params` accepts a trailing comma before `>`, matching how `parse_fn` already accepts a trailing comma in the parameter list.
- `parse_where_clause` stops the predicate loop when the next token is `LBrace` (body start). It does NOT currently handle `Semi` (trait/extern function declarations without a body are not yet supported by `parse_fn`).
- The `TraitBound`'s `path` is built as a single-segment path, with `generic_args: Vec::new()`. The outer `TraitBound.generic_args` field stays empty for now — generic args on trait paths (e.g. `Foo<T>`) are deferred to the path-types slug.
- `TypeParam.bounds` is populated when `: bounds` follows the param name. The verify test uses bare params `<T, U>` so this branch is exercised at the where-clause level only; the test-driven coverage is intentional.
- The existing helper `type_span` is reused for span computation in the where predicate and bound segments where convenient.
- `#[allow(dead_code)]` is removed from `Generics` because `FnDef` now references it; leaving the attribute would warn under `-D warnings` once the lint gate slug lands. The other helper structs (`TypeParam`, `WhereClause`, `WherePred`, `TraitBound`) become reachable via `Generics` and will lose `dead_code` warnings transitively.

## Blockers
Blockers: none

## Summary
Teach `parse_fn` to recognize `<T, U[: bounds]>` after the function name and `where T: Bound + Bound` before the body, recording both on a new `FnDef.generics` field, with a `fn_generics_and_where` unit test pinning the shape.

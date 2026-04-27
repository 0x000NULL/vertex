# Plan: parse-enum-items-with-all-variant-kinds

## Goal
Extend `EnumDef` with full structure and add `parse_enum` that recognizes unit/tuple/struct variants, optional discriminants, and generics — locked in by `parser::item::tests::enum_all_variant_kinds`.

## Steps
1. In `vertex_stage0/src/ast/item.rs`, replace the placeholder `EnumDef` (currently just `id` + `span`) with a full record:
   - `name: String`
   - `generics: Option<Generics>`
   - `variants: Vec<EnumVariant>`
   - keep `id` + `span`.
   Add two new public types in the same file:
   - `pub enum VariantKind { Unit, Tuple(Vec<Type>), Struct(Vec<Field>) }`
   - `pub struct EnumVariant { pub id: NodeId, pub span: Span, pub name: String, pub kind: VariantKind, pub discriminant: Option<Expr> }` (import `Expr` via `crate::ast::expr::Expr`). Tag both `#[allow(dead_code)]` and `#[derive(Debug, Clone)]` to match siblings.
2. In `vertex_stage0/src/parser/item.rs`, add a `pub fn parse_enum(&mut self) -> Result<Item, CompileError>` modeled on `parse_struct`:
   - `expect(&TokenKind::Enum)` → captures `start_span`.
   - `expect(&TokenKind::Ident(...))` → variant-list owner name.
   - Optional generics: same `Lt`/`parse_generics_params` block as in `parse_fn`/`parse_struct` (no `where` here per spec line 3263; do not consume `Where`).
   - `expect(&TokenKind::LBrace)`; loop until `RBrace`:
     - parse variant name as `Ident`, capture name span.
     - look at next token:
       - `LParen` → tuple variant. Consume; parse comma-separated `parse_type()` until `RParen`; allow trailing comma; collect into `Vec<Type>`; build `VariantKind::Tuple`. End-span = `RParen`.
       - `LBrace` → struct variant. Consume; parse comma-separated fields using the same shape as `parse_struct`'s record-arm (optional `pub`, ident, `:`, `parse_type()`, push `Field { is_pub, name, ty, ... }`); allow trailing comma; expect `RBrace`. Build `VariantKind::Struct(fields)`.
       - otherwise → `VariantKind::Unit`. End-span = name span (updated below if discriminant present).
     - optional discriminant: if next token is `Eq`, consume it, then expect an `IntLiteral(value, suffix)` token and build `Expr::IntLit(IntLit { id, span, value, suffix })`; record it in `discriminant` and update end-span.
     - assemble the `EnumVariant` (`name_span.merge(&end_span)`); push to vec.
     - if `eat(Comma)`, continue; else break (RBrace expected next).
   - `expect(&TokenKind::RBrace)` → `end_span`.
   - Build `generics` from `generics_list_span` exactly like `parse_struct` (no where-clause; pass `where_clause: None`).
   - Return `Item::Enum(EnumDef { id, span, name, generics, variants })`.
3. Update the `Item::Enum` handling in `Item::id()` and `Item::span()` is already correct (still reads `i.id` / `i.span`); no change needed there. Audit anywhere else `EnumDef` is constructed (likely none — it was a placeholder) and adjust if the compiler complains.
4. Add a `#[cfg(test)] mod tests` test `enum_all_variant_kinds` in `parser/item.rs` with a single test fn that exercises, in sequence on fresh parsers:
   - `enum E { A, B(i32, i32), C { x: i32, y: i32 } }` — asserts unit/tuple/struct kinds, field/type names, no discriminants, no generics, no errors, EOF reached.
   - `enum E { Foo = 5, Bar = 7, }` — asserts unit kind on each, discriminant `Expr::IntLit(value=5)` and `value=7`, trailing comma accepted.
   - `enum Result<T, E> { Ok(T), Err(E) }` — asserts generics has 2 params (`T`, `E` with empty bounds, no where-clause), tuple variants of length 1 each, type name on each.
   Use the existing `tok` / `ident_tok` / `int_tok` helpers; add a small helper `as_enum(item) -> EnumDef` mirroring `as_struct`.

## Files
- `vertex_stage0/src/ast/item.rs` — flesh out `EnumDef`; add `EnumVariant`, `VariantKind`; import `Expr`.
- `vertex_stage0/src/parser/item.rs` — add `parse_enum`; extend imports (`EnumDef`, `EnumVariant`, `VariantKind`, `IntLit` from `crate::ast::expr`); add the `enum_all_variant_kinds` unit test.

## Risks
- `parse_type` is the stopgap that only accepts a single bare ident — generic-arg types like `Result<T, E>::Ok(T)` happen to work because `T` and `E` are bare idents, but anything like `Vec<i32>` in a variant body would not. The plan keeps the test inputs within the stopgap's vocabulary.
- `Lt`/`Gt` handling inherits the same caveat as `parse_struct` — no support for `>>` (Shr) closing nested generics. Test inputs avoid this.
- `discriminant` is typed `Option<Expr>` for forward compatibility, but the parser only accepts a single `IntLiteral` token. Anything else (e.g. `Foo = -5`, `Foo = SOME_CONST`) will fail with the `expect` error from `IntLiteral(...)`. This matches the current capability of the tree (no expression parser yet).
- Adding fields to `EnumDef` will break any existing literal construction of `EnumDef { id, span }`. Per the current source there are none, but `cargo check` will surface any.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::item::tests::enum_all_variant_kinds
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- `EnumDef` currently being a near-empty placeholder (only `id` + `span`) means we are free to redesign it; replacing it does not break sibling work because no other code constructs or inspects its fields.
- Discriminants on non-unit variants are accepted by the parser without diagnostic. Spec grammar (line 3265) doesn't forbid them syntactically; semantic validation will be done later. Keep the parser permissive.
- Trailing comma after the last variant is permitted (matches spec line 3264 and `parse_struct` behavior).
- Discriminant expressions are limited to a single `IntLiteral` token for this milestone; richer expressions land with the expression-parsing items.
- No `where` clause is parsed for `enum` — spec `enum_def` (line 3263) does not include one; keep the door open by only conditionally building `Generics` when params exist (mirroring `parse_struct`).
- Tests live alongside the parser in `parser/item.rs`'s existing `mod tests`; the verify path `parser::item::tests::enum_all_variant_kinds` matches that location.
- The crate is `vertex_stage0` (manifest under `vertex_stage0/Cargo.toml`), not the workspace root — `cargo test --lib` therefore needs `--manifest-path` to target it directly.
- The existing `type_span` helper already covers the `Type::Path` and `Type::Ref` shapes the stopgap `parse_type` produces; tuple/struct-variant field spans can reuse it.

## Blockers
Blockers: none

## Summary
Promote `EnumDef` to a real AST node and implement `parse_enum` covering unit, tuple, struct variants, explicit discriminants, and generics, pinned by a single `enum_all_variant_kinds` test.

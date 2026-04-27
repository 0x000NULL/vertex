# Plan: parse-trait-items

## Goal
Extend `TraitDef` with a real shape (name, generics, supertraits, body items) and add a `parse_trait` that recognizes trait headers with optional generics + supertrait bounds and trait bodies containing required/default methods, associated types, and associated consts, locked in by a single `trait_with_assoc` unit test.

## Steps
1. In `vertex_stage0/src/ast/item.rs`, replace the placeholder `TraitDef { id, span }` with real fields: `name: String`, `generics: Option<Generics>`, `supertraits: Vec<TraitBound>`, `items: Vec<TraitItem>`. Add a `TraitItem` enum with three variants and their carrier structs:
   - `TraitItem::Fn(TraitItemFn)` — `id`, `span`, `name`, `generics: Option<Generics>`, `params: Vec<Param>`, `ret_ty: Option<Type>`, `default: Option<Block>` (None = required, Some = default body).
   - `TraitItem::Type(TraitItemType)` — `id`, `span`, `name` (bounds/default left out for this slice).
   - `TraitItem::Const(TraitItemConst)` — `id`, `span`, `name`, `ty: Type` (default value left out).
   - Use `#[allow(dead_code)] #[derive(Debug, Clone)]` to match neighbors.
2. In `vertex_stage0/src/parser/item.rs`, add a `pub fn parse_trait(&mut self) -> Result<Item, CompileError>` that:
   - Expects `Trait`, then ident name, then optional `<...>` via existing `parse_generics_params` (record `generics_list_span`).
   - On `Colon`, consume it and call `parse_bounds()` to fill `supertraits` (reuses the `+`-separated bound parser).
   - On `Where`, call `parse_where_clause()` and merge into the `Generics` value (matching the pattern used in `parse_fn`/`parse_struct`).
   - Expects `LBrace`, then loops: dispatch on `peek()`:
     - `Fn` → call a new helper `parse_trait_method()` (see step 3).
     - `Type` → consume keyword, expect ident, expect `Semi`, push `TraitItem::Type`.
     - `Const` → consume keyword, expect ident, `Colon`, call `parse_type()`, expect `Semi`, push `TraitItem::Const`.
     - Anything else → `expected_one_of_error(&[Fn, Type, Const, RBrace])` and break.
   - Expects `RBrace`; merges spans; builds and returns `Item::Trait(TraitDef { ... })`.
3. Add private `fn parse_trait_method(&mut self) -> Result<TraitItem, CompileError>` that mirrors the prelude of `parse_fn` but does not run the modifier loop and does not require a body:
   - Expect `Fn`, ident name, optional `<...>`, optional `(...)` params (reuse the same `try_parse_self_param` + named-param loop already in `parse_fn`), optional `-> Type`, optional where clause.
   - On `Semi`: consume it, set `default = None`.
   - On `LBrace`: call `self.parse_block()`, take the inner `Block`, set `default = Some(block)`.
   - Otherwise: `expect_one_of(&[Semi, LBrace])`.
   - To keep this lean, factor the shared "params + ret + where" body of `parse_fn` into a private helper (e.g. `fn parse_fn_signature_tail(&mut self) -> Result<(Vec<Param>, Option<Type>, Option<WhereClause>), CompileError>`) called by both `parse_fn` and `parse_trait_method`. Modifier handling stays in `parse_fn` only.
4. Add `#[test] fn trait_with_assoc()` in the existing `tests` module of `vertex_stage0/src/parser/item.rs`. Build a token stream for:
   ```
   trait Name<T>: Super + Super2 {
       fn req(&self);
       fn def(&self) { }
       type Item;
       const MAX: usize;
   }
   ```
   Assert: name == "Name"; one generic param `T`; two supertraits with idents `Super`, `Super2`; four items in order — `Fn{name:"req", default:None}`, `Fn{name:"def", default:Some(_)}`, `Type{name:"Item"}`, `Const{name:"MAX", ty == path "usize"}`. Assert no errors and that `peek() == Eof`.
5. Run `cargo build --lib` to surface unused-field warnings (the new structs already carry `#[allow(dead_code)]`) and `cargo test --lib parser::item::tests::trait_with_assoc` to lock the test.

## Files
- `vertex_stage0/src/ast/item.rs` — replace stub `TraitDef` with real fields; add `TraitItem` enum and carrier structs `TraitItemFn`, `TraitItemType`, `TraitItemConst`.
- `vertex_stage0/src/parser/item.rs` — add `parse_trait`, private `parse_trait_method`, optional `parse_fn_signature_tail` shared with `parse_fn`; add `trait_with_assoc` unit test in the `tests` module.

## Risks
- `parse_trait_bound` is still the single-ident stopgap, so supertraits and bounds with generic args (`Super<T>`) won't parse here. The test must use bare idents. This will be replaced by `parse-path-types-with-generic-args` later — leave a one-line comment pointing at that slug.
- `parse_generics_params` does not handle nested `>>`, so the test header keeps generics shallow (`<T>`).
- Refactoring `parse_fn` into a `parse_fn_signature_tail` helper risks breaking existing `parse_fn` tests (`plain_fn`, `fn_modifiers`, `self_params`, `fn_generics_and_where`). Keep the helper byte-for-byte equivalent to the inlined code it replaces and run those tests as part of `cargo test --lib`.
- `Block` is reused as the default body. `parse_block` returns `Expr::Block(b)`; unwrap with the same `match` pattern already used in `parse_fn` (line 408 of `parser/item.rs`), and panic with `unreachable!` on the unexpected variant — matches the existing convention.
- Trait items that begin with a `pub` visibility marker are not yet supported anywhere; this slice does not add them. Spec line for this slice does not include `pub`, so omit it.

## Prereqs
Prereqs: none

## Verify
```
cargo build --lib --manifest-path vertex_stage0/Cargo.toml
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::trait_with_assoc
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests
```

## Assumptions
- Trait header `Name<T>: Super + Super2` uses the existing `parse_bounds` helper for the `:`-separated supertrait list (single-ident stopgap) — generic supertraits are out of scope for this slice.
- Optional `where` clause on the trait header is supported because `parse_where_clause` already exists; not exercised by the new test, but allowed by the parser to match `parse_fn`/`parse_struct` shape.
- `TraitItemFn` carries `default: Option<Block>` rather than reusing `FnDef` (whose `body` is non-optional). This avoids changing `FnDef`'s public shape, which other items (`parse_fn` callers, future `parse_impl`) depend on.
- Associated types and consts in this slice are minimal: `type Item;` and `const NAME: Ty;` only — no bounds on associated types, no default values on consts. Richer forms land in later slices and can extend `TraitItemType` / `TraitItemConst` then.
- Trait methods accept `self` parameters via the existing `try_parse_self_param`, so `fn req(&self);` reuses the same path as inherent methods.
- Trailing comma between trait items is not part of the grammar; items are terminated by `;` or `}` (default body). The parse loop simply re-peeks after each item and breaks on `RBrace`.
- The shared `parse_fn_signature_tail` helper is added as a private method on `Parser` in the same `impl` block; if extracting it complicates this slice it can be inlined twice instead — the test surface is identical.
- The new unit test lives in the existing `tests` mod inside `parser/item.rs`, reusing `tok`, `ident_tok`, and other helpers.
- No new error code is needed; reuse `E0100`/`Syntax` via `expect`/`expect_one_of`.
- The TraitDef change touches `Item::trait_*` arms in `Item::id()`/`Item::span()` only because they already destructure by `i.id`/`i.span`; field additions on `TraitDef` keep those arms compiling.

## Blockers
Blockers: none

## Summary
Promote `TraitDef` from a placeholder to a real AST node and add `parse_trait` covering trait headers with generics + supertraits and trait bodies containing required/default methods, associated types, and associated consts, pinned by a single `trait_with_assoc` unit test.

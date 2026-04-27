# Plan: parse-inherent-and-trait-impls

## Goal
Promote `ImplDef` from a placeholder to a real AST node and add `parse_impl` covering inherent impls (`impl<T> Name<T> { ... }`), trait impls (`impl<T> Clone for Name<T> where T: Clone { ... }`), and impl bodies that mix methods with associated type/const bindings, all pinned by an `impl_inherent_and_trait` unit test.

## Steps
1. Expand `ImplDef` in `src/ast/item.rs` with real fields: `name: String` (the implementing-Self single-segment ident), `generics: Option<Generics>`, `trait_ref: Option<TraitBound>` (`Some` for trait impls; reuses the existing `TraitBound` shape — single-segment path + optional generic args), `self_ty_args: Vec<GenericArg>` (the `<...>` after the Self ident), `items: Vec<ImplItem>`.
2. In the same file, add `ImplItem` enum with three variants: `Fn(FnDef)` (reuses the existing `FnDef`), `Type(ImplItemType { id, span, name, ty })`, `Const(ImplItemConst { id, span, name, ty, value: Expr })`. Keep `#[allow(dead_code)]` on the new structs to match neighbours; do not change `Item::Impl` dispatch in `Item::id()`/`span()` (they already delegate to `i.id`/`i.span`).
3. In `src/parser/item.rs`, add a small local helper `parse_simple_path_type(&mut self) -> (String, Vec<GenericArg>, Span)` that parses `Ident` optionally followed by `<` generic-arg-list `>`, where each generic arg is a single `Ident` (treated as `GenericArg::Placeholder`, mirroring `parse_self_explicit_type`). This supplies both the trait reference and the Self-type ident under the current type-parsing stopgap. Comma-separated args are accepted so `Result<T, E>` round-trips even though only one arg is needed for the test.
4. Add `pub fn parse_impl(&mut self) -> Result<Item, CompileError>`:
   - Expect `Impl`; record `start_span`.
   - Optional `<...>` generic params (reuse `parse_generics_params`); track `generics_list_span`.
   - Parse a simple path type via the helper. Call this `head`.
   - If next is `For`, consume it and parse a second simple path type as the Self type; build a `TraitBound { path, generic_args }` from `head` and store in `trait_ref`. Otherwise `head` becomes the Self type and `trait_ref = None`.
   - Optional `where` clause via existing `parse_where_clause`.
   - Expect `LBrace`; loop dispatching on `peek`:
     - `Fn`: call `parse_fn`, destructure `Item::Fn(FnDef)`, push `ImplItem::Fn`.
     - `Type`: parse `type IDENT = TYPE ;` (note the `=`, unlike trait body) and push `ImplItem::Type`.
     - `Const`: parse `const IDENT : TYPE = EXPR ;` (using the stopgap `parse_type` and `parse_expr`) and push `ImplItem::Const`.
     - `RBrace`: break.
     - else: emit `expected_one_of_error(&[Fn, Type, Const, RBrace])` and break.
   - Expect `RBrace`; build `Generics` from `generics_list_span` + `where_clause` (mirroring `parse_trait`); return `Item::Impl(ImplDef { ... })`.
5. Add `#[test] fn impl_inherent_and_trait()` covering all four sub-steps in one test:
   - `impl<T> Name<T> { fn new() {} }` — assert no `trait_ref`, generic param `T`, Self ident `Name`, one `ImplItem::Fn` named `new`.
   - `impl<T> Clone for Name<T> where T: Clone { fn clone(&self) -> Self {} type Item = i32; const MAX: i32 = 5i32; }` — assert `trait_ref.path.segments[0].ident == "Clone"`, Self ident `Name`, where-clause has one predicate on `T: Clone`, four items: `Fn(clone)`, `Type{name:"Item", ty=Path("i32")}`, `Const{name:"MAX", ty=Path("i32"), value=IntLit(5)}`.
   - Verify `p.errors.is_empty()` and `peek() == Eof` for both.

## Files
- `vertex_stage0/src/ast/item.rs` — expand `ImplDef`; add `ImplItem`, `ImplItemType`, `ImplItemConst`. Re-export `Path`/`GenericArg` already imported via existing modules — add `use crate::ast::expr::{Expr, GenericArg}` if missing.
- `vertex_stage0/src/parser/item.rs` — import `ImplDef`, `ImplItem`, `ImplItemConst`, `ImplItemType` plus existing `Item`; add `parse_simple_path_type` helper, `parse_impl`, and the `impl_inherent_and_trait` test.

## Risks
- `Const` in the body is shared between "associated const" and the start of a `const fn` modifier chain. Mirroring `parse_trait`, this plan treats `Const` strictly as associated const — methods cannot be `const fn` in this scope. Acceptable because the current `parse-fn-modifiers` task only exercises top-level `fn`, and the verify test does not require it.
- The stopgap `parse_simple_path_type` cannot consume `>>`, so nested generics like `Vec<Vec<T>>` in an impl head would fail. Same constraint as the rest of the type-parsing stopgaps; the test stays single-arg.
- Extracting `FnDef` from `parse_fn`'s `Item::Fn` works only because `parse_fn` always returns `Item::Fn`. Use `match { Item::Fn(f) => f, other => unreachable!(...) }` (mirroring how `parse_block` is destructured today). Any future change to `parse_fn`'s return shape would break the unwrap — add a clear `unreachable!` panic message.
- Item-level dispatch from a future `parse_item` is *not* updated here (no such function exists yet). `parse_impl` becomes callable only via tests for now; that's consistent with `parse_trait`/`parse_struct` today.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib parser::item::tests::impl_inherent_and_trait
cargo build -p vertex_stage0
```

## Assumptions
- The test name `impl_inherent_and_trait` is a single `#[test]` covering all four sub-steps, matching how `trait_with_assoc`/`enum_all_variant_kinds`/`struct_tuple_unit` bundle multiple shapes into one test.
- `ImplDef` carrying a `name: String` plus separate `self_ty_args: Vec<GenericArg>` is acceptable since the rest of the codebase uses single-segment `Path` everywhere under the type-parsing stopgap; richer Self types (e.g. `(A, B)`, `&T`, `[T]`) arrive when `parse-path-types-with-generic-args` and friends land.
- For trait impls, the trait reference is stored as a `TraitBound` (id/span/path/generic_args) — same shape used for supertraits and where-clause bounds — to keep one consistent representation.
- `ImplItemConst.value` is typed as `Expr` (not `Block` or `Option<Expr>`) and parsed via `parse_expr` — current expression parser already handles integer literals, which is sufficient for the test.
- Associated `type X = T;` in impl bodies is stored as `ImplItemType { name, ty }`. This intentionally diverges from `TraitItemType` (no rhs) to capture the binding required by the spec's "Associated type/const bindings inside impl bodies" sub-step.
- `parse_impl` is added as a `pub fn` on `Parser` (matching `parse_fn`/`parse_struct`/`parse_enum`/`parse_trait`); wiring it into a future top-level `parse_item` dispatch is out of scope here.
- `Item::Impl(_)`'s existing `id()`/`span()` arms continue to work because the new `ImplDef` keeps `id` and `span` fields.
- `cargo test --lib` runs with the workspace's single library member (`vertex_stage0`), so no `-p` flag is needed.

## Blockers
Blockers: none

## Summary
Turn `ImplDef` into a real AST node and implement `parse_impl` for inherent and trait impls (including associated type/const bindings in impl bodies), pinned by an `impl_inherent_and_trait` unit test.

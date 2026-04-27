# Plan: parse-const-items

## Goal
Promote `ConstDef` from a placeholder to a real AST node and add `parse_const` recognizing `const NAME: T = expr;`, pinned by a single `const_item` unit test in `parser::item::tests`.

## Steps
1. In `src/ast/item.rs`, extend `ConstDef` with `name: String`, `ty: Type`, and `value: Expr` (alongside the existing `id` and `span`), keeping `#[derive(Debug, Clone)]` and the `#[allow(dead_code)]` attribute. Keep the `Item::Const(ConstDef)` variant and the `id()`/`span()` arms unchanged.
2. In `src/parser/item.rs`, add `pub fn parse_const(&mut self) -> Result<Item, CompileError>` that:
   - expects `TokenKind::Const` (capture its span as the start span);
   - expects an `Ident` and extracts its name;
   - expects `Colon` and parses the type with the existing stopgap `self.parse_type()` (same pattern used by `parse_struct`/`parse_fn`);
   - expects `Eq`;
   - parses the initializer with `self.parse_expr()`;
   - expects `Semi` (capture its span as the end span);
   - merges the start and end spans, allocates a new `NodeId`, and returns `Item::Const(ConstDef { id, span, name, ty, value })`.
3. Add a `#[test] fn const_item()` in `parser::item::tests` (alongside `use_simple_and_alias`, `mod_external_vs_inline`, etc.) that drives `parse_const` over a hand-built token stream representing `const N: i32 = 1i32;` and asserts:
   - the resulting `Item` is `Item::Const`,
   - `name == "N"`,
   - the type is a single-segment path of `i32` (use the existing `type_ident` helper),
   - the value is an `Expr::IntLit` with `value == 1`,
   - `p.errors` is empty and the next token is `Eof`.
4. Run `cargo test --lib parser::item::tests::const_item` to confirm the new test passes; run `cargo test --lib` once to make sure no neighboring item test regresses (the `ConstDef` field additions touch only construction sites that don't yet exist outside the new code path).

## Files
- `vertex_stage0/src/ast/item.rs` -- Replace placeholder fields on `ConstDef` with `id`, `span`, `name: String`, `ty: Type`, `value: Expr`. The existing `use crate::ast::expr::{Block, Expr};` and `use crate::ast::ty::Type;` imports are already present, so no new imports are needed.
- `vertex_stage0/src/parser/item.rs` -- Add `ConstDef` to the `crate::ast::item::{...}` import block; add the new `parse_const` method on `impl Parser`; add the `const_item` test in the `tests` submodule (it can reuse the existing `tok`, `ident_tok`, `int_tok`, `as_*` helpers; add an `as_const(item: Item) -> ConstDef` helper next to the others).

## Risks
- **Stopgap `parse_type` limitation.** The local `parse_type` only accepts a bare identifier, so the test must use a single-segment type like `i32`; richer types arrive with `parse-path-types-with-generic-args`. Mitigation: keep the test's type a plain identifier and document the stopgap reuse in the same way `parse_struct` / `parse_enum` do.
- **`Item::Const` is already a variant.** The `id()` / `span()` arms already match `Item::Const(i) => i.id` / `i.span`, so renaming/adding fields on `ConstDef` does not need wiring updates there. Risk is limited to any other crate code that constructs `ConstDef` directly — a quick `grep` showed only the AST definition itself touches it today.
- **Visibility (`pub const ...`)** is intentionally out of scope for this item (handled by `add-visibility-pub-pub-crate-pub-super-pub-in-path`). If `pub` lands in front of `const`, the caller dispatch will route it; `parse_const` itself stays minimal.
- **`parse_expr` recursion.** The initializer uses the full pratt-parsed expression grammar, so any pre-existing expression bug surfaces here. Mitigation: the test uses a simple `i32` literal, matching the precedent set by `enum_all_variant_kinds` for discriminant exprs.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::const_item
cargo test --lib --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- `ConstDef` should mirror the field shape used by other promoted item structs (`UseDef`, `ModDef`, `StructDef`): include `id`, `span`, plus the semantic fields `name`, `ty`, `value`. No `is_pub` field is added in this item — visibility is owned by the dedicated visibility task.
- The grammar form to recognize is `const NAME: T = expr;` exactly (with trailing `;`), matching the TODO sub-step text and the convention of every other already-implemented item parser. The spec sample lines (`const BUFFER_SIZE: usize = 1024`) omit the `;` but the parser tests exclusively use `;`-terminated forms, which is the source of truth for this task.
- Use the file-local stopgap `parse_type` (like `parse_fn`, `parse_struct`, `parse_enum`, `parse_trait` already do) rather than waiting on `parse-path-types-with-generic-args`. The test stays single-segment to fit that limitation.
- Use `parse_expr` for the initializer; do not invent a constant-expression sub-grammar at the parser layer (compile-time evaluability is enforced later, not here).
- The required test name (`const_item`) lives in `vertex_stage0/src/parser/item.rs` under the existing `#[cfg(test)] mod tests` block. The verify command's path `parser::item::tests::const_item` matches that location since the crate is the library inside `vertex_stage0`.
- The cargo manifest sits at `vertex_stage0/Cargo.toml` (the root-level `Cargo.toml` is a workspace shim), so verify uses `--manifest-path vertex_stage0/Cargo.toml`. This matches how `cargo test` was being run elsewhere in this repo.
- `as_const` test helper is added inline in the test module rather than exported, mirroring `as_fn`, `as_use`, etc.

## Blockers
Blockers: none

## Summary
Promote `ConstDef` to a real AST node, add `parse_const` for `const NAME: T = expr;`, and lock the form in with one `const_item` parser unit test.

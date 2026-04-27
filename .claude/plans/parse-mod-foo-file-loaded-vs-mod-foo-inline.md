# Plan: parse-mod-foo-file-loaded-vs-mod-foo-inline

## Goal
Promote `ModDef` to a real AST node with a `name` and a `ModKind` (`External` for `mod foo;`, `Inline(items)` for `mod foo { ... }`) and add `parse_mod` that recognizes both shapes, pinned by a `mod_external_vs_inline` unit test.

## Steps
1. In `vertex_stage0/src/ast/item.rs`, add a new public `ModKind` enum with two variants: `External` and `Inline(Vec<Item>)`. Place it just above `ModDef`. Annotate with `#[allow(dead_code)]` and `#[derive(Debug, Clone)]` to match neighboring nodes.
2. Extend `ModDef` (currently only `id` + `span`) with `name: String` and `kind: ModKind`. Keep the existing `Item::Mod(ModDef)` variant and the `id()` / `span()` arms in the `Item` impl unchanged.
3. In `vertex_stage0/src/parser/item.rs`, add `pub fn parse_mod(&mut self) -> Result<Item, CompileError>`:
   - `expect(&TokenKind::Mod)` → capture `start_span`.
   - `expect(&TokenKind::Ident(...))` → capture `name` and `name_span`.
   - Dispatch on `self.peek()`:
     - `TokenKind::Semi`: `bump()`, `kind = ModKind::External`, `end_span = semi.span`.
     - `TokenKind::LBrace`: `bump()`, then loop parsing items via a small private `parse_mod_inline_item` dispatcher (Step 4) until `RBrace`; `expect(&TokenKind::RBrace)` for `end_span`; `kind = ModKind::Inline(items)`.
     - Other: call `expect_one_of(&[Semi, LBrace])` to surface a syntax error and return its `Err`.
   - Build `ModDef { id: new_node_id(), span: start_span.merge(&end_span), name, kind }` and return `Item::Mod(...)`.
4. Add a private helper `fn parse_mod_inline_item(&mut self) -> Result<Item, CompileError>` that dispatches on `self.peek()` to existing parsers: `Fn → parse_fn`, `Struct → parse_struct`, `Enum → parse_enum`, `Trait → parse_trait`, `Mod → parse_mod` (recursive). For unsupported leading tokens, fall through to `expect_one_of(&[Fn, Struct, Enum, Trait, Mod, RBrace])` so the error message names the legal item-starters this stage actually parses. Other item kinds (`Use`, `Const`, `Type`, `Impl`, modifiers, `Pub`) are intentionally out of scope for this todo and will be wired in by the corresponding future items (`parse-use-items-simple-paths`, `parse-const-items`, `add-modifiers-...`, etc.).
5. Update the import line at the top of `vertex_stage0/src/parser/item.rs` to bring in `ModDef` and `ModKind` from `crate::ast::item`.
6. Inside the existing `#[cfg(test)] mod tests` in `vertex_stage0/src/parser/item.rs`, add helper `fn as_mod(item: Item) -> ModDef` and the `#[test] fn mod_external_vs_inline()` test that:
   - Parses `mod foo;` and asserts `name == "foo"`, `matches!(m.kind, ModKind::External)`, `p.errors.is_empty()`, and trailing `Eof`.
   - Parses `mod bar { fn x() {} }` and asserts `name == "bar"`, that `kind` is `ModKind::Inline(items)` with one item that destructures as `Item::Fn(f)` where `f.name == "x"`, `p.errors.is_empty()`, trailing `Eof`.
   - Parses a recursive case `mod outer { mod inner; }` and asserts the inner item is `Item::Mod` with `kind: External`.

## Files
- `vertex_stage0/src/ast/item.rs` -- add `ModKind` enum; extend `ModDef` with `name` and `kind` fields; no change to `Item` enum or its `id()`/`span()` impls.
- `vertex_stage0/src/parser/item.rs` -- import `ModDef`/`ModKind`; add `parse_mod` + private `parse_mod_inline_item` dispatcher; add `as_mod` helper and `mod_external_vs_inline` test in the existing `tests` module.

## Risks
- The inline-mod body dispatcher only handles the keywords whose parsers exist today (`Fn`, `Struct`, `Enum`, `Trait`, `Mod`). A test using `Use`, `Impl`, `Const`, attributes, or `Pub` inside an inline mod would error out. Mitigation: scope the new test to a single `fn` body and a recursive `mod`, both of which are supported now. Future items will broaden the dispatcher when they land.
- `is_sync_point` in `parser/mod.rs` already lists `TokenKind::Mod`, so error recovery treats `mod` as a sync point — no change needed there. If a future broader item dispatcher is added, this helper should be reused, but that refactor is out of scope here.
- The `ModDef` shape change is observed only by tests (no consumer reads `name`/`kind` yet), so no downstream code needs updating.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::mod_external_vs_inline
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate manifest lives at `vertex_stage0/Cargo.toml` (consistent with the directory layout) so verify commands pass `--manifest-path` rather than relying on workspace root.
- `ModKind::Inline` carries `Vec<Item>` directly (not a wrapper struct), matching how `VariantKind::Tuple(Vec<Type>)` and `VariantKind::Struct(Vec<Field>)` are modeled in the same file.
- `External` (rather than `File` or `Loaded`) is the variant name implied by the todo title's "(file-loaded)" gloss; it matches Rust's mental model of "external file module" and is what the spec verb in the todo uses.
- The inline-body dispatcher does NOT call `recover_to_sync` on bad item-starter tokens; it propagates the `expect_one_of` error so the test signal stays clean. Recovery semantics for inline `mod` bodies will be tightened in `end-to-end-recovery-test`.
- No visibility (`pub mod foo;`) parsing is added here — that arrives with `add-visibility-pub-pub-crate-pub-super-pub-in-path`.
- No attribute parsing on `mod` items here — that arrives with `add-attribute-parsing`.
- Test token sequences use existing `tok` / `ident_tok` helpers in the test module; no new helpers needed beyond `as_mod`.
- `Item` is already imported in `parser/item.rs` (it is — see the existing `use crate::ast::item::{...Item, ...}` line), so adding `ModDef`, `ModKind` to that same `use` group is sufficient.

## Blockers
Blockers: none

## Summary
Promotes `ModDef` to carry `name` + `ModKind` and adds `parse_mod` that recognizes both `mod foo;` and `mod foo { ... }`, with a single test (`mod_external_vs_inline`) covering both forms plus a recursive case.

# Plan: parse-plain-function-items

## Goal
Add a `parser::item` module with a `parse_fn` that recognises `fn name(p: Type, ...) -> RetTy { body }` and returns a populated `Item::Fn`, locked in by a `parser::item::tests::plain_fn` unit test.

## Steps
1. Flesh out `FnDef` in `vertex_stage0/src/ast/item.rs` to carry the signature data: `name: String`, `params: Vec<Param>`, `ret_ty: Option<Type>`, `body: Block`, alongside the existing `id` / `span`. Add a sibling `Param { id: NodeId, span: Span, name: String, ty: Type }` (no patterns — slug explicitly says "Param: name: Type (no patterns yet)"). Leave other Item variants (Struct, Enum, ...) as the existing placeholders so we don't fight the rest of the per-item plans.
2. Create `vertex_stage0/src/parser/item.rs`. Add a minimal `parse_type` helper local to this module that recognises a single-segment path type built from an `Ident` (returning `Type::Path(Path { segments: [PathSegment { ident, generic_args: vec![] }], ... })`). Note in a one-line comment that this is a stopgap to be replaced by `parse-path-types-with-generic-args`.
3. Add `parse_fn(&mut self) -> Result<Item, CompileError>` on `Parser`:
   - `expect(TokenKind::Fn)` — capture span start
   - `expect(TokenKind::Ident(_))` — extract name string from the bumped token
   - `expect(TokenKind::LParen)`; loop: when peek is `RParen` stop; otherwise `expect(Ident)` for the param name, `expect(Colon)`, `parse_type` for the param type; comma is separator with optional trailing comma; collect into `Vec<Param>`. `expect(TokenKind::RParen)`.
   - Optional return type: if `eat(&TokenKind::Arrow)`, `parse_type` and store as `Some(ty)`; else `None`.
   - `parse_block` (already exists) for the body; pattern-match the returned `Expr::Block(b)` and use `b` directly so `FnDef.body: Block` (not `Expr`).
   - Compute span as `fn_kw_span.merge(&body.span)`, allocate a fresh `NodeId`, and return `Item::Fn(FnDef { ... })`.
4. Wire `pub mod item;` into `vertex_stage0/src/parser/mod.rs` next to `pub mod expr;` / `pub mod stmt;`.
5. Add `parser::item::tests::plain_fn` covering at least:
   - `fn f() {}` — zero params, no return.
   - `fn id(x: i32) -> i32 { x }` — one param + return type + tail-expression body. (`x` body uses an ident, but since there is no path-expression parsing yet, fall back to a literal-bodied case `fn k() -> i32 { 1i32 }` if needed to keep this test self-contained — see Assumptions.)
   - `fn add(a: i32, b: i32,) -> i32 { 0i32 }` — multiple params + trailing comma.
   - Also assert that `Parser::errors` is empty and the parser is at `Eof` after each call.
   The test builds tokens directly the way `parser::stmt::tests` and `parser::tests` already do (no lexer round-trip needed).

## Files
- `vertex_stage0/src/ast/item.rs` — flesh out `FnDef` (add `name`, `params`, `ret_ty`, `body`); add new `Param` struct; keep other Item variants untouched.
- `vertex_stage0/src/parser/mod.rs` — add `pub mod item;`.
- `vertex_stage0/src/parser/item.rs` — new file: `parse_fn`, local `parse_type` helper, and `mod tests` with the `plain_fn` test.

## Risks
- Changing `FnDef`'s shape may collide with later item plans (`add-modifiers-...`, `add-visibility-...`, `add-attributes-...`, `add-self-parameters`, `add-generics-...`). Mitigation: those plans are designed to *extend* this struct, so we keep the field set small and forward-compatible (no `vis`/`generics`/`attrs` yet — they'll be added by their own slugs). Body type is `Block` (a struct) rather than `Expr`, so future "fn body is an expression" surprises are unlikely — the grammar requires a block.
- The local `parse_type` helper duplicates work that `parse-path-types-with-generic-args` will eventually do. Mitigation: comment marking it a stopgap; the call site is centralised so the later slug can swap it for a richer `Parser::parse_type` without touching `parse_fn`.
- `parse_block` returns `Expr::Block(Block)`, not `Block`. We pattern-match and unwrap; if `parse_block` ever returned a different variant the unwrap would panic, but its current contract guarantees the variant.
- Trailing comma handling in the parameter list is easy to get wrong (loop must check `RParen` at top of each iteration, after each comma).

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::item::tests::plain_fn
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate root for the parser lives at `vertex_stage0/Cargo.toml` (confirmed by file layout); `cargo test --lib` from the workspace root needs `--manifest-path vertex_stage0/Cargo.toml` because the workspace setup is unknown. If a workspace `Cargo.toml` at the repo root forwards correctly, the bare command also works — both should pass.
- `FnDef` is allowed to grow new fields (not a stable public API yet — every variant currently has only `id`/`span` placeholders, and follow-on slugs explicitly extend it).
- `Param` is added as a *new* type, not re-purposing `ClosureParam` / `Pat` — patterns are out of scope per the slug's "no patterns yet".
- Minimal `parse_type` accepts only single-segment ident paths (`i32`, `String`, `MyType`). Multi-segment paths, generics, refs, slices, etc. are left to their respective slugs. The local helper will be visible only inside `parser::item`; once `parse-path-types-with-generic-args` lands it will replace this helper.
- Test bodies are built from int literals (already-parsed primitive) rather than path expressions, since `parse-path-expressions` hasn't shipped yet. The signature/body shape (zero-stmt block, tail-only block) is what we want to pin; the body's *contents* are incidental.
- Function bodies use `Block` directly (extracted from `parse_block`'s `Expr::Block`); there is no need to store the body as a generic `Expr`.
- Visibility, attributes, modifiers (`pub`, `unsafe`, `extern`, `const`), generics, where-clauses, and `self` parameters are all explicitly out-of-scope for this plain item — each has its own pending slug.

## Blockers
Blockers: none

## Summary
Wires up the smallest possible function-item parser (`fn name(p: T) -> R { body }`) so subsequent item-modifier slugs have something concrete to extend, with a `plain_fn` unit test pinning the signature shape.

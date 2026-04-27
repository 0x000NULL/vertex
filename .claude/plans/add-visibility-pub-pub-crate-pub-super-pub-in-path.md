# Plan: add-visibility-pub-pub-crate-pub-super-pub-in-path

## Goal
Parse Rust-style visibility modifiers (`pub`, `pub(crate)`, `pub(super)`, `pub(in path)`) in front of function items, surfaced as a `Visibility` enum stored on `FnDef`.

## Steps
1. In `src/ast/item.rs`, add a `Visibility` enum:
   ```
   pub enum Visibility {
       Inherited,
       Public(Span),
       Crate(Span),
       Super(Span),
       Restricted { path: Path, span: Span },
   }
   ```
   Mark `#[allow(dead_code)]` and `Debug, Clone` like the surrounding types. Import `Path` from `crate::ast::expr` (the file already imports `Block` from there, so the cross-module reference is consistent).
2. Add `pub vis: Visibility` to `FnDef`. Update the existing `fn_modifiers` and `plain_fn` tests' field bindings only if they pattern-match on `FnDef` exhaustively (current tests use `f.field` access, so they're unaffected).
3. Re-export `Visibility` from `src/ast/mod.rs` next to the existing `pub use item::Item;`.
4. In `src/parser/item.rs`, add `parse_visibility(&mut self) -> Visibility`:
   - If `peek() != Pub`, return `Inherited` without consuming.
   - Bump the `pub` token, capture its span as `start`.
   - If next is `LParen`: bump it, then dispatch on the inner peek:
     - `Ident("crate")` → bump, expect `RParen`, return `Crate(start.merge(rparen.span))`.
     - `Ident("super")` → bump, expect `RParen`, return `Super(...)`.
     - `In` → bump, parse a simple path (see step 5), expect `RParen`, return `Restricted { path, span }`.
     - anything else → call `expected_one_of_error(&[Ident, Ident, In])`-equivalent (use a single descriptive error via `expected_one_of_error`), then return `Public(start)` as a recovery so parsing continues.
   - If no `(`: return `Public(start)`.
5. Add a private helper `parse_simple_path(&mut self) -> Result<Path, CompileError>` next to the local `parse_type` stopgap, parsing `ident (:: ident)*` into a `Path` whose `PathSegment`s have empty `generic_args`. Drop a `// TODO:` comment noting it will be replaced by the proper path parser from slug `parse-path-expressions`. (Mirrors the pattern of the existing stopgap `parse_type`.)
6. In `parse_fn`, call `let vis = self.parse_visibility();` BEFORE the existing modifier loop. If `vis` is non-`Inherited`, derive a leading span from it and seed `first_modifier_span` (rename the local concept to "first leading span" only if needed; minimal change is to also seed it from the visibility span when present).
7. Pass `vis` into the constructed `FnDef`.
8. Add `parser::item::tests::fn_visibility` covering:
   - bare `fn f() {}` → `Visibility::Inherited`
   - `pub fn f() {}` → `Visibility::Public(_)`
   - `pub(crate) fn f() {}` → `Visibility::Crate(_)`
   - `pub(super) fn f() {}` → `Visibility::Super(_)`
   - `pub(in foo::bar) fn f() {}` → `Visibility::Restricted { path, .. }` with `segments == ["foo", "bar"]`
   - `pub const fn f() {}` → visibility `Public` AND `is_const == true` (visibility precedes modifiers per spec grammar)
   - `pub() fn f() {}` (malformed inner) → produces an `E0100` syntax error but still parses to a function
9. Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and the new test.

## Files
- `src/ast/item.rs` — add `Visibility` enum; add `vis: Visibility` field to `FnDef`.
- `src/ast/mod.rs` — `pub use item::Visibility;`.
- `src/parser/item.rs` — add `parse_visibility`, `parse_simple_path`, wire into `parse_fn`, set `vis` on the produced `FnDef`, add `fn_visibility` test.

## Risks
- `crate` and `super` are NOT lexer keywords (verified in `src/lexer/scan.rs:584-617` — only `self`/`Self` are special). They come back as `TokenKind::Ident("crate")` / `Ident("super")`, so dispatch must string-match the ident, not look for a token kind. Future work that promotes them to keywords would touch this code.
- The existing `is_sync_point` already includes `TokenKind::Pub`, so error recovery from a malformed visibility correctly stops at the next item.
- `FnDef` gains a field; any other code constructing `FnDef` would need updating. Only `parse_fn` constructs it today (verified by `Grep "FnDef \{|FnDef \{"` mentally — the only producer is `src/parser/item.rs`).
- The simple path parser duplicates work that `parse-path-expressions` will do. Acceptable per the existing precedent (`parse_type` stopgap on lines 11–27 of `src/parser/item.rs`); we mark it with a `TODO` referencing the future slug so it's removed when that lands.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib parser::item::tests::fn_visibility
cargo test --lib parser::item::tests::plain_fn parser::item::tests::fn_modifiers
cargo build
cargo clippy --all-targets -- -D warnings
```

## Assumptions
- Per `vertex_v1_spec.md:3238-3243`, visibility precedes the `const`/`unsafe`/`extern` modifier set. We parse it once at the start of `parse_fn`, not inside the modifier loop. `const pub fn` is therefore rejected (the loop will exit on `pub` and `expect(Fn)` will error).
- `crate`/`super`/`in` after `pub(` are matched by: `Ident("crate")`, `Ident("super")`, and `TokenKind::In` (a real keyword). Other `Ident`s inside `pub(...)` are an error.
- `pub(in path)` accepts `ident (:: ident)*`. No leading `::`, no generics, no `self`/`super`/`crate` segments — the spec example doesn't use them and richer path parsing is the dedicated job of `parse-path-expressions`.
- A new `vis` field on `FnDef` is non-default; `parse_fn` is the sole producer today, so no other constructor needs updating.
- `Visibility::Inherited` is used when no `pub` is present (rather than `Option<Visibility>`), matching the existing convention of representing absence with an enum variant rather than `Option`.
- Empty `pub()` is recovered as `Public` plus a syntax error so that downstream item parsing still completes — same recovery philosophy as `Expr::Error` from the prior `insert-placeholder-expr-error-nodeid-span-and-continue` slug.
- The local `parse_simple_path` returns `crate::ast::expr::Path`. When `parse-path-expressions` lands, the helper is removed and call sites switch to the canonical parser.
- Tests cover one combination with a modifier (`pub const fn`) to pin ordering; we don't enumerate every cross-product since `fn_modifiers` already covers the modifier matrix.

## Blockers
Blockers: none

## Summary
Adds a `Visibility` AST enum and parser support so `fn` items can carry `pub`, `pub(crate)`, `pub(super)`, or `pub(in path)`, locked in by `parser::item::tests::fn_visibility`.

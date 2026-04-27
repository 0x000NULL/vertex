# Plan: parse-static-and-static-mut-items

## Goal
Recognize `static NAME: T = expr;` and `static mut NAME: T = expr;` as items by promoting `StaticDef` to a real AST node and adding `parse_static`, which requires teaching the lexer the new `static` keyword.

## Steps
1. **Lexer keyword.** In `src/lexer/token.rs` add a new `Static` variant to `TokenKind` (placed alphabetically between `SelfUpper` and `Struct` to match the existing ordering style). In `src/lexer/scan.rs` add `"static" => TokenKind::Static,` to the keyword match arm in `scan_ident_or_keyword` (alongside `"self" => TokenKind::SelfLower`, etc.). Extend the `keywords_take_priority_over_idents` table with `("static", TokenKind::Static)` and add a `lex_eq!("static", vec![TokenKind::Static]);` line near the existing `lex_eq!("const", ...)` so the lexer test suite stays consistent.
2. **AST node.** In `src/ast/item.rs`, replace the placeholder `StaticDef { id, span }` with a real definition:
   ```
   pub struct StaticDef {
       pub id: NodeId,
       pub span: Span,
       pub name: String,
       pub ty: Type,
       pub value: Expr,
       pub is_mut: bool,
   }
   ```
   `Item::Static(StaticDef)` is already in the enum and the `id()`/`span()` match arms already cover it — no change needed there.
3. **Parser.** In `src/parser/item.rs`, add `pub fn parse_static(&mut self) -> Result<Item, CompileError>` mirroring `parse_const` (lines 1067–1093):
   - `expect(&TokenKind::Static)` — capture `start_span`.
   - `let is_mut = self.eat(&TokenKind::Mut);` — optional `mut`.
   - `expect(&TokenKind::Ident(String::new()))` for the name (extract via the same `match` pattern used by `parse_const`).
   - `expect(&TokenKind::Colon)`, then `parse_type()`.
   - `expect(&TokenKind::Eq)`, then `parse_expr()`.
   - `expect(&TokenKind::Semi)` — end_span comes from the semicolon.
   - `span = start_span.merge(&end_span)`, `id = self.new_node_id()`, return `Ok(Item::Static(StaticDef { id, span, name, ty, value, is_mut }))`.
   - Update the `use crate::ast::item::{ ... }` import list at the top of `parser/item.rs` to include `StaticDef`.
4. **Test.** Add a `static_item` `#[test]` to the existing `tests` module in `src/parser/item.rs`, immediately after `const_item` (around line 2237). Add an `as_static` helper next to `as_const`. Cover both forms with one combined test:
   - Form A: `static N: i32 = 1i32;` — assert `name == "N"`, `type_ident(&s.ty) == "i32"`, `s.value` is `Expr::IntLit { value: 1 }`, `s.is_mut == false`, `p.errors.is_empty()`, peek is `Eof`.
   - Form B: `static mut N: i32 = 1i32;` — same checks but `s.is_mut == true`.
   Use the existing `tok`, `ident_tok`, `int_tok`, and `type_ident` helpers; use `Parser::new(...)` followed by `p.parse_static().expect("parse_static")`.

## Files
- `src/lexer/token.rs` — add `Static` variant to `TokenKind`.
- `src/lexer/scan.rs` — recognize `"static"` keyword in `scan_ident_or_keyword` and extend the keyword test table + `lex_eq!` line.
- `src/ast/item.rs` — flesh out `StaticDef` with `name`, `ty`, `value`, `is_mut` fields.
- `src/parser/item.rs` — import `StaticDef`, add `pub fn parse_static`, add `as_static` helper and `static_item` test.

## Risks
- Adding `static` as a keyword shadows it as an identifier. A repo-wide grep for the literal word `static` in source files turns up only Rust meta-uses (`static EOF_KIND: TokenKind`, `'static`, `assert "static"` in lexer span test), all of which are in *our* Rust code, not in lexer-input strings — so the change is safe. The only lexer input that contains the word `static` is the `all_tokens_have_nonzero_span` test src string, which only checks span shape, not kinds.
- `parse_static` (like `parse_const`) is a public method but is **not** wired into the top-level item dispatch (`parse_mod_inline_item`); leaving it that way matches the existing `parse_const` pattern. A later "wire all item kinds into the top-level dispatcher" task can pick this up.
- `StaticDef` is only consumed via `Item::Static` arms in `id()`/`span()`; no downstream pass exists yet, so widening the struct cannot break anything.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::static_item
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The Cargo manifest lives at `vertex_stage0/Cargo.toml` (matches existing repo layout). The verify command in the todo (`cargo test --lib parser::item::tests::static_item`) is given without `--manifest-path`; I add `--manifest-path vertex_stage0/Cargo.toml` because the runner executes from the repo root, where bare `cargo test` would fail. The test path itself (`parser::item::tests::static_item`) is preserved verbatim.
- The single `static_item` test is intended to cover **both** `static` and `static mut` forms in one `#[test]` (the spec lists two forms and one verify command). This matches the precedent set by `mod_external_vs_inline` and `use_simple_and_alias`, which combine related forms in a single test.
- `StaticDef` should mirror `ConstDef` (`id, span, name, ty, value`) plus a single new `is_mut: bool` field. Visibility (`pub`), attributes, and `extern` linkage are left for later items (the same way `ConstDef` is currently bare).
- `parse_type()` (the stopgap single-segment path parser) and `parse_expr()` are sufficient to cover the test inputs `i32` and `1i32`; richer types/expressions land via the listed pending items and will flow through automatically.
- The lexer keyword can be added without coordination with other items because the keyword recognition is purely additive: no existing token disappears, and no existing tests rely on `static` lexing as an `Ident`.
- I extend the lexer's `keywords_take_priority_over_idents` test table even though the verify command doesn't run it, to keep the lexer test suite consistent with the new keyword. `cargo build` in verify will not catch a missing entry, but `cargo test` (without filter) elsewhere in the run would, so adding it preempts breakage.

## Blockers
Blockers: none

## Summary
Promote `StaticDef` to a real AST node, teach the lexer the `static` keyword, add `parse_static` for both `static NAME: T = expr;` and `static mut NAME: T = expr;`, and lock the form in with one `static_item` parser unit test.

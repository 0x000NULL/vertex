# Plan: parse-use-items-simple-paths

## Goal
Add a `parse_use` method that recognizes `use foo::bar;` and `use foo::bar as baz;`, extending `UseDef` to carry the parsed path segments and optional alias and pinning both forms with one `use_simple_and_alias` unit test.

## Steps
1. In `src/ast/item.rs`, extend the `UseDef` struct from its current `{ id, span }` placeholder to also carry `name: String` segments (use `pub segments: Vec<String>`) and `pub alias: Option<String>`. Keep `#[allow(dead_code)]` and `#[derive(Debug, Clone)]`.
2. In `src/parser/item.rs`, add `pub fn parse_use(&mut self) -> Result<Item, CompileError>`:
   - `expect(&TokenKind::Use)` → record `start_span`.
   - Parse first path segment: `expect(&TokenKind::Ident(String::new()))`, push the string to `segments`.
   - Loop while `eat(&TokenKind::ColonColon)`: `expect(&TokenKind::Ident(String::new()))`, push.
   - Optional alias: peek for `TokenKind::Ident(s)` where `s == "as"`. If matched, `bump()` the `as` ident, then `expect(&TokenKind::Ident(String::new()))` and store the resulting string in `alias`. (`as` is not a reserved kw in `lexer/token.rs:32-114`, so it lexes as `Ident("as")`.)
   - `expect(&TokenKind::Semi)` → `end_span`.
   - Build `Item::Use(UseDef { id, span: start_span.merge(&end_span), segments, alias })`.
3. Add a `use_simple_and_alias` test inside the existing `tests` module in `src/parser/item.rs` (mirroring style of `mod_external_vs_inline`, `struct_tuple_unit`, etc.) that drives `parse_use` for two synthetic token streams:
   - `use foo::bar;` → asserts `segments == ["foo", "bar"]`, `alias.is_none()`, no errors, peek is `Eof`.
   - `use foo::bar as baz;` → asserts `segments == ["foo", "bar"]`, `alias == Some("baz")`, no errors, peek is `Eof`.

## Files
- `src/ast/item.rs` -- extend `UseDef` with `segments: Vec<String>` and `alias: Option<String>`.
- `src/parser/item.rs` -- add `parse_use` to `impl Parser`; add `use_simple_and_alias` test plus `as_use(item: Item) -> UseDef` test helper, and import `UseDef` in the `use` line at the top of the file (already imports from `crate::ast::item`).

## Risks
- `as` is not a keyword in the lexer (`scan.rs:584-617`), so it lexes as `Ident("as")`. Recognising it via string-match is correct now but may collide with `parse-indexing-cast-try` once that item introduces an `As` keyword — that follow-up will need to convert this peek to a kw match. Documented as an assumption, not a blocker for this item.
- Nested-tree forms (`use foo::{bar, baz}` and `use foo::*`) are deliberately out of scope and will be handled by `parse-use-items-nested-glob`. The new `UseDef` shape (flat `segments` + `alias`) is additive enough to be replaced/extended later without breaking other items, since `UseDef` is currently only constructed inside `parse_use`.
- `parse_mod_inline_item` (`parser/item.rs:949-968`) does not currently dispatch on `TokenKind::Use`. This task does not require nested `use` inside `mod { ... }`, so we leave that helper alone; will be revisited when top-level item dispatch is wired.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::use_simple_and_alias
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate root is `vertex_stage0/` (Cargo manifest lives there); `--manifest-path` is the cleanest way to invoke `cargo` from the repo root for verify.
- `as` remains an ordinary identifier at the lexer level for now; recognising it as `Ident(s) where s == "as"` is acceptable until a future item promotes it to a keyword. Since `mem::discriminant` matching in `eat`/`expect` ignores the inner string, we must handle this with an explicit `if let TokenKind::Ident(s) = self.peek().clone() { ... }` style peek before bumping.
- `UseDef` is currently `{ id, span }` only and is constructed nowhere yet (no other call-sites to update besides `Item::Use` arms in `Item::id`/`Item::span`, which keep working unchanged).
- The test file naming uses `parser::item::tests::use_simple_and_alias`, following the style of existing tests like `parser::item::tests::mod_external_vs_inline`.
- Single-segment paths (`use foo;`) are also accepted by the loop (zero `::` iterations) — not asked for explicitly, but a free property that costs nothing and matches a normal Rust shape.
- We do not need to touch `parser/mod.rs` (`is_sync_point` already lists `TokenKind::Use`).

## Blockers
Blockers: none

## Summary
Extends `UseDef` and adds `parse_use` so `use foo::bar;` and `use foo::bar as baz;` round-trip into AST, locked in by a `use_simple_and_alias` unit test.

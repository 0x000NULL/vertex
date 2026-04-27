# Plan: add-attribute-parsing

## Goal
Introduce a generic `Attribute { path, args, span }` AST node, lexer/parser support for outer attributes (`#[...]`), and wire it onto `FnDef` so `parse_fn` accepts `#[no_mangle]`, `#[inline]`, `#[derive(Clone, Debug)]` prefixes, pinned by `parser::item::tests::fn_attributes`.

## Steps
1. Add `Pound` to `TokenKind` in `src/lexer/token.rs`. The lexer currently rejects `#` as `Error("invalid character: #")` (confirmed at scan.rs:850). The only existing `#` consumer is the `r#"..."#` raw-string scan (scan.rs:740–744), which runs *before* general byte dispatch and is therefore unaffected.
2. In `src/lexer/scan.rs::next_token` (after the doc-comment / raw-string / string / char fast paths), emit `TokenKind::Pound` for a bare `b'#'` byte.
3. Add the `Pound => "\`#\`"` arm to `describe(&TokenKind)` in `src/parser/mod.rs` so error messages stay coherent.
4. Create `src/ast/attr.rs` with `Attribute { id: NodeId, span: Span, path: Path, args: AttrArgs }` and `AttrArgs::{ Empty, Delimited(Vec<Token>) }`. `Empty` covers `#[no_mangle]`/`#[inline]`; `Delimited` stores the raw token stream between `(` and `)` for `#[derive(Clone, Debug)]`/`#[inline(always)]` so downstream consumers (derive validator, repr parser) can re-interpret it.
5. Wire the new module into `src/ast/mod.rs` (`pub mod attr; pub use attr::{Attribute, AttrArgs};`).
6. Add `attrs: Vec<Attribute>` (default `Vec::new()`) to `FnDef` in `src/ast/item.rs`.
7. Add `Parser::parse_attribute` and `Parser::parse_outer_attributes` in `src/parser/item.rs`. Grammar: `'#' '[' ident ('(' tokens-until-matching-')' )? ']'`. Use a single-segment `Path` for now (sufficient for the listed attributes). The args grabber tracks paren depth so nested forms like `cfg(any(a, b))` round-trip; bound the loop on `Eof` and emit `E0100` on imbalance.
8. Modify `parse_fn` to call `parse_outer_attributes` *before* modifiers; if any attribute was parsed, fold `attrs[0].span` into `start_span`. Populate `FnDef.attrs`.
9. Add `parser::item::tests::fn_attributes` covering the three spec examples (`#[no_mangle]`, `#[inline]`, `#[derive(Clone, Debug)]`) plus a stack `#[no_mangle] #[inline] fn k() {}`. Assert `attrs.len()`, first-segment ident, `AttrArgs::Empty` vs `Delimited`, and that `FnDef.span` starts at the `#`.

## Files
- `src/lexer/token.rs` -- add `Pound` to `TokenKind`.
- `src/lexer/scan.rs` -- emit `TokenKind::Pound` for `#` in `next_token`.
- `src/parser/mod.rs` -- add the `Pound` arm in `describe`.
- `src/ast/attr.rs` -- new file: `Attribute`, `AttrArgs`.
- `src/ast/mod.rs` -- declare `pub mod attr` and re-export.
- `src/ast/item.rs` -- add `attrs: Vec<Attribute>` to `FnDef`.
- `src/parser/item.rs` -- add `parse_attribute`, `parse_outer_attributes`; call from `parse_fn`; populate `attrs: Vec::new()` in existing test paths; add the `fn_attributes` test.

## Risks
- Adding a `TokenKind` variant breaks any exhaustive match outside the parser. Known consumers: `describe`, `is_sync_point` (no change needed — `#` is not a sync point), and the lexer itself. Need to grep for stray exhaustive matches; the fix is mechanical.
- `AttrArgs::Delimited(Vec<Token>)` requires `Token: Clone` (already derived) and stores whole `Span`s — heavier than ideal but fine for stage0; tighten if it becomes a hot path.
- Paren-depth scan must terminate on `Eof` to avoid an infinite loop on malformed input; raise `E0100` and recover to the next sync point.
- Doc-comment scan (`scan.rs:734`) runs before token dispatch; `#` inside line comments is untouched. Inside raw strings the existing `r#"..."#` consumer wins because it fires before the generic dispatch.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib -p vertex_stage0 parser::item::tests::fn_attributes
cargo test --lib -p vertex_stage0
```

## Assumptions
- The lexer needs a new `Pound` token because `#` is currently lexed as `Error("invalid character: #")`. Adding it is part of this slug — there is no separate slug for it, and `parser::item::tests::fn_attributes` cannot be written otherwise.
- `AttrArgs` is `Empty | Delimited(Vec<Token>)` rather than a structured tree; downstream slugs (`validate-derive-allow-list`, `add-repr-parsing-on-structs`) re-interpret the raw tokens themselves, avoiding premature meta-syntax commitment.
- Attribute path is a single-segment ident for now. Multi-segment paths (e.g. `#[foo::bar]`) aren't in any v1-spec attribute example; swap in the real path parser once `parse-path-expressions` lands.
- Attributes attach only to `FnDef` in this slug. Other `*Def`s gain `attrs` fields when their own slugs need them (`add-repr-parsing-on-structs`, etc.).
- Inner attributes (`#![...]`) are out of scope.
- Existing `parse_fn` tests pass unchanged — they construct `FnDef` only via `parse_fn`, and never inspect `attrs`. The constructor populates `attrs: Vec::new()` when no `#` precedes the function.
- Test lives in the existing `mod tests` in `src/parser/item.rs`, matching the placement of `plain_fn` and `fn_modifiers`.

## Blockers

### Blocker: lexer lacks `#` token
- severity: cross-item
- affects: lexer, attribute-parsing, repr-parsing, derive-validation, cfg-attributes
- question: Is it acceptable to introduce `TokenKind::Pound` and the corresponding scan rule as part of this slug, or should that be split into a dedicated prerequisite item?
- default_assumption: Add `TokenKind::Pound` and emit it from the scanner inside this slug, since no other pending slug introduces `#` and the verify test cannot run without it.

### Blocker: shape of AttrArgs
- severity: cross-item
- affects: attribute-parsing, repr-parsing, derive-validation, cfg-attributes
- question: Should `AttrArgs` store a raw `Vec<Token>` or a structured form (ident list, name=value, nested call tree)?
- default_assumption: `AttrArgs::Delimited(Vec<Token>)` — raw tokens. Each consumer slug interprets the slice for its own attribute; keeps stage0 simple and defers meta-syntax design.

## Summary
Adds outer-attribute parsing (`#[...]`) plus a generic `Attribute` AST node, wires `attrs` onto `FnDef`, and pins it with `parser::item::tests::fn_attributes` — including the lexer-side `Pound` token that the parser requires.

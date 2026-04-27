# Plan: parse-use-items-nested-glob

## Goal
Extend `UseDef` and `parse_use` to handle nested groups (`use { a, b::c, d::{e, f} };`), glob (`use foo::*;`), and a leading `pub` visibility (`pub use bar;`), pinned by a single `use_nested_glob_pub` test in `parser::item::tests`.

## Steps
1. In `src/ast/item.rs`, introduce a recursive `UseTree` enum with three variants:
   - `Simple { segments: Vec<String>, alias: Option<String> }`
   - `Glob { segments: Vec<String> }` (the `segments` is the prefix before `::*`; may be empty for a bare `use *;` though that's not in scope)
   - `Nested { segments: Vec<String>, items: Vec<UseTree> }` (`segments` is the prefix path before `{...}`; may be empty when `use {...}` is written directly without a prefix)
   Replace the current flat `UseDef { segments, alias }` shape with `UseDef { id, span, is_pub: bool, tree: UseTree }`. Keep `#[allow(dead_code)] #[derive(Debug, Clone)]` on both types and re-export nothing new beyond updating `crate::ast::item` consumers' import line in `src/parser/item.rs`.
2. In `src/parser/item.rs`, rewrite `parse_use` to:
   1. Optionally consume a leading `TokenKind::Pub` and remember `is_pub`. Start span begins at `pub` if present, else at `use`.
   2. Expect `TokenKind::Use`.
   3. Parse a `UseTree` via a new helper `parse_use_tree(&mut self) -> Result<UseTree, CompileError>`.
   4. Expect `TokenKind::Semi`; merge end span.
   5. Build `Item::Use(UseDef { id, span, is_pub, tree })`.
3. Implement `parse_use_tree`:
   - If next token is `TokenKind::LBrace`, parse a nested group with empty prefix `segments = vec![]` (the case `use { a, b };`).
   - Otherwise expect a leading `Ident` and accumulate `segments` while the next token is `TokenKind::ColonColon`. After consuming `::`, peek the next token:
     - If it is `TokenKind::Star`, bump it and return `UseTree::Glob { segments }`.
     - If it is `TokenKind::LBrace`, fall through to the nested-group path below using the accumulated `segments` as the prefix.
     - Otherwise expect another `Ident` segment and continue the loop.
   - After the segment loop, if not glob/nested, look for an optional `as` alias the same way the current code does (`Ident("as")` followed by `Ident(name)`) and return `UseTree::Simple { segments, alias }`.
   - Nested-group helper: consume `LBrace`; loop, calling `parse_use_tree` recursively, separated by `Comma`; allow an optional trailing comma; expect `RBrace`. Return `UseTree::Nested { segments, items }`.
4. Update the existing `use_simple_and_alias` test to assert against `u.tree` via a `UseTree::Simple { segments, alias }` match (it still has to pass; the rename is purely a shape change, not a removal).
5. Add a single new test `#[test] fn use_nested_glob_pub()` covering all three forms in one function:
   - Tokens for `use { a, b::c, d::{e, f} };` — assert `is_pub == false`, `tree` is `UseTree::Nested { segments: empty, items.len() == 3 }`, item 0 is `Simple {["a"], None}`, item 1 is `Simple {["b","c"], None}`, item 2 is `Nested { segments=["d"], items=[Simple{["e"]}, Simple{["f"]}] }`.
   - Tokens for `use foo::*;` — assert `tree` is `UseTree::Glob { segments=["foo"] }`, `is_pub == false`.
   - Tokens for `pub use bar;` — assert `is_pub == true`, `tree` is `UseTree::Simple { segments=["bar"], alias=None }`.
   - After each parse, assert `p.errors.is_empty()` and `matches!(p.peek(), TokenKind::Eof)`.
6. Run `cargo test --lib parser::item::tests::use_simple_and_alias` and the new `use_nested_glob_pub` together via `cargo test --lib parser::item::tests::use_` while iterating, then confirm with the canonical verify line below.

## Files
- `vertex_stage0/src/ast/item.rs` — Add `UseTree` enum; replace `UseDef` fields with `is_pub: bool` and `tree: UseTree`; keep `id`/`span`. No change to `Item::Use` arm or `Item::id`/`Item::span` impls.
- `vertex_stage0/src/parser/item.rs` — Rewrite `parse_use`; add `parse_use_tree` helper; update import list to keep `UseDef` and add `UseTree`. Update the existing `use_simple_and_alias` test to match the new shape; add the new `use_nested_glob_pub` test.

## Risks
- Breaking the existing `use_simple_and_alias` test by changing `UseDef` shape — mitigated by updating that test in the same commit.
- Other crates/modules might already inspect `UseDef.segments` / `UseDef.alias`. Grep confirms no consumers outside `parser/item.rs` and `ast/item.rs` (resolve, typecheck, mir, codegen are stubs). Safe to refactor.
- `Pub` consumed by `parse_use` only — items dispatched through `parse_mod_inline_item` won't yet recognize `pub use` because the dispatcher matches on `TokenKind::Use`, not `TokenKind::Pub`. That's fine: this todo's verify calls `parse_use` directly with a `Pub` lead token; broader `pub`-on-items handling is the dedicated `add-visibility-pub-pub-crate-pub-super-pub-in-path` item. Add a one-line comment near `parse_use` noting the local `pub` handling will be subsumed by that item.
- `parse_use_tree` recursion is unbounded; pathological deeply-nested input could overflow the stack. Acceptable for stage0; matches the recursive style already used by `parse_mod` for inline mods.
- Glob in non-leading position (e.g., `use foo::{bar, *}`) is not covered by the spec's example tokens. The plan limits glob to the trailing-segment form `use prefix::*;`. Nested-group children are restricted to `Simple` and `Nested` (no `*` inside braces) for this todo; broader coverage can be added later without a schema change because `UseTree` already permits it structurally — only the parser is conservative.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib parser::item::tests::use_nested_glob_pub
```

## Assumptions
- The existing `parse_use` keyword-handling for `as` (matching an `Ident("as")` rather than a dedicated keyword token) remains the project's convention — applied to both `Simple` and any future cases — since there is no `As` `TokenKind`.
- Replacing `UseDef`'s flat `segments`/`alias` fields with a `UseTree` is preferred over adding parallel fields, since stage0 has no external consumers of those fields and a tree is the canonical representation needed for nested/glob.
- The new `use_nested_glob_pub` test lives in the existing `#[cfg(test)] mod tests` block in `vertex_stage0/src/parser/item.rs` (same module as `use_simple_and_alias`), so the test path `parser::item::tests::use_nested_glob_pub` resolves.
- Adding an `is_pub: bool` to `UseDef` now (rather than waiting for the `add-visibility-...` item) is acceptable; that future item will likely generalize `is_pub` into a richer `Visibility` enum across all items, including `UseDef`. This local boolean is a stepping stone, not a permanent design.
- Top-level `use { ... };` (no prefix segments) is in scope per the spec example; `parse_use_tree` returns `UseTree::Nested { segments: vec![], items }` for that form.
- No span data on individual `UseTree` nodes for now — only `UseDef` carries a `Span`. Per-tree spans can be added later without breaking callers.
- Glob is only accepted as the trailing component (`prefix::*`) and not inside a brace group, since the spec example does not require it. The `UseTree::Glob` variant remains structurally sufficient if we relax this later.

## Blockers
Blockers: none

## Summary
Promotes `UseDef` to carry a recursive `UseTree` plus an `is_pub` flag, and rewrites `parse_use` (with a new `parse_use_tree` helper) to recognize nested groups, glob imports, and a leading `pub`, all pinned by one new `use_nested_glob_pub` test alongside the updated `use_simple_and_alias` test.

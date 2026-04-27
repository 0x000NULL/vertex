# Plan: parse-tuple-types

## Goal
Extend `parse_type` to recognize a leading `(` and produce `Type::Tuple` for `()` (unit) and `(T, U, V)` while keeping `(T)` as a parenthesized grouping, locked in by one `tuple_types` unit test.

## Steps
1. In `vertex_stage0/src/parser/ty.rs`, add a leading `LParen` branch at the top of `parse_type` (before the `Amp`/`Star`/`LBracket` arms) that delegates to a new private `parse_tuple_or_grouped_type` helper.
2. Implement `parse_tuple_or_grouped_type`:
   - `bump()` the `LParen`.
   - If the next token is `RParen`, `bump()` and return `Type::Tuple(Vec::new())` (unit type).
   - Otherwise parse one `Type` via `self.parse_type()`.
   - If the next token is `RParen` (no comma seen), `bump()` and return the inner type unchanged — `(T)` is grouping, not a tuple. This mirrors Rust semantics and keeps the 1-tuple form `(T,)` available without making `(T)` ambiguous.
   - If the next token is `Comma`, `bump()` it; then loop: if `RParen`, stop (handles trailing comma / 1-tuple `(T,)`); otherwise parse another `Type`, then expect either `Comma` (continue) or `RParen` (stop). Use `self.expect(&TokenKind::RParen)?` at the end.
   - Return `Type::Tuple(elems)`.
3. Extend the `type_span` helper to cover `Type::Tuple`: when non-empty, fall back to the span of the first element; for empty tuple, this helper currently has no caller that needs a span (only `parse_ref_type` calls it, and `&()` would otherwise hit the `unreachable!`). To be safe, return the span of the first element if present, else use a synthesized empty span via `Span::new(FileId(0), 0, 0)` — actually simpler: only `parse_ref_type` consumes this; for empty tuple we can merge with the `&` span by reusing the start span. Pragmatically, add a `Type::Tuple(elems)` arm that returns `type_span(&elems[0])` when non-empty and otherwise reuses the start span path — since the only path is via `parse_ref_type` and we have `start_span` there, accept that `&()` will need a sentinel; simplest viable choice is to return the first element's span if any, else `Span::new(FileId(0), 0, 0)` (matching what test helpers already use). Document via inline reasoning that real spans arrive when the lexer wires actual offsets in a later phase.
4. Add a `tuple_types` `#[test]` in the existing `tests` module covering: `()` → `Type::Tuple(empty)`; `(i32, u8, bool)` → `Type::Tuple` with three `Type::Path` segments asserting each `ident`; and `(i32)` → `Type::Path` with `ident == "i32"` (grouping, not a tuple). Each case asserts `p.errors.is_empty()` and the parser landed on `Eof`.

## Files
- `vertex_stage0/src/parser/ty.rs` -- add `LParen` branch in `parse_type`, new `parse_tuple_or_grouped_type` helper, `Type::Tuple` arm in `type_span`, and a `tuple_types` unit test.

## Risks
- `(T)` vs `(T,)` distinction: a naive implementation would treat `(T)` as a 1-tuple. The plan explicitly distinguishes them by inspecting whether a comma was seen before the `RParen`.
- `type_span` currently `unreachable!`s on uncovered variants; forgetting to add the `Tuple` arm would crash any future caller that wraps a tuple in a reference. Adding the arm now prevents a hidden trap when `parse-reference-types` interacts with tuple inners.
- Empty-tuple span has no natural inner element; choosing a sentinel `Span::new(FileId(0), 0, 0)` is consistent with the test helpers already in this file. Real spans land when the lexer phase wires offsets.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::ty::tests::tuple_types
```

## Assumptions
- `Type::Tuple(Vec<Type>)` in `vertex_stage0/src/ast/ty.rs` already exists (verified) and is the intended carrier for both unit `()` and n-ary tuples; unit is encoded as an empty vec rather than a separate `Type::Unit` variant.
- `(T)` parses as the grouped type `T` (matching Rust); only `(T,)` would yield a 1-tuple. The spec text `(T, U, V)`, `()` does not mention `(T)`, but distinguishing them is necessary to avoid ambiguity with future parenthesized tuple-pattern/expression parsing and to match Rust precedent.
- The verify command uses the workspace's `vertex_stage0` package (the only `Cargo.toml` location, given the source tree under `vertex_stage0/src/`); a bare `cargo test --lib` from the repo root would fail because there is no top-level Cargo manifest.
- Empty tuple `()` does not need a meaningful span in this stopgap because no currently planned caller relies on it; using `Span::new(FileId(0), 0, 0)` keeps `type_span` total.
- The test mirrors the structural style of the existing `ref_types` / `slice_and_array_types` tests (manual `Token` vector via the `tok`/`ident_tok` helpers), since the lexer is not invoked from parser unit tests in this file.

## Blockers
Blockers: none

## Summary
Adds a tuple/grouping arm to `parse_type` so `()` and `(T, U, V)` produce `Type::Tuple` while `(T)` remains a grouping, pinned by one new unit test.

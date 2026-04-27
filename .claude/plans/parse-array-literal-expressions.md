# Plan: parse-array-literal-expressions

## Goal
Add `parse_array_literal` to the expression parser so that `[a, b, c]` produces `Expr::ArrayLit` and `[value; count]` produces `Expr::ArrayRepeat`, wired into the primary head so existing postfix indexing remains unaffected.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, add a new private method `parse_array_literal(&mut self) -> Result<Expr, CompileError>` that:
   - Asserts the head is `TokenKind::LBracket` (mirroring `parse_paren_or_tuple`'s shape) and bumps the `[`, capturing its span.
   - Handles the empty case `[]` by returning `Expr::ArrayLit` with `elems: vec![]` and the merged `[`..`]` span.
   - Otherwise calls `self.parse_expr()` for the first element, then peeks:
     - `Semi` → bump, parse the count expression with `self.parse_expr()`, expect `RBracket`, return `Expr::ArrayRepeat { value: Box::new(first), count: Box::new(count), ... }`.
     - `Comma` → bump, loop collecting more elements (`parse_expr` then optional trailing `Comma`) until `RBracket` or `Eof`, expect `RBracket`, return `Expr::ArrayLit` with the collected `elems`.
     - `RBracket` → bump, return single-element `Expr::ArrayLit` with `elems: vec![first]`.
     - Otherwise produce an `unexpected_token_error("`,` , `;` , or `]`")`.
   - Allocates the `NodeId` via `self.new_node_id()` and merges the `[` span with the `]` span for the resulting node `span`.
2. Extend `parse_primary_for_paren` (lines 562–571) with a `TokenKind::LBracket => self.parse_array_literal(),` arm so an array literal is recognised as a primary head. Postfix indexing (`parse_postfix`'s existing `LBracket` arm) is unaffected because that arm only fires after a primary has already been produced.
3. Add an `array_literal_and_repeat` unit test in the existing `tests` module (`vertex_stage0/src/parser/expr.rs`) with one sub-case per form:
   - `[]` → `Expr::ArrayLit` with `elems.len() == 0`, asserts position consumed both brackets, no errors.
   - `[1i32]` → `Expr::ArrayLit` with one element.
   - `[1i32, 2i32, 3i32]` → `Expr::ArrayLit` with three elements.
   - `[1i32, 2i32,]` (trailing comma) → `Expr::ArrayLit` with two elements.
   - `[0i32; 4i32]` → `Expr::ArrayRepeat` with `value` = IntLit 0 and `count` = IntLit 4.
   - Negative case: `[1i32` (missing `]`) → `parse_array_literal` returns `Err`.

## Files
- `vertex_stage0/src/parser/expr.rs` -- add `parse_array_literal`, add `TokenKind::LBracket` arm in `parse_primary_for_paren`, add `array_literal_and_repeat` unit test.

## Risks
- **Conflict with postfix indexing**: `parse_postfix`'s `LBracket` arm consumes `[expr]` as indexing on whatever primary just parsed. Adding `LBracket` to the primary set means the very first `[` is always an array literal (correct: `[1,2,3][0]` should be index of an array, and the postfix loop will run *after* `parse_primary_for_paren` returns the array). No regression expected, but worth eyeballing the existing `index_cast_try` test path.
- **Range-prefix RHS starter set** (`range_rhs_starts_here`, expr.rs:593): does not include `LBracket`, so `..[1,2]` would currently surface as a bare `..` followed by stray tokens. The TODO comment already flags this; out of scope for this item but worth noting.
- Empty-array `[]` is accepted (consistent with the `()` unit/tuple shape). If the spec disallows empty arrays at the parser level, the test case for `[]` would need to assert an error instead.

## Prereqs
Prereqs: none

(The `ArrayLit` and `ArrayRepeat` AST variants already exist at `vertex_stage0/src/ast/expr.rs:259–274` and are wired into the `Expr` enum's `id`/`span` impls, so no AST work is required.)

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::expr::tests::array_literal_and_repeat
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate root is `vertex_stage0/` (per `Cargo.toml` location implied by `vertex_stage0/src/...`); both verify commands pass `--manifest-path` to be unambiguous when `bash -c` runs from the repo root.
- `[]` parses as a zero-element `ArrayLit` (no separate `Empty` variant exists; the AST has only `ArrayLit { elems: Vec<Expr> }` and `ArrayRepeat`).
- Trailing commas are accepted in the comma-separated form (mirrors `parse_paren_or_tuple`'s trailing-comma handling at expr.rs:129–134).
- The repeat form parses both `value` and `count` with the full `parse_expr` (not `parse_binary` or a literal-only path), matching how `parse_postfix`'s indexing arm already calls `parse_expr` for the index expression at expr.rs:459.
- `LBracket` is added to `parse_primary_for_paren` (the current literal-only stub) rather than to a new `parse_primary` — the stub is the actual primary entry point until the larger Pratt/primary refactor lands.
- The new test follows the existing `Token::new` + `Span::new(FileId(0), 0, 0)` pattern and uses `IntSuffix::I32` literals, matching every other test in the module.
- No update to `range_rhs_starts_here` is in scope; that touches range-prefix behaviour and would belong with a "primary head set" cleanup item.
- Use `IntLiteral(0, IntSuffix::I32)` rather than `IntLiteral(0, IntSuffix::Usize)` for the repeat-count test, since the parser does not type-check suffixes at this stage and the existing tests consistently use `I32`.

## Blockers
Blockers: none

## Summary
Adds `parse_array_literal` plus a `LBracket` head arm in `parse_primary_for_paren`, with a six-case `array_literal_and_repeat` unit test covering empty, single, multi, trailing-comma, repeat, and missing-`]` error forms.

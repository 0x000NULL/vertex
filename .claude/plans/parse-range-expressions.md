# Plan: parse-range-expressions

## Goal
Wire up `..` and `..=` so `parse_expr` produces `Expr::Range` for the five spec forms (`a..b`, `a..=b`, `a..`, `..b`, `..`), without breaking any existing parser test.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, add `Range` to the import list pulled from `crate::ast::expr`.
2. Replace `parse_expr`'s body so it dispatches into a new private `parse_range` layer instead of jumping straight to `parse_binary(0)`. Keep `parse_binary`/`parse_cast`/`parse_unary` unchanged.
3. Implement `parse_range` with two arms:
   - **Prefix** (peek is `DotDot` or `DotDotEq`): bump the operator, capture its span, set `inclusive` from the kind. If `range_rhs_starts_here(peek)` is true, parse the end via `parse_binary(0)` and build `Expr::Range { start: None, end: Some(box end), inclusive, span = op_span.merge(end.span()) }`. Otherwise build `Expr::Range { start: None, end: None, inclusive, span = op_span }`.
   - **Infix**: call `parse_binary(0)` for the LHS, then peek; if it's `DotDot`/`DotDotEq`, bump, capture span, set `inclusive`, and either parse RHS via `parse_binary(0)` (when `range_rhs_starts_here`) or emit a no-RHS `Range`. If no `DotDot[Eq]` follows the LHS, just return the LHS unchanged so existing non-range expressions are untouched.
4. Add a private free function `range_rhs_starts_here(kind: &TokenKind) -> bool` that returns true for the FIRST set the existing parser actually accepts: `IntLiteral`, `FloatLiteral`, `CharLiteral`, `StringLiteral`, `RawStringLiteral`, `True`, `False`, `LParen`, `Minus`, `Not`, `Star`, `Amp`. (Anything outside this set means "no RHS"; this is the same accept-set as `parse_primary_for_paren` plus the four unary-prefix tokens accepted by `parse_unary`.)
5. Allocate a fresh `NodeId` for every `Range` node via `self.new_node_id()`, mirroring the pattern in `parse_unary`/`parse_postfix`.
6. Add `#[test] fn range_forms()` in `mod tests` covering all five spec forms with `IntLiteral(_, IntSuffix::I32)` operands plus a trailing `Eof`. Each case asserts: variant is `Expr::Range`, `inclusive` matches, `start`/`end` are the expected `Some(IntLit{value=..})`/`None`, `p.pos` consumed exactly the range tokens, and `p.errors.is_empty()`. Use the existing `int_tok` and `tok` helpers already defined at the top of the test module.

## Files
- `vertex_stage0/src/parser/expr.rs` — add `Range` to the import set, rewrite `parse_expr`, add `parse_range`, add `range_rhs_starts_here`, add the `range_forms` unit test.

## Risks
- **Precedence vs assignment is technically wrong.** Per Rust, range binds tighter than assignment, but we layer range OUTSIDE `parse_binary(0)` (which still owns assignment). So `a = b..c` will parse as `Range(Assign(a, b), c)` instead of the correct `Assign(a, Range(b, c))`. No existing or new test in this item exercises this mix. Pulling assignment out of the binary table is out of scope — flag for a future cleanup item.
- **Chained ranges (`a..b..c`) silently leave trailing tokens.** With the single-shot infix check, `a..b..c` parses as `Range(a, b)` and leaves `..c` in the stream. The caller of `parse_expr` will eventually choke. Not exercised by `range_forms`. If we want to be strict here, after building the range we could peek for another `DotDot`/`DotDotEq` and emit `E0100` "range expressions are non-associative" — analogous to the existing comparison guard. Defer unless `range_forms` requires it.
- **`range_rhs_starts_here` set drift.** The FIRST set is hand-maintained against the current parser surface (literals + `(` + four unary-prefix ops). When later items add `Ident`/`If`/`Match`/`Loop`/`Block`/`[`/path heads as expression starters, this list MUST be extended or `a.. <new-form>` will be misparsed as `a..` followed by stray tokens. Add a TODO comment on the helper noting this.
- The new layer adds another stack frame to every expression parse; trivial perf cost only.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::expr::tests::range_forms
cargo test --lib --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate root is `vertex_stage0/Cargo.toml`; `cargo test --lib` from the repo root would not find the package, so the manifest path is required.
- The `Expr::Range(Range { id, span, start, end, inclusive })` AST variant from `src/ast/expr.rs` is the intended shape — already defined with `start: Option<Box<Expr>>`, `end: Option<Box<Expr>>`, `inclusive: bool`. No AST change needed.
- The `range_forms` test name and module path (`parser::expr::tests::range_forms`) follows the convention used by sibling tests `unary_prefix`, `operator_precedence`, `call_method_field`, `index_cast_try`, etc., all defined in the same `#[cfg(test)] mod tests` block in `src/parser/expr.rs`.
- Calling `parse_binary(0)` for both the LHS and (optional) RHS of a range is acceptable even though it allows assignment to slip into the LHS; see the precedence risk above. This matches the minimal-blast-radius design.
- `..=` (prefix inclusive-to-end) is supported symmetrically with `..b` since the lexer already produces `DotDotEq`. The spec only enumerates `..b`, but allowing `..=b` requires zero extra code and is consistent with Rust.
- An "absent RHS" is detected by token-class lookahead, not by a speculative parse-and-rewind; the helper enumerates exactly the tokens today's `parse_unary`/`parse_primary_for_paren` accept. False negatives become "stops too early"; false positives would be "tries to parse and errors". The chosen set errs on the side of false negatives, which matches Rust's behavior where `for x in 0.. { ... }` correctly stops at `{`.
- No new `pub` API is introduced; `parse_range` and `range_rhs_starts_here` are private to the module.

## Blockers
Blockers: none

## Summary
Adds a `parse_range` wrapper around `parse_binary(0)` plus a `range_forms` unit test, producing `Expr::Range` for the five spec forms while leaving all existing expression parsing paths intact.

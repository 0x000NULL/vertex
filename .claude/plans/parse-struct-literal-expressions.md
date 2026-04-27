# Plan: parse-struct-literal-expressions

## Goal
Add a `parse_struct_lit` arm so the primary parser produces `Expr::StructLit { path, fields, base }` for `Path { f: v, ..base }` forms (including empty, shorthand, and trailing comma), and add a `Parser::restrict_struct_literal` flag that suppresses this interpretation so future `if`/`while`/`for`/`match` heads can disambiguate `if Foo { ... }` per Rust convention.

## Steps
1. In `vertex_stage0/src/parser/mod.rs`, add a `pub(crate) restrict_struct_literal: bool` field to `Parser`, initialized to `false` in `Parser::new`. This is the single piece of state that downstream control-flow-head items will toggle; this plan only introduces the lever and consumes it at the struct-literal site — it does NOT touch `parse_if`/`parse_while` (those are separate pending items).
2. In `vertex_stage0/src/parser/expr.rs`, extend the primary head: after `parse_path` produces a `Path`, peek for `LBrace`. If present AND `!self.restrict_struct_literal` AND the next token after `{` is a plausible struct-literal head (`Ident`, `DotDot`, or `RBrace` for the empty case), consume it as a struct literal via a new `parse_struct_lit_after_path(path)` helper. Otherwise return `Expr::Path(path)` unchanged. Wire the path arm into `parse_primary_for_paren` for the `Ident(_)` and `SelfUpper` heads (assumes `parse_path` from prereq is callable; if it returns just a single-segment `Path` for non-`::` inputs, that is fine).
3. Implement `parse_struct_lit_after_path(path)`:
   - `expect(&LBrace)`.
   - Loop until `RBrace`/`Eof`:
     - If `peek == DotDot`: bump; parse `base = parse_expr()`; break (must be the last element). Track an `Option<Box<Expr>>`.
     - Else expect `Ident(name)`; bump.
       - If `peek == Colon`: bump; parse `value = parse_expr()`; push `StructLitField { name, value }`.
       - Else (shorthand per spec `field_init = identifier [ ":" expression ]`): synthesize `value = Expr::Path(Path { single segment named `name`, span = ident span })` and push the field. Use `new_node_id()` for the synthesized path id.
     - If `peek == Comma`: bump; continue. Else break.
   - `expect(&RBrace)`. Build span = `path.span.merge(&rbrace.span)`. Return `Expr::StructLit(StructLit { id, span, path, fields, base })`.
4. Add unit test `struct_literal` under `parser::expr::tests` covering: `Foo {}`; `Foo { x: 1 }`; `Foo { x: 1, y: 2 }`; trailing comma `Foo { x: 1, }`; field shorthand `Foo { x }`; base only `Foo { ..base }`; mixed `Foo { x: 1, ..base }`; multi-segment path `a::b::Foo { x: 1 }`; and a disambiguation case where the parser is constructed and `p.restrict_struct_literal = true;` is set before parsing `Foo { x: 1 }` — assert the result is `Expr::Path` with the `{ ... }` left unconsumed (peek == `LBrace`).
5. Span/id discipline: every new `StructLit` gets a fresh id from `new_node_id`; span runs from the path's first token through the closing `}`. Synthesized shorthand path values also get a fresh id. No reuse of ids.
6. Run the verify command.

## Files
- `vertex_stage0/src/parser/mod.rs` — add `restrict_struct_literal: bool` field on `Parser` (default `false`).
- `vertex_stage0/src/parser/expr.rs` — import `StructLit`, `StructLitField`, `Path`, `PathSegment` from `crate::ast::expr`; add `parse_struct_lit_after_path`; route the path-head arm of `parse_primary_for_paren` (Ident/SelfUpper) through `parse_path` and then optionally into the struct-literal branch; add the `struct_literal` test.

## Risks
- **Path arm regression.** `parse_primary_for_paren` is the same primary used by `parse_postfix`. If the path arm is added incorrectly, every postfix (`f()`, `x.y`, `x[i]`, etc.) on an identifier could break. Mitigation: keep the struct-literal lookahead conditional and *return the bare path* when the lookahead doesn't match, leaving postfix handling untouched.
- **Block stub collision.** The closure body uses `parse_block_stub` only when `peek == LBrace`. Closure parameter heads like `|x| Foo { ... }` could now be interpreted as struct literals — that's actually the correct Rust behavior (closure body extends maximally), so no special-case needed there.
- **Empty-body ambiguity.** `Foo {}` is a valid struct literal but `{}` after a path inside an `if`/`while` head is exactly the case we want suppressed. The flag handles this; without the flag the heuristic "next-after-`{` is `Ident`/`DotDot`/`RBrace`" still matches `{}`, so the flag is the load-bearing disambiguator (not the lookahead).
- **`parse_path` shape.** This plan assumes `parse_path` returns `Result<Path, CompileError>` and that `Path` is the same type defined in `ast::expr`. If the prereq item stores path elsewhere (e.g., as `Expr::Path` only), step 2 must extract the inner `Path` from `Expr::Path` instead. Either adaptation is local to `parse_primary_for_paren`.
- **Spec vs. AST scope.** Spec grammar (line 3356) does not list `..base`, but the AST already has `base: Option<Box<Expr>>` and the todo bullet explicitly requires it. Following the todo bullet.

## Prereqs
- parse-path-expressions

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::expr::tests::struct_literal
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The prereq `parse-path-expressions` lands a `Parser::parse_path(&mut self) -> Result<Path, CompileError>` (or returns `Expr::Path` whose inner `Path` is trivially extractable) and routes `Ident`/`SelfUpper` heads through it from `parse_primary_for_paren`. If it does not, this plan additionally adds a single-segment `parse_path` shim sufficient for the test corpus.
- `restrict_struct_literal` is added as a plain `bool` field on `Parser` rather than a stack/scope guard; future control-flow-head items will save-and-restore around their head expressions. No RAII helper is added now to avoid premature abstraction.
- Field shorthand `Foo { x }` is supported per spec line 3357 (`field_init = identifier [ ":" expression ]`) by synthesizing an `Expr::Path` value with a single segment matching the field name. The synthesized path uses `generic_args: vec![]`.
- `..base` is parsed eagerly as `parse_expr()`; a trailing comma after the base (`{ ..base, }`) is rejected as a syntax error, matching Rust.
- `Foo { ..base, x: 1 }` (base before fields) is rejected: once we enter the `..` arm we break the loop and require `RBrace` next.
- Multi-segment path heads use the `parse_path` from the prereq; tuple-struct/unit-struct construction (`Foo` / `Foo(1)`) is intentionally out of scope — they remain `Expr::Path` and `Expr::Call`, which downstream resolution turns into ctor calls.
- Tests live in the existing `parser::expr::tests` module with the test name `struct_literal` exactly, matching the verify command.
- `Parser::restrict_struct_literal` is `pub(crate)` (or has a `pub(crate)` setter) so future control-flow-head code in the same crate can flip it; the unit test in `parser::expr::tests` likewise has crate-internal access.

## Blockers
### Blocker: shape of parse_path return type
- severity: cross-item
- affects: parse-path-expressions, parse-struct-literal-expressions, parse-function-call-method-call-field-access
- question: Does the prereq `parse-path-expressions` plan return `Result<Path, CompileError>` (the `ast::expr::Path` struct) or `Result<Expr, CompileError>` wrapping `Expr::Path(Path)`?
- default_assumption: Assume it returns `Result<Path, CompileError>`. If it returns `Result<Expr, CompileError>` instead, destructure `Expr::Path(p) => p` at the call site in `parse_primary_for_paren`; this is a one-line adaptation and does not change the rest of the plan.

### Blocker: scope of disambiguation lever in this commit
- severity: local
- affects: parse-if-else-expressions, parse-loop-while-for-expressions, parse-match-expressions
- question: Should this commit also wire `restrict_struct_literal=true` into the (currently nonexistent) `if`/`while`/`for`/`match` head parsers, or only land the flag and let those items consume it?
- default_assumption: Land the flag and the consumer site only; do NOT modify `if`/`while`/`for`/`match` parsers (they don't exist yet). Document in the test that the flag suppresses struct-literal interpretation, leaving full integration to the dedicated control-flow items.

## Summary
Adds `parse_struct_lit_after_path` plus a `restrict_struct_literal` parser flag, producing `Expr::StructLit` for `Path { … }` forms (including shorthand and `..base`) while giving future `if`/`while` heads a single switch to suppress the struct-literal interpretation per Rust convention.

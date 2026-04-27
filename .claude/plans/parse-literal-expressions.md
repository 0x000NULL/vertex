# Plan: parse-literal-expressions

## Goal
Add `Parser::parse_int_lit`, `parse_float_lit`, `parse_char_lit`, `parse_str_lit`, and `parse_bool_lit` in a new `src/parser/expr.rs` submodule that turns lexer literal tokens into `Expr::IntLit`/`FloatLit`/`CharLit`/`StrLit`/`BoolLit` AST nodes.

## Steps
1. Add a `next_node_id: u32` field to `Parser` (initialized to 0 in `Parser::new`) and a private `Parser::new_node_id(&mut self) -> NodeId` helper that post-increments and wraps in `NodeId`. This is required because the literal-expression constructors all take a `NodeId` and no allocator exists on the parser yet.
2. Create `src/parser/expr.rs` and declare it as `pub mod expr;` from `src/parser/mod.rs`.
3. In `expr.rs`, write `impl Parser { ... }` adding five methods, each returning `Result<Expr, CompileError>`:
   - `parse_int_lit`: peek for `TokenKind::IntLiteral(value, suffix)`. If matched, bump and build `Expr::IntLit(IntLit { id, span, value, suffix })`. Otherwise call `expected_token_error` against a sentinel `IntLiteral(0, IntSuffix::Unsuffixed)` and return the same `CompileError` (but for the non-batched method shape, just call `self.expect(...)`-style: produce the error via the parser's standard error path). Use a local `match self.bump()` after `peek` to extract the value/suffix without cloning.
   - `parse_float_lit`: same pattern but with `FloatLiteral(value, suffix)` → `Expr::FloatLit(FloatLit { id, span, value, suffix })`.
   - `parse_char_lit`: same pattern but with `CharLiteral(c)` → `Expr::CharLit(CharLit { id, span, value: c })`.
   - `parse_str_lit`: accept either `StringLiteral(s)` or `RawStringLiteral(s)`; both produce `Expr::StrLit(StrLit { id, span, value: s })`. Cloning the `String` out of the borrowed peek is avoided by branching on `peek` then pattern-matching on the bumped `Token` (move out of `tok.kind`).
   - `parse_bool_lit`: accept `TokenKind::True` → `value=true`, `TokenKind::False` → `value=false`, build `Expr::BoolLit(BoolLit { id, span, value })`.
4. Each method consumes exactly one token, allocates one `NodeId`, and uses the consumed token's `Span` as the literal's span. None of these methods push to `self.errors`; they return a `Result` so the caller decides.
5. Add a `#[cfg(test)] mod tests` to `src/parser/expr.rs` containing a single test function `literal_expressions` that:
   - For each of the five parsers, builds a `Vec<Token>` with one literal token followed by `Eof`, constructs a `Parser`, calls the method, asserts `Ok(Expr::XxxLit(_))` is returned with the correct value/suffix, and asserts `parser.pos == 1` (token was consumed) and `parser.errors.is_empty()`.
   - Exercises both `StringLiteral` and `RawStringLiteral` through `parse_str_lit`.
   - Exercises a "wrong token" path for one method (e.g., feed `Plus` to `parse_int_lit`) to confirm it returns `Err(_)` and does not advance `pos`.

## Files
- `src/parser/mod.rs` -- declare `pub mod expr;`; add `next_node_id: u32` field to `Parser`; initialize it in `Parser::new`; add private `fn new_node_id(&mut self) -> NodeId`.
- `src/parser/expr.rs` -- new file: imports (`crate::ast::expr::*`, `crate::ast::NodeId`, `crate::error::{CompileError, ErrorCode, ErrorKind}`, `crate::lexer::token::{TokenKind, IntSuffix, FloatSuffix}`), the five `impl Parser` methods, and `mod tests` with `fn literal_expressions`.

## Risks
- `parse_int_lit` returns a `Result` rather than calling `expected_token_error` (which pushes to `errors` and recovers). This differs from the existing `expected_token_error` recovery path — but most expression-level parsers in Rust-style compilers prefer to bubble errors so the caller can decide between recovery and fallthrough (e.g., `parse_primary` will try other alternatives before erroring). If a later item (e.g., `pratt-parser-for-binary-operators`) prefers the push-and-recover style, it can wrap these.
- Owning the `String` payload of `StringLiteral`/`RawStringLiteral` requires moving out of the bumped token's `kind`; the existing `bump` clones the whole token. This is fine functionally but is a small extra allocation per string literal — acceptable for stage0.
- Adding `next_node_id` to `Parser` is a small public-shape change; later items expecting a different allocator (e.g., centralized in `Arena`) would need to refactor.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::expr::tests::literal_expressions
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate manifest is `vertex_stage0/Cargo.toml` (verified: `vertex_stage0/src/lib.rs` exists; the runner's verify uses `cargo test --lib parser::expr::tests::literal_expressions`, which succeeds when run from `vertex_stage0/` or with `--manifest-path`). I include `--manifest-path vertex_stage0/Cargo.toml` so the verify works regardless of CWD.
- `NodeId` allocation lives on the `Parser` (not in `Arena`). The pending `wire-arena-allocator-into-ast` item is independent; today `Arena` only stores `()`, so nothing in tree provides node IDs. A simple `u32` counter on `Parser` is the lowest-risk choice and trivially refactorable later.
- The five parser methods return `Result<Expr, CompileError>`, not `Result<Expr, ()>` and not pushing to `errors`. The error span is the current span; the error code is `E0100` ("unexpected token") with `ErrorKind::Syntax`, matching `expect`'s convention.
- `parse_str_lit` accepts both regular and raw string literals because both lower to `StrLit` (the lexer has already stripped `r#""#` framing per the lexer items). The AST has no separate "raw" flag.
- `parse_bool_lit` reads `TokenKind::True` / `TokenKind::False` (keywords), not a hypothetical `BoolLiteral` variant — confirmed by `src/lexer/token.rs:39,58`.
- The test name in the verify command (`literal_expressions`) corresponds to a single `#[test] fn literal_expressions(...)` containing all five parser checks (not five separate tests). The verify runs that one test by exact path.
- No changes to `lexer/`, `ast/expr.rs`, or `ast/mod.rs` are required; the AST already exposes `IntLit`, `FloatLit`, `CharLit`, `StrLit`, `BoolLit`, all `pub`, all carrying `id`, `span`, and the relevant payload.

## Blockers
Blockers: none

## Summary
Adds five literal-expression parser methods in a new `parser::expr` module plus a per-`Parser` `NodeId` counter, unblocking primary-expression parsing for all subsequent expression items.

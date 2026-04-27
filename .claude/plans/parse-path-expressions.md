# Plan: parse-path-expressions

## Goal
Add `Parser::parse_path` to `src/parser/expr.rs` that parses `a::b::c` and `Type::<T>::method` turbofish paths into the existing `ast::expr::Path` / `PathSegment`, plus a `path_with_turbofish` unit test.

## Steps
1. In `src/parser/expr.rs`, add `pub fn parse_path(&mut self) -> Result<Expr, CompileError>` on the existing `impl Parser` block.
2. Validate the head token: accept `TokenKind::Ident(_)`, `TokenKind::SelfLower`, or `TokenKind::SelfUpper`; on mismatch return `self.unexpected_token_error("path")` without advancing.
3. Bump the head token, capture its span, and convert it to the segment's `ident` String (`Ident(s)` → `s`, `SelfLower` → `"self"`, `SelfUpper` → `"Self"`); start a `Vec<PathSegment>` with `generic_args: Vec::new()`.
4. Loop: while `peek() == ColonColon`, look at `peek_at(1)`:
   - If `Lt` (turbofish `::<`): bump `::`, bump `<`, then parse a comma-separated argument list — for each arg, bump exactly one token (any kind) and push `GenericArg::Placeholder`; allow trailing comma; expect `Gt` and bump it; attach the args vector to the *last* segment already in the list (i.e. the one preceding the `::<`). Update running span to the `>` token's end.
   - Else if next is `Ident`/`SelfLower`/`SelfUpper`: bump `::`, bump the ident, push a new `PathSegment` with that name and empty generic args. Update running span.
   - Else: stop the loop (do not consume the `::`); fall out so callers can decide.
5. Allocate a `NodeId`, build `Path { id, span: merged_span, segments }`, return `Ok(Expr::Path(path))`.
6. Add a `#[test] fn path_with_turbofish` inside the existing `tests` module that drives:
   - `a :: b :: c` → 3 segments named `"a"`, `"b"`, `"c"`, all with empty generic args.
   - `Type :: < T > :: method` → 2 segments: `"Type"` with one `GenericArg::Placeholder`, and `"method"` with empty args.
   - Assert `p.pos` points at the trailing `Eof`, `p.errors.is_empty()`, and that the result matches `Expr::Path(_)`.

## Files
- `vertex_stage0/src/parser/expr.rs` — add `parse_path` method on `impl Parser` and the `path_with_turbofish` unit test in `mod tests`. No changes to AST, lexer, or parser/mod.

## Risks
- The placeholder turbofish-arg parser swallows exactly one token per arg, so a multi-token arg like `Vec<T>` inside a turbofish would mis-tokenize. Acceptable today: the resolved blocker explicitly says use `GenericArg::Placeholder` and let the generics migration redo it; the verify test uses single-ident args only.
- A bare `::` followed by `<` mid-path is committed (we bump `::` before checking for the head ident); if a non-turbofish `<` followed `::`, that would be a parse error anyway, so committing is fine.
- Span merging assumes `Span::merge` works across the head and final consumed token (it does — see `span.rs`).

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::expr::tests::path_with_turbofish
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- Per the resolved blocker, each turbofish argument is stored as `GenericArg::Placeholder`; the actual type parsing arrives with the generics-item plan.
- Per the resolved blocker, `Self` and `self` keyword tokens are accepted as a path head segment, with their ident materialized as the strings `"Self"` and `"self"`.
- The verify test name `path_with_turbofish` belongs in `parser::expr::tests` (the existing `tests` submodule of `src/parser/expr.rs`), matching the literal-tests pattern already established there.
- The Cargo manifest lives at `vertex_stage0/Cargo.toml` (only crate in the workspace), so verify commands target it explicitly.
- Consuming exactly one token per turbofish arg is sufficient because the verify test only needs `<T>`; richer arg parsing is out of scope until a real type parser exists.
- A bare `::` not followed by a head token or `<` ends the path (the loop stops without consuming `::`); the caller sees an unconsumed `::` and can recover. We do *not* synthesize an error for this case in `parse_path` itself.
- A trailing comma inside `::<...>` is tolerated to keep the placeholder loop simple; the future generics work can tighten this.

## Blockers
Blockers: none

## Summary
Implements `Parser::parse_path` for multi-segment paths and `Type::<T>::method` turbofish, populating existing `Path`/`PathSegment`/`GenericArg::Placeholder` AST nodes, plus the unit test the verify line requires.

Here is the plan.

# Plan: implement-parser-struct-in-src-parser-mod-rs

## Goal
Introduce the foundational `Parser` struct in `vertex_stage0/src/parser/mod.rs` with token cursor state, error accumulation, and the basic peek/bump/eat/expect primitives that all subsequent parser items will build on.

## Steps
1. In `vertex_stage0/src/parser/mod.rs`, add `use` imports for `crate::error::{CompileError, ErrorAccumulator, ErrorCode, ErrorKind}`, `crate::lexer::token::{Token, TokenKind}`, and `crate::span::Span`.
2. Define `pub struct Parser` with fields: `tokens: Vec<Token>`, `pos: usize`, `errors: ErrorAccumulator`. (Match the field types named in the spec verbatim.)
3. Add a `pub fn new(tokens: Vec<Token>) -> Self` constructor that initializes `pos = 0` and `errors = ErrorAccumulator::new()`.
4. Add `pub fn peek(&self) -> &TokenKind` — returns the kind of `tokens[pos]`. Since the lexer guarantees a trailing `Eof` token, indexing past the end is a bug; if `pos >= tokens.len()`, fall back to a static `&TokenKind::Eof` (so callers never panic on a well-formed token stream that was over-consumed by a recovery path).
5. Add `pub fn peek_at(&self, offset: usize) -> &TokenKind` — same fallback semantics for `tokens[pos + offset]`.
6. Add `pub fn bump(&mut self) -> Token` — clones and returns the current token, then advances `pos` (saturating at `tokens.len()`). Cloning is acceptable because `Token` is `#[derive(Clone)]` already; the parser doesn't take ownership of the source `Vec` slot.
7. Add `pub fn eat(&mut self, kind: &TokenKind) -> bool` — if `peek()` matches `kind` by discriminant (use `std::mem::discriminant` so payload-bearing kinds like `Ident(String)` compare on variant only), call `bump()` and return `true`; else return `false`.
8. Add `pub fn expect(&mut self, kind: &TokenKind) -> Result<Token, CompileError>` — if discriminant matches, return `Ok(self.bump())`; else build a `CompileError` with `ErrorCode::E0100` ("unexpected token"), `ErrorKind::Syntax`, the current token's span, and a message of the form `"expected {expected}, found {found}"` using a small local `describe(&TokenKind) -> &'static str` helper that returns short stable names for variants. Do **not** auto-push to `self.errors`; let callers decide (the spec says it returns the error).
9. Add a `#[cfg(test)] mod tests` block with a single test `peek_and_bump_basics` that builds a `Parser` from a hand-rolled token stream of `[Plus, Minus, Eof]` (each with a dummy span), then asserts: initial `peek()` is `Plus`; `peek_at(1)` is `Minus`; `bump()` returns a token whose kind is `Plus`; subsequent `peek()` is `Minus`; `eat(&Minus)` returns `true`; `eat(&Star)` returns `false`; `peek()` is `Eof`; `expect(&Eof)` returns `Ok`.
10. Run `cargo test --lib parser::tests::peek_and_bump_basics` to confirm.

## Files
- `vertex_stage0/src/parser/mod.rs` — fill in the currently empty file with the `Parser` struct, the six methods, and the `peek_and_bump_basics` test module.

## Risks
- The spec calls `peek` and `peek_at` without specifying return type. Returning `&TokenKind` (rather than `Option<&Token>`) is a bet that's friendlier for the upcoming Pratt parser items but is a one-way ergonomic choice; if a later item needs the *span* of the peeked token it will need a separate `peek_token`/`peek_span` accessor (cheap to add later).
- `eat` taking `&TokenKind` means callers who want to match payload-bearing variants (e.g. a *specific* `Ident("foo")`) cannot use it. That's deliberate — `eat`/`expect` are for fixed punctuation/keywords; payload-bearing matches will go through dedicated parser methods. Document this implicitly via the `discriminant` comparison.
- Cloning whole `Token`s on every `bump` allocates for `StringLiteral`/`Ident`. Acceptable for stage0; can be revisited (e.g. `mem::replace` with a sentinel) if profiling later flags it.

## Prereqs
- define-token-struct
- define-tokenkind-enum-in-src-lexer-token-rs-keyword-variants
- implement-span-struct-in-src-span-rs
- define-errorcode-and-errorkind-in-src-error-rs
- define-compileerror-struct-in-src-error-rs
- implement-erroraccumulator-in-src-error-rs

(All six are already merged on `main` per the file inspection above, so this plan is unblocked in practice — they're listed only because they are upstream dependencies.)

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::tests::peek_and_bump_basics
cargo build --manifest-path vertex_stage0/Cargo.toml
grep -q "pub struct Parser" vertex_stage0/src/parser/mod.rs
grep -q "tokens: Vec<Token>" vertex_stage0/src/parser/mod.rs
grep -q "errors: ErrorAccumulator" vertex_stage0/src/parser/mod.rs
grep -q "fn expect" vertex_stage0/src/parser/mod.rs
```

## Assumptions
- Working directory for `cargo` is the repo root, which contains a workspace `Cargo.toml`. Using `--manifest-path vertex_stage0/Cargo.toml` makes the verify commands location-independent and unambiguous (the workspace also resolves `cargo test --lib` from the root, but the explicit form survives any future workspace shuffles).
- `peek`/`peek_at` returning `&TokenKind` (not `Option<&TokenKind>` and not `&Token`) is the most ergonomic shape for upcoming Pratt-style parsing where callers branch on kind. Spans of the current token are still reachable via `&self.tokens[self.pos].span` in a future helper.
- `eat`/`expect` compare by `mem::discriminant`, since the spec phrasing "advance if match" only makes sense for variant-level matching when many variants carry payloads.
- `expect` returns the error to the caller without auto-pushing it onto `self.errors`. This keeps the primitive composable; recovery code in later items will decide whether to push or attach as a label.
- The test `peek_and_bump_basics` is not pre-written by the runner, so this plan creates it. The verify command’s exact path `parser::tests::peek_and_bump_basics` matches a `mod tests` inside `parser/mod.rs` containing `fn peek_and_bump_basics`.
- No changes are needed to `lib.rs` — `pub mod parser;` is already declared.
- A trailing `Eof` token in the stream is the lexer's contract (already used in `Scanner` design), so the static `Eof` fallback in `peek` is defensive only and won't trigger in well-formed inputs.

## Blockers
Blockers: none

## Summary
Creates the `Parser` cursor + error-accumulator scaffold and the peek/bump/eat/expect primitives that every subsequent parser item depends on, plus a smoke test proving they wire together.

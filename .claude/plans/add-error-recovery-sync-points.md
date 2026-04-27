# Plan: add-error-recovery-sync-points

## Goal
Add a `Parser::recover_to_sync` method that advances past garbage tokens to the next safe synchronization point, plus an `expected_token_error` helper that uses it, so future parser items can recover from syntax errors without losing track.

## Steps
1. In `vertex_stage0/src/parser/mod.rs`, add a private helper `is_sync_point(&TokenKind) -> bool` that returns true for `Semi`, `RBrace`, EOF, and item-start keywords (`Fn`, `Struct`, `Enum`, `Trait`, `Impl`, `Mod`, `Use`, `Const`, `Static`, `Type`, `Pub`, `Unsafe`, `Extern`).
2. Add `pub fn recover_to_sync(&mut self)` to `impl Parser`. Loop: while `peek()` is not a sync point and not EOF, call `bump()`. After the loop, if peek is `Semi`, also `bump()` past the semicolon (so caller resumes on the next statement); if peek is `RBrace` or an item-start keyword or EOF, leave the cursor on it (so the caller's enclosing parser sees the close brace / next item).
3. Add `pub fn expected_token_error(&mut self, expected: &TokenKind)` that constructs the same `CompileError` that today's `expect` returns (E0100, Syntax, current span, "expected X, found Y"), pushes it onto `self.errors`, then calls `self.recover_to_sync()`. This gives subsequent parser items one canonical "report and resync" entry point.
4. Refactor `expect` to delegate to `expected_token_error` on the failure branch instead of building the error inline — `expect` still returns `Result<Token, CompileError>` for callers that want to bail, but the error is also accumulated and the cursor is advanced. (Alternative: leave `expect` alone and only have new callers use `expected_token_error`. Pick the non-refactor option to keep the change tight — see Assumptions.)
5. Add the unit test `recovery_advances_past_garbage` inside `mod tests` in `parser/mod.rs`. Build a token stream like `[Plus, Star, Star, Semi, Fn, Eof]` (a few "garbage" tokens, then a sync point), call `recover_to_sync`, and assert the cursor lands on the token immediately after the `Semi` (i.e. `peek() == Fn`). Add a second sub-assertion: starting at `[Plus, Star, RBrace, Eof]`, recovery stops with `peek() == RBrace` (cursor not advanced past it). Add a third: `[Plus, Star, Fn, Eof]` stops with `peek() == Fn`.
6. `cargo fmt` and `cargo test --lib parser::tests::recovery_advances_past_garbage`.

## Files
- `vertex_stage0/src/parser/mod.rs` — add `is_sync_point` helper, `recover_to_sync` method, `expected_token_error` method, and the `recovery_advances_past_garbage` test.

## Risks
- **Sync set choice**: spec language ("synchronization points: semicolons, braces, item boundaries") doesn't fully enumerate which keywords count as item starts. If a later item adds `pub(crate)` parsing or treats `async`/`async fn` as item starts, the sync set may need expansion. Mitigation: keep the helper private and centralized so it's easy to extend.
- **Off-by-one on the semicolon**: deciding whether to consume the semicolon or stop on it changes downstream parser behavior. Stopping *after* the semi matches typical recovery (caller's "parse next statement" loop sees the next stmt's first token); stopping *on* `RBrace`/item-start lets the enclosing parser close its block naturally. Mismatch here will cascade into every parser item that uses recovery.
- **Test brittleness**: the test names tokens but not spans. Using `Span::new(FileId(0), 0, 0)` for every token is fine for behavior, but the error pushed into the accumulator will dedup on `(code, file_id, start)`, which is irrelevant to this test.
- **`expect` refactor scope**: rewriting `expect` to push-and-recover changes its contract for the existing `peek_and_bump_basics` test. Keep `expect` as-is for now (see Assumptions).

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::tests::recovery_advances_past_garbage
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The Cargo workspace lives at `vertex_stage0/`, so verify commands pass `--manifest-path vertex_stage0/Cargo.toml`. (Confirmed via `vertex_stage0/Cargo.toml`.)
- "Item-start keywords" means the keywords that begin a top-level item per the spec: `fn`, `struct`, `enum`, `trait`, `impl`, `mod`, `use`, `const`, `static`, `type`, plus the modifiers `pub`, `unsafe`, `extern` that can lead an item. Existing `TokenKind` covers all of these except `Static` — and `TokenKind` does not currently have a `Static` variant, so it is omitted from the sync set for now (will be added when the static-item parser item lands).
- `recover_to_sync` consumes a trailing `;` (so the caller's stmt-loop resumes on the *next* stmt) but stops *on* `}` and item-start keywords (so the enclosing block/item parser sees the boundary).
- `expected_token_error` is a new method, not a rename of the existing failure path inside `expect`. `expect` is left unchanged so the existing `peek_and_bump_basics` test still passes; future parser items will call `expected_token_error` directly when they want to push-and-recover instead of bailing.
- The test lives in the existing `mod tests` block in `parser/mod.rs` (matches the verify command's `parser::tests::recovery_advances_past_garbage` path).
- `bump()` at EOF is safe: it returns the last token without advancing past `tokens.len()`, so a recovery loop that hits EOF terminates cleanly via the `is_sync_point(EOF) == true` check rather than relying on `bump()`'s saturation.

## Blockers
Blockers: none

## Summary
Adds `Parser::recover_to_sync` and `Parser::expected_token_error` plus a unit test, giving downstream parser items a single push-and-resync entry point at semicolons, closing braces, and item-start keywords.

# Plan: expected-one-of-messages

## Goal
Add an `expect_one_of`-style helper on `Parser` that, when the current token doesn't match any kind in a candidate follow-set, emits an `E0100`/`Syntax` `CompileError` whose message enumerates every expected candidate using the existing `describe` formatting.

## Steps
1. In `vertex_stage0/src/parser/mod.rs`, add a free helper `format_candidate_list(kinds: &[&TokenKind]) -> String` (or `&[TokenKind]` — see Assumptions) that runs each kind through the existing `describe` and joins with Oxford-comma rules: 1 → `X`; 2 → `X or Y`; 3+ → `X, Y, or Z`. Empty input → `"token"` fallback.
2. Add a method `Parser::expect_one_of(&mut self, kinds: &[TokenKind]) -> Result<Token, CompileError>` that:
   - On match (any kind whose `mem::discriminant` equals the peeked token's), `bump()` and return the token.
   - On mismatch, build `format!("expected {}, found {}", format_candidate_list(kinds), describe(self.peek()))`, wrap in `CompileError::new(ErrorCode::E0100, ErrorKind::Syntax, self.current_span(), …)`, and return `Err(...)`. Do NOT push to `self.errors` — keep symmetry with the single-kind `expect`, which only returns the error and lets the caller decide whether to accumulate/recover.
3. Add a sibling `expected_one_of_error(&mut self, kinds: &[TokenKind])` that mirrors `expected_token_error`: builds the same message, pushes onto `self.errors`, then calls `self.recover_to_sync()`. Useful when the caller wants accumulator-driven recovery instead of `?`-propagation.
4. In the existing `parser::tests` module (`vertex_stage0/src/parser/mod.rs`), add a unit test `expected_message_lists_candidates` covering:
   - Single-kind input → `"expected `+`, found `;`"` shape.
   - Two-kind input → `"expected `,` or `)`, found `;`"` shape.
   - Three-kind input → `"expected `,`, `;`, or `]`, found `}`"` shape (Oxford comma) — matches the hand-rolled string already used at `expr.rs:211`.
   - Asserts `err.code == ErrorCode::E0100` and `err.kind == ErrorKind::Syntax`.
   - Also asserts that on a *match*, `expect_one_of` returns `Ok(...)` and advances `pos` (so the helper isn't accidentally pure-error-only).
5. Do NOT migrate the existing hand-rolled `unexpected_token_error("`,`, `;`, or `]`")` call sites in `parser/expr.rs`. That's behavior-preserving churn outside this todo's scope and risks colliding with other in-flight items (e.g. range/array/struct parsing). Callers can opt in incrementally.

## Files
- `vertex_stage0/src/parser/mod.rs` — add `format_candidate_list` free fn, `Parser::expect_one_of` and `Parser::expected_one_of_error` methods, plus the `expected_message_lists_candidates` test inside the existing `#[cfg(test)] mod tests`.

## Risks
- **Message-style drift.** Existing strings like `` "expected `,`, `;`, or `]`" `` were hand-written; if `format_candidate_list` produces a slightly different join (extra space, missing Oxford comma) future migrations of those sites will produce diff noise in the golden-file harness. Mitigation: the test pins the exact 3-candidate shape, and we explicitly mirror the Oxford-comma pattern already in use.
- **Empty slice.** A caller passing `&[]` is almost certainly a bug. Picking a `"token"` fallback keeps the parser from panicking but masks the bug; documenting the precondition with a one-line debug_assert is fine.
- **Discriminant-only matching.** Like the existing `expect`/`eat`, this helper compares by `mem::discriminant`, so payload-bearing kinds (`Ident("foo")` vs `Ident("bar")`) are equal. That matches existing semantics; no new risk, but worth noting so future migrations don't expect value-equality.
- **No accumulator pollution.** If a future caller forgets the difference between the `Result`-returning `expect_one_of` and the accumulator-pushing `expected_one_of_error`, they could double-report. The naming mirrors existing `expect` vs `expected_token_error`, so the asymmetry is conventional rather than novel.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib -p vertex_stage0 parser::tests::expected_message_lists_candidates
cargo build -p vertex_stage0
```

## Assumptions
- The new helper is added to `parser/mod.rs` (alongside the existing `expect`/`expected_token_error`), not a new file — keeps the helper next to its siblings and keeps the `describe`/`current_span` helpers private as today.
- The test goes in the existing `parser::tests` module in `parser/mod.rs` (the module path the verify command points at).
- Helper signature is `&[TokenKind]` (owned slice of kinds, callers pass `&[TokenKind::Comma, TokenKind::Semi, TokenKind::RBracket]`). This keeps call sites readable; using `&[&TokenKind]` would force `&&` patterns for static literals.
- Message shape uses Oxford comma to match existing strings at `expr.rs:146,211,318`. Single-candidate output is plain `"expected X, found Y"` (no "one of") to keep parity with the existing single-kind `expect`.
- `expect_one_of` returns `Result` (matching `expect`); `expected_one_of_error` pushes-and-recovers (matching `expected_token_error`). Both exist so callers can pick the recovery model without recomputing the message.
- `ErrorCode::E0100` / `ErrorKind::Syntax` are correct — that's what every existing parser-mismatch error uses (`mod.rs:78`, `mod.rs:90`, `expr.rs:915`).
- Existing hand-rolled `unexpected_token_error("...")` call sites in `parser/expr.rs` are deliberately left alone in this commit; migrating them is a separate, larger pass and risks rebasing conflicts with other pending parser items.
- The verify command runs from the workspace root (which is the working directory). `-p vertex_stage0` selects the only crate; the filter pins down the new test.

## Blockers
Blockers: none

## Summary
Add `Parser::expect_one_of` / `expected_one_of_error` plus a `format_candidate_list` helper so mismatch errors enumerate every expected token from the follow-set, locked in by a `parser::tests::expected_message_lists_candidates` unit test.

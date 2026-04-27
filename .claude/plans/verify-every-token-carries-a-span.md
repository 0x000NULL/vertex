# Plan: verify-every-token-carries-a-span

## Goal
Add a `lexer::scan::tests::all_tokens_have_nonzero_span` test that drives `Scanner::next_token` across a representative source until EOF and asserts every emitted non-EOF token has `span.start < span.end`.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, inside the existing `#[cfg(test)] mod tests` block, add a new `#[test] fn all_tokens_have_nonzero_span()`.
2. Build a string covering each scanner branch: a doc comment, keywords, identifiers, an underscore, a leading-underscore identifier, a decimal int, a hex int, a binary int, a float, a char literal, a regular string, a raw string with hashes, every operator/punctuation form (single-, two-, and three-char variants like `..=`, `<<`, `>>`, `==`, `=>`, `->`, `::`, `!=`), and grouping delimiters. Reuse the program from `tokenizes_full_program` (line 1439) as a starting basis and supplement so each scanning branch is exercised.
3. Loop calling `s.next_token()` collecting tokens until a token with `kind == TokenKind::Eof` is produced; push the EOF token last, then break.
4. Iterate the collected tokens. For every token whose kind is not `TokenKind::Eof`, assert `t.span.start < t.span.end` (and check `t.span.file_id` matches the `FileId` passed to the scanner). For the trailing `Eof` token, assert `t.span.start == t.span.end` and `t.span.start as usize == src.len()` to pin the documented zero-length-EOF invariant.
5. Also assert the loop produced at least one non-EOF token, so an empty-input regression cannot silently pass the test.

## Files
- `vertex_stage0/src/lexer/scan.rs` -- append `#[test] fn all_tokens_have_nonzero_span()` to the existing `mod tests` block. No production code changes.

## Risks
- **EOF span ambiguity.** The literal sub-step says "every `Token.span.start < Token.span.end`", but `Scanner::next_token` deliberately emits a zero-length `Eof` (scan.rs:729) and the existing `tokenizes_full_program` test enforces that invariant. Asserting `<` on the EOF token would contradict the existing design and break the suite. Resolution: scope the strict inequality to non-EOF tokens (matches the spirit of "every token carries a real span" and keeps the EOF invariant intact).
- **Coverage gaps.** If the source string omits a branch (e.g. `0b...`, raw strings with hashes, `..=`), a future regression that emits a zero-width token from that branch would not be caught. Mitigation: assemble the source explicitly to hit every branch in `next_token`, including the underscore/`_bar` distinction.
- **Error-recovery tokens.** `next_token` can emit `TokenKind::Error(...)` tokens for unterminated strings, lone `'`, or unknown bytes. The current implementation always advances `self.pos` by at least one byte before producing the error span, so the assertion holds; staying within valid syntax avoids depending on that detail.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml lexer::scan::tests::all_tokens_have_nonzero_span
```

## Assumptions
- The verify command's `--lib lexer::scan::tests::all_tokens_have_nonzero_span` path means the test lives in the `tests` submodule of `src/lexer/scan.rs`, not in `tests/integration/`. The todo's wording "integration test" is descriptive (an end-to-end driver of `next_token`), not a directive to use the `tests/` directory; otherwise the verify filter would not match.
- "Every `Token.span.start < Token.span.end`" applies to non-EOF tokens. The EOF token is intentionally zero-width and the existing `tokenizes_full_program` test (scan.rs:1510) enforces that; reversing it would break passing tests. The new test will explicitly pin `start == end` for EOF and `start < end` for everything else.
- `cargo test` is invoked from the workspace root; passing `--manifest-path vertex_stage0/Cargo.toml` makes the verify deterministic regardless of cwd. (`cargo test --lib lexer::scan::tests::...` from root would also work via the workspace, but the explicit manifest is safer.)
- No production-code change is needed: every existing scanner branch already produces a span whose `end` exceeds `start` (the `Span::new(... start, self.pos as u32)` calls follow at least one byte advance). The test is purely a regression guard.
- The test source string will be authored fresh (rather than reusing the `tokenizes_full_program` constant verbatim) so we can guarantee branch coverage including raw strings with `#`, binary literals, `..=`, and `_bar`-style identifiers.

## Blockers
Blockers: none

## Summary
Add a single regression test that drives the full scanner over a source covering every `next_token` branch and asserts every non-EOF token has a non-empty span (with EOF pinned to start==end at end-of-input).

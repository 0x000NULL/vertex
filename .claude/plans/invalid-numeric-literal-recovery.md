# Plan: invalid-numeric-literal-recovery

## Goal
When `next_token` enters the integer-literal arm with a `0x`/`0b` prefix but `scan_int_hex`/`scan_int_bin` rejects the input, emit a single `TokenKind::Error("invalid numeric literal: …")` spanning the entire malformed run and advance past it, rather than silently falling through to decimal scanning that consumes just the leading `0`.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, inside `Scanner::next_token`'s digit branch, change the `0x`/`0X`/`0b`/`0B` handling so that on `None` from `scan_int_hex`/`scan_int_bin`, instead of falling through, advance past the prefix (2 bytes) plus any contiguous identifier-continuation bytes (`is_ascii_alphanumeric` or `_`) and return a `Token::new(TokenKind::Error("invalid numeric literal: <lex>"), span)`. The Error message body should embed the actual rejected lexeme so callers can render it. Make sure pos is advanced unconditionally even when zero trailing chars exist (so `0x<eof>` still consumes the two-byte `0x`).
2. Add a `#[test] fn invalid_numeric_recovers` near `invalid_char_recovers` that drives `next_token` over a source containing each rejection class — `0x`, `0xg`, `0x_`, `0b`, `0b2`, `0b_` — interleaved with valid identifiers to confirm the scanner resumes cleanly. Assert that each malformed run yields exactly one `TokenKind::Error` whose span covers the entire run (start = position of `0`, end = position just past the last alphanumeric/underscore byte), that following identifiers are tokenized normally, that pos progresses monotonically, and that a final `Eof` sits at `src.len()` with an empty span. Mirror the structure of `invalid_char_recovers` and `unterminated_string_recovers`.
3. Re-check existing tests: `tokenizes_full_program`, `all_tokens_have_nonzero_span`, and `hex_and_bin_literals` (which calls `scan_int_hex`/`scan_int_bin` directly and asserts `pos==0` on rejection). Because the recovery logic lives in `next_token` and not in the sub-scanners, the unit-level rejection contract for `scan_int_hex`/`scan_int_bin` (returns `None`, leaves `pos` at start) must be preserved.

## Files
- `vertex_stage0/src/lexer/scan.rs` — edit the `0x`/`0X`/`0b`/`0B` arms of `next_token` to emit an `Error` token + advance past the run on rejection; add `invalid_numeric_recovers` test in the existing `tests` module.

## Risks
- **Suffix collision:** Identifier-continuation eating could swallow what should be a suffix on a *valid* hex literal. Mitigation: only run the eat-and-error path when `scan_int_hex`/`scan_int_bin` already returned `None`; valid hex/bin still parses via the existing `Some(...)` branch.
- **Decimal fallthrough still desired for non-`0x`/`0b` cases:** `123abc` is not in scope — the spec sub-step is specifically about `0x`/`0b` parse failure. Leaving decimal scanning untouched preserves the existing `IntLiteral + Ident` behavior for plain decimals; this is intentional.
- **Re-flagging well-formed input:** A standalone `0` followed by an unrelated identifier (e.g., `0 x`) should not error. The trigger is `peek == 0` and `peek_at(1) in {x,X,b,B}`, which is unchanged from current code; only the failure branch changes.
- **`scan_int_hex`/`scan_int_bin` contract:** They reset `pos` on `None`. The new code re-advances explicitly using a fresh start anchor, so the reset is harmless.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml lexer::scan::tests::invalid_numeric_recovers
cargo test --lib --manifest-path vertex_stage0/Cargo.toml lexer::scan::tests
```

## Assumptions
- The crate manifest lives at `vertex_stage0/Cargo.toml` (confirmed by repo layout); the verify command needs `--manifest-path` because the workspace root has no top-level `Cargo.toml`. If a workspace `Cargo.toml` exists at the repo root, `cargo test --lib lexer::scan::tests::invalid_numeric_recovers` from the root will also work — the explicit path is the safer bet.
- "Offending run" for `0x`/`0b` rejection means: the two-byte prefix plus all immediately-following identifier-continuation bytes (`is_ascii_alphanumeric() || == b'_'`). This matches the natural lexical extent a human reader would group as "the bad number".
- The Error message format `"invalid numeric literal: <lex>"` mirrors the existing `"invalid character: <ch>"` style from `invalid_char_recovers`. Consistent prefix makes the renderer's job simpler later.
- We do NOT modify `scan_int_hex`/`scan_int_bin` themselves — they keep returning `None` with `pos` reset, preserving the contract pinned by `hex_and_bin_literals`.
- We do NOT extend recovery to `scan_float` failures (e.g., `1.0e`) — the sub-step text and verify test name reference only the integer-prefix case, and `scan_float` already cleanly falls back to `scan_int_decimal` for legitimate inputs like `1` or `.5` rejected as floats. Treating `1.0e` is out of scope unless the test specifically targets it.
- The test name `invalid_numeric_recovers` (singular) goes inside the existing `mod tests` block in `scan.rs`, reachable via the path `lexer::scan::tests::invalid_numeric_recovers`.
- A leading `0` is a valid decimal int (`IntLiteral(0)`), so the recovery path must only fire when `peek_at(1)` is one of `x|X|b|B` AND the sub-scanner returned None — `0` alone or `01` should remain decimal.

## Blockers
Blockers: none

## Summary
Make `next_token` emit a single `Error` token spanning malformed `0x`/`0b` runs and advance past them, pinned by a new `invalid_numeric_recovers` test.

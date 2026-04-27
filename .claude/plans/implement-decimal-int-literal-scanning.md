# Plan: implement-decimal-int-literal-scanning

## Goal
Add `Scanner::scan_int_decimal` that consumes a base-10 integer literal with `_` separators and an optional integer suffix, returning `(u64, IntSuffix, Span)`.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, add `pub fn scan_int_decimal(&mut self) -> (u64, IntSuffix, Span)` to `impl<'a> Scanner<'a>`. Import `IntSuffix` and `Span` (`use crate::lexer::token::IntSuffix;` and `use crate::span::Span;`). Precondition: caller has positioned `self.pos` at the first decimal digit (`0..=9`).
2. Record `start = self.pos as u32`. Caller guarantees first byte is a digit, so a leading `_` is implicitly rejected (no need for explicit guard at entry — `_` simply will not be consumed unless preceded by a digit).
3. Loop while `self.peek()` is `Some(b)` with `b.is_ascii_digit()` or `b == b'_'`: if digit, push the digit char into a small `String` accumulator (or fold into `u64` directly via `value = value.checked_mul(10).and_then(|v| v.checked_add((b - b'0') as u64))`); if `_`, just skip. Track an `overflow` flag if any `checked_*` returns `None`. Always advance `self.pos` so the span covers the whole digit/underscore run regardless of overflow.
4. After the digit loop, attempt to parse a suffix by peeking the next byte(s). Recognised suffixes: `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize`. Implementation: if `peek()` is `b'i'` or `b'u'`, try to match the longest known suffix from a fixed table (e.g. `["isize","i64","i32","i16","i8","usize","u64","u32","u16","u8"]`) by comparing `self.bytes[self.pos..]` with `starts_with`; on match, advance `self.pos` by the suffix length and set the corresponding `IntSuffix`. Otherwise `IntSuffix::Unsuffixed`.
5. Compute `end = self.pos as u32`, build `Span::new(self.file_id, start, end)`, and return `(value, suffix, span)`. (Overflow handling: for this sub-task, on overflow return `u64::MAX` as a placeholder value; an upcoming `invalid-numeric-literal-recovery` item will replace this with proper error reporting. Note this in Assumptions.)
6. Add a `#[test] fn decimal_int_with_underscores_and_suffix()` in the existing `mod tests` of `scan.rs` covering: `123` → `(123, Unsuffixed)`, `1_000_000` → `(1_000_000, Unsuffixed)`, `42u32` → `(42, U32)`, `0i64` → `(0, I64)`, `9_isize` → `(9, ISize)`, and `1_2_3u8` → `(123, U8)`. Also assert the returned `Span` covers the full literal including the suffix and that `self.pos` is advanced to just past it.
7. Run `cargo test --lib lexer::scan::tests::decimal_int_with_underscores_and_suffix` and `cargo build` to confirm.

## Files
- `vertex_stage0/src/lexer/scan.rs` -- add `use` imports for `IntSuffix` and `Span`; add `scan_int_decimal` method on `Scanner`; add the named unit test inside the existing `mod tests`.

## Risks
- Suffix matching order must be longest-first (`isize` before `i8`, `u64` before `u8`, etc.) or short prefixes will swallow long ones. Mitigation: explicit longest-first table.
- `_` separator rule: a trailing `_` (e.g. `5_`) is allowed by this loop because we eat any `_` after a digit. Spec sub-step only forbids *leading* `_`; permissive trailing-`_` matches what most lexers do at the scanning stage and can be tightened by validation later. Recorded as assumption.
- Identifier collision: `123abc` would be tokenised by the operator/identifier driver as `IntLiteral(123, Unsuffixed)` followed by `Ident("abc")`. That's the driver's job, not this method's; this method only consumes a suffix if it exactly matches one of the 10 fixed tokens. An invalid suffix like `123i7` would consume `123` and leave `i7` for the identifier path, which is acceptable for this sub-step.
- Overflow on values > `u64::MAX`: handled with a saturating placeholder until the dedicated recovery item lands.

## Prereqs
- implement-scanner-struct-in-src-lexer-scan-rs
- add-literal-variants-to-tokenkind
- implement-span-struct-in-src-span-rs

(All three are already merged per recent commits, but listed because this plan directly references `Scanner`, `IntSuffix`, and `Span`.)

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml lexer::scan::tests::decimal_int_with_underscores_and_suffix
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate manifest lives at `vertex_stage0/Cargo.toml` (consistent with existing module paths). If the workspace root has a top-level `Cargo.toml` that re-exports this crate, plain `cargo test --lib lexer::scan::tests::...` will also work; the `--manifest-path` form is the safe superset.
- `IntSuffix` and `Span` are already defined and re-exported as shown in the read of `token.rs` and `span.rs` — no edits to those files are needed.
- `scan_int_decimal` only handles base-10. Hex (`0x`) and binary (`0b`) are out of scope and belong to the next item (`implement-hex-0x-and-binary-0b-int-literal-scanning`). The decimal scanner does not need to peek for `0x`/`0b`; the eventual `next_token` driver will dispatch on the leading `0[xb]` before calling this method.
- Floating-point disambiguation (e.g. `1.0`, `1e10`) is out of scope for this item and handled by the future float-scanning item; this method stops at the first non-digit, non-underscore, non-recognised-suffix byte. The driver decides whether to upgrade to a float.
- "Reject leading `_`" is enforced by the precondition that callers only invoke `scan_int_decimal` when `peek()` is an ASCII digit. The driver enforces this when dispatching; bare `_` is the `Underscore` token, not a numeric literal.
- On overflow we return `u64::MAX` as a placeholder. Span covers the whole literal so a later pass can re-examine the source text. A proper error will be emitted by the dedicated `invalid-numeric-literal-recovery` item.
- Trailing `_` (e.g. `5_`) is permitted at the scanner level; tightening, if desired, is deferred to numeric-literal recovery.
- The test name in the verify command (`decimal_int_with_underscores_and_suffix`) is taken verbatim from the sub-step spec and is therefore the required test function name.

## Blockers
Blockers: none

## Summary
Adds `Scanner::scan_int_decimal` with `_`-separator and 10-suffix support, plus the named unit test the spec verifies against.

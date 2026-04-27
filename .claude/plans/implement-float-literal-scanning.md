# Plan: implement-float-literal-scanning

## Goal
Add `Scanner::scan_float` that recognizes `digit+(_|digit)* . digit+(_|digit)* [eE [+-]? digit+(_|digit)*] [f32|f64]?` and the `float_literal_forms` unit test verifying happy paths, exponent forms, and the `.5` rejection.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, add `pub fn scan_float(&mut self) -> Option<(f64, FloatSuffix, Span)>` to `Scanner<'a>`.
   - Save `start = self.pos`. If `peek()` is not an ASCII digit, return `None` (covers `.5`, EOF, non-numeric).
   - Consume leading digit run: digits and `_` (mirrors `scan_int_decimal`'s loop) into a scratch `String` buffer (skip underscores so `f64::from_str` accepts it).
   - Require a fractional part: `peek_at(0) == Some(b'.')` AND `peek_at(1)` is an ASCII digit. If not, rewind `self.pos = start` and return `None` (lets the driver fall back to `scan_int_decimal`; avoids stealing `1..2` ranges and `1.foo()` method calls).
   - Consume the `.` (push to buffer) and the fractional digit/underscore run (digits → buffer; `_` skipped).
   - Optional exponent: if `peek()` is `b'e'` or `b'E'`, push `'e'`, advance; if next is `b'+'` or `b'-'`, push it and advance; then require at least one ASCII digit — if absent, rewind to `start` and return `None`. Consume digit/underscore run into buffer.
   - Call a new private `scan_float_suffix(&mut self) -> FloatSuffix` that matches `f32` / `f64` (only when leading byte is `b'f'`) and returns `FloatSuffix::Unsuffixed` otherwise; do not write the suffix into the parse buffer.
   - Parse the scratch buffer with `<f64 as core::str::FromStr>::from_str(&buf).ok()?`. Build `Span::new(self.file_id, start as u32, self.pos as u32)` and return `Some((value, suffix, span))`.
2. Add a `#[test] fn float_literal_forms()` in the existing `tests` module covering:
   - `"1.0"` → `1.0`, `Unsuffixed`, span covers full input, pos advanced to end.
   - `"1.0e10"` → `1.0e10`, `Unsuffixed`.
   - `"1.0E-3"` → `1.0e-3`, `Unsuffixed`.
   - `"3.14f32"` → `3.14_f64` value, `FloatSuffix::F32`.
   - `"2.5f64"` → `2.5`, `FloatSuffix::F64`.
   - `"1_000.000_5"` → `1000.0005`, `Unsuffixed` (digit separator).
   - `"1.0e+2"` → `100.0`.
   - `".5"` → `scan_float` returns `None`, `pos` unchanged at 0.
   - `"1"` (no dot) → `None`, `pos == 0` (so the driver can fall back to int).
   - `"1.0e"` (exponent missing digits) → `None`, `pos == 0`.
   - Use `(f64 - expected).abs() < 1e-12` (or exact equality for nice values) to compare.

## Files
- `vertex_stage0/src/lexer/scan.rs` -- add `Scanner::scan_float` + private `scan_float_suffix` helper, import `FloatSuffix` from `crate::lexer::token`, add the `float_literal_forms` test to the existing `tests` module.

## Risks
- Stealing `.` from method-call / range syntax (e.g., `1.foo()`, `1..2`). Mitigated by requiring the byte after `.` to be an ASCII digit before consuming the dot.
- Underscores around the exponent or trailing the fractional run could confuse `f64::from_str` if accidentally pushed into the parse buffer; we skip every `_` to avoid that.
- `f64::from_str` on a hand-built buffer is deterministic, but we must not include the suffix in the parse buffer — handled by deferring suffix scan until after value parsing.
- Test name must match the verify command exactly: `lexer::scan::tests::float_literal_forms`.

## Prereqs
- add-literal-variants-to-tokenkind
- implement-scanner-struct-in-src-lexer-scan-rs

## Verify
```
cargo test --lib lexer::scan::tests::float_literal_forms
```

## Assumptions
- `scan_float` returns `Option<(f64, FloatSuffix, Span)>` and rewinds `self.pos` on `None` so the driver can fall through to `scan_int_decimal`. (This matches `scan_int_hex` / `scan_int_bin`'s rewind-on-fail convention.)
- A leading `.` (no integer part) is rejected: `scan_float` returns `None` for inputs starting with `.` regardless of what follows. The lexer driver will not invoke `scan_float` for `.5`-style starts; if it does, this guard makes the rejection explicit.
- `1.` (trailing dot, no fractional digit) is treated as integer-then-dot (returns `None`, pos rewound). Spec text only lists `1.0`-style forms.
- The exponent must have at least one digit (`1e` / `1.0e+` reject and rewind).
- Underscores are permitted inside any digit run (integer, fractional, exponent) for consistency with `scan_int_decimal` / `scan_int_hex` / `scan_int_bin`. They are stripped before calling `f64::from_str`.
- Suffix recognition only fires when the post-numeric byte is `b'f'`; anything else (including `e`/`E` of an exponent) is left for the caller. Only `f32` and `f64` are accepted; `f16`, `f128`, etc. yield `Unsuffixed` and leave bytes for the caller.
- `f64::from_str` failure (should be unreachable given our grammar) is treated as `None` with rewind, defensively.
- The test compares floats with a tolerance for inexact values (e.g., `3.14`) and exact equality for round values (`1.0`, `100.0`).
- We do not need to update `vertex_stage0/src/lexer/mod.rs` — `scan.rs` is already wired in, and `FloatSuffix`/`FloatLiteral` already exist on `TokenKind`.
- Cargo workspace test invocation `cargo test --lib lexer::scan::tests::float_literal_forms` (without `-p`) is the verify command verbatim from the spec; it resolves to the single library crate (`vertex_stage0`) that contains the path.

## Blockers
Blockers: none

## Summary
Adds a rewind-on-failure `Scanner::scan_float` recognizing `digits . digits [exp] [f32|f64]?` plus a `float_literal_forms` test, leaving `.5` and `1.foo` alone for the driver.

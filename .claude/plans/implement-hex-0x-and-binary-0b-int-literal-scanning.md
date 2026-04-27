# Plan: implement-hex-0x-and-binary-0b-int-literal-scanning

## Goal
Add hex (`0x…`) and binary (`0b…`) integer literal scanning to `Scanner`, with `_` separators and empty-digit-run rejection, mirroring the existing `scan_int_decimal` style.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, add two new public methods on `Scanner<'a>`:
   - `scan_int_hex(&mut self) -> Option<(u64, IntSuffix, Span)>` — consumes the `0x` prefix itself, then a run of `[0-9a-fA-F_]`. Reject (return `None` and rewind `self.pos` to the start) when the digit run after the prefix is empty (i.e. the next byte after `0x` is not a hex digit). Track overflow by saturating to `u64::MAX` exactly like the decimal helper. After collecting digits, call the existing `scan_int_suffix` to optionally consume an `i*`/`u*` suffix. Return `Some((value, suffix, Span::new(file_id, start, end)))`.
   - `scan_int_bin(&mut self) -> Option<(u64, IntSuffix, Span)>` — same shape, but consumes the `0b` prefix and a run of `[01_]`. Same overflow-saturation and same suffix call. Reject + rewind on empty digit run.
2. Implement digit handling so that one or more leading `_` interspersed between digits is allowed but at least one real digit must appear; that is, after the prefix we require the first non-`_` byte we examine to be a valid digit for the radix. The simplest correct rule: scan digits/`_` as long as we see them, but require that we consumed at least one real digit (not just `_`s) — if not, rewind `self.pos = start as usize` and return `None`. (This matches the spec's "reject empty digit run".)
3. Reuse the existing `scan_int_suffix` helper unchanged — it already covers all `i*`/`u*` suffixes and is radix-agnostic.
4. Add one `#[test] fn hex_and_bin_literals()` in the existing `tests` module that covers:
   - Happy path: `0x1F` → 31 unsuffixed, `0xff_ffu32` → 65535 U32, `0xDEAD_BEEFi64` → 0xDEADBEEF I64, `0b0` → 0, `0b1010_1010` → 170, `0b1111_1111u8` → 255, mixed-case hex `0xAbCd`.
   - Span: `span.file_id`, `span.start == 0`, `span.end == input.len()`, `s.pos == input.len()` for each happy case.
   - Empty-digit-run rejection: `Scanner::new("0x", _).scan_int_hex()` and `Scanner::new("0xg", _).scan_int_hex()` and `Scanner::new("0x_", _).scan_int_hex()` return `None` and leave `s.pos == 0`. Same shape for `0b`, `0b2`, `0b_` against `scan_int_bin`.
   - Overflow saturation: `0xFFFF_FFFF_FFFF_FFFF_F` saturates to `u64::MAX`.
5. Leave `scan_int_decimal` untouched so its existing test (`decimal_int_with_underscores_and_suffix`) still passes; the driver task (later item `wire-all-scanners-into-scanner-next-token-driver`) will dispatch on the `0x`/`0b` prefix and call the new helpers.

## Files
- `vertex_stage0/src/lexer/scan.rs` -- add `scan_int_hex` and `scan_int_bin` methods plus the `hex_and_bin_literals` unit test in the existing `#[cfg(test)] mod tests` block.

## Risks
- Underscore-only digit runs (e.g. `0x_`) must be rejected; a naive "consume `[0-9a-fA-F_]*`" loop would silently accept and produce value 0 — guard with a "saw at least one real digit" flag.
- On rejection we must rewind `self.pos` to the original start so a future driver can either emit an `Error` token over the prefix bytes or retry scanning; otherwise the prefix bytes are silently swallowed and the rest of the lexer desynchronises.
- `scan_int_suffix` reads from `self.pos`, so it must be called only on the success branch — calling it on rejection paths would corrupt position after the rewind.
- Mixed-case hex (`0xDEADbeef`) must be accepted; using `u8::is_ascii_hexdigit()` covers both cases. `b - b'0'`-style digit conversion does NOT work for hex — use a small helper that maps `0..=9`, `a..=f`, `A..=F` to `0..=15`.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml lexer::scan::tests::hex_and_bin_literals
cargo test --lib --manifest-path vertex_stage0/Cargo.toml lexer::scan::tests::decimal_int_with_underscores_and_suffix
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate lives at `vertex_stage0/` (confirmed by reading `vertex_stage0/src/lexer/scan.rs`); `--manifest-path vertex_stage0/Cargo.toml` is needed because `cargo test --lib` from the workspace root needs to know the package.
- The new methods return `Option<(u64, IntSuffix, Span)>` rather than `(u64, IntSuffix, Span)`. Reasoning: the `error` module is not yet built (slug `define-errorcode-and-errorkind-in-src-error-rs` is still pending), so we cannot emit a structured diagnostic from the scan helper. Returning `None` is the simplest contract that lets the future `next_token` driver detect rejection and synthesise an `Error` token. The existing `scan_int_decimal` keeps its current `(u64, IntSuffix, Span)` shape — decimal scanning has no "empty digit run" failure mode because it is only called when the driver has already seen an ASCII digit.
- The two helpers consume the `0x`/`0b` prefix themselves (rather than expecting the caller to have already advanced past it). This keeps the rewind-on-rejection semantics local to the helper and avoids the caller needing to remember how many bytes to roll back.
- Hex literals accept both lowercase and uppercase a–f; mixed-case is permitted (matching Rust's lexical grammar, which the surrounding spec is modelled on).
- Overflow handling matches `scan_int_decimal`: saturate the accumulated value to `u64::MAX`, do not return an error. A future task in the `invalid-numeric-literal-recovery` slug will refine this once errors exist.
- The verify test name `hex_and_bin_literals` lives at module path `lexer::scan::tests::hex_and_bin_literals` because the `tests` module is `#[cfg(test)] mod tests` inside `scan.rs`.

## Blockers
Blockers: none

## Summary
Adds `scan_int_hex` / `scan_int_bin` to `Scanner` with `_`-separator and `i*`/`u*` suffix support, rejecting and rewinding on empty digit runs, plus a `hex_and_bin_literals` unit test verifying happy-path values, spans, suffix capture, mixed-case hex, overflow saturation, and rejection of `0x`/`0b`/`0x_`/`0b_`/`0xg`/`0b2`.

# Plan: fuzz-style-robustness-test

## Goal
Add a deterministic, seeded fuzz test that drives `Scanner` over 1000 random byte sequences and asserts the scanner never panics, runs forever, or fails to terminate at EOF.

## Steps
1. Open `vertex_stage0/src/lexer/scan.rs` and locate the existing `#[cfg(test)] mod tests { … }` block (already present at line 888).
2. Inside that module, add a tiny self-contained xorshift64 PRNG `fn next_rand(state: &mut u64) -> u64` so the test pulls in no new crate dependencies and is fully reproducible from a fixed seed.
3. Add `#[test] fn fuzz_random_bytes_no_panic()`:
   - Initialize the PRNG with a fixed seed (e.g. `0x9E3779B97F4A7C15`).
   - Loop 1000 iterations. Each iteration:
     - Draw a random length in `0..=256`.
     - Fill a `Vec<u8>` with random bytes from the PRNG.
     - Convert via `String::from_utf8_lossy(&bytes).into_owned()` (Scanner::new requires `&str`; lossy conversion is required because random bytes are usually not valid UTF-8).
     - Construct `Scanner::new(&s, FileId(0))`.
     - Loop calling `next_token()` until `TokenKind::Eof` is returned, with a hard iteration cap (e.g. `4 * s.len() + 16`) and a "pos must advance OR be Eof" assertion to guarantee termination — otherwise a non-advancing scanner bug would hang the test instead of failing it.
   - Each iteration includes the PRNG seed/iteration index in the panic/assert messages so any discovered failure is reproducible.
4. Confirm the rest of the file is untouched.

## Files
- `vertex_stage0/src/lexer/scan.rs` -- add `fn next_rand` helper + `#[test] fn fuzz_random_bytes_no_panic` inside the existing `mod tests` block. No changes outside that module.

## Risks
- A latent scanner bug that fails to advance on some byte sequence could turn this test into an infinite loop. The iteration cap + "pos advanced or Eof" assertion bounds the loop so such a bug fails fast with a clear message instead of hanging CI.
- `Scanner::new` takes `&str`, so we cannot fuzz arbitrary non-UTF-8 byte streams directly — `String::from_utf8_lossy` replaces ill-formed sequences with U+FFFD before they reach the scanner. This is documented in the test as an assumption; it still exercises every codepoint the scanner can actually receive.
- A fixed seed means we re-fuzz the same 1000 inputs every run; this is intentional for determinism, not a defect.

## Prereqs
- implement-scanner-struct-in-src-lexer-scan-rs
- wire-all-scanners-into-scanner-next-token-driver

## Verify
```
cargo test --lib lexer::scan::tests::fuzz_random_bytes_no_panic
```

## Assumptions
- "1000 random byte sequences (PRNG-seeded)" means a deterministic seeded PRNG inside the test, not adding `rand`/`proptest`/`quickcheck` crates as dependencies (the project currently has no `[dependencies]` section in `vertex_stage0/Cargo.toml` and is intentionally lean).
- The intent is to fuzz the full scanner pipeline, not just the trivial `Scanner::new` constructor (which only stores fields and cannot panic). The test therefore drives `next_token()` to EOF on each random input — that's where bugs would actually surface.
- Since `Scanner::new` requires `&str`, random bytes are funneled through `String::from_utf8_lossy` to produce valid UTF-8 inputs. This matches how the scanner is invoked in production (input comes from a `&str` source file).
- Random input lengths up to 256 bytes are sufficient breadth; the goal is panic-coverage of branches in `next_token` and its sub-scanners, not throughput stress.
- The test belongs in the existing `tests` submodule of `scan.rs` (path `lexer::scan::tests::fuzz_random_bytes_no_panic` matches the verify command exactly), not in a new integration test file.
- An iteration cap proportional to input length is acceptable safety scaffolding; if it ever trips, that itself is a real scanner bug worth surfacing as a test failure.

## Blockers
Blockers: none

## Summary
Adds one deterministic, dependency-free fuzz test that hammers the scanner with 1000 PRNG-generated inputs and proves it always terminates without panicking.

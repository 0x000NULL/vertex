# Plan: snapshot-test-helper-macro-in-src-lexer-test-util-rs

## Goal
Add a `lex_eq!` declarative macro in a new `vertex_stage0/src/lexer/test_util.rs` module that drives the scanner over an input string and asserts the resulting `TokenKind` sequence (spans dropped) matches an expected slice, plus a single self-test that the runner can verify.

## Steps
1. Create `vertex_stage0/src/lexer/test_util.rs` containing a public `macro_rules! lex_eq` macro exported via `#[macro_export]`. The macro takes `($input:expr, $expected:expr)`, constructs a `Scanner` over the input with a fixed `FileId(0)`, repeatedly calls `next_token()` collecting `token.kind` until `TokenKind::Eof` is observed (Eof not pushed into the collected vec), then compares the collected `Vec<TokenKind>` to `$expected` using `assert_eq!` with a clear panic message including the input.
2. Wire the new module into the lexer module tree by adding `#[cfg(test)] pub mod test_util;` to `vertex_stage0/src/lexer/mod.rs` so the helper compiles only under test (matches lib's existing convention of test-only utilities) but is reachable from sibling tests via `crate::lexer::test_util::*` and from `#[macro_export]` at the crate root.
3. Inside `test_util.rs`, add a `#[cfg(test)] mod tests` containing one test named `macro_works` that exercises the macro on a small input (e.g. `"let x"` → `[TokenKind::Let, TokenKind::Ident("x".into())]`) so the verify command `cargo test --lib lexer::test_util::tests::macro_works` actually finds and passes a test.
4. Use only types already exported by the crate: `crate::lexer::scan::Scanner`, `crate::lexer::token::{Token, TokenKind}`, `crate::span::FileId`. No new dependencies.

## Files
- `vertex_stage0/src/lexer/test_util.rs` -- new file: `#[macro_export] macro_rules! lex_eq { ... }` plus `#[cfg(test)] mod tests { fn macro_works() { ... } }`.
- `vertex_stage0/src/lexer/mod.rs` -- add `#[cfg(test)] pub mod test_util;`.

## Risks
- `#[macro_export]` puts `lex_eq` at the crate root (`vertex_stage0::lex_eq!`), not under `crate::lexer::test_util`; tests inside the same module reference it via the locally-visible `macro_rules!` definition, so this is fine, but downstream callers must use `use vertex_stage0::lex_eq;` — acceptable for a test helper.
- Macro hygiene: the macro must use absolute paths (`$crate::lexer::scan::Scanner`, `$crate::lexer::token::TokenKind`, `$crate::span::FileId`) so it works whether invoked from inside `test_util.rs` or sibling lexer tests.
- `TokenKind::Ident(String)` and string-literal variants force callers to construct owned `String`s in the expected slice; that's a known ergonomic cost, not a correctness risk.
- Comparing the collected `Vec<TokenKind>` with `$expected` requires `$expected` be a `Vec<TokenKind>` or coercible slice; the macro will accept any expression and rely on `PartialEq` between the two (already derived on `TokenKind`).

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib lexer::test_util::tests::macro_works
test -f vertex_stage0/src/lexer/test_util.rs
grep -q "macro_rules! lex_eq" vertex_stage0/src/lexer/test_util.rs
```

## Assumptions
- Crate lives at `vertex_stage0/` (workspace member); `cargo test --lib` must be scoped via `--manifest-path vertex_stage0/Cargo.toml` since the repo root `Cargo.toml` may be a workspace.
- The macro lives in `lexer/test_util.rs` as instructed (not in `tests/`), so `#[cfg(test)]`-gating the module in `mod.rs` is the correct visibility model — it keeps the helper from bloating release builds while making it reachable from any in-crate test module.
- "Compare token kind list (drop spans for snapshot brevity)" means collect `token.kind` into a `Vec<TokenKind>` and compare with `assert_eq!`; no third-party snapshot crate (insta, etc.) is introduced.
- `Eof` is excluded from the collected vec so callers write `[Let, Ident("x".into())]` instead of `[Let, Ident("x".into()), Eof]`. This is a local convention; documented in a one-line `//` comment on the macro.
- `FileId(0)` is hard-coded inside the macro — the spec says spans are dropped, so the file id is irrelevant to comparisons.
- The verify test name `macro_works` is the path the runner specified (`lexer::test_util::tests::macro_works`); the test must exist under exactly that path.

## Blockers
Blockers: none

## Summary
Introduces a `lex_eq!` macro and self-test in a new `src/lexer/test_util.rs`, giving downstream lexer tests a one-liner way to assert a scanner's `TokenKind` sequence without span boilerplate.

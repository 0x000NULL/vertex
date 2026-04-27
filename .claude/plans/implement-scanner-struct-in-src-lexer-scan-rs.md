# Plan: implement-scanner-struct-in-src-lexer-scan-rs

## Goal
Create a `Scanner<'a>` struct in `vertex_stage0/src/lexer/scan.rs` that owns the source slice plus a byte cursor and exposes the cursor primitives (`peek`, `peek_at`, `bump`, `eat_while`) the rest of the lexer will build on.

## Steps
1. Create `vertex_stage0/src/lexer/scan.rs` containing:
   - `pub struct Scanner<'a> { pub src: &'a str, pub bytes: &'a [u8], pub pos: usize, pub file_id: FileId }`
   - `impl<'a> Scanner<'a>` with:
     - `pub fn new(src: &'a str, file_id: FileId) -> Self` — captures `src.as_bytes()` once into `bytes` and starts `pos` at 0.
     - `pub fn peek(&self) -> Option<u8>` — returns `bytes.get(pos).copied()`.
     - `pub fn peek_at(&self, offset: usize) -> Option<u8>` — returns `bytes.get(pos + offset).copied()`.
     - `pub fn bump(&mut self) -> Option<u8>` — returns the byte at `pos` (if any) and advances `pos` by 1.
     - `pub fn eat_while<F: Fn(u8) -> bool>(&mut self, pred: F)` — advances `pos` while `peek()` matches `pred`.
   - Bring `crate::span::FileId` into scope.
2. Add `pub mod scan;` to `vertex_stage0/src/lexer/mod.rs` next to the existing `pub mod token;`.
3. Add a small `#[cfg(test)] mod tests` inside `scan.rs` covering construction, `peek`/`peek_at` past EOF returning `None`, `bump` advancing `pos`, and `eat_while` consuming a run (e.g., ASCII whitespace) — small enough to keep the commit coherent and ensure the helpers are not dead code per `-D warnings`.

## Files
- `vertex_stage0/src/lexer/scan.rs` — new file with `Scanner` struct, `new`, byte-level helpers (`peek`, `peek_at`, `bump`, `eat_while`), and unit tests.
- `vertex_stage0/src/lexer/mod.rs` — add `pub mod scan;`.

## Risks
- Byte-cursor semantics interact with multi-byte UTF-8: `bump`/`peek` return raw bytes, not `char`s. Subsequent literal/identifier scanners must decode UTF-8 themselves; documenting this implicitly via the byte signatures is sufficient at this stage.
- Future scanner sub-tasks may want a `Scanner::span_from(start)` helper or a `current_span()` method; this plan deliberately stops at the four required helpers to match the spec.
- If clippy is set to deny warnings, unused helpers would fail; the `#[cfg(test)]` exercises mitigate this.

## Prereqs
Prereqs: none

## Verify
```
cargo build -p vertex_stage0
test -f vertex_stage0/src/lexer/scan.rs
grep -q 'pub struct Scanner' vertex_stage0/src/lexer/scan.rs
grep -q 'pub mod scan' vertex_stage0/src/lexer/mod.rs
cargo test -p vertex_stage0 --lib lexer::scan
```

## Assumptions
- The crate root is `vertex_stage0/` (workspace member); the spec's `src/lexer/scan.rs` resolves to `vertex_stage0/src/lexer/scan.rs`. The verify uses that path.
- `FileId` is the existing `crate::span::FileId` newtype already used by `Token`/`Span`; no new type is introduced.
- `bytes` is initialized once from `src.as_bytes()` in `new` and kept in sync by construction (only `pos` mutates). All four helpers operate on bytes, not `char`s; per-codepoint decoding is the responsibility of later literal/identifier scanners.
- `peek`/`peek_at` return `Option<u8>` so callers can branch on EOF without bounds-check noise; `bump` likewise returns `Option<u8>` and is a no-op at EOF.
- `eat_while` takes `F: Fn(u8) -> bool` (a closure over a byte) — sufficient for ASCII-class predicates that cover whitespace, digits, and identifier-continue bytes; non-ASCII identifier-continue checks will need a `char`-based variant added in a later sub-task and are out of scope here.
- Fields are `pub` so neighboring lexer modules in this crate can read `src`/`pos` when constructing spans without going through accessors. Visibility can be tightened later if a stable internal API emerges.
- A minimal `#[cfg(test)]` block is included so the helpers aren't reported as dead code under `-D warnings`; tests are scoped to the helpers only and do not pre-empt later scanner tests.

## Blockers
Blockers: none

## Summary
Adds the `Scanner<'a>` cursor type and its four byte-level primitives in a new `vertex_stage0/src/lexer/scan.rs`, wired into the `lexer` module, giving subsequent lexer sub-tasks a concrete object to extend.

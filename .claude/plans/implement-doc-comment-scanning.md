Now I have full context. Let me write the plan.

# Plan: implement-doc-comment-scanning

## Goal
Add a `Scanner::scan_doc_comment` method that recognizes `///` (outer) and `//!` (inner) line doc comments and emits them as `TokenKind::DocComment(String, DocStyle)`, with `DocStyle::{Outer, Inner}` defined in `token.rs`.

## Steps
1. In `vertex_stage0/src/lexer/token.rs`, add `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub enum DocStyle { Outer, Inner }`.
2. In `vertex_stage0/src/lexer/token.rs`, add a new variant `DocComment(String, DocStyle)` to `TokenKind` (placed near `StringLiteral`/`Ident` for grouping).
3. In `vertex_stage0/src/lexer/scan.rs`, add `pub fn scan_doc_comment(&mut self) -> Option<(String, DocStyle, Span)>`:
   - Return `None` (no rewind needed; nothing consumed) unless current bytes are `//` and next byte is `/` (Outer) or `!` (Inner). Importantly, exclude `////` (4+ slashes) — per common Rust convention `////` is a regular line comment, not a doc; keep behavior conservative by treating any `///` prefix as a doc (matching the spec grammar literally), but still allow `////...` to be picked up as doc since the spec says only `///` prefix. (See assumptions.)
   - Record `start = self.pos`, advance past the 3-byte prefix (`///` or `//!`).
   - Collect content bytes until `\n` or EOF (do not include the newline) into a `String`.
   - Build span from `start..self.pos` and return `Some((content, style, span))`.
4. Update `Scanner::skip_comments` documentation/behavior: it already declines to consume `///` and `//!`; leave it untouched so the driver routes to `scan_doc_comment` first. (No code change needed beyond confirming the existing guard at scan.rs:58-62.)
5. Add a `#[cfg(test)] mod tests` test named `doc_comments_preserved` that:
   - Verifies `///` outer doc with content (incl. trailing-newline boundary, EOF-terminated, leading space, empty body), each producing `DocStyle::Outer` and matching content.
   - Verifies `//!` inner doc cases for `DocStyle::Inner`.
   - Verifies that `// regular` and `/* block */` and `////` (4+ slashes — see assumptions) yield `None` from `scan_doc_comment` and leave `pos` unchanged.
   - Verifies span correctness (`file_id`, `start`, `end`) and `pos` advancement.
6. Run `cargo test --lib lexer::scan::tests::doc_comments_preserved` and `cargo test --lib` to make sure nothing else regresses.

## Files
- `vertex_stage0/src/lexer/token.rs` — add `DocStyle` enum and `TokenKind::DocComment(String, DocStyle)` variant.
- `vertex_stage0/src/lexer/scan.rs` — add `scan_doc_comment` method and `doc_comments_preserved` unit test; add `use crate::lexer::token::DocStyle;` at the top.

## Risks
- Adding a non-exhaustive `TokenKind` variant could break `match` arms elsewhere. Currently no match on `TokenKind` exists outside `token.rs` (`grep` of project finds none in the lexer module), so risk is minimal — but verify with `cargo check`.
- `////` ambiguity: spec grammar literally says `///` so 4+ slashes still match. We emit those as outer doc; if downstream wants 4-slash to be a regular comment, that is a future concern (not in scope here).
- Boundary with `skip_comments`: if the wiring driver step (separate todo) does not call `scan_doc_comment` before `skip_comments`, doc comments will fall through. The current `skip_comments` already returns `false` for `///`/`//!`, so the order-of-operations contract is preserved; no change here.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib lexer::scan::tests::doc_comments_preserved
cargo test --manifest-path vertex_stage0/Cargo.toml --lib
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The Rust crate lives at `vertex_stage0/`; `cargo` invocations must use `--manifest-path vertex_stage0/Cargo.toml`.
- `DocStyle` belongs in `token.rs` alongside `IntSuffix`/`FloatSuffix` (consistent with the existing module layout).
- `TokenKind::DocComment` carries the comment body **without** the `///` / `//!` prefix and **without** the trailing newline. This matches how `StringLiteral` carries the unescaped content rather than the surrounding quotes.
- `////...` (four or more slashes) is treated as an outer doc comment, since the spec grammar `"///" { non_newline }` literally matches it. Driver-level disambiguation can be added later if the spec clarifies otherwise.
- EOF terminates a doc comment cleanly — no rewind required, since the body has no closing delimiter to be missing.
- `scan_doc_comment` does not need to handle CRLF specially; the body is captured up to the next `\n`, and any trailing `\r` is left as part of the body (matches how other scanners treat raw byte content).
- Test name is `doc_comments_preserved` exactly, in `lexer::scan::tests`, so the verify command resolves it.
- No need to wire `scan_doc_comment` into a `next_token` driver in this commit — that is the separate `wire-all-scanners-into-scanner-next-token-driver` item.

## Blockers
Blockers: none

## Summary
Adds `DocStyle` and `TokenKind::DocComment` plus a rewind-safe `Scanner::scan_doc_comment` so `///` and `//!` are preserved instead of being skipped, covered by the `doc_comments_preserved` unit test.

# Plan: implement-error-pretty-printer-in-src-error-render-rs

## Goal
Add a `render(&CompileError, &SourceMap) -> String` pretty-printer at `vertex_stage0/src/error/render.rs` that emits rustc-style diagnostics (header, snippet, caret, notes, help), bootstrapping the still-missing `ErrorCode` / `ErrorKind` / `CompileError` types since the prerequisite TODO items have not landed yet (see needs-review.md entries on lines 83–202).

## Steps
1. Convert the single-file `vertex_stage0/src/error.rs` (currently only `Suggestion`) into a directory module: delete `src/error.rs`, create `src/error/mod.rs`. `lib.rs`'s `pub mod error;` stays unchanged.
2. In `src/error/mod.rs`, re-declare `Suggestion` as it exists today (message + optional replacement + span), and bootstrap the prerequisite types using shapes from `compiler_architecture.md` §6 + TODO §2 lines 78–90 so a future re-attempt of items "define ErrorCode/ErrorKind" and "define CompileError" reconciles cleanly:
   - `pub struct ErrorCode(pub u32);` with `Copy, Clone, PartialEq, Eq, Debug` and assoc consts `E0308`, `E0502` (enough for tests + future plumbing).
   - `pub enum ErrorKind { Lexical, Syntax, NameResolution, Type, BorrowCheck, Other }` with `Copy, Clone, PartialEq, Eq, Debug`.
   - `pub struct CompileError { pub code: ErrorCode, pub kind: ErrorKind, pub span: Span, pub message: String, pub suggestions: Vec<Suggestion>, pub notes: Vec<String> }` with `Debug, Clone`.
   - Builder methods: `pub fn new(code, kind, span, msg) -> Self`, `pub fn with_suggestion(self, s) -> Self`, `pub fn with_note(self, n) -> Self`.
   - Add `pub mod render;`.
3. Write `src/error/render.rs` with `pub fn render(err: &CompileError, src: &SourceMap) -> String`. Format (no color — see step 4):
   - Header: `error[E{code:04}]: {message}` (E0308 zero-padded to 4 digits).
   - Location: `  --> {filename}:{line}:{col}` using `SourceMap::line_col` on `err.span.start` and `SourceFile::name.display()`.
   - Snippet block: blank gutter line `   |`, then `{line:>4} | {line_text}`, then caret line `     | {spaces}{carets} {primary_label_msg}` where carets span the column range covered by the span on the primary line (clamp to end-of-line for multi-line spans; for v1 a single-line span is sufficient — multi-line falls back to caret at start col only). Trailing blank gutter line.
   - For each note: `   = note: {text}`.
   - For each suggestion: `   = help: {message}` (replacement preview deferred to a later item — current `Suggestion` has no rendered location yet beyond the help line; multi-label rendering is the explicitly separate next TODO item).
4. Color gating: detect `std::env::var_os("NO_COLOR").is_some()` OR stdout-not-tty → plain text. Implement the actual code path as plain-text-only for now (no `termcolor` dep added) — the isatty check returns "no color" deterministically, and the test sets `NO_COLOR=1` for belt-and-suspenders. A `// TODO: termcolor when stdout is a tty and !NO_COLOR` line is acceptable per project policy (concrete, non-obvious why).
5. Tests module in `render.rs`:
   - `renders_e0308_format`: at the top, `std::env::set_var("NO_COLOR", "1");`. Build a `SourceMap`, register a one-line file `"src/main.vx"` with content `"    x + \"hello\""` (or similar). Build `CompileError::new(ErrorCode::E0308, ErrorKind::Type, span_over_full_expr, "type mismatch")` with `.with_note("cannot add integer and string")` and `.with_suggestion(Suggestion { message: "convert the string to a number with: x + \"hello\".parse()?".into(), replacement: None, span: <same> })`. Assert the rendered string `contains` each of: `error[E0308]: type mismatch`, `--> src/main.vx:1:`, the source line text, `^`, `= note: cannot add integer and string`, `= help: convert the string to a number`.

## Files
- `vertex_stage0/src/error.rs` — delete (replaced by directory module).
- `vertex_stage0/src/error/mod.rs` — new; carries `Suggestion`, `ErrorCode`, `ErrorKind`, `CompileError` (+ builders), and `pub mod render;`.
- `vertex_stage0/src/error/render.rs` — new; `pub fn render` + private helpers + `#[cfg(test)] mod tests` containing `renders_e0308_format`.

## Risks
- **Bootstrapped types may conflict on re-attempt of prior TODO items.** Mitigation: field/method shapes mirror `compiler_architecture.md` §6 and TODO §2 exactly so a re-attempt is a no-op reconcile rather than a rewrite. This mirrors the resolution pattern already approved in needs-review.md for `implement-erroraccumulator`.
- **File-to-directory conversion.** If `src/error.rs` is not actually deleted (e.g., left behind by a stale checkout), Cargo will error with "file found for module `error` at both `error.rs` and `error/mod.rs`". The Steps explicitly delete the old file.
- **Caret column math with multi-byte source.** `SourceMap::line_col` returns 1-based char columns, but caret rendering needs to know how many display cells to underline. For the v1 test the source is ASCII, so char count == display count. Risk only matters when later inputs include multi-byte chars; flagged for the multi-label follow-up item.
- **Termcolor not added.** Output is always plain. Acceptable for now since the verify test forbids color anyway; later items can add the dep when wiring up the binary's terminal output path.
- **Cargo manifest path.** The repo root is not a Cargo workspace (see needs-review.md entries 76–93). All `cargo` commands must use `--manifest-path vertex_stage0/Cargo.toml`.

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::render::tests::renders_e0308_format
test -f vertex_stage0/src/error/mod.rs
test -f vertex_stage0/src/error/render.rs
```

## Assumptions
- `pub mod error;` in `lib.rs` does not need editing — Cargo resolves it against either `error.rs` or `error/mod.rs`.
- The actual `Span` field name is `file` (per current `src/span.rs`), not `file_id` (which TODO.md still says); the renderer uses `err.span.file`.
- `SourceFile.name` is a `PathBuf`; `name.display()` is acceptable for the `--> path:line:col` line.
- "Secondary labels" in the task bullet refers to forward-looking work (the explicit next TODO item is "Add multi-label support"), so this item renders only the primary span. Help lines come from `suggestions`, note lines from `notes`.
- Single-line spans are sufficient for the verify test; multi-line span rendering is deferred (clamped behavior described in step 3).
- No new dependency added (no `termcolor`); color gating is implemented as "always plain" with a TODO comment.
- A new `Span` constructor isn't needed — the test uses the existing `Span { file, start, end }` fields directly (the `Span::new` method in `span.rs:14` is also available).
- Bootstrapped `ErrorCode` only needs `E0308` and `E0502` constants for this item; full E0001..E1999 enumeration belongs to the original `define-errorcode-and-errorkind` item.
- Test forces `NO_COLOR=1` even though the implementation is always plain — defensive against a future contributor adding color without updating tests.

## Blockers
Blockers: none

## Summary
Land `error::render::render(&CompileError, &SourceMap) -> String` plus the minimum `ErrorCode`/`ErrorKind`/`CompileError` scaffolding it needs to compile, in the `src/error/` directory module, with one passing snapshot-style test asserting the E0308-shaped output.

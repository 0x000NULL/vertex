# Plan: add-multi-label-support-to-renderer

## Goal
Extend `CompileError` and `render` so a single error can carry multiple `Label { span, message, primary }` entries, where the primary label displays the source snippet+caret and secondary labels are referenced by `file:line:col` line-number citations.

## Steps
1. In `vertex_stage0/src/error/mod.rs`, add a public `Label` struct with fields `span: Span`, `message: String`, `primary: bool` (derives `Debug, Clone`).
2. Add a `pub labels: Vec<Label>` field to `CompileError`; initialize it as empty in `CompileError::new`. Add a builder method `with_label(self, label: Label) -> Self` (and a convenience `with_secondary_label(self, span, message)` that pushes a non-primary label) that pushes onto `labels`. Keep `code`, `kind`, `span`, `message`, `notes`, `suggestions` unchanged so the existing E0308 test continues to pass.
3. In `vertex_stage0/src/error/render.rs`, change `render` so that:
   - The "primary" snippet block is driven by the first `Label` with `primary == true` if any exist; otherwise fall back to `err.span` (preserves the existing E0308 test). Render its message (if non-empty) on the caret line as `^^^^ <message>`; if no message, just carets — matching today's behavior.
   - After the primary block, iterate over `labels.iter().filter(|l| !l.primary)` and emit one line each in the format `  ::: <file>:<line>:<col>: <message>` using `SourceMap::line_col` (no snippet rendering for secondaries). Use `:::` (rustc's convention) so the new lines are unambiguous and easy to match in tests.
   - Notes and suggestions continue to render after labels, in their existing order.
4. Add a unit test `multi_label_layout` in `error::render::tests` that:
   - Builds a `SourceMap` with one file containing multiple lines.
   - Constructs a `CompileError` with one primary `Label` on line N and at least two secondary `Label`s on different lines (and/or a different file).
   - Asserts the rendered output (a) contains the primary line's snippet text and a caret row, (b) contains a `:::` line referencing each secondary label's `file:line:col` and message, and (c) does NOT include the secondary snippet text on its own source line (i.e. secondaries are referenced, not snippet-rendered).
5. Run `cargo test --lib error::render::tests::multi_label_layout` and the existing `renders_e0308_format` test to confirm both pass.

## Files
- `vertex_stage0/src/error/mod.rs` — add `Label` struct, add `labels: Vec<Label>` field on `CompileError`, add `with_label` / `with_secondary_label` builder methods.
- `vertex_stage0/src/error/render.rs` — drive the primary snippet block from the first primary label (fallback to `err.span`), append `:::`-prefixed reference lines for each secondary label using `SourceMap::line_col`, add new `multi_label_layout` test in `tests` module.

## Risks
- The existing `renders_e0308_format` test creates a `CompileError` without any labels; the renderer must still produce the same output by falling back to `err.span` when `labels` is empty or has no primary entry. Mitigation: keep the fallback path identical to today's logic.
- Adding a non-`Default` field to `CompileError` is fine because all construction goes through `CompileError::new`, but any external callers that struct-literal-initialize `CompileError` would break. Quick grep should confirm none exist (only `error/render.rs::tests` uses `CompileError::new`).
- Multi-byte column accounting for secondary labels: reuse `SourceMap::line_col`, which is already UTF-8-aware (validated by `span::tests::line_col_handles_multibyte`), so no new char-counting code is needed.
- The exact format `:::` / `<file>:<line>:<col>: <msg>` is a guess at rustc-style output; the test must assert against substrings the implementation actually emits. Mitigation: write test and renderer together, asserting on stable substrings (`":::"`, `"file.vx:"`, the message text) rather than full-line equality.

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::render::tests::multi_label_layout
cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::render::tests::renders_e0308_format
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate `vertex_stage0` is the only crate; `cargo test --lib` from the workspace root may not find the test by itself, so `--manifest-path vertex_stage0/Cargo.toml` is used. (If a workspace `Cargo.toml` exists at the repo root, plain `cargo test --lib error::render::tests::multi_label_layout` would also work — the manifest-path form is strictly safer.)
- "Reference by line number" in the spec means a single-line citation in the form `<file>:<line>:<col>: <message>` (rustc's `:::` style), not a full snippet block. This matches the spirit of "primary shows snippet; secondary references by line number."
- `Label` fields use owned `String` for `message` (consistent with `Suggestion::message` and `CompileError::message`).
- The `primary` flag is just a bool — no enum needed; the renderer treats the first `primary == true` label as THE primary and renders any others (if a caller mistakenly marks two as primary) the same as the first, while still pushing extras as secondary references would over-engineer; simpler is to render the first primary as the snippet block and treat the rest as secondaries regardless of their `primary` flag. I'll go with: "first primary label drives the snippet; every other label (primary or not) is rendered as a `:::` reference line."
- Backward compatibility: callers that don't supply any labels still get today's exact output (snippet driven by `err.span`, no `:::` lines). This keeps `renders_e0308_format` green without touching it.
- The new `Label` type and `with_label` builder are added as `pub` so other modules (lexer/parser/typecheck) can adopt them later, but no existing code is migrated to multi-label form in this commit — that's out of scope.
- `with_secondary_label` is added as a small ergonomic helper because the test will likely use it; if it turns out unused, it can be inlined later. Adding it now keeps the test readable.

## Blockers
Blockers: none

## Summary
Adds a `Label` type and `labels: Vec<Label>` to `CompileError`, and teaches the renderer to draw a snippet+caret for the primary label and `:::`-prefixed `file:line:col: message` reference lines for each secondary label, verified by a new `multi_label_layout` test.

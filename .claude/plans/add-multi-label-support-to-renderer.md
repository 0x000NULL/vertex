# Plan: add-multi-label-support-to-renderer

## Goal
Ensure the diagnostic renderer in `vertex_stage0/src/error/render.rs` walks all `Label` entries on a `CompileError`, shows a source snippet with carets for the primary label, and prints secondary labels as `:::`-prefixed line/column references — verified by `error::render::tests::multi_label_layout`.

## Steps
1. Confirm `CompileError.labels: Vec<Label>` and `Label { span, message, primary }` are present in `vertex_stage0/src/error/mod.rs` (already added) and that `with_label` / `with_secondary_label` builders exist.
2. In `render::render`, locate the primary label by scanning `err.labels` for `primary == true`; fall back to `err.span` with an empty message if no primary label is present (preserves existing single-span E0308 path).
3. Render the header (`error[Ennnn]: msg`) and the `--> file:line:col` location using the resolved primary span.
4. Compute the primary line slice from `SourceFile::line_starts`, clamp the span end to the line end, derive the caret count as `(end_col - start_col).max(1)`, and emit the `   |`, `LL | <line>`, `     | <pad><^^^> <message>` block. Omit the trailing message segment when the primary label has no text.
5. After the primary block, iterate the remaining labels in original order, skipping the primary index. For each secondary label, resolve `(line, col)` via `SourceMap::line_col` and emit `  ::: <path>:<line>:<col>: <message>` — no source snippet, no carets.
6. Render `notes` as `   = note: …` and `suggestions` as `   = help: …` exactly as today.
7. Keep behavior for cross-file secondary labels: each `Label` carries its own `file_id`, and the `:::` line uses that file's path.
8. Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and the verify test below to confirm everything is clean.

## Files
- `vertex_stage0/src/error/render.rs` -- multi-label rendering loop: primary snippet + caret block, then `:::` reference lines for secondaries; existing `renders_e0308_format` and new `multi_label_layout` tests live here.
- `vertex_stage0/src/error/mod.rs` -- (read-only reference) confirms `Label`, `CompileError.labels`, `with_label`, `with_secondary_label` exist; do not modify.

## Risks
- Off-by-one in caret math when the span ends past the newline of the primary line — clamp `span_end` to `line_end` before counting chars.
- UTF-8 columns: `col` is a char count via `SourceMap::line_col`, but `pad` uses `" ".repeat(col-1)`. Wide chars may visually misalign; acceptable for stage 0, mirrors rustc's basic alignment.
- Secondary label whose `file_id` differs from the primary must call `src.file(label.span.file_id)` (not the cached primary file) — easy to regress.
- A `CompileError` with zero labels must still render via the fallback to `err.span` so `renders_e0308_format` keeps passing.
- A secondary label sharing the primary's line should NOT inline a snippet (test asserts `beta = 2` does not appear), so the secondary loop must never emit source text.

## Prereqs
- implement-span-struct-in-src-span-rs
- define-errorcode-and-errorkind-in-src-error-rs
- define-compileerror-struct-in-src-error-rs

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::render::tests::multi_label_layout
cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::render::tests::renders_e0308_format
cargo clippy --manifest-path vertex_stage0/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path vertex_stage0/Cargo.toml -- --check
```

## Assumptions
- The crate lives at `vertex_stage0/` and uses `--manifest-path vertex_stage0/Cargo.toml`; the bare `cargo test --lib error::render::tests::multi_label_layout` from the spec is interpreted as targeting that crate.
- Secondary labels reference the source by `<path>:<line>:<col>: <message>` with a leading `  ::: ` (matching rustc); the test asserts `:::`, the path, and `line:col` substrings, so this format satisfies it.
- Label iteration order matches insertion order (the test relies on `secondary_a` then `secondary_b` appearing as constructed; `Vec` preserves push order).
- Primary label uses the label's own span when present, otherwise falls back to `err.span` — preserves the existing E0308 test where no `Label` was attached.
- Caret count is at least 1 even for zero-width spans (e.g. EOF errors) via `.max(1)`.
- `NO_COLOR` is honored implicitly because the current renderer emits no ANSI codes; the TODO at the top of the file remains for a future ticket.
- The `multi_label_layout` test already exists in-tree (added alongside this change); the implementation just needs to satisfy it. If the test were missing, it would be added in this same commit.
- Notes and suggestions render unchanged after the secondary label block, matching the existing E0308 expectations.

## Blockers
Blockers: none

## Summary
Renderer iterates `CompileError.labels`, drawing the primary as a snippet with carets and message, and each secondary as a `:::`-prefixed `file:line:col: message` reference (no snippet), satisfying `multi_label_layout` while keeping `renders_e0308_format` green.

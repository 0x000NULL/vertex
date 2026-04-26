# Plan: define-suggestion-struct-in-src-error-rs

## Goal
Introduce a `Suggestion` struct in `vertex_stage0/src/error.rs` that pairs a human-readable message and an optional replacement with the `Span` it applies to, giving downstream diagnostic code a stable shape for fix-it hints.

## Steps
1. Add a `use crate::span::Span;` import at the top of `vertex_stage0/src/error.rs` (file is currently empty).
2. Define `pub struct Suggestion` with fields `message: String`, `replacement: Option<String>`, and `span: Span`. Derive `Debug` and `Clone` to match the conventions already used on `SourceFile` in `span.rs`.
3. Run the verify steps to confirm the crate still builds and the struct is publicly declared in the expected file.

## Files
- `vertex_stage0/src/error.rs` — currently empty; add `use crate::span::Span;` and a `#[derive(Debug, Clone)] pub struct Suggestion { pub message: String, pub replacement: Option<String>, pub span: Span }`.

## Risks
- The `error` module is referenced from `vertex_stage0/src/lib.rs` (line 7). Adding new items to a previously-empty module will not break any existing call sites because nothing currently imports from `error`.
- Choosing `Debug, Clone` derives is a low-risk default; `Copy` is impossible due to `String`/`Option<String>` fields. No `PartialEq`/`Eq` needed yet — can be added when first consumer demands it.

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
grep -q 'pub struct Suggestion' vertex_stage0/src/error.rs
```

## Assumptions
- Fields should be `pub` so consumers in sibling modules (lexer, parser, etc.) can construct/read them directly, mirroring the all-`pub` field style used on `Span` and `SourceFile` in `span.rs`.
- The `Span` to import is `crate::span::Span` (the only `Span` in the crate per `span.rs:7`).
- Standard derives are `#[derive(Debug, Clone)]` — `Debug` for diagnostics output, `Clone` because suggestions will likely be cloned when emitted alongside errors. No `Copy` (owned `String`s).
- Verify commands run from the repo root (`C:\Users\Ethan\vertex`), so `--manifest-path vertex_stage0/Cargo.toml` is needed for `cargo build` and the `grep` path is `vertex_stage0/src/error.rs`.
- No tests are required by the sub-step spec; structural definition is sufficient for this item.

## Blockers
Blockers: none

## Summary
Creates the `Suggestion` diagnostic struct (message + optional replacement + span) in the previously-empty `error.rs`, unblocking future diagnostic work.

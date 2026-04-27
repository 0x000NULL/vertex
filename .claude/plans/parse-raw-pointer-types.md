# Plan: parse-raw-pointer-types

## Goal
Extend `parse_type` to recognize `*const T` and `*mut T` and produce `Type::Ptr`, locked in by a single `parser::ty::tests::raw_ptr_types` unit test.

## Steps
1. In `vertex_stage0/src/parser/ty.rs`, add a leading `TokenKind::Star` branch to `parse_type` that dispatches to a new `parse_ptr_type` helper (mirroring the existing `parse_ref_type` structure).
2. Implement `parse_ptr_type`: bump the `*`, then require either `TokenKind::Const` or `TokenKind::Mut` (set `mutable` to `true` for `mut`, `false` for `const`); if the next token is neither, push a `CompileError` via the parser's existing error helper and recover by treating it as `const` so the inner type still parses. Recursively call `self.parse_type()` for the inner type and return `Type::Ptr { mutable, ty: Box::new(inner) }`.
3. Add a `Type::Ptr { ty, .. }` arm to the local `type_span` helper that returns the inner type's span — `Ptr` has no span field per the spec, and `type_span` is itself flagged as a stopgap.
4. Add the `raw_ptr_types` test inside `mod tests` covering: `*const i32` → `Ptr { mutable: false }` wrapping a single-segment path `i32`; `*mut i32` → `Ptr { mutable: true }` wrapping `i32`; and a nested `*const *mut i32` to verify the recursive case. Each case asserts `p.errors.is_empty()` and that the parser is parked on `Eof`.

## Files
- `vertex_stage0/src/parser/ty.rs` -- add `Star` dispatch in `parse_type`, add `parse_ptr_type` helper, extend `type_span` with a `Ptr` arm, add `raw_ptr_types` unit test.

## Risks
- Mis-classifying which keyword sets `mutable=true`: `*mut` is mutable, `*const` is not. Easy to invert; the test's three cases catch it.
- If a future `parse-reference-types` rewrite reshapes `parse_ref_type`'s span handling, the `type_span` Ptr arm may need to be revisited — acceptable since `type_span` is a self-described stopgap.
- Recovery on a malformed `*` (neither `const` nor `mut`) needs to avoid an infinite loop. Defaulting to `mutable=false` and continuing into `parse_type()` is consistent with the rest of the parser's tolerant style; not exercised by the test but worth keeping the recovery deterministic.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::ty::tests::raw_ptr_types
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate's manifest lives at `vertex_stage0/Cargo.toml` (matches the existing source layout); the verify commands pass `--manifest-path` so the run works from the repo root.
- `TokenKind::Star`, `TokenKind::Const`, and `TokenKind::Mut` already exist in the lexer (confirmed: `Star` is at `src/lexer/token.rs:75`; `Const`/`Mut` are present per existing `parse_ref_type` and the const/static items already shipped).
- The `Type::Ptr` AST variant stays exactly `Ptr { mutable: bool, ty: Box<Type> }` per `compiler_architecture.md:200` and `TODO.md:285` — no `span`/`id` fields are added, even though `Ref` carries them.
- `parse_ptr_type` recursively delegates to `parse_type` for the inner type, so `*const *mut T` and `*const &T` compose for free.
- The error-recovery path for `*` not followed by `const`/`mut` falls back to `mutable=false` and continues parsing the inner type. Not asserted by the test; chosen for consistency with the parser's existing tolerant style.
- `type_span`'s `Ptr` arm returns the inner type's span (the `*const`/`*mut` prefix is dropped). This is acceptable because `type_span` is documented as a stopgap that will be replaced when path-types/array/tuple/fn types land.
- The new test reuses the local `tok`, `ident_tok`, and `assert_path_ident` helpers already defined in `mod tests`.

## Blockers
Blockers: none

## Summary
Add `*const T` / `*mut T` parsing to `parse_type` (with a stopgap `type_span` arm) and a single `raw_ptr_types` unit test pinning the form.

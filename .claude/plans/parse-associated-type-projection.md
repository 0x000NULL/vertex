# Plan: parse-associated-type-projection

## Goal
Parse `<T as Iterator>::Item` as a qualified-path projection in `parse_type`, producing a new `Type::QPath` AST variant locked in by one unit test.

## Steps
1. Add a `Type::QPath { self_ty: Box<Type>, trait_: Path, segments: Vec<PathSegment>, span: Span, id: NodeId }` variant to the `Type` enum in `vertex_stage0/src/ast/ty.rs` (gated `#[allow(dead_code)]` like its siblings). `segments` holds the trailing `::Item[::More]` projection (length ≥ 1).
2. In `vertex_stage0/src/parser/ty.rs`, add a leading `TokenKind::Lt` arm at the top of `parse_type` that delegates to a new `parse_qpath_type` helper. Place it before the existing arms so it is unambiguous (a `<` at type position can only start a qualified path today; generic-arg `<` follows an ident and is handled by the upcoming `parse-path-types-with-generic-args`).
3. In `parse_qpath_type`:
   - Bump `<`, remember `start_span`.
   - Recursively call `self.parse_type()` for the self type (handles `&T`, `*const T`, paths, etc.).
   - Expect `TokenKind::Ident(s)` with `s == "as"` (the lexer emits `as` as an identifier — see the precedent at `parser/expr.rs:688`); on mismatch push an `E0100` Syntax error and recover via `recover_to_sync`.
   - Parse the trait body by calling `self.parse_type()` and matching the result: if it is `Type::Path(p)`, take `p`; otherwise push an `E0100` error ("expected trait path after `as`") and recover. (Stopgap is fine since the current `parse_type` path-body produces only `Type::Path`; the `parse-path-types-with-generic-args` item will broaden this naturally.)
   - Expect `TokenKind::Gt`.
   - Expect `TokenKind::ColonColon`, then parse one or more `Ident` segments separated by `::`, each producing a `PathSegment { ident, generic_args: Vec::new() }` (mirroring the stopgap path-type body). Loop while we see another `::` followed by an `Ident`.
   - Compute `span = start_span.merge(&last_segment_span)` and assign a `new_node_id()`.
4. Update the `type_span` helper in `vertex_stage0/src/parser/ty.rs` to add a `Type::QPath { span, .. } => *span` arm so the `_ => unreachable!` does not fire when a QPath appears inside `&`, `*`, `[…]`, `(…)`, or `fn` types.
5. Add a single `#[test] fn assoc_projection()` in the existing `parser::ty::tests` module that feeds the token sequence `[ Lt, Ident("T"), Ident("as"), Ident("Iterator"), Gt, ColonColon, Ident("Item"), Eof ]`, calls `p.parse_type()`, asserts:
   - the result matches `Type::QPath { self_ty, trait_, segments, .. }`,
   - `*self_ty` is `Type::Path` whose single segment ident equals `"T"`,
   - `trait_.segments` has one segment with ident `"Iterator"` and empty generic_args,
   - `segments` has length 1 with ident `"Item"` and empty generic_args,
   - `p.errors` is empty and the next peek is `Eof`.

## Files
- `vertex_stage0/src/ast/ty.rs` -- add `QPath` variant to the `Type` enum (with `self_ty`, `trait_: Path`, `segments: Vec<PathSegment>`, `span`, `id`).
- `vertex_stage0/src/parser/ty.rs` -- add `Lt` arm to `parse_type`, add `parse_qpath_type` helper, extend `type_span` to cover `Type::QPath`, add `assoc_projection` unit test.

## Risks
- A future `parse-path-types-with-generic-args` may want `<` to start generic args; that case is always preceded by an ident, while this arm only fires at the *start* of `parse_type`. No conflict, but worth a code comment near the `Lt` branch.
- If `parse_type` is later extended so the trait body can produce a non-`Type::Path` (e.g. a `&Trait` object), the `Type::Path` extraction here will need to widen. The stage-0 spec does not require trait objects in QPath, so the narrowed match is acceptable today.
- `assert_path_ident` already assumes a single-segment path; reusing it for the inner self-ty / trait checks works for `T` and `Iterator` but not for paths with generics. The test stays on bare idents to match the stopgap.
- The `as` keyword is not in `TokenKind`; relying on `Ident("as")` matches the established precedent (`parser/expr.rs:688`, `parser/item.rs:995`). When/if a real `As` token is introduced, this site needs updating alongside `parser/expr.rs` cast handling.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::ty::tests::assoc_projection
cargo check --lib --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate manifest lives at `vertex_stage0/Cargo.toml` (workspace layout — confirmed by file listing). `cargo test --lib` without `--manifest-path` would fail from the repo root.
- `as` continues to be lexed as `TokenKind::Ident("as")`. No `TokenKind::As` exists today.
- `Type::QPath` is the right shape (self_ty + trait Path + projection segments). Storing the trait as `Path` (not `Box<Type>`) keeps the data model honest: a trait reference is a path, not an arbitrary type.
- Length-1 projection segment is sufficient for the named verify test, but the loop accepts longer projections (`<T as Foo>::A::B`) for forward compatibility — costs nothing and matches Rust's grammar.
- Putting the `Lt` arm at the top of `parse_type` (before `Amp`/`Star`/`LBracket`/`LParen`/`Fn`/`Extern`) is correct because none of those start with `<`.
- The new `Type::QPath` variant carries `span` and `id` (matching `Type::Ref`), so `type_span` returns the stored span rather than recursing.
- The existing `_ => unreachable!` in `type_span` covers `Type::Infer` today; adding `QPath` keeps that fallback semantics (only `Infer` remains unreachable from the parser).
- One-test-per-item discipline: a single `#[test] fn assoc_projection` is added, matching the cadence of `ref_types`, `raw_ptr_types`, etc.

## Blockers
Blockers: none

## Summary
Adds `<T as Iterator>::Item` parsing to `parse_type` via a new `Type::QPath` AST variant and a single `assoc_projection` unit test.

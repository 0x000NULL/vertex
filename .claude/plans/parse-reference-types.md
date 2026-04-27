# Plan: parse-reference-types

## Goal
Extend `parse_type` in `src/parser/ty.rs` so a leading `&` produces a `Type::Ref { mutable, ty, span, id }` for `&T`, `&mut T`, and (where lexer support exists) `&'lifetime T`, with the lifetime parsed-and-discarded.

## Steps
1. In `src/parser/ty.rs`, add a leading-token dispatch at the top of `parse_type`: when `peek()` is `TokenKind::Amp`, delegate to a new private `parse_ref_type` helper; otherwise fall through to the existing path-type code from `parse-path-types-with-generic-args`.
2. Implement `fn parse_ref_type(&mut self) -> Result<Type, CompileError>`:
   - `bump()` the `Amp` and remember `start_span`.
   - **Optional lifetime swallow:** if the next token is an `Ident(name)` whose name begins with `'` (the convention adopted once the lexer learns lifetimes; today no such token is produced, so this branch is a defensive no-op), `bump()` it and discard. The AST `Type::Ref` carries no lifetime field, matching the "Stage 0 simplification — lifetime parsed but ignored semantically" note.
   - If `peek()` is `TokenKind::Mut`, `bump()` it and set `mutable = true`; else `mutable = false`.
   - Recursively call `self.parse_type()?` to obtain the inner `Type` (this naturally supports nested `&&T`, `&mut &T`, `&Vec<T>`, etc.).
   - Compute `inner_span` via a small local `fn type_span(&Type) -> Span` that matches the variants `parse_type` can produce so far (`Path`, `Ref`); panic on unknown variants with a clear message (consistent with the existing helper in `item.rs`).
   - Build `Type::Ref { mutable, ty: Box::new(inner), span: start_span.merge(&inner_span), id: self.new_node_id() }`.
3. Update the existing `tests` module in `parser/ty.rs` (created by the prereq) with a single `#[test] fn ref_types` that exercises three sub-cases by building token vectors directly (mirroring the style of `path_with_generics` and the existing `parse_self_param_ref` tests in `item.rs`):
   - `&i32` → `Type::Ref { mutable: false, ty: Path("i32"), .. }`
   - `&mut i32` → `Type::Ref { mutable: true, ty: Path("i32"), .. }`
   - `&&i32` → outer `Type::Ref { mutable: false, ty: Type::Ref { mutable: false, ty: Path("i32") } }` (proves recursion)
   - Each sub-case asserts `p.errors.is_empty()` and `p.peek() == TokenKind::Eof`.
4. Confirm `is_sync_point` need not be modified (`Amp` is not a sync token and reference types appear inside expressions/types where the existing sync set is correct).
5. Confirm `type_span` in `item.rs` already handles `Type::Ref` (it does, line ~1175) so existing call sites in `parse_where_clause` keep working unchanged once a `Ref` flows through there.

## Files
- `src/parser/ty.rs` — extend `parse_type` with the `Amp` branch, add `fn parse_ref_type`, add a small local `type_span` helper if the prereq did not already provide one, and add the `ref_types` test inside the existing `tests` module.

## Risks
- **Lexer has no lifetime token yet.** `'static` would today lex as `Error("'") + Static + …`, so `&'static str` cannot be exercised end-to-end. The plan handles this by writing a defensive lifetime-swallow branch that activates once lexer support lands; until then, the test does not cover lifetimes. If a reviewer wants live `&'static str` coverage, the lexer must learn lifetime tokens first — out of scope for this slug.
- **Recursive `parse_type` and span widths.** The merged span depends on the inner type's `span`. If the inner type is a `Type::Path` with a multi-token span, `type_span` must read it from the path; this matches the existing helper.
- **Span on `Type::Ref` inner.** Using a single `type_span` helper inside `parser/ty.rs` duplicates the one in `item.rs`. Mitigation: leave `item.rs`'s copy alone (it gates on the stopgap variants) and add a private one in `ty.rs` covering only `Path` and `Ref`. The two copies stay until tuple/array/etc. types arrive and a shared helper is justified.
- **Ordering with prereq.** This item assumes `parser/ty.rs` and a real `parse_type` exist. If `parse-path-types-with-generic-args` lands later, this plan must be re-sequenced.

## Prereqs
parse-path-types-with-generic-args

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::ty::tests::ref_types
cargo build --manifest-path vertex_stage0/Cargo.toml
cargo test --lib --manifest-path vertex_stage0/Cargo.toml
test -f vertex_stage0/src/parser/ty.rs
```

## Assumptions
- `parser/ty.rs` is created by `parse-path-types-with-generic-args`; this slug only adds an `Amp` branch and a test there. If the prereq has not run, this plan is blocked, not worked around.
- Lifetime tokens are not yet emitted by the lexer (verified: `src/lexer/token.rs` has no `Lifetime` variant; `scan_char` rejects `'static`). The plan therefore restricts test coverage to `&T`, `&mut T`, and nested `&&T`. The defensive `Ident`-starting-with-`'` swallow is forward-compatible with the most common lifetime-token shape but is untested today.
- `Type::Ref` keeps its existing AST shape (`mutable: bool`, `ty: Box<Type>`, `span: Span`, `id: NodeId`) — no new lifetime field. This matches what `parse_self_param_ref` already constructs.
- Test `ref_types` is a single `#[test] fn` with three internal sub-cases, mirroring `path_with_generics`.
- The `Cargo.toml` lives at `vertex_stage0/Cargo.toml`, so `--manifest-path vertex_stage0/Cargo.toml` is required; the workspace root is not a crate.
- Recursive `parse_type` already handles arbitrary inner types — no special case for path heads after the optional `mut`.
- No changes are needed to `is_sync_point`, `recover_to_sync`, or any other parser scaffolding; reference parsing fails fast through the existing `expect`/`?` plumbing.
- The `&'static T` form mentioned in the spec snippet is documented as deferred (lexer dependency) rather than implemented as a token-stream hack involving `Error("'")`, which would be brittle and would not survive when lifetime tokens are introduced.

## Blockers
Blockers: none

## Summary
Adds a leading-`&` branch to `parse_type` that builds `Type::Ref` for `&T` and `&mut T` (with a forward-compatible lifetime-skip for when the lexer emits lifetimes), locked in by a single `parser::ty::tests::ref_types` test.

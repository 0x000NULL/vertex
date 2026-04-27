# Plan: parse-slice-array-types

## Goal
Extend `parse_type` to recognize `[T]` (only as the pointee of `&` per spec) and `[T; N]` (array with const-expr length), reusing the existing `Type::Slice` / `Type::Array` AST variants and locking the form in with one `slice_and_array_types` unit test.

## Steps
1. In `vertex_stage0/src/parser/ty.rs`, extend `Parser::parse_type` to dispatch on `TokenKind::LBracket` to a new private `parse_bracketed_type`.
2. Implement `parse_bracketed_type`:
   - `bump()` the `[`.
   - Call `self.parse_type()` to read the element type `T`.
   - If the next token is `TokenKind::Semi`, `bump()` it, parse the length via `self.parse_expr()`, then `expect(&TokenKind::RBracket)`, and return `Type::Array { elem: Box::new(T), len: Box::new(len_expr) }`.
   - Otherwise `expect(&TokenKind::RBracket)` and return `Type::Slice { elem: Box::new(T) }`.
3. Extend the local `type_span` helper to handle `Type::Array`/`Type::Slice` by recursing into the element type (consistent with the existing `Ptr` arm — the helper is the stopgap for `&[T]`'s span computation in `parse_ref_type` and is documented to disappear once path types land).
4. Add a `#[test] fn slice_and_array_types()` in `parser::ty::tests` covering:
   - `&[i32]` → `Type::Ref { mutable: false, ty: Type::Slice { elem: Path("i32") } }`.
   - `[i32; 4]` → `Type::Array { elem: Path("i32"), len: Expr::IntLit(4) }`.
   - Each case asserts `errors.is_empty()` and that the parser is positioned at `Eof`.

## Files
- `vertex_stage0/src/parser/ty.rs` — add `[`-arm to `parse_type`, add `parse_bracketed_type`, extend `type_span` for `Array`/`Slice`, add the `slice_and_array_types` unit test.

## Risks
- `Type::Slice` and `Type::Array` carry no `span` field, so `type_span`'s fallback (recurse into the element) loses the `[`/`]` extents. This matches the existing `Ptr` arm's looseness and is acceptable for the stopgap; the real path-type/whole-type pass will replace it.
- `parse_expr` is fairly broad (range, comparison, etc.) — for `[T; N]` this is fine because the `]` cannot be a continuation of a Pratt expression, so the length parse will stop cleanly before `]`. The test pins this with a plain integer literal to avoid any surprise.
- Pending `parse-array-literal-expressions` (expression-level `[1, 2, 3]`) is dispatched from `parse_expr`, not `parse_type`, so the two `[`-entry points don't collide.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::ty::tests::slice_and_array_types
```

## Assumptions
- The verify path of "ONE test named `slice_and_array_types` in `parser::ty::tests`" is the contract; the prompt says "single … unit test" and gives that exact path. Both the `&[T]` and `[T; N]` cases live inside that one test function (mirroring the precedent set by `ref_types`, which already covers `&i32`, `&mut i32`, and `&&i32` together).
- `Type::Slice`/`Type::Array` are reused as-is — no AST schema changes. They already exist on `crate::ast::ty::Type`.
- The length expression of `[T; N]` is parsed with the existing `Parser::parse_expr`. The plan does not narrow it to a "const-expr" subset; const-ness is a later semantic pass, and the spec sub-step's parenthetical "(where `N` is a const expr)" is informational.
- The `&[T]` case must work even though `parse_ref_type` calls `type_span(&inner)` on the slice — the new `Slice`/`Array` arms in `type_span` make that legal. The exact span value is not asserted by the test (the existing `ref_types` test also doesn't assert spans).
- The `cargo test` invocation needs `--manifest-path vertex_stage0/Cargo.toml` because the cargo workspace lives in that subdirectory (root has `Cargo.lock`/`Cargo.toml` but the crate sources are under `vertex_stage0/`). If the workspace already wires this up so a top-level `cargo test` works, the extra flag is harmless.

## Blockers
Blockers: none

## Summary
Add `[T]` (slice) and `[T; N]` (array, length parsed via `parse_expr`) handling to `parse_type`, locked in by one `slice_and_array_types` unit test.

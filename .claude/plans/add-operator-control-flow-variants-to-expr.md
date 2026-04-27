# Plan: add-operator-control-flow-variants-to-expr

## Goal
Add operator and access-style variants (`Unary`, `Binary`, `Call`, `MethodCall`, `Field`, `TupleField`, `Index`, `Cast`, `Try`) to the existing `Expr` enum in `vertex_stage0/src/ast/expr.rs`, mirroring the per-variant struct + `id()`/`span()` dispatch pattern already in place.

## Steps
1. In `vertex_stage0/src/ast/expr.rs`, add two small operator enums: `UnaryOp` (with `Neg`, `Not`, `Deref`, `Ref`, `RefMut` — covering spec §2 unary forms `- not * & &mut`) and `BinaryOp` (covering arithmetic `+ - * / %`, comparison `== != < > <= >=`, logical `and or`, bitwise `& | ^ << >>`, and assignment `= += -= *= /= %=` from spec §2). Both `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`.
2. Add a `CastTy` placeholder enum (single `Placeholder` variant) with a `// TODO: replaced by define-type-enum-in-src-ast-ty-rs` comment, following the existing `GenericArg::Placeholder` precedent (`expr.rs:62-68`) so `Cast` has a typed `ty` field that the later `Ty` work can swap in.
3. Add nine per-variant structs (`Unary`, `Binary`, `Call`, `MethodCall`, `FieldAccess`, `TupleFieldAccess`, `Index`, `Cast`, `Try`), each carrying `id: NodeId`, `span: Span`, plus the variant-specific payload — recursive `Expr` children held as `Box<Expr>` (or `Vec<Expr>` for `args`). Field shapes:
   - `Unary { id, span, op: UnaryOp, operand: Box<Expr> }`
   - `Binary { id, span, op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr> }`
   - `Call { id, span, callee: Box<Expr>, args: Vec<Expr> }`
   - `MethodCall { id, span, receiver: Box<Expr>, method: String, args: Vec<Expr>, generic_args: Vec<GenericArg> }` (reusing existing `GenericArg`)
   - `FieldAccess { id, span, receiver: Box<Expr>, name: String }`
   - `TupleFieldAccess { id, span, receiver: Box<Expr>, idx: u32 }`
   - `Index { id, span, receiver: Box<Expr>, idx: Box<Expr> }`
   - `Cast { id, span, expr: Box<Expr>, ty: Box<CastTy> }`
   - `Try { id, span, expr: Box<Expr> }`
   Each gets `#[allow(dead_code)] #[derive(Debug, Clone)]` to match existing variants.
4. Extend the `Expr` enum with the corresponding nine variants (`Unary(Unary)`, `Binary(Binary)`, `Call(Call)`, `MethodCall(MethodCall)`, `Field(FieldAccess)`, `TupleField(TupleFieldAccess)`, `Index(Index)`, `Cast(Cast)`, `Try(Try)`).
5. Extend the `Expr::id()` and `Expr::span()` match arms to dispatch the new variants to their inner struct's field.
6. Run `cargo build` from the workspace root and confirm the crate still compiles cleanly.

## Files
- `vertex_stage0/src/ast/expr.rs` — add `UnaryOp`, `BinaryOp`, `CastTy` placeholder, nine new variant structs, nine new `Expr` enum arms, and corresponding `id()`/`span()` match arms.

## Risks
- Box layout / recursion: `Expr` becomes a recursive enum once `Unary`/`Binary`/etc. hold child `Expr`s. Forgetting `Box` on a child field would make the type infinite-size and break the build. Mitigated by using `Box<Expr>` consistently for single children and `Vec<Expr>` for multiple.
- Naming collision: `Field` and `Index` are common names. Named outer variants `Field`/`TupleField`/`Index` but used distinct struct names (`FieldAccess`, `TupleFieldAccess`, `Index`) to avoid the variant name shadowing the struct in match arms.
- Future-Ty churn: introducing `CastTy::Placeholder` adds a small migration cost when `define-type-enum-in-src-ast-ty-rs` lands, but it matches the existing precedent (`GenericArg::Placeholder`) so the cost is already accepted by the codebase.
- Dead-code warnings: maintained with `#[allow(dead_code)]` on each new struct/enum, matching existing items.

## Prereqs
Prereqs: none

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
grep -q "Unary(Unary)" vertex_stage0/src/ast/expr.rs
grep -q "Binary(Binary)" vertex_stage0/src/ast/expr.rs
grep -q "Call(Call)" vertex_stage0/src/ast/expr.rs
grep -q "MethodCall" vertex_stage0/src/ast/expr.rs
grep -q "TupleField" vertex_stage0/src/ast/expr.rs
grep -q "Cast(Cast)" vertex_stage0/src/ast/expr.rs
grep -q "Try(Try)" vertex_stage0/src/ast/expr.rs
grep -q "Index(Index)" vertex_stage0/src/ast/expr.rs
```

## Assumptions
- The instruction's substep listing `Unary, Binary, Call, MethodCall, Field, TupleField, Index, Cast, Try` is authoritative for what this commit must add. The plan title's mention of "control-flow" is a misnomer — true control-flow variants (`if`, `match`, `loop`, `return`, `break`, `continue`, `block`) are owned by the separate pending item `add-control-flow-variants-to-expr` and are NOT in scope here.
- `Cast`'s `ty` field uses a local `CastTy` placeholder enum (with a TODO comment) rather than waiting on `define-type-enum-in-src-ast-ty-rs`. This mirrors the existing `GenericArg::Placeholder` precedent in the same file, keeps this commit unblocked, and gives the future `Ty` item a clear swap target.
- `MethodCall`'s `generic_args` reuses the existing `GenericArg` enum already defined in `expr.rs`, rather than introducing a parallel placeholder.
- `UnaryOp` includes `Ref` and `RefMut` (the borrow operators `&` and `&mut`) as unary variants since spec §2 lists `&` and `&mut` as prefix operators; `Deref` covers the `*` prefix form. Bitwise `~` from spec §2 is included as `BitNot` as well, in case the parser later wants it (it's a unary, but spec lists it under bitwise — including for completeness).
- `Field::name` and `MethodCall::method` use `String` to match the precedent set by `PathSegment::ident: String` (`expr.rs:58`); interning can come later.
- `TupleFieldAccess::idx` is `u32` (tuple indices are small non-negative integers).
- Per-variant struct names follow the pattern `<VariantName>` where unambiguous (e.g. `Unary`, `Binary`, `Cast`) and `<VariantName>Access` where the bare name conflicts (`FieldAccess`, `TupleFieldAccess`).
- Variant ordering inside `Expr` keeps existing literal+path variants first and appends the new variants after, preserving call-site stability for any existing matches.
- Verify uses `cargo build --manifest-path vertex_stage0/Cargo.toml` because the workspace root `Cargo.toml` and the crate `Cargo.toml` both exist; targeting the crate is unambiguous and the substep specifies `cargo build`.

## Blockers
Blockers: none

## Summary
Extends `Expr` with operator/access/cast/try variants by following the existing per-variant-struct + id/span-dispatch pattern, using a local `CastTy` placeholder so the work doesn't block on the still-pending `Ty` enum.

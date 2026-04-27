# Plan: add-self-parameters

## Goal
Extend `parse_fn` to recognize the five self-parameter forms (`self`, `&self`, `&mut self`, `self: Box<Self>`, `self: Rc<Self>`) as the optional first parameter of a function, and pin the behavior with a `self_params` unit test.

## Steps
1. In `vertex_stage0/src/ast/item.rs`, add `pub is_self: bool` to `Param` so the AST distinguishes `self` parameters from ordinary ones (default `false`).
2. In `vertex_stage0/src/parser/item.rs`:
   - Add a helper `try_parse_self_param(&mut self) -> Option<Result<Param, CompileError>>` that uses `peek` / `peek_at` to detect, without consuming on the `None` path:
     - `SelfLower` → bare `self`
     - `Amp` followed by `SelfLower` → `&self`
     - `Amp` followed by `Mut` then `SelfLower` → `&mut self`
   - When a self-form is detected, consume the relevant tokens and synthesize the type:
     - bare `self` (no `:`) → `Type::Path(Path { segments: [{ ident: "Self", generic_args: vec![] }], … })`
     - `&self` / `&mut self` → `Type::Ref { mutable, ty: Box::new(Type::Path(Self)), span, id }`
     - bare `self` followed by `:` → call a small new helper `parse_self_explicit_type` that accepts a single-segment path with one optional generic argument, e.g. `Box<Self>` / `Rc<Self>` (build a `PathSegment { ident: "Box"|"Rc", generic_args: vec![GenericArg::Placeholder] }`, accepting either `Ident` or `SelfUpper` inside the angle brackets, expecting `Lt` … `Gt`).
   - In `parse_fn`'s param loop, call `try_parse_self_param` once at the very start (before any normal params); if it returns `Some`, push the resulting `Param` and require either a `Comma` (continue with normal params) or proceed to expect `RParen`. Disallow a self-form anywhere except the first slot.
   - Update the existing normal-param construction site to set `is_self: false`.
   - Compute spans by merging the first consumed token's span with the last consumed token's span (e.g., `&` … `self`, or `self` … `>`).
3. Add a new `#[test] fn self_params` in the `tests` mod of `parser/item.rs` covering all five forms inside a `fn m(<form>) {}` and asserting:
   - `f.params.len() == 1`, `f.params[0].name == "self"`, `f.params[0].is_self == true`.
   - `&self` / `&mut self` produce `Type::Ref { mutable, … }` with the expected mutability and an inner `Type::Path` segment of `"Self"`.
   - bare `self` produces `Type::Path` with a single `"Self"` segment, no generic args.
   - `self: Box<Self>` and `self: Rc<Self>` produce `Type::Path` with one segment whose `ident` is `"Box"` / `"Rc"` and `generic_args.len() == 1`.
   - `p.errors.is_empty()` and trailing token is `Eof` for each case.
   - Optionally, a sixth case `fn m(&self, x: i32) {}` to confirm self + trailing comma + a normal param still parses cleanly with `is_self == false` on the second param.

## Files
- `vertex_stage0/src/ast/item.rs` — add `is_self: bool` to `Param`.
- `vertex_stage0/src/parser/item.rs` — add `try_parse_self_param` and `parse_self_explicit_type` helpers; wire them into `parse_fn`'s param loop; set `is_self: false` on the existing normal-param construction; add the `self_params` unit test.

## Risks
- The existing stopgap `parse_type` does not accept `SelfUpper` or generic args, so the `self: Box<Self>` / `self: Rc<Self>` path requires a self-only mini-type helper. When `parse-path-types-with-generic-args` lands, that helper should be deleted in favor of the general parser — flag this in the helper's doc-comment.
- Adding `is_self` is a non-breaking field addition, but every `Param { … }` literal must be updated; only `parse_fn` constructs `Param` today, so the surface is small.
- Span correctness on `&mut self` (three-token consume) and on `self: Box<Self>` (five-token consume) — must use the first and last consumed token's spans, not the `LParen`/`Colon`.
- Lookahead must NOT consume tokens when the param is not a self-form; rely on `peek_at(1)` / `peek_at(2)` only.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::self_params
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::plain_fn
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::item::tests::fn_modifiers
```

## Assumptions
- Self parameters only appear as the first positional parameter; the parser does not enforce "method signatures must live in `impl`/`trait`" — that's a later semantic check. The parser accepts a self-form in any free-standing `fn` for now (matches what the task spec asks for).
- The existing `Param` struct is the right home for self params, with an added `is_self: bool` flag, rather than introducing a separate `SelfParam` AST node. This keeps `FnDef.params` homogeneous and avoids ripples in downstream consumers.
- Bare `self` (no type annotation) is encoded as `Type::Path` with a single `"Self"` segment, mirroring how the spec treats `self` as shorthand for `self: Self`.
- `self: Box<Self>` / `self: Rc<Self>` are encoded as `Type::Path` with a single segment carrying one `GenericArg::Placeholder` (the AST's current placeholder enum), since `GenericArg` cannot yet carry a nested `Type` payload. The presence/count of generic args is what the test will assert.
- Inside `<…>` we accept either `SelfUpper` or any `Ident` as the generic argument token (so `Box<Self>`, `Rc<Self>`, and even `Box<T>` are parsed identically at this stage); strictness around `Self`-only is deferred.
- Errors on a disallowed self-form (e.g., `self` appearing after another param) are out of scope here; we simply do not look for self-form past the first slot.
- The `Param::span` for `&mut self` is `&` … `self`, computed by merging the first and last consumed token spans.
- The verify command in TODO.md (`cargo test --lib parser::item::tests::self_params`) needs `--manifest-path vertex_stage0/Cargo.toml` because the workspace root has no `Cargo.toml` driving the library; the existing `cargo test` invocations in this repo target the `vertex_stage0` crate directly.

## Blockers
Blockers: none

## Summary
Adds `is_self` to `Param` and teaches `parse_fn` to recognize all five self-parameter forms as the optional first parameter, locked in by a `parser::item::tests::self_params` unit test.

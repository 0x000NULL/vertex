I have enough context to write the plan. The current state is a stopgap `parse_type` in `parser/item.rs` that only handles a bare identifier. The verify path (`parser::ty::tests::path_with_generics`) implies the new code lives in `src/parser/ty.rs`. `GenericArg` is currently a placeholder enum and needs to be promoted to carry a `Type`.

# Plan: parse-path-types-with-generic-args

## Goal
Replace the stopgap `parse_type` with a real path-type parser in a new `parser/ty.rs` module that recognizes multi-segment paths and `<...>` generic argument lists, including nested generics whose closing `>>` is lexed as `Shr`.

## Steps
1. Promote `GenericArg` in `src/ast/expr.rs` from `Placeholder` to a real variant carrying a `Type` (e.g. `pub enum GenericArg { Type(Type) }`). Drop the stale "TODO: replaced/merged…" comment. Add a `pub use` re-export through `ast/mod.rs` if useful. Re-route the `use crate::ast::expr::GenericArg` reference in `ast/generics.rs` so the import keeps working (no structural change there — `TraitBound.generic_args: Vec<GenericArg>` stays, just stays empty for now).
2. Create `src/parser/ty.rs` with `impl Parser { pub fn parse_type(...) -> Result<Type, CompileError> { ... } }`. Behavior:
   - Parse one or more `Ident` segments separated by `::`; record the first segment span as the start span and the closing token span as the end span.
   - On any segment, if the next token is `Lt`, parse a comma-separated list of generic arguments by calling `parse_type` recursively for each; require at least one argument; allow an optional trailing comma; close with `>` (see step 3 for `>>`).
   - Build `Type::Path(Path { id, span, segments })` where each segment populates `generic_args: Vec<GenericArg>` (empty when no `<...>`).
3. Add a private `fn eat_gt_or_split_shr(&mut self) -> Option<Span>` helper in `parser/ty.rs`:
   - If `peek()` is `Gt`, bump and return its span.
   - If `peek()` is `Shr` (the `>>` produced by the lexer when nested generics close), mutate `self.tokens[self.pos].kind` in place to `Gt` and shrink its span to the second `>` half (start = original_start + 1, end = original_end), then return the span of the first `>` (start = original_start, end = original_start + 1). Do **not** advance `self.pos` so the second `Gt` is consumed by the outer level. This is the standard rustc trick.
   - Use this helper in step 2's closer.
4. Add a `mod tests` block in `parser/ty.rs` with the required `path_with_generics` test:
   - Build tokens for `Vec<T>` and assert `Type::Path` with one segment `Vec`, one generic arg that is `GenericArg::Type(Type::Path)` with segment `T`.
   - Build tokens for `HashMap<String, Vec<i32>>` (closing `>>` lexed as `Shr`) and assert two outer args (`String`, then nested `Vec<i32>`), nested arg is one segment `Vec` with one inner arg `i32`.
   - Optionally include a `std::vec::Vec<i32>` multi-segment case.
   - Each test asserts `p.errors.is_empty()` and `p.peek() == Eof`.
5. Wire it up in `src/parser/mod.rs` by adding `pub mod ty;` next to `pub mod expr; pub mod item; pub mod stmt;`.
6. Delete the stopgap `fn parse_type` (lines ~16–32) and the helper `fn parse_self_explicit_type` (lines ~38–66) in `src/parser/item.rs`. Update `try_parse_self_param` / `parse_self_param_value` callers (the only `parse_self_explicit_type` user) to invoke the new `self.parse_type()`. The new path parser handles `Box<Self>`/`Rc<Self>` natively because `Self` should be accepted as a path head — extend the head-token check in step 2 to accept `Ident` **or** `SelfUpper` (treat `Self` as the segment ident `"Self"`).
7. Keep the `type_span` helper in `item.rs` (or inline it) — the new parser still produces `Type::Path` with a known span, so existing call sites in `parse_where_clause` continue to work.
8. Update the existing `self_params` test assertions in `item.rs` if they break (e.g. the one currently checking `generic_args.len() == 1` against the old `Placeholder` should still pass since `GenericArg::Type(...)` still has length 1; no semantic change to count assertions).
9. Remove the now-stale `// Note: nested generics like Vec<Vec<T>> would lex >> as Shr…` comment on `parse_generics_params` (line ~188) since that limitation is lifted at the type level (the comment specifically calls out the type-side; `parse_generics_params` itself parses *params*, not args, and is unaffected).

## Files
- `src/ast/expr.rs` — replace `GenericArg::Placeholder` with `GenericArg::Type(Type)`; remove stale TODO comment; add `use crate::ast::ty::Type;` if not already imported (watch for cycle: `ty.rs` already imports `expr::Path`, so the cycle is already there and benign).
- `src/parser/mod.rs` — add `pub mod ty;`.
- `src/parser/ty.rs` — **new file**: `impl Parser { pub fn parse_type, fn parse_path_type, fn eat_gt_or_split_shr }` plus `#[cfg(test)] mod tests` containing `path_with_generics`.
- `src/parser/item.rs` — delete stopgap `parse_type` and `parse_self_explicit_type`; redirect their lone caller (`parse_self_param_value`) to `self.parse_type()`; drop the `parse-path-types-with-generic-args` TODO comment; update import line for `GenericArg` if needed.

## Risks
- **`Shr` splitting with span fidelity.** Mutating a token in place is unusual; if other code re-reads the same position after the split, the original `Shr` info is gone. Mitigation: only mutate after we have committed to consuming the closing `>`, and confine the mutation to the type parser. The half-span produced for the consumed `>` can be slightly off (we don't know byte-level lexer width), but at the AST level only the merged span is observable; tests only check structure not exact span bytes.
- **Cycle in AST imports.** `ast::ty::Type` already imports `ast::expr::Path`, and now `ast::expr::GenericArg` will import `ast::ty::Type`. Rust allows cycles inside one crate but it's worth ensuring both modules use full paths consistently.
- **Greedy multi-segment parsing in non-type contexts.** `parse_type` is also called by `parse_where_clause` with paths followed by `:`; multi-segment `::` parsing must stop at non-`::` tokens, which the design already does. No change in behavior expected for existing where-clause tests.
- **`SelfUpper` in head position** must be accepted to keep `Box<Self>` / `Rc<Self>` self-param tests green; if missed, those tests fail.
- **Empty generic arg list `<>`.** The spec is fuzzy; choosing to require ≥ 1 arg matches Rust. If a later item needs `<>`, this can be relaxed.
- **Existing call sites that still pass `GenericArg::Placeholder`** (none after this change is applied — a grep confirms only `parse_self_explicit_type` produced it, and that function is being deleted).

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::ty::tests::path_with_generics
cargo build --manifest-path vertex_stage0/Cargo.toml
cargo test --lib --manifest-path vertex_stage0/Cargo.toml
test -f vertex_stage0/src/parser/ty.rs
```

## Assumptions
- The crate manifest lives at `vertex_stage0/Cargo.toml` (verified by `ls`); `cargo test --lib` from the workspace root would not target it, so `--manifest-path` is required.
- The test name `path_with_generics` is a single `#[test] fn` covering both `Vec<T>` and `HashMap<String, Vec<i32>>` (the bundle phrases them as one verify line).
- `GenericArg` should hold a `Type` directly (not boxed) — `Type` is moderately sized but already used unboxed in `Vec<Type>` for tuples and where-pred fields, so an unboxed variant is consistent.
- `Self` in type position is parsed as a `Path` segment with the literal ident `"Self"` (matches what `synth_self_path_type` and existing `assert_self_path` already encode).
- The `>>` splitting trick mutates the token stream in place; this is acceptable because the parser owns `tokens: Vec<Token>` and no other code depends on the original `Shr` token after consumption.
- `parse_type` is the single public entrypoint for type parsing; subsequent items (`parse-reference-types`, `parse-tuple-types`, etc.) will extend it by branching on the leading token. That structural choice is implicit in this todo's slug name.
- Do not update `MethodCall.generic_args` or `TraitBound.generic_args` populators — those are still empty `Vec::new()` at all call sites and remain so until their respective parser items run.
- The full `cargo build` / `cargo test --lib` runs are part of verify because changing `GenericArg` ripples into all crates that match on it; running the whole test suite catches any miss.

## Blockers
Blockers: none

## Summary
Adds a real path-type parser in `src/parser/ty.rs` that handles multi-segment paths and (possibly nested) generic argument lists, retiring the stopgap `parse_type` in `item.rs` and promoting `GenericArg` to carry an actual `Type`.

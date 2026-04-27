# Plan: parse-function-types

## Goal
Extend `parse_type` in `vertex_stage0/src/parser/ty.rs` to recognize bare `fn(T, U) -> V` and `extern "C" fn(...)` function types, producing `Type::Fn` with parsed parameter and return types.

## Steps
1. In `vertex_stage0/src/parser/ty.rs`, add a top-of-`parse_type` branch that fires when the next token is `TokenKind::Fn`, or `TokenKind::Extern` (with `Fn` directly after, or after a `StringLiteral` ABI). Dispatch to a new private `parse_fn_type` helper.
2. Implement `parse_fn_type`: if the current token is `Extern`, bump it; if a `StringLiteral` ABI follows, bump it (the ABI value is parsed but, given `Type::Fn` carries no `abi` field today, recorded in a local and discarded so we exercise the syntax). Then `expect(&TokenKind::Fn)`, `expect(&TokenKind::LParen)`, parse a comma-separated list of `parse_type()` until `RParen`, allowing a trailing comma. Then if the next token is `TokenKind::Arrow`, bump it and call `parse_type()` for the return; otherwise default the return to `Type::Tuple(Vec::new())` (the unit type) per spec convention.
3. Construct and return `Type::Fn { params, ret: Box::new(ret) }`.
4. Extend the file-local `type_span` helper with a `Type::Fn { params, ret, .. } => ...` arm so any callers that wrap a function type (e.g. `&fn(i32) -> i32`) keep working: use `type_span(ret)` if params/ret available, falling back to `Span::new(FileId(0), 0, 0)` if there are no params and ret is unit-tuple.
5. Add a `#[test] fn fn_types()` to the existing `tests` module covering: (a) `fn() -> i32` (zero params, explicit return), (b) `fn(i32, u8) -> bool` (multi-param), (c) `fn(i32)` (no `-> T`, return is `Type::Tuple(vec![])`), (d) `extern "C" fn(i32) -> i32` (verifying the `extern "C"` form parses and yields a `Type::Fn`). Each case asserts `parse_type` returns a `Type::Fn` with the expected `params.len()`, the expected leaf path idents, the expected return shape, no accumulated errors, and that `peek()` is at `Eof`.

## Files
- `vertex_stage0/src/parser/ty.rs` -- new dispatch arm in `parse_type`, new `parse_fn_type` helper, `Type::Fn` arm in `type_span`, new `fn_types` unit test.

## Risks
- `Type::Fn` in `vertex_stage0/src/ast/ty.rs` does not currently carry an `abi` field, so `extern "C"` is parsed-and-discarded; later items that need ABI distinction will have to extend the AST and any prior tests. Documented as an assumption.
- The ABI string is delivered through `TokenKind::StringLiteral(String)`; if downstream items decide ABI strings should be validated against an allow-list, that validation must be added later. We accept any string here.
- Trailing-comma handling in the parameter list mirrors `parse_tuple_or_grouped_type`; an empty parameter list `fn()` must parse cleanly.
- Adding the `Type::Fn` arm to `type_span` is required; otherwise `&fn(...)` would hit the `unreachable!` and panic.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::ty::tests::fn_types
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- This item should not modify `Type::Fn` in `src/ast/ty.rs`; adding an `abi` field is the responsibility of a later item. The parsed ABI string is therefore intentionally discarded (the syntax is exercised; the value is dropped). Tests assert only the `Type::Fn` shape and parameter/return contents.
- A function type with no `-> T` (e.g. `fn(i32)`) returns `Type::Tuple(vec![])` (unit) as the implicit return, matching how Rust-style function signatures default. The test pins this.
- `extern` without a following string literal (e.g. `extern fn(...)`) is permitted and treated as default ABI; we will not add a separate test for that form to keep the test name `fn_types` focused on the spec sub-bullets, but the parser will not reject it.
- Allowing a trailing comma in the parameter list is acceptable and matches the tuple-type behavior already in this file.
- `parse_fn_type` is added to the same `impl Parser` block in `ty.rs`; it does not need to be `pub`.
- The `Fn` and `Extern` keywords are real `TokenKind` variants (confirmed in `lexer/token.rs`), so the parser can match them with `matches!(self.peek(), TokenKind::Fn)` etc.
- The new dispatch arm is placed before the path-type stopgap fallback so that `fn` (which is also a valid identifier shape only in legacy contexts) is captured as a function type, not an identifier.
- `type_span` for `Type::Fn` will return the return type's span; this is a reasonable approximation until a later item threads an explicit `span` field through `Type::Fn`.

## Blockers
Blockers: none

## Summary
Adds `fn(...) -> T` and `extern "C" fn(...) -> T` parsing in `parse_type`, returning `Type::Fn`, and locks the form in with one `parser::ty::tests::fn_types` unit test.

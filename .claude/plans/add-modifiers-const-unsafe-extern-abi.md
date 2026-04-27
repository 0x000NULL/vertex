Now I have enough context. Let me draft the plan.

# Plan: add-modifiers-const-unsafe-extern-abi

## Goal
Extend `Parser::parse_fn` (in `vertex_stage0/src/parser/item.rs`) to recognize a leading set of function modifiers (`const`, `unsafe`, `extern "ABI"`) — singly and in valid combinations — and record them on `FnDef`, with a `parser::item::tests::fn_modifiers` test pinning the behavior.

## Steps
1. Extend `FnDef` in `vertex_stage0/src/ast/item.rs` with three optional/boolean fields: `is_const: bool`, `is_unsafe: bool`, and `extern_abi: Option<String>` (None = not extern; `Some("C")` for `extern "C"`; `Some("Rust")` if no string literal followed `extern` to mirror Rust's implicit-Rust default — or alternatively use `Some(None)` with a richer enum; pick the simpler `Option<String>` shape and write None=no-extern, Some(s)=extern with named ABI, with `s == ""` reserved for bare `extern`).
2. Update existing `FnDef` constructors / matches accordingly. The only known constructor is in `parse_fn`; the only known consumer pattern is the test helper `as_fn` in `parser::item::tests`. The struct is `#[derive(Debug, Clone)]` with no other readers, so adding fields is local.
3. In `Parser::parse_fn`, before `expect(&TokenKind::Fn)`, parse a leading modifier sequence in any order but each at most once:
   - `const` → set `is_const = true`
   - `unsafe` → set `is_unsafe = true`
   - `extern` → set `extern_abi`. If the next token is `TokenKind::StringLiteral(s)`, consume it and store `Some(s)`. Otherwise store `Some(String::new())` to denote bare `extern` (implicit `"Rust"`).
   Use the modifier span (or the first modifier's span) instead of `fn_kw.span` as the starting point for `FnDef.span`.
4. Detect duplicates: if the same modifier appears twice, emit a syntax error via `expected_token_error` style (or a focused `CompileError` with `ErrorCode::E0100`) and continue. Keep the first seen value.
5. Stop the modifier loop the first time `peek()` is not `Const`/`Unsafe`/`Extern`; then proceed with the existing `expect(&TokenKind::Fn)` flow unchanged.
6. Update existing `plain_fn` test if needed (only if signature changes break it; with default field values it should still pass).
7. Add a new `fn_modifiers` test under `parser::item::tests` covering each single modifier and the canonical combination `const unsafe extern "C" fn …`. Each case asserts the corresponding flag/abi on `FnDef` and that the parser consumed the body successfully with no accumulated errors.

## Files
- `vertex_stage0/src/ast/item.rs` -- add `is_const: bool`, `is_unsafe: bool`, `extern_abi: Option<String>` to `FnDef`.
- `vertex_stage0/src/parser/item.rs` -- parse leading modifier sequence in `parse_fn`; populate the new fields; widen `FnDef.span` to start at the first modifier; add the `fn_modifiers` test.

## Risks
- Any other code that constructs a `FnDef` literally would fail to compile after the field addition; given the codebase is at the very early parser stage and a grep should confirm `FnDef { … }` is only constructed in `parse_fn`, this is low-risk.
- `extern` without a following string literal is ambiguous with `extern { … }` blocks; this plan only addresses the `extern "ABI" fn` form (since `parse_fn` is invoked only when an `fn` item is expected). We do NOT attempt to disambiguate item-level `extern { … }` here.
- If `TokenKind::StringLiteral` interns the inner string differently than expected (e.g., escapes), the ABI string may not match the user's literal byte-for-byte; for now we trust the lexer's already-decoded value.
- The duplicate-modifier branch deliberately emits an error but does not reorder/restart; downstream `Item::Fn` would still be produced. This matches the recovery philosophy established by prior tasks.

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::item::tests::fn_modifiers
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::item::tests::plain_fn
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The crate root is `vertex_stage0/` (confirmed by file tree); cargo invocations need `--manifest-path vertex_stage0/Cargo.toml`. The verify command in the todo (`cargo test --lib parser::item::tests::fn_modifiers`) is specified relative to the crate, so the runner should be invoked from `vertex_stage0/`; explicit `--manifest-path` makes the verify robust regardless of cwd.
- `TokenKind::StringLiteral(String)` already exists in the lexer (confirmed at `vertex_stage0/src/lexer/token.rs:68`) and the lexer surfaces it for `"C"`.
- The simplest data shape for the ABI is `Option<String>`: `None` = no `extern`, `Some("C")` = `extern "C"`, `Some(String::new())` = bare `extern fn`. A future plan can lift this to a stronger `Abi` enum once more ABIs are supported; preserving `Option<String>` keeps churn minimal here.
- Modifier order is permissive (any order, each at most once). We do not enforce a canonical order; that's the typechecker's or a later linter's job. Duplicates produce an `E0100` syntax error but recovery continues.
- The existing `plain_fn` test should remain passing because the new fields have natural default values (`false`, `false`, `None`); we only need to populate them in `parse_fn`.
- We do NOT add `async` here; the lexer does not yet have an `Async` token, and the spec for this todo names only `const`, `unsafe`, and `extern "ABI"`.
- The `FnDef.span` should cover the modifier prefix; if modifiers are present, use the earliest modifier's span as the start, otherwise `fn_kw_span`.

## Blockers
Blockers: none

## Summary
Adds parser support for `const`/`unsafe`/`extern "ABI"` function modifiers (any combination, each at most once), records them on `FnDef`, and pins the behavior with a `fn_modifiers` unit test.

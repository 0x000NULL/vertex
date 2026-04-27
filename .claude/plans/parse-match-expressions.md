# Plan: parse-match-expressions

## Goal
Parse `match scrut { pat if guard => expr, ... }` into `Expr::Match`, wire it into the primary-expression head dispatch, and add a `match_basic` unit test alongside the existing `if`/`loop`/`while`/`for` tests.

## Steps
1. In `src/parser/expr.rs`, extend the existing `use crate::ast::expr::{...}` import to include `Match` and `MatchArm` (alongside the existing `Pat`, which we'll keep using as a placeholder for the pattern slot).
2. Add a private `parse_match` method on `Parser` modeled on the existing `parse_if` / `parse_for` shape:
   - `expect(&TokenKind::Match)` to capture the start span.
   - Parse the scrutinee with `self.parse_expr()` (matches the `parse_while` / `parse_for` convention; reuse the same TODO comment about a future "no-struct-literal" context now that struct-literal heads aren't parsed yet).
   - `expect(&TokenKind::LBrace)` to open the arm list.
   - Loop while `peek()` is neither `RBrace` nor `Eof`, calling a small `parse_match_arm` helper for each arm.
   - After each arm, eat an optional `Comma`. If the next token isn't `Comma` or `RBrace`, fall through to `expect(&TokenKind::RBrace)` so the user gets the standard "expected `,` or `}`" recovery via the existing error machinery.
   - `expect(&TokenKind::RBrace)` to close, merge spans, mint a NodeId, return `Expr::Match(Match { id, span, scrutinee: Box::new(...), arms })`.
3. Add a private `parse_match_arm` helper:
   - Capture the start span via the same `if self.pos < self.tokens.len()` idiom used in `parse_closure` / `parse_unary` / `parse_binary`.
   - Pattern slot stub: accept one of `Ident(_)`, `Underscore`, `IntLiteral`, `FloatLiteral`, `CharLiteral`, `StringLiteral`/`RawStringLiteral`, `True`, `False` by consuming a single token (mirrors the single-bare-ident stub in `parse_for` and `parse_closure_param`). Store `Pat::Placeholder`. Anything else returns `unexpected_token_error("pattern")`. Add a TODO comment that this is replaced when the real pattern parser lands (see `parse-literal-patterns` / `parse-ident-patterns-mut-sub-binding` / `parse-or-patterns-and-wildcard`).
   - Optional guard: if `peek()` is `If`, bump it and parse the guard expression with `self.parse_expr()`, store it in `Option<Box<Expr>>`.
   - `expect(&TokenKind::FatArrow)` for `=>`.
   - Parse body with `self.parse_expr()`.
   - Merge spans, mint a NodeId, return `MatchArm { id, span, pattern: Pat::Placeholder, guard, body: Box::new(body) }`.
4. Wire `match` into the primary-expression head: in `parse_primary_for_paren`, add a `TokenKind::Match => self.parse_match(),` arm next to the other keyword heads. Update the head-set TODO comment above `range_rhs_starts_here` (it already lists "Match"), but add `TokenKind::Match` to the `range_rhs_starts_here` allow-set so `a..match s { _ => 1 }` parses sensibly; mirrors how `If` is included today.
5. In the existing `#[cfg(test)] mod tests` block, add a `#[test] fn match_basic()` next to `loop_while_for` / `if_else_chain`. Cover (using the existing `tok` / `int_tok` / `int_value` helpers and `Pat`):
   - `match 1i32 { 1i32 => 2i32, _ => 3i32 }` → asserts `Expr::Match` with two arms, scrutinee is `IntLit(1)`, arm 0 has `Pat::Placeholder` + `guard.is_none()` + body `IntLit(2)`, arm 1 body `IntLit(3)`.
   - `match 1i32 { x if true => 2i32 }` → arm has `guard.is_some()` whose body is `BoolLit(true)`, body `IntLit(2)`. Confirms `if guard` is consumed before `=>`, not as a nested `if`-expression.
   - Trailing comma allowed: same as case 1 with a trailing comma after the second arm.
   - Error: missing `=>` (e.g. `match 1i32 { _ 2i32 }`) → `Err` with `E0100`.
   - Error: unexpected `}` mid-arm (e.g. `match 1i32 { _ => }`) → `Err` (the body `parse_expr` fails).

## Files
- `vertex_stage0/src/parser/expr.rs` -- add `parse_match` + `parse_match_arm`, dispatch arm in `parse_primary_for_paren`, extend `range_rhs_starts_here`, expand the `use` import to include `Match`/`MatchArm`, and add the `match_basic` test.

## Risks
- `parse_expr()` for the scrutinee may try to consume a `{` as a block expression head once struct-literal heads land. Mitigated by leaving the same TODO comment used in `parse_while`/`parse_for`; today `LBrace` heading a primary parses as a block, which doesn't apply because `match` already requires `{` before arms — but the scrutinee comes *before* that brace, so `match s { ... }` works as long as the scrutinee itself doesn't start with `{`. Documented as a known limitation.
- Pattern stub accepts a single token; or-patterns (`A | B => ...`) and tuple-struct patterns (`Some(x) => ...`) are out of scope. Tests must avoid those forms; future pattern items will replace the stub.
- The guard parses with `parse_expr()`, which can recursively parse a nested `if`. That's fine — Rust's grammar likewise allows `if guard_expr` where the guard is any expression — but we should not let the optional `if` keyword be confused with a fresh `if`-expression head. Because `parse_match_arm` greedily checks `If` *before* calling `parse_expr` for the body, the guard `if` is bound here, not at the body slot.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml parser::expr::tests::match_basic
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- The Cargo manifest lives at `vertex_stage0/Cargo.toml` (confirmed by file layout). The verify command therefore needs `--manifest-path vertex_stage0/Cargo.toml`; the spec-style `cargo test --lib parser::expr::tests::match_basic` from the todo description is shorthand and would only work if run inside `vertex_stage0/`. I include the manifest-path form so the runner's `bash -c` works from the repo root, which matches how earlier items have been set up (the test path itself is unchanged).
- The pattern slot uses the legacy `Pat::Placeholder` enum that already lives in `src/ast/expr.rs` and is already used by `parse_for` and `parse_closure_param`. The richer `Pattern` enum in `src/ast/pat.rs` is not wired into `MatchArm.pattern` yet and that re-wiring is owned by separate items (`parse-literal-patterns` etc.), not this one.
- The `match` keyword token (`TokenKind::Match`) and `=>` token (`TokenKind::FatArrow`) exist in the lexer (confirmed via the `describe` helper in `src/parser/mod.rs`).
- Trailing comma after the last arm is accepted; comma between arms is required only when the next token is not `}`. This matches Rust precedent and the way `parse_paren_or_tuple` and `parse_array_literal` handle their separators today.
- `match` becoming a valid range RHS head (`a..match ...`) is desirable and consistent with how `If` is handled in `range_rhs_starts_here`. If not, omit that single-line addition — it's not required for `match_basic` to pass.
- `Pat`, `Match`, `MatchArm` are all re-exported from `crate::ast::expr` (confirmed by reading `src/ast/expr.rs`).
- No new tokens, AST nodes, or error codes are introduced; everything reuses `E0100` via `unexpected_token_error` / `expect`.
- The `match_basic` test does not need to assert spans, only structural shape (matches the depth of assertions in `loop_while_for` and `if_else_chain`).

## Blockers
Blockers: none

## Summary
Adds `parse_match` + `parse_match_arm` (with a single-token pattern stub matching the existing `for`/closure stub strategy), dispatches `TokenKind::Match` from the primary-expression head, and locks in shape with a `match_basic` unit test covering simple arms, guards, trailing commas, and two error cases.

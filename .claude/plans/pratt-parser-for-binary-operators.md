# Plan: pratt-parser-for-binary-operators

## Goal
Add a Pratt-style infix driver (`parse_expr` + `parse_binary`) on top of the existing `parse_unary` primary so binary operators `*  / %  +  -  << >>  &  ^  |  comparisons  and  or  =/+=/-=/*=//=/%=` parse with the spec precedence table, and reject chained comparisons like `a < b < c`.

## Steps
1. In `vertex_stage0/src/ast/expr.rs` — confirm `BinaryOp` already has every needed variant (Add/Sub/Mul/Div/Rem, Shl/Shr, BitAnd/BitOr/BitXor, Eq/Ne/Lt/Gt/Le/Ge, And, Or, Assign/AddAssign/SubAssign/MulAssign/DivAssign/RemAssign — all present, no AST change needed).
2. In `vertex_stage0/src/parser/expr.rs`, add private `infix_binding_power(kind: &TokenKind) -> Option<(u8, u8, BinaryOp, OpClass)>` returning `(left_bp, right_bp, op, class)` where `OpClass` is a small private enum `{ Comparison, Assignment, Other }`. Precedence rungs (low→high) using even/odd left/right BP for left/right associativity:
   - Assignment `= += -= *= /= %=` — right-assoc, lowest (e.g. left=2, right=1), `class = Assignment`
   - `or` — left-assoc (left=3, right=4)
   - `and` — left-assoc (left=5, right=6)
   - Comparisons `== != < > <= >=` — non-associative, `class = Comparison`, identical left/right BP (e.g. left=7, right=8) — non-associativity enforced separately, not via BP
   - `|` (Pipe) — left (left=9, right=10)
   - `^` (Caret) — left (left=11, right=12)
   - `&` (Amp) — left (left=13, right=14)
   - `<<`/`>>` (Shl/Shr) — left (left=15, right=16)
   - `+`/`-` (Plus/Minus) — left (left=17, right=18)
   - `*`/`/`/`%` (Star/Slash/Percent) — left (left=19, right=20)
3. Add `pub fn parse_expr(&mut self) -> Result<Expr, CompileError>` that calls `parse_binary(0)`.
4. Add `fn parse_binary(&mut self, min_bp: u8) -> Result<Expr, CompileError>`:
   - Parse `lhs = self.parse_unary()?` (existing — handles unary prefix and falls through to literal/paren primary).
   - Loop:
     - Peek next; if `infix_binding_power(peek)` is `None` or `left_bp < min_bp`, break.
     - If `class == Comparison` AND `lhs` is already `Expr::Binary(b)` with `b.op` in the comparison set → emit `CompileError::new(ErrorCode::E0100, ErrorKind::Syntax, op_span, "chained comparison operators require parentheses")` and return `Err`. (Per resolved Q: `(1 < 2) < 3` is accepted because the inner `(1 < 2)` is a wrapped expression, not a `Binary` directly — `parse_paren_or_tuple` already unwraps `(expr)` to the inner `Expr`, so the outer `<` sees `Expr::Binary` and rejects only the unparenthesized chain. Document this in a code comment.)
     - Bump the operator token, recurse into `parse_binary(right_bp)` for the rhs, build `Expr::Binary { op, lhs, rhs, span = lhs.span().merge(&rhs.span()) }`, set as new `lhs`.
   - Return `lhs`.
5. Wire BinaryOp mapping for each token:
   `Plus→Add, Minus→Sub, Star→Mul, Slash→Div, Percent→Rem, EqEq→Eq, BangEq→Ne, Lt→Lt, Gt→Gt, Le→Le, Ge→Ge, And→And, Or→Or, Amp→BitAnd, Pipe→BitOr, Caret→BitXor, Shl→Shl, Shr→Shr, Eq→Assign, PlusEq→AddAssign, MinusEq→SubAssign, StarEq→MulAssign, SlashEq→DivAssign, PercentEq→RemAssign`.
6. Add `#[test] fn operator_precedence()` exercising at minimum:
   - `1 + 2 * 3` → `Add(1, Mul(2,3))`
   - `1 * 2 + 3` → `Add(Mul(1,2), 3)`
   - `1 - 2 - 3` → `Sub(Sub(1,2), 3)` (left-assoc)
   - `a = b = c` → `Assign(a, Assign(b, c))` (right-assoc) — use `IntLiteral` placeholders since path parsing isn't done yet
   - `1 | 2 & 3` → `BitOr(1, BitAnd(2,3))`
   - `1 == 2 and 3 == 4` → `And(Eq(1,2), Eq(3,4))`
   - `1 and 2 or 3` → `Or(And(1,2), 3)`
   - `1 << 2 + 3` → `Shl(1, Add(2,3))`? — verify: `+` (17) > `<<` (15), so `1 << (2 + 3)` ✓
7. Add `#[test] fn comparison_non_associative_rejected()` confirming `1 < 2 < 3` returns `Err` with `ErrorCode::E0100`, and that `(1 < 2) < 3` (using already-implemented `parse_paren_or_tuple` for the inner) parses successfully.
8. Run `cargo fmt --all` and `cargo clippy --lib --all-targets -- -D warnings` clean.

## Files
- `vertex_stage0/src/parser/expr.rs` — add `parse_expr`, `parse_binary`, `infix_binding_power`, private `OpClass` enum, two `#[cfg(test)]` tests `operator_precedence` and `comparison_non_associative_rejected`.

## Risks
- Path expressions aren't parsed yet, so tests can't use `a < b`; must use integer literals as operand placeholders. Acceptable — `parse_unary` already handles literal heads.
- The existing `parse_unary` consumes `*` and `&` as prefix — fine, since `parse_binary` only inspects infix tokens AFTER a primary is parsed; the primary call to `parse_unary` will eat any leading `*`/`&` correctly, and infix `*`/`&` after a primary disambiguates as multiplication / bitwise-AND.
- Ambiguity at the `(expr) < x` boundary: `parse_paren_or_tuple` returns the inner unwrapped `Expr`, which means the chained-comparison check (which inspects `Expr::Binary`) still fires for `(a < b) < c`. The resolved Q accepts this — flag in a code comment as a known TODO if real code trips it.
- Span construction relies on `Expr::span()` (already implemented) and `Span::merge` (used elsewhere — assumed present).
- Rejection error uses generic `E0100` per the resolved Q (no dedicated code).

## Prereqs
Prereqs: none

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::expr::tests::operator_precedence
cargo test --manifest-path vertex_stage0/Cargo.toml --lib parser::expr::tests::comparison_non_associative_rejected
cargo clippy --manifest-path vertex_stage0/Cargo.toml --lib --all-targets -- -D warnings
```

## Assumptions
- The crate is `vertex_stage0` (Cargo manifest at `vertex_stage0/Cargo.toml`); tests are scoped via `--manifest-path`. The existing `run.log` and prior commits all touch `vertex_stage0/src/...`.
- Tests are written using `IntLiteral` operands (e.g. `1 + 2 * 3`) since `parse_path` isn't in this item; this satisfies the verify lines without depending on later items.
- `Span::merge(&Span)` is the existing combiner used by `parse_unary` (line 170 of `parser/expr.rs`); reuse it for binary spans.
- The chained-comparison check looks at the *immediate* `lhs` being `Expr::Binary` with a comparison `op`; parenthesized comparisons are unwrapped by `parse_paren_or_tuple` into `Expr::Binary`, so `(a < b) < c` will also be rejected by the strict check. Per resolved Q the default is to accept the strict behavior here — documented as a TODO comment for follow-up if real code hits it.
- `=`, `+=`, `-=`, `*=`, `/=`, `%=` are right-associative and looser than `or` per the resolved Q; they participate in the same infix loop as the rest.
- No new error variant is added — generic `ErrorCode::E0100` reused, message `"chained comparison operators require parentheses"`. (Alternative: emit two errors — accept reject as Err and let caller recover. Prefer single Err for simplicity since `parse_binary` is fail-fast like `parse_unary`.)
- `OpClass` is a private enum local to `parser/expr.rs`, not exported.
- Numeric BP values are an internal contract — even/odd pairing encodes left vs right associativity for any future reviewer.

## Blockers
Blockers: none

## Summary
Implements the spec's binary-operator precedence as a Pratt loop wrapping the existing unary parser, with non-associative comparisons enforced via an `lhs`-shape check, validated by the two required unit tests.

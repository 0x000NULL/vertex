# Plan: parse-function-call-method-call-field-access

## Goal
Add postfix parsing for `f(args)`, `x.method(args)`, `x.field`, and `x.0` so that postfix call/field/method-call/tuple-field bind tighter than unary, layered on top of the existing literal-only primary stub.

## Steps
1. In `vertex_stage0/src/parser/expr.rs`, add a new private `parse_postfix(&mut self) -> Result<Expr, CompileError>` method on `Parser`. It begins with `let mut expr = self.parse_primary_for_paren()?;` (same head set as today — literals only, intentional per the resolved blocker), then runs a `loop { match self.peek() { ... } }` that returns `expr` once no more postfix tokens are available.
2. In the loop, on `TokenKind::LParen` build a `Call`:
   - Bump the `(`.
   - Parse zero-or-more arguments via `self.parse_expr()`, each followed by an optional `,`. Allow an immediate `)` (empty args) and tolerate a trailing comma (loop guard `while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof)` mirrors `parse_paren_or_tuple`).
   - `expect(&TokenKind::RParen)`. Span = `callee.span().merge(&rparen.span)`. Wrap into `Expr::Call(Call { id, span, callee: Box::new(expr), args })`. Re-assign to `expr` and continue the loop.
3. In the loop, on `TokenKind::Dot` build a Field/MethodCall/TupleField:
   - Bump the `.`.
   - If peek is `TokenKind::Ident(_)`: bump it, extract the name. If the *new* peek is `TokenKind::LParen`, reuse the same arg-parsing block from step 2 to produce `Expr::MethodCall { receiver: Box::new(expr), method: name, args, generic_args: vec![] }` (turbofish intentionally deferred per resolved blocker — leave `generic_args` empty). Otherwise produce `Expr::Field(FieldAccess { receiver: Box::new(expr), name })` with span `receiver.span().merge(&ident_tok.span)`.
   - Else if peek is `TokenKind::IntLiteral(v, _)`: bump it. If `v > u32::MAX as u64`, return `Err(CompileError::new(ErrorCode::E0100, ErrorKind::Syntax, intlit_tok.span, "tuple field index exceeds u32::MAX"))` (per resolved blocker — error, not silent truncation). Otherwise produce `Expr::TupleField(TupleFieldAccess { receiver: Box::new(expr), idx: v as u32 })` with span `receiver.span().merge(&intlit_tok.span)`.
   - Else: return `Err(self.unexpected_token_error("identifier or integer literal"))`.
4. Modify `parse_unary` so its fall-through arm (currently `_ => return self.parse_primary_for_paren(),` at line 229) becomes `_ => return self.parse_postfix(),`. This locates postfix tighter than unary, matching spec § "Operator Precedence" (`. ()` is rank 1, unary is rank 2). Recursive operands of `Unary` therefore also receive postfix wrapping (`-x.field` → `Neg(Field(x, "field"))`). Do **not** touch `parse_paren_or_tuple`'s use of `parse_primary_for_paren` — leave the two stubs separate per the resolved blocker.
5. Add `#[test] fn call_method_field()` to the existing `tests` mod. Reuse the existing `tok` / `int_tok` helpers and exercise the four required forms via `parse_unary` (since that is the entry point that now wires `parse_postfix`):
   - **Call empty args** `42()` → `Expr::Call { callee: IntLit(42), args: [] }`.
   - **Call w/ args + trailing comma** `42(1, 2,)` → `Expr::Call` with two `IntLit` args.
   - **Field access** `42 . foo` → `Expr::Field { receiver: IntLit(42), name: "foo" }`.
   - **Method call** `42 . foo (7)` → `Expr::MethodCall { receiver: IntLit(42), method: "foo", args: [IntLit(7)], generic_args: [] }`.
   - **Tuple field** manually composed `IntLit(42), Dot, IntLit(0, Unsuffixed), Eof` → `Expr::TupleField { receiver: IntLit(42), idx: 0 }`. (Hand-built tokens sidestep the lexer's `1.0`-as-FloatLiteral issue.)
   - **Tuple field u32 overflow** `IntLit(42), Dot, IntLit(u64::MAX, Unsuffixed)` → `Err(E0100)`.
   - **Dot + bad RHS** `IntLit(42), Dot, Plus` → `Err(E0100)`.
   - **Chain** `IntLit(42), Dot, Ident("foo"), LParen, RParen, Dot, Ident("bar"), Dot, IntLit(0, Unsuffixed)` → outer `TupleField` whose receiver is `Field("bar")` whose receiver is `MethodCall("foo")` on `IntLit(42)`. Confirms the loop accumulates correctly.
   - In each `Ok` case assert `p.errors.is_empty()` and assert `p.pos` advanced past every input token (consistent with sibling tests).

## Files
- `vertex_stage0/src/parser/expr.rs` — add `parse_postfix`, swap `parse_unary`'s fall-through to call it, and add the `call_method_field` test in `mod tests`. No other module is touched: `Call`, `MethodCall`, `FieldAccess`, `TupleFieldAccess`, and their `Expr` variants/`span()`/`id()` arms already exist in `src/ast/expr.rs`.

## Risks
- **Recursive `parse_expr` inside args** can blow up stacks on deeply nested calls; this is consistent with the existing Pratt driver, so accept the same risk envelope.
- **Span on chained postfix**: each step extends span via merge; if one segment's span is `0..0` (synthetic test tokens), the merged span will be similarly degenerate. Tests don't assert on offsets, only structure, so this is fine.
- **Span captured before `bump`**: must read `self.tokens[self.pos].span` *before* calling `self.bump()` for the closing paren / ident / int-literal so we don't span-merge past the consumed token. Mirrors the pattern already used in `parse_unary`.
- **`parse_primary_for_paren` still literal-only**: if someone wires `Ident` heads into a different primary later, the loop's behavior must remain agnostic to head type — it only inspects what comes *after* an expression. This is already the case.
- **Trailing comma after no args**: `()` is empty, `(,)` should not be tolerated. The loop guard handles this — on RParen we never enter the loop body, so the comma branch is unreachable for the empty case.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib parser::expr::tests::call_method_field
cargo check --all-targets
```

## Assumptions
- `MethodCall.generic_args` is left as `vec![]` here; turbofish (`x.method::<T>(args)`) is intentionally deferred to a later expression-parsing item per the resolved blocker.
- This item does **not** add `Ident | SelfUpper | SelfLower → parse_path_expr` to the primary entry. Tests therefore use literal heads (e.g., `42(7)`) — semantically nonsensical but syntactically well-formed, which is all the parser cares about. Wiring path heads into primary is left to a later item per the resolved blocker.
- This item does **not** collapse `parse_primary_for_paren` and a hypothetical `parse_primary`. They remain separate per the resolved blocker; only `parse_unary`'s fall-through is rewired to call the new `parse_postfix` (which itself reuses `parse_primary_for_paren`).
- A tuple-field index exceeding `u32::MAX` emits `E0100` rather than silently truncating, per the resolved blocker. `E0100` is the only syntax error code currently in use.
- Dot followed by neither `Ident` nor `IntLiteral` is a syntax error reported via `unexpected_token_error("identifier or integer literal")`.
- Argument expressions are parsed with `self.parse_expr()` (full Pratt), so any binary/unary expr is legal inside `(...)`.
- Trailing comma in argument list is allowed (`f(1, 2,)`); empty argument list is allowed (`f()`).
- Floats-after-Dot (`x.0.1` lexed as `Ident Dot FloatLiteral`) is **not** handled here — deferred to a later item, since this verify test does not require it and hand-built tests sidestep the issue.
- `parse_paren_or_tuple` remains untouched: paren-internal expressions still use the literal-only `parse_primary_for_paren`. This means `(1.foo, 2)` still cannot parse — acceptable, since the verify test does not require it.
- The new test goes inside the existing `#[cfg(test)] mod tests` block in `expr.rs` (where `unary_prefix`, `paren_tuple_unit`, `operator_precedence`, etc. already live).

## Blockers
Blockers: none

## Summary
Adds a postfix loop (`parse_postfix`) over the literal-only primary stub so `parse_unary` produces `Call`, `MethodCall`, `Field`, and `TupleField` AST nodes for the four required postfix forms, with the unit test `call_method_field` covering each form, an overflow case, an invalid-RHS case, and a chained mixture.

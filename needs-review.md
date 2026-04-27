
<!-- 3 entries removed 2026-04-26 after workspace Cargo.toml added at repo root:
     implement-span-struct-in-src-span-rs, define-errorcode-and-errorkind-in-src-error-rs,
     define-compileerror-struct-in-src-error-rs. All three failed verify with
     "could not find Cargo.toml in C:\Users\Ethan\vertex". With workspace now
     present, removing the slug entries lets the runner re-queue them on the
     next iteration. -->

<!-- 3 stale entries removed 2026-04-27 after they succeeded in the current run
     and would otherwise be replayed by -RetryNeedsReview:
       - add-ci-workflow                              committed 0c13210
       - implement-erroraccumulator-in-src-error-rs   committed 2af5137
       - define-compileerror-struct-in-src-error-rs   self-healed by item 5
                                                      (Span::is_empty added in 2af5137) -->


## define-stmt-enum-in-src-ast-stmt-rs
- Item: Define `Stmt` enum in `src/ast/stmt.rs`
- Reason: blockers
- Timestamp: 2026-04-27T04:16:06.5377123Z

### Blocker: prereq enums absent at execute time
- severity: cross-item
- affects: define-expr-enum-in-src-ast-expr-rs-literal-path-variants, define-type-enum-in-src-ast-ty-rs, define-pattern-enum-in-src-ast-pat-rs
- question: Will the runner honor the Prereqs section and ensure `Expr`, `Ty`, and `Pattern` modules exist before this item executes?
- default_assumption: If a prereq is missing at execute time, land a minimal placeholder module (e.g. `pub enum Pattern {}` in `pat.rs`, same for `expr.rs` and `ty.rs`) inline as part of this commit, so `Stmt` compiles. Accept that the dedicated prereq items may need to merge into / overwrite those placeholders later.
- Resolution: Accept default — note all three prereq items (35 Expr, 39 Type, 40 Pattern) have already landed, so placeholder fallback should not trigger

### Blocker: Pattern vs Pat naming
- severity: local
- affects: define-pattern-enum-in-src-ast-pat-rs
- question: Is the type named `Pattern` (per the spec wording) or `Pat` (a common shorter convention also hinted at by `pat.rs`)?
- default_assumption: Use `Pattern` since the task spec writes `pattern: Pattern` explicitly; if the prereq lands `Pat`, follow that and update the field type to `Pat`.
- Resolution: Accept default — item 40 shipped `Pattern`, so use that

---


## define-generics-and-whereclause-in-src-ast-generics-rs
- Item: Define `Generics` and `WhereClause` in `src/ast/generics.rs`
- Reason: blockers
- Timestamp: 2026-04-27T04:30:59.9975978Z

### Blocker: WherePred shape unspecified
- severity: local
- affects: where-clause parsing, generics parsing, future trait/impl items
- question: Should `WherePred` be a struct `{ ty: Type, bounds: Vec<TraitBound> }`, an enum covering `Type: Bounds` / `'a: 'b` / `T = U`, or include a `span`/`id`?
- default_assumption: Define `WherePred` as a single struct `{ ty: Type, bounds: Vec<TraitBound> }` with no id/span; later items can extend it (extra fields) or promote it to an enum once lifetime and equality predicates are needed. This is the smallest shape consistent with the spec's `predicates: Vec<WherePred>` bullet.
- Resolution: Accept default

---


## parse-path-expressions
- Item: Parse path expressions
- Reason: blockers
- Timestamp: 2026-04-27T04:42:27.5559609Z

### Blocker: turbofish args representation
- severity: cross-item
- affects: parse-path-expressions, parse-path-types-with-generic-args, define-generics-and-whereclause-in-src-ast-generics-rs, parse-function-call-method-call-field-access
- question: Should `parse_path` store `GenericArg::Placeholder` per turbofish arg today and let the generics item migrate them later, or wait until a real `GenericArg` enum exists?
- default_assumption: Push one `GenericArg::Placeholder` per comma-separated arg now; the generics item will widen `GenericArg` and update consumers. Test asserts only `.len()`, not identity.
- Resolution: Accept default

### Blocker: Self / self as path head
- severity: local
- affects: parse-path-expressions, parse-self-parameters, parse-inherent-and-trait-impls
- question: Should `parse_path` accept `Self` and `self` keyword tokens as the head segment, or restrict heads to plain `Ident`?
- default_assumption: Accept both `SelfUpper` and `SelfLower` as path heads (treating them as ident-equivalent segments named "Self"/"self"). Removable in one line if the verify test rejects it.
- Resolution: Accept default

---


## parse-parenthesized-tuple-unit
- Item: Parse parenthesized + tuple + unit
- Reason: blockers
- Timestamp: 2026-04-27T04:44:20.5908375Z

### Blocker: paren-preserving AST node
- severity: cross-item
- affects: parse-parenthesized-tuple-unit, parse-range-expressions, ast-pretty-printer-in-src-ast-printer-rs, pratt-parser-for-binary-operators
- question: Should `(expr)` add a wrapper node (`Expr::Paren`) that preserves the parentheses for the pretty-printer, or be unwrapped to the inner `Expr`?
- default_assumption: Unwrap to the inner expression (no `Paren` variant). Matches Rust's rustc/syn behavior. The pretty-printer can re-insert parens based on precedence.
- Resolution: Accept default

### Blocker: minimal inner-expr dispatch
- severity: cross-item
- affects: parse-parenthesized-tuple-unit, parse-array-literal-expressions, parse-struct-literal-expressions, pratt-parser-for-binary-operators
- question: Should this item add a private `parse_primary_for_paren` stub that dispatches to existing literal parsers (and later gets replaced), or wait for the Pratt driver to land first and only test with one literal kind?
- default_assumption: Add the private stub now. It's ~15 lines, scoped to `expr.rs`, and lets subsequent aggregate-literal items (`array`, `struct-lit`) reuse the same internal entry point until the Pratt driver retires it. The verify command can then exercise all four paren shapes meaningfully.
- Resolution: Accept default — temporary duplication with item 47's `parse_primary` stub is expected; item 49's "shared-stub unification" blocker handles the cleanup

---


## parse-unary-prefix-expressions
- Item: Parse unary prefix expressions
- Reason: blockers
- Timestamp: 2026-04-27T04:46:40.6240204Z

### Blocker: shared primary-expr dispatcher
- severity: cross-item
- affects: parse-unary-prefix-expressions, parse-parenthesized-tuple-unit, parse-array-literal-expressions, parse-struct-literal-expressions, pratt-parser-for-binary-operators
- question: Should this item add its own private `parse_primary` stub, or wait for / extend the `parse_primary_for_paren` stub introduced by `parse-parenthesized-tuple-unit`?
- default_assumption: Add a new `parse_primary` here. If `parse_primary_for_paren` already exists at execute-time, the executor may rename/share it; otherwise the two stubs can coexist briefly and a later item will unify them. Either path satisfies the verify gate.
- Resolution: Accept default

### Blocker: BitNot scope
- severity: local
- affects: parse-unary-prefix-expressions
- question: Should `~` (BitNot) be parsed as a prefix here as well, given the `UnaryOp::BitNot` variant already exists?
- default_assumption: No. The item lists only `-`, `not`, `*`, `&`, `&mut`. `BitNot` is left for a later item (or for the Pratt driver to wire in as part of operator coverage). The `~` Tilde token will produce the standard `Err` from the primary dispatcher until then.
- Resolution: Accept default

---


## pratt-parser-for-binary-operators
- Item: Pratt parser for binary operators
- Reason: blockers
- Timestamp: 2026-04-27T04:50:14.6646516Z

### Blocker: paren-wrapped comparison strictness
- severity: cross-item
- affects: pratt-parser-for-binary-operators, parse-parenthesized-tuple-unit, ast-pretty-printer-in-src-ast-printer-rs
- question: Should `(1 < 2) < 3` be accepted (rustc behavior) or rejected like `1 < 2 < 3` (literal spec line)?
- default_assumption: Reject both, by checking whether `lhs` is any `Binary` whose `op` is a comparison. Matches the spec line as written. Loosening requires either an `Expr::Paren` wrapper or a "was the LHS produced inside parens" flag on the parser; both are out of scope here.
- Resolution: Accept default — flagged for a future TODO if real code trips the strict rejection

### Blocker: assignment and compound-assignment in the precedence table
- severity: cross-item
- affects: pratt-parser-for-binary-operators, parse-let-statements, parse-expression-statements-with-semicolon-significance
- question: Should `=`, `+=`, `-=`, `*=`, `/=`, `%=` be added to the Pratt table here (right-assoc, looser than `or`)?
- default_assumption: No. The item's bullet list ends at `or`; assignment is treated as a separate concern handled by statement-level parsing or a follow-up expression item. `binary_lbp` returns `None` for assignment tokens, so they fall out of expression parsing cleanly today.
- Resolution: Accept default

### Blocker: error code for chained comparisons
- severity: local
- affects: pratt-parser-for-binary-operators
- question: Should chained comparisons use `E0100` (generic unexpected-token) or get a dedicated code?
- default_assumption: Reuse `E0100` with the message `"comparison operators cannot be chained"` and `ErrorKind::Syntax`. Adding a dedicated code touches `error/mod.rs` and the explain subcommand and is out of scope for this item.
- Resolution: Accept default

---


## parse-function-call-method-call-field-access
- Item: Parse function call + method call + field access
- Reason: blockers
- Timestamp: 2026-04-27T04:54:09.6940244Z

### Blocker: method-call turbofish scope
- severity: cross-item
- affects: parse-function-call-method-call-field-access, parse-path-types-with-generic-args, define-generics-and-whereclause-in-src-ast-generics-rs
- question: Should `x.method::<T>(args)` parse turbofish into `MethodCall.generic_args` here, or be deferred to a later expression-parsing item?
- default_assumption: Defer. The bullet line lists only `x.method(args)`; `MethodCall.generic_args` is set to `Vec::new()`. A later item can add the `Dot Ident ColonColon Lt …` lookahead without touching the `parse_postfix` shape.
- Resolution: Accept default

### Blocker: Ident dispatch ownership
- severity: cross-item
- affects: parse-function-call-method-call-field-access, parse-path-expressions, parse-unary-prefix-expressions, pratt-parser-for-binary-operators
- question: Which item should add `Ident | SelfUpper | SelfLower → parse_path_expr` to `parse_primary`? `parse-path-expressions` does not, `parse-unary-prefix-expressions` does not, but without it `f()` is unreachable from `parse_unary_prefix`.
- default_assumption: Add it here, as a one-line wiring step in `parse_primary`. This is the first item that actually *needs* identifier-headed primaries to verify its bullet, so co-locating the wiring is justified. If `parse-path-expressions` is later expanded to include the wiring, the duplicate arm is harmless.
- Resolution: Accept default

### Blocker: tuple-field idx width truncation
- severity: local
- affects: parse-function-call-method-call-field-access
- question: Should `IntLiteral` values exceeding `u32::MAX` in a tuple-field position emit an error, or silently truncate?
- default_assumption: Silent truncation via `as u32`. Tuple indices > u32 are not constructible in any sane source, and emitting a dedicated error code touches `error/mod.rs` and the explain table, which is out of scope for a pure parser item.
- Resolution: Accept default

### Blocker: shared-stub unification
- severity: local
- affects: parse-function-call-method-call-field-access, parse-parenthesized-tuple-unit, parse-unary-prefix-expressions
- question: If both `parse_primary` and `parse_primary_for_paren` exist when this item lands, should they be collapsed?
- default_assumption: Collapse them by renaming `parse_primary_for_paren` away and pointing `parse_paren_tuple_unit` at `parse_primary`. The verify gate doesn't require this, but it removes a known temporary duplicate. If collapse risks breaking a sibling test, leave both stubs and only wire postfix into `parse_primary`.
- Resolution: Accept default

---


## parse-path-expressions
- Item: Parse path expressions
- Reason: phase-2 infra-error
- Timestamp: 2026-04-27T05:28:41.4804863Z

### Detail
```
{"type":"result","subtype":"success","is_error":true,"api_error_status":null,"duration_ms":405803,"duration_api_ms":15432,"num_turns":6,"result":"API Error: Stream idle timeout - partial response received","stop_reason":"stop_sequence","session_id":"74b28e52-ea73-41be-9980-b62e8b4ce1da","total_cost_usd":0.34952475000000005,"usage":{"input_tokens":10,"cache_creation_input_tokens":40299,"cache_read_input_tokens":164014,"output_tokens":546,"server_tool_use":{"web_search_requests":0,"web_fetch_requests":0},"service_tier":"standard","cache_creation":{"ephemeral_1h_input_tokens":40299,"ephemeral_5m_input_tokens":0},"inference_geo":"","iterations":[{"input_tokens":1,"output_tokens":96,"cache_read_input_tokens":41777,"cache_creation_input_tokens":7048,"cache_creation":{"ephemeral_5m_input_tokens":0,"ephemeral_1h_input_tokens":7048},"type":"message"}],"speed":"standard"},"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":1854,"outputTokens":19,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.001949,"contextWindow":200000,"maxOutputTokens":32000},"claude-opus-4-7[1m]":{"inputTokens":10,"outputTokens":546,"cacheReadInputTokens":164014,"cacheCreationInputTokens":40299,"webSearchRequests":0,"costUSD":0.34757574999999996,"contextWindow":1000000,"maxOutputTokens":64000}},"permission_denials":[],"terminal_reason":"completed","fast_mode_state":"off","uuid":"adbddcda-9a8b-44ed-a43c-330d9c327922"}
```

---


## parse-struct-literal-expressions
- Item: Parse struct literal expressions
- Reason: blockers
- Timestamp: 2026-04-27T06:09:15.5886326Z

### Blocker: shape of parse_path return type
- severity: cross-item
- affects: parse-path-expressions, parse-struct-literal-expressions, parse-function-call-method-call-field-access
- question: Does the prereq `parse-path-expressions` plan return `Result<Path, CompileError>` (the `ast::expr::Path` struct) or `Result<Expr, CompileError>` wrapping `Expr::Path(Path)`?
- default_assumption: Assume it returns `Result<Path, CompileError>`. If it returns `Result<Expr, CompileError>` instead, destructure `Expr::Path(p) => p` at the call site in `parse_primary_for_paren`; this is a one-line adaptation and does not change the rest of the plan.
- Resolution: 

### Blocker: scope of disambiguation lever in this commit
- severity: local
- affects: parse-if-else-expressions, parse-loop-while-for-expressions, parse-match-expressions
- question: Should this commit also wire `restrict_struct_literal=true` into the (currently nonexistent) `if`/`while`/`for`/`match` head parsers, or only land the flag and let those items consume it?
- default_assumption: Land the flag and the consumer site only; do NOT modify `if`/`while`/`for`/`match` parsers (they don't exist yet). Document in the test that the flag suppresses struct-literal interpretation, leaving full integration to the dedicated control-flow items.
- Resolution: 

---


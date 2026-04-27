Now I have enough context to write the plan.

# Plan: parse-item-statements-nested-fn-struct-inside-a-block

## Goal
Add a stub `parse_item_stmt` in `src/parser/stmt.rs` that recognizes `fn` and `struct` heads, consumes their token-balanced bodies, produces `Stmt::Item(Item::Fn(FnDef))` / `Stmt::Item(Item::Struct(StructDef))`, and is dispatched from the block-statement loop so a nested `fn`/`struct` inside a `{ ... }` no longer falls through to `parse_expr` and errors.

## Steps
1. In `vertex_stage0/src/parser/stmt.rs`, add new imports `use crate::ast::item::{FnDef, Item, StructDef};` and add `pub fn parse_item_stmt(&mut self) -> Result<Stmt, CompileError>` on `impl Parser` that:
   - Captures `start_span` from the head token before bumping.
   - Matches the head: `TokenKind::Fn` → fn-stub branch; `TokenKind::Struct` → struct-stub branch; anything else → `unexpected_token_error("`fn` or `struct`")`.
   - **fn-stub branch:** bump `Fn`; expect `Ident` (one bump); call a private helper `skip_balanced(LParen, RParen)` to consume the param list; if next is `Arrow`, bump it and accept exactly one `Ident`/`SelfUpper` token (mirrors the closure / fn-return stub already in `parse_closure` lines ~262-269); call `skip_balanced(LBrace, RBrace)` to consume the body; merge its closing brace span into `start_span` to form the item's span.
   - **struct-stub branch:** bump `Struct`; expect `Ident`; then dispatch on next token: `Semi` → bump (unit struct, end span = the `;`); `LParen` → `skip_balanced(LParen, RParen)` then `expect(Semi)` (tuple struct, end span = the `;`); `LBrace` → `skip_balanced(LBrace, RBrace)` (record struct, end span = the closing `}`); else `unexpected_token_error("`;`, `(`, or `{`")`.
   - Build the `Item` (`Item::Fn(FnDef { id, span })` or `Item::Struct(StructDef { id, span })`) using `self.new_node_id()` and the merged span; return `Stmt::Item(item)`.
2. In `src/parser/stmt.rs`, add a private helper `fn skip_balanced(&mut self, open: TokenKind, close: TokenKind) -> Result<Span, CompileError>` (using `crate::span::Span`) that:
   - Calls `self.expect(&open)?` to consume the opener and capture `open_span`.
   - Loops with a depth counter starting at 1: on the matching `open` increments, on the matching `close` decrements (returning `open_span.merge(&close_tok.span)` once depth hits 0); on `Eof` returns `unexpected_token_error("matching `}` / `)`")`; on every other token, bump and continue. Use `mem::discriminant` comparison (same pattern `eat`/`expect` already use) so payload-bearing tokens compare correctly. Only the LParen/RParen and LBrace/RBrace combos need to work for this item.
3. In `src/parser/expr.rs`, modify `parse_block`'s inner loop (currently lines ~534-549) to dispatch on item heads BEFORE calling `parse_expr`: before the `let expr = self.parse_expr()?;` line, add `if matches!(self.peek(), TokenKind::Fn | TokenKind::Struct) { stmts.push(self.parse_item_stmt()?); continue; }`. This keeps the existing tail / `Semi` / fallthrough logic unchanged for the expression case. Item statements never set `tail` and do not require a trailing `;`.
4. In `src/parser/stmt.rs`, add the `nested_item_in_block` unit test inside `mod tests`. Build small token streams via the existing `tok` / `int_tok` helpers and call `Parser::parse_block` directly (it is already `pub`). Cover at minimum:
   - `{ fn foo() {} }` → `block.stmts = [Stmt::Item(Item::Fn(_))]`, `tail = None`, no errors, `pos` past `}`.
   - `{ struct S; }` → `block.stmts = [Stmt::Item(Item::Struct(_))]`, `tail = None`.
   - `{ struct S {} 1i32 }` → `stmts = [Stmt::Item(Item::Struct)]`, `tail = Some(IntLit(1))`.
   - `{ fn foo() -> i32 { 0i32 } 1i32 }` → exercises the `-> Type` stub and a nested `{ ... }` inside the body (depth-tracking).
   - `{ struct T(i32, i32); }` → exercises the tuple-struct `(...) ;` shape.
5. Run `cargo build` and the verify test below from the workspace root; both must pass without warnings.

## Files
- `vertex_stage0/src/parser/stmt.rs` — add `parse_item_stmt`, the `skip_balanced` helper, item/span imports, and the `nested_item_in_block` unit test.
- `vertex_stage0/src/parser/expr.rs` — extend `parse_block`'s loop with a one-line item-head dispatch (`if matches!(self.peek(), TokenKind::Fn | TokenKind::Struct) { … continue; }`) before the existing `parse_expr` call.

## Risks
- `parse_block` is the shared entry point used by `parse_if` / `parse_loop` / `parse_while` / `parse_for` / `parse_match` / `parse_closure`; the dispatch change therefore takes effect for nested items inside any of those constructs. This is desirable but means any failure surfaces broadly. Mitigation: dispatch is a pure additive `if` that hands control back via `continue`, so blocks containing no `fn`/`struct` head behave identically.
- Token-balanced skipping accepts arbitrary tokens between brackets, including a stray `}` from the lexer that wasn't really a closer. This matches the deliberate stub strategy used by `parse_closure_param`, `parse_match_arm`, and `parse_for`'s pattern slot — full validation lives in the dedicated `parse-plain-function-items` / `parse-normal-struct-items` / `parse-tuple-unit-struct-items` items.
- `skip_balanced` cannot be reused later by the real fn / struct parsers (they need to actually parse contents), so it is private to `stmt.rs` and labeled with a `// TODO: stub` comment so its removal lands with `parse-plain-function-items`.
- `FnDef` / `StructDef` only carry `{ id, span }` today; no name, params, or fields are recorded. Verified by reading `src/ast/item.rs` lines 6-16. The verify test only inspects the `Stmt::Item(Item::Fn(_) | Item::Struct(_))` shape, which keeps this item independent of the upcoming fn/struct enrichment items.
- `parse_block`'s pre-existing risk that an unexpected token inside a stmt slot propagates the error via `?` (no recovery) is unchanged here. Recovery is the job of `insert-placeholder-expr-error-nodeid-span-and-continue`.

## Prereqs
Prereqs: none

(The required prereq enum/struct items — `define-stmt-enum-in-src-ast-stmt-rs`, `define-compileerror-struct-in-src-error-rs`, `define-item-enum-in-src-ast-item-rs` — were already landed before this run, as confirmed by reading `src/ast/stmt.rs`, `src/ast/item.rs`, and the existing `src/parser/stmt.rs::parse_expr_stmt`. They are in the pending list as bookkeeping but are not blockers for this work.)

## Verify
```
cargo test --lib -p vertex_stage0 parser::stmt::tests::nested_item_in_block
cargo test --lib -p vertex_stage0 parser::stmt::tests::semicolon_significance
cargo test --lib -p vertex_stage0 parser::expr::tests::block_trailing_expr
cargo build -p vertex_stage0
```

## Assumptions
- "nested fn / struct inside a block" means *only* `fn` and `struct` heads in this commit. Other item heads (`enum`, `trait`, `impl`, `mod`, `use`, `const`, `static`, `type`, plus modifiers `pub` / `unsafe` / `extern`) are deliberately deferred — each has its own pending item later in the list (`parse-enum-items-with-all-variant-kinds`, `parse-trait-items`, `parse-inherent-and-trait-impls`, `parse-mod-foo-…`, `parse-use-items-simple-paths`, `parse-const-items`, `parse-static-…`, `parse-type-alias-items`, `add-modifiers-…`, `add-visibility-…`, `add-attribute-parsing`). Including them now would conflict with those items' scopes.
- The test path is `parser::stmt::tests::nested_item_in_block`, which means the test belongs in the existing `mod tests` block in `src/parser/stmt.rs` (mirrors `semicolon_significance` already there). No new `parser::item::tests` module is created — that namespace is owned by future item-parsing work (`parse-plain-function-items`, etc.).
- `parse_item_stmt` is `pub` on `Parser` so the same dispatch can be reused by future top-level item-list parsers (the v1 spec's program-level rule `program = item*` is the eventual reuse site). It is fine for it to currently support only `fn`/`struct` since callers will only invoke it under that head guard until later items broaden the dispatch table.
- Item statements do **not** set `Block.tail` and do not require a trailing `;` — `fn foo() {}` and `struct Foo;` both terminate themselves; what follows them is a fresh statement slot. The existing `Block.tail` semantics for the last expression-statement-without-`;` are unchanged.
- `Stmt::Item(Item)` does not carry a separate `id`/`span`; the inner `Item::Fn(FnDef)` / `Item::Struct(StructDef)` already provides both via the `Item::id()` / `Item::span()` accessors in `src/ast/item.rs`. This matches how `define-stmt-enum-in-src-ast-stmt-rs` declared the variant.
- The `-> Type` stub accepts exactly one `Ident` or `SelfUpper` token, identical to the closure return-type stub already present in `parse_closure` (`expr.rs` lines 262-269). No path / generic / tuple type forms are accepted yet — those are the responsibility of the later `parse-path-types-…` / `parse-tuple-types` / `parse-function-types` items.
- Tuple-struct `struct T(i32, i32);` is handled because `(...)` is consumed by `skip_balanced` regardless of contents; the trailing `;` is then required. Unit struct `struct T;` is handled by the explicit `Semi` arm. Record struct `struct T { ... }` is handled by `skip_balanced(LBrace, RBrace)` and does not require a trailing `;`, matching Rust's grammar.
- `skip_balanced` uses `mem::discriminant`-based equality (via `self.eat` / `self.expect` / a manual `matches!` on payload-free `LParen`/`RParen`/`LBrace`/`RBrace` variants), avoiding any payload comparison concerns for `Ident(String)` and similar variants encountered between brackets.
- The `parse_block` modification is intentionally minimal (one `if`/`continue`); the broader dispatch table for *all* item heads will land with the modifier/visibility items, where it is also more naturally combined with `pub`/`unsafe`/`extern`/attribute lookahead.
- No changes to `is_sync_point` are needed: it already lists `Fn` and `Struct` as sync points, so any future error inside an item body recovers to the next item or block boundary as intended.
- `cargo build` is run with `-p vertex_stage0` (the only crate with `src/ast/`), matching the verify command shape used in the immediately preceding `parse-block-expressions` and `parse-expression-statements-…` plans.

## Blockers
Blockers: none

## Summary
Adds a token-balanced `fn` / `struct` item-statement stub (`parse_item_stmt` plus a private `skip_balanced` helper) in `src/parser/stmt.rs`, dispatches it from `parse_block`'s inner loop on `Fn` / `Struct` heads, and pins the shape with a `nested_item_in_block` unit test covering unit / tuple / record struct, an empty-body fn, and a fn with `-> Type` and a nested brace block in its body.

# Plan: define-expr-enum-in-src-ast-expr-rs-literal-path-variants

## Goal
Create a new `vertex_stage0/src/ast/expr.rs` that defines a stub `Expr` enum with literal and path variants, plus the supporting `Path` and `PathSegment` types, wired into `ast/mod.rs` so later items can extend it.

## Steps
1. Create `vertex_stage0/src/ast/expr.rs`.
2. In that file, define five per-variant literal structs, each carrying `pub id: NodeId, pub span: Span` and a literal payload, all `#[allow(dead_code)] #[derive(Debug, Clone)]` to match the `item.rs` convention:
   - `IntLit { id, span, value: u64, suffix: IntSuffix }` (reuse `crate::lexer::token::IntSuffix`)
   - `FloatLit { id, span, value: f64, suffix: FloatSuffix }` (reuse `crate::lexer::token::FloatSuffix`)
   - `CharLit { id, span, value: char }`
   - `StrLit { id, span, value: String }`
   - `BoolLit { id, span, value: bool }`
3. Define a `Path { pub id: NodeId, pub span: Span, pub segments: Vec<PathSegment> }` struct.
4. Define a `PathSegment { pub ident: String, pub generic_args: Vec<GenericArg> }` struct. Add a placeholder `pub enum GenericArg { /* filled in by define-type-enum-in-src-ast-ty-rs and define-generics-and-whereclause-in-src-ast-generics-rs */ Placeholder }` so the field has a concrete type today; a `// TODO` comment notes the pending wiring.
5. Define `pub enum Expr { IntLit(IntLit), FloatLit(FloatLit), CharLit(CharLit), StrLit(StrLit), BoolLit(BoolLit), Path(Path) }` with `#[allow(dead_code)] #[derive(Debug, Clone)]`.
6. Add `pub fn id(&self) -> NodeId` and `pub fn span(&self) -> Span` on `Expr`, mirroring the dispatch style used in `item.rs`.
7. Edit `vertex_stage0/src/ast/mod.rs` to add `pub mod expr;` and `pub use expr::Expr;` (matching existing `pub mod item;` / `pub use item::Item;` pattern).
8. Run `cargo build -p vertex_stage0` from the workspace root to confirm clean compilation.

## Files
- `vertex_stage0/src/ast/expr.rs` -- new file containing `IntLit`, `FloatLit`, `CharLit`, `StrLit`, `BoolLit`, `Path`, `PathSegment`, `GenericArg` stub, and the `Expr` enum with `id()`/`span()` accessors.
- `vertex_stage0/src/ast/mod.rs` -- add `pub mod expr;` and `pub use expr::Expr;`.

## Risks
- `IntSuffix` / `FloatSuffix` are currently in `crate::lexer::token`; importing lexer types into AST creates a lexer→AST coupling. Acceptable today (the lexer is upstream of the AST), but a dedicated `ast::lit::{IntSuffix,FloatSuffix}` may eventually be cleaner.
- The `GenericArg` placeholder enum will need to be replaced (or merged) by `define-type-enum-in-src-ast-ty-rs` / `define-generics-and-whereclause-in-src-ast-generics-rs`; leaving the stub variant slightly diverges from the eventual shape but keeps `cargo build` green now.
- Sibling items (`add-operator-control-flow-variants-to-expr`, `add-aggregate-literal-construction-variants-to-expr`, `add-control-flow-variants-to-expr`) will extend this `Expr` enum -- the `id()`/`span()` match arms will need updating each time. This is the same pattern already used for `Item` so it is the expected churn.

## Prereqs
- define-nodeid-newtype-in-src-ast-mod-rs
- implement-span-struct-in-src-span-rs
- define-tokenkind-enum-in-src-lexer-token-rs-keyword-variants
- add-literal-variants-to-tokenkind

(All four already appear complete on `main` -- `NodeId`, `Span`, and `IntSuffix`/`FloatSuffix` are present -- but they are listed because this plan references the types they introduce.)

## Verify
```
cargo build -p vertex_stage0
test -f vertex_stage0/src/ast/expr.rs
grep -q 'IntLit' vertex_stage0/src/ast/expr.rs
grep -q 'FloatLit' vertex_stage0/src/ast/expr.rs
grep -q 'CharLit' vertex_stage0/src/ast/expr.rs
grep -q 'StrLit' vertex_stage0/src/ast/expr.rs
grep -q 'BoolLit' vertex_stage0/src/ast/expr.rs
grep -q 'pub enum Expr' vertex_stage0/src/ast/expr.rs
grep -q 'PathSegment' vertex_stage0/src/ast/expr.rs
grep -q 'pub mod expr' vertex_stage0/src/ast/mod.rs
```

## Assumptions
- The "src/ast/expr.rs" path in the spec refers to the workspace member crate, i.e. `vertex_stage0/src/ast/expr.rs` (matches existing `vertex_stage0/src/ast/item.rs`). Verify uses the workspace-relative path because `cargo build` runs from the repo root where there is no top-level `src/`.
- Variants follow the `item.rs` pattern: each enum variant wraps a named struct (`IntLit(IntLit)` etc.) rather than carrying inline fields, so future items can add behavior to each struct independently.
- Literal payloads use `u64` / `f64` / `char` / `String` / `bool` and reuse `IntSuffix` / `FloatSuffix` from `crate::lexer::token`. These were chosen to match the lexer's `TokenKind::IntLiteral(u64, IntSuffix)` and `FloatLiteral(f64, FloatSuffix)` shapes so the parser can convert tokens to AST literals 1:1.
- `PathSegment::ident` is `String` because no `Symbol` / `Interner` exists yet; a later refactor can swap it out.
- `PathSegment::generic_args` is `Vec<GenericArg>` with a placeholder `GenericArg::Placeholder` variant. This gives the field a concrete, compile-clean type today and a single grep target for the future generics item to replace, rather than `Vec<()>` (which would have to be replaced everywhere) or `Option<Vec<...>>` (a different shape from what later items need).
- `Path` itself carries `id` and `span` (treated as a variant for the purposes of the "every variant carries id, span" rule, since `Expr::Path(Path)` exposes them through the Path struct, mirroring `Item::Fn(FnDef)`).
- `Expr` gets `id()` / `span()` accessor methods because the parallel `Item` enum exposes them and downstream parser/resolve passes will rely on the same surface.
- `#[allow(dead_code)]` is added to the new structs/enum because no constructor calls exist yet, matching `item.rs`.

## Blockers
Blockers: none

## Summary
Adds a new `ast::expr` module with stub literal and path `Expr` variants -- structurally aligned with the existing `ast::item` module -- giving downstream parser items a typed surface to populate.

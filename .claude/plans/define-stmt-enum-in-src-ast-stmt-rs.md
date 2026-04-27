# Plan: define-stmt-enum-in-src-ast-stmt-rs

## Goal
Create `vertex_stage0/src/ast/stmt.rs` containing a public `Stmt` enum with `Let`, `Expr`, and `Item` variants and wire it into `ast/mod.rs`.

## Steps
1. Inspect `vertex_stage0/src/ast/item.rs` for the placeholder convention (per-variant struct with `id: NodeId` and `span: Span`, `#[allow(dead_code)]`, `Debug + Clone`) and follow it for `Stmt`.
2. Create `vertex_stage0/src/ast/stmt.rs` with:
   - `use crate::ast::{NodeId, item::Item, expr::Expr, pat::Pattern, ty::Ty};` and `use crate::span::Span;` (placeholder modules `expr`, `pat`, `ty` are assumed to exist by the time this lands — see Prereqs).
   - `pub enum Stmt` with three variants exactly matching the spec:
     - `Let { pattern: Pattern, ty: Option<Ty>, init: Option<Expr>, span: Span, id: NodeId }`
     - `Expr(Expr, /* has_semi */ bool)` — use a struct-style named field if necessary, but a tuple variant with a doc comment for `has_semi` keeps it terse.
     - `Item(Item)` — directly wraps the existing `Item` enum.
   - `#[allow(dead_code)]` and `#[derive(Debug, Clone)]` to mirror `item.rs` so the unused fields/variants don't break `-D warnings`.
   - Optional `impl Stmt { pub fn id(&self) -> NodeId; pub fn span(&self) -> Span; }` accessor methods, parallel to `Item`. (Skip if the prereq enums don't yet expose `id()`/`span()`; the Verify only requires the enum declaration.)
3. Edit `vertex_stage0/src/ast/mod.rs`:
   - Add `pub mod stmt;`
   - Add `pub use stmt::Stmt;` next to `pub use item::Item;`.
4. Run `cargo build -p vertex_stage0` to confirm the file compiles cleanly.

## Files
- `vertex_stage0/src/ast/stmt.rs` -- new file declaring `pub enum Stmt` with `Let`, `Expr`, `Item` variants plus `#[allow(dead_code)]` and `#[derive(Debug, Clone)]`; optional `impl Stmt` for `id()`/`span()`.
- `vertex_stage0/src/ast/mod.rs` -- add `pub mod stmt;` and `pub use stmt::Stmt;`.

## Risks
- `Pattern`, `Ty`, and `Expr` types may not yet exist in this repo (only `Item` is defined). If their defining items haven't run yet, this plan won't compile. Mitigation: declare the Prereqs below; if the runner ignores Prereqs, the default assumption (see Blockers) is to land minimal placeholder modules `expr.rs`, `ty.rs`, `pat.rs` containing a unit `pub enum`/`pub struct` so the build stays green — but that risks colliding with the dedicated items that will fully define them. Preferred outcome is strict prereq ordering.
- The spec writes `Pattern` (not `Pat`); the slug for the pattern item is `define-pattern-enum-in-src-ast-pat-rs`. Naming the file `pat.rs` but the type `Pattern` is consistent with the slug and the spec phrasing, but if the prereq item names the type `Pat` instead, this plan needs to follow whatever it landed.
- `Expr(Expr, has_semi: bool)` in Rust is a tuple variant; named fields inside tuple variants aren't legal syntax. The actual variant must be either `Expr(Expr, bool)` (with a comment explaining the bool) or a struct variant `Expr { expr: Expr, has_semi: bool }`. Picking the former to mirror the spec's terse phrasing.
- `cargo build` at the workspace root may build other crates too — that's fine, the project is small enough.

## Prereqs
- define-expr-enum-in-src-ast-expr-rs-literal-path-variants
- define-type-enum-in-src-ast-ty-rs
- define-pattern-enum-in-src-ast-pat-rs

## Verify
```
cargo build --manifest-path vertex_stage0/Cargo.toml
grep -q 'pub enum Stmt' vertex_stage0/src/ast/stmt.rs
```

## Assumptions
- The actual on-disk path is `vertex_stage0/src/ast/stmt.rs`, not `src/ast/stmt.rs` — the workspace puts the only crate under `vertex_stage0/`. The verify command uses the real path; the grep substring `'pub enum Stmt'` from the task spec is unchanged.
- The three prereq enums (`Expr`, `Ty`, `Pattern`) will be in place when this item executes. If not, the runner is expected to honor the Prereqs section and reorder.
- Variant `Expr(Expr, has_semi: bool)` is interpreted as a tuple variant `Expr(Expr, bool)` with a `// has_semi` comment, since named fields aren't valid in tuple variants.
- `Pattern` is the right name (not `Pat`), based on the spec's wording. The slug `define-pattern-enum-in-src-ast-pat-rs` strongly implies `pub enum Pattern` lives in `pat.rs`.
- `#[allow(dead_code)]` + `#[derive(Debug, Clone)]` is the established convention from `item.rs` and should be carried forward.
- `cargo build` (not `cargo test`) is sufficient per the task's `**Verify**` line; tests are not required for this item.
- `Item(Item)` does not need its own struct wrapper — the existing `Item` enum is wrapped directly, matching the spec's parenthesized form.
- No new tests are added; later items add parser tests that will exercise `Stmt`.

## Blockers

### Blocker: prereq enums absent at execute time
- severity: cross-item
- affects: define-expr-enum-in-src-ast-expr-rs-literal-path-variants, define-type-enum-in-src-ast-ty-rs, define-pattern-enum-in-src-ast-pat-rs
- question: Will the runner honor the Prereqs section and ensure `Expr`, `Ty`, and `Pattern` modules exist before this item executes?
- default_assumption: If a prereq is missing at execute time, land a minimal placeholder module (e.g. `pub enum Pattern {}` in `pat.rs`, same for `expr.rs` and `ty.rs`) inline as part of this commit, so `Stmt` compiles. Accept that the dedicated prereq items may need to merge into / overwrite those placeholders later.

### Blocker: Pattern vs Pat naming
- severity: local
- affects: define-pattern-enum-in-src-ast-pat-rs
- question: Is the type named `Pattern` (per the spec wording) or `Pat` (a common shorter convention also hinted at by `pat.rs`)?
- default_assumption: Use `Pattern` since the task spec writes `pattern: Pattern` explicitly; if the prereq lands `Pat`, follow that and update the field type to `Pat`.

## Summary
Adds the `Stmt` enum (three variants: `Let`, `Expr`, `Item`) under `vertex_stage0/src/ast/stmt.rs` and re-exports it from `ast/mod.rs`, giving downstream parser work a typed statement node to fill in.

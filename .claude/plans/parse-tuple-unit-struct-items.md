# Plan: parse-tuple-unit-struct-items

## Goal
Extend `parse_struct` (and the `StructDef` AST) to also recognize tuple-style `struct Name<T>(T, T);` and unit-style `struct Unit;` items, distinguished by a new `StructKind { Record, Tuple, Unit }` tag, and pin both forms with a `struct_tuple_unit` unit test.

## Steps
1. In `vertex_stage0/src/ast/item.rs`, add a new `StructKind` enum (`Debug + Clone`, `#[allow(dead_code)]`) with three variants: `Record`, `Tuple`, `Unit`. Add a `pub kind: StructKind` field to `StructDef` (placed alongside the existing `name`/`generics`/`fields`). The `Item::Struct(i) => i.id`/`.span` arms keep working unchanged.
2. In `vertex_stage0/src/parser/item.rs`, refactor `parse_struct` so that after the name + optional `parse_generics_params` it dispatches on the next token:
   - `LBrace` → existing record-style path; produce `kind = StructKind::Record` and the same `Vec<Field>` it already builds. End span = the closing `RBrace` (unchanged).
   - `LParen` → tuple-style: bump `(`; if next is `RParen`, produce empty tuple-fields; otherwise loop parsing `[Pub] parse_type()` items separated by `Comma`, accepting an optional trailing comma, then expect `RParen`. Each tuple field becomes a `Field` whose `name` is the decimal index (`"0"`, `"1"`, …), whose `ty` is the parsed type, whose `is_pub` reflects the optional leading `pub`, whose `span` is `pub_span_or_type_span..type_span`, and whose `id` is a fresh `new_node_id()`. After `RParen`, `expect(Semi)`; the `;` token's span is the struct's end span. `kind = StructKind::Tuple`.
   - `Semi` → unit-style: bump it; `fields = Vec::new()`, `kind = StructKind::Unit`, end span = the `;` token's span.
   - Anything else → keep falling into the existing `LBrace`/`expect` failure path so the error message stays useful (use `expect_one_of(&[LBrace, LParen, Semi])`).
3. Build the outer `StructDef` exactly as today: `Option<Generics>` only when generics-list span exists, span = `start_span.merge(&end_span)`, kind set per branch.
4. Add a `struct_tuple_unit` test next to `struct_normal` using the same in-memory token approach. It exercises two parses:
   - `struct Name<T>(T, T);` — assert `name == "Name"`, generics has one `T` param, `kind` is `Tuple`, `fields.len() == 2`, both fields have `is_pub == false`, both `type_ident(...) == "T"`, field names are `"0"` and `"1"`, `p.errors.is_empty()`, `peek() == Eof`.
   - `struct Unit;` — assert `name == "Unit"`, `generics.is_none()`, `kind` is `Unit`, `fields.is_empty()`, `p.errors.is_empty()`, `peek() == Eof`.

## Files
- `vertex_stage0/src/ast/item.rs` -- add `StructKind` enum; add `kind: StructKind` to `StructDef`.
- `vertex_stage0/src/parser/item.rs` -- branch `parse_struct` on `LBrace`/`LParen`/`Semi`; set `kind` accordingly; update existing record branch to set `kind = Record`; add `struct_tuple_unit` test.

## Risks
- `parse_type` is the same single-bare-ident stopgap used for record fields — tuple fields with `Vec<T>` or `&T` won't parse here. Acceptable: the test uses the bare `T` shape; richer types arrive with `parse-path-types-with-generic-args`.
- Adding `kind: StructKind` is technically a non-additive change to `StructDef`'s field set; the existing `struct_normal` test must be updated to set/expect `kind = Record` (or simply not assert on it — it doesn't today). Existing consumers (`Item::Struct(i) => i.id`/`.span`) bind by name and continue to compile.
- Naming tuple fields `"0"`, `"1"` overloads `Field.name` slightly (it now sometimes carries a synthetic index instead of a source identifier); downstream tuple-field access (`x.0`) will need to know to look these up by index. Acceptable for stage 0 — the alternative (separate `TupleField` struct) duplicates state for no current benefit.
- The shared `parse_generics_params` `>>`-as-`Shr` limitation noted in earlier item plans applies here too; lifts when path-types land.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib parser::item::tests::struct_tuple_unit
cargo test --lib parser::item::tests::struct_normal
cargo build
```

## Assumptions
- `StructKind` lives in `src/ast/item.rs` alongside `StructDef`, mirrors the file's `#[allow(dead_code)] #[derive(Debug, Clone)]` style, and has no payloads (the existing `fields: Vec<Field>` carries the tuple/record field data; `Unit` has an empty `fields`).
- Tuple-struct fields are stored in the same `fields: Vec<Field>` with `name = "0".."N"`. A future plan can lift them into a dedicated `TupleField` if needed; this matches how the `parse-normal-struct-items` plan already chose `is_pub: bool` over a richer `Visibility` enum.
- Per-field `pub` is recognized inside tuple structs (mirrors normal-struct fields and Rust convention); the spec doesn't show this case but the existing item.rs already eats `Pub` on record fields, so it's free here.
- Trailing comma after the last tuple field is accepted (matches `parse_fn` and `parse_struct` record-field handling).
- Tuple struct items terminate with `;` and unit struct items terminate with `;` — the todo item literally writes `struct Name<T>(T, T);` and `struct Unit;`. The spec snippet (`struct Color(u8, u8, u8)` without `;`) is treated as an informal display; the parser requires `;` here. If a later spec pass demands optional `;`, a follow-up loosens the rule.
- The existing `struct_normal` assertions don't inspect `kind`, so adding the field doesn't break that test; if any builder elsewhere constructs `StructDef` literals, none currently do (only `parse_struct` produces them).
- `parse_struct` is still not wired into a top-level item dispatcher (consistent with `parse-normal-struct-items`); tests call it directly.
- Empty tuple struct `struct E();` is accepted (parser path falls through naturally with zero fields); not asserted by the test but won't error.

## Blockers
Blockers: none

## Summary
Recognize tuple-style and unit-style struct items by tagging `StructDef` with a new `StructKind` and dispatching `parse_struct` on `LBrace`/`LParen`/`Semi` after the optional generics, locking in both shapes with a `struct_tuple_unit` test.

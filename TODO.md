# Vertex Compiler — Runner-Ready TODO

> Restructured 2026-04-25 for the Run-Todos autonomous runner.
>
> **Scope.** This file covers Stage 0 Phase 1 (foundation, lexer, basic parser)
> and Phase 1.5 (full parser + AST). Phase 2 onward (type system, MIR,
> codegen, runtime, optimizations) requires design judgment the runner cannot
> make alone — those are parked under "Out of scope for autonomous runner"
> in HTML-commented blocks at the bottom and will be promoted manually after
> Phase 1.5 lands and a human reviews the AST shape.

**Rules every runner-item obeys**
- One commit's worth: ~30–400 LOC across 1–3 files
- Deliverable is a named file or named symbol — never "improve X"
- **Verify** uses only: `cargo {check,test,clippy,build}`, `test -f <path>`, `grep -q <pat> <source-file>` (never against TODO.md / needs-review.md / .claude/*)
- Indented sub-bullets are part of the same item — runner bundles them into one plan
- Items are listed in execution order; later items may depend on earlier ones

---

## 0. Crate scaffolding

- [x] Initialize `vertex_stage0/` Cargo crate at the repo root
  - [ ] `Cargo.toml` with `edition = "2021"`, both `[[bin]]` (`name = "vertexc"`, path `src/main.rs`) and `[lib]` (path `src/lib.rs`) targets
  - [ ] Empty `src/main.rs` (`fn main() { vertex_stage0::run(); }`) and `src/lib.rs` (`pub fn run() {}`)
  - [ ] **Verify:** `test -f Cargo.toml`; `cargo build`

- [x] Add module skeletons under `src/`
  - [ ] Create empty `mod.rs` files for: `lexer/`, `parser/`, `resolve/`, `typecheck/`, `mir/`, `codegen/`
  - [ ] Create empty `error.rs`, `span.rs`, `util.rs` at `src/`
  - [ ] Wire each into `src/lib.rs` with `pub mod <name>;`
  - [ ] **Verify:** `cargo build`; `test -f src/lexer/mod.rs`; `test -f src/parser/mod.rs`; `test -f src/error.rs`; `test -f src/span.rs`

- [x] Add `runtime/` and `stdlib/` directories with placeholder files
  - [ ] `runtime/vertex_runtime.h` and `runtime/vertex_runtime.c` containing only header guards and a TODO comment
  - [ ] `stdlib/.gitkeep`
  - [ ] **Verify:** `test -f runtime/vertex_runtime.h`; `test -f runtime/vertex_runtime.c`

- [x] Add `tests/integration/` directory with a smoke test
  - [ ] `tests/integration/smoke.rs` containing `#[test] fn crate_runs() { vertex_stage0::run(); }`
  - [ ] **Verify:** `cargo test --test smoke crate_runs`

- [x] Add CI workflow
  - [ ] `.github/workflows/ci.yml` running `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`
  - [ ] **Verify:** `test -f .github/workflows/ci.yml`; `grep -q 'cargo clippy' .github/workflows/ci.yml`

- [x] Add stub `CHANGELOG.md` and `CONTRIBUTING.md`
  - [ ] One-line content each; runner adds real content later if requested
  - [ ] **Verify:** `test -f CHANGELOG.md`; `test -f CONTRIBUTING.md`

---

## 1. Source-location infrastructure

- [x] Implement `FileId` newtype in `src/span.rs`
  - [ ] `pub struct FileId(pub u32);` with `Copy, Clone, PartialEq, Eq, Hash, Debug`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub struct FileId' src/span.rs`

- [x] Implement `Span` struct in `src/span.rs`
  - [ ] Fields: `file_id: FileId`, `start: u32`, `end: u32`
  - [ ] Methods: `pub fn new(file_id, start, end)`, `pub fn len(&self)`, `pub fn merge(&self, other: &Span) -> Span`
  - [ ] Derives: `Copy, Clone, PartialEq, Eq, Debug`
  - [ ] **Verify:** `cargo test --lib span::tests::span_merge_takes_outer_bounds`

- [x] Implement `SourceMap` struct in `src/span.rs`
  - [ ] Field: `files: Vec<SourceFile>` where `SourceFile { id: FileId, name: PathBuf, content: String, line_starts: Vec<u32> }`
  - [ ] Methods: `pub fn add_file(&mut self, name, content) -> FileId`, `pub fn snippet(&self, span) -> &str`, `pub fn line_col(&self, file, byte_offset) -> (u32, u32)`
  - [ ] **Verify:** `cargo test --lib span::tests::source_map_round_trip_ascii_and_utf8`

- [x] Add multi-byte UTF-8 line/column tests
  - [ ] In `span.rs` tests module: include input with em-dash and emoji; assert line/col correct
  - [ ] **Verify:** `cargo test --lib span::tests::line_col_handles_multibyte`

---

## 2. Error reporting

- [x] Define `ErrorCode` and `ErrorKind` in `src/error.rs`
  - [ ] `ErrorCode(pub u32)` newtype with associated consts `E0001..E1999` ranges (lex / syntax / resolve / type / borrow / other)
  - [ ] `ErrorKind` enum: `Lexical, Syntax, NameResolution, Type, BorrowCheck, Other`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub struct ErrorCode' src/error.rs`; `grep -q 'pub enum ErrorKind' src/error.rs`

- [x] Define `Suggestion` struct in `src/error.rs`
  - [ ] Fields: `message: String`, `replacement: Option<String>`, `span: Span`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub struct Suggestion' src/error.rs`

- [x] Define `CompileError` struct in `src/error.rs`
  - [ ] Fields: `code: ErrorCode`, `kind: ErrorKind`, `span: Span`, `message: String`, `suggestions: Vec<Suggestion>`, `notes: Vec<String>`
  - [ ] Methods: `pub fn new(code, kind, span, msg)`, `pub fn with_suggestion(self, s) -> Self`, `pub fn with_note(self, n) -> Self`
  - [ ] **Verify:** `cargo test --lib error::tests::compile_error_builder_chains`

- [x] Implement `ErrorAccumulator` in `src/error.rs`
  - [ ] Methods: `pub fn new()`, `pub fn push(&mut self, e: CompileError)`, `pub fn into_result<T>(self, ok: T) -> Result<T, Vec<CompileError>>`
  - [ ] Cap at 100 errors (silent drop after, but increment a counter)
  - [ ] Dedupe by `(code, span.file_id, span.start)`
  - [ ] **Verify:** `cargo test --lib error::tests::accumulator_caps_at_100`; `cargo test --lib error::tests::accumulator_dedupes`

- [x] Implement error pretty-printer in `src/error/render.rs`
  - [ ] `pub fn render(err: &CompileError, src: &SourceMap) -> String`
  - [ ] Render: `error[E0308]: <message>`, source snippet, caret under primary span, secondary labels, `note:` and `help:` lines
  - [ ] No color in tests; gate `termcolor` behind isatty check (tests force off via env var `NO_COLOR=1`)
  - [ ] **Verify:** `cargo test --lib error::render::tests::renders_e0308_format`

- [x] Add multi-label support to renderer
  - [ ] Render multiple `Label { span, message, primary: bool }` entries
  - [ ] Primary label shows source snippet; secondary labels reference by line number
  - [ ] **Verify:** `cargo test --lib error::render::tests::multi_label_layout`

- [x] Implement `--explain E0XXX` subcommand
  - [ ] `src/explain.rs` containing `pub fn explain(code: &str) -> Option<&'static str>`
  - [ ] Stub entries for E0080, E0133, E0277, E0308, E0369, E0382, E0425, E0433, E0499, E0502, E0503, E0505, E0599, E0608
  - [ ] Each entry: 3-paragraph string with explanation + minimal example
  - [ ] Wire into `main.rs` arg parsing
  - [ ] **Verify:** `cargo test --lib explain::tests::explain_e0308_returns_text`

---

## 3. Token enum

- [x] Define `TokenKind` enum in `src/lexer/token.rs` — keyword variants
  - [ ] 29 keyword variants: `Break, Const, Continue, Else, Enum, Extern, False, Fn, For, If, Impl, In, Let, Loop, Match, Mod, Mut, Not, Or, Pub, Return, SelfLower, SelfUpper, Struct, Trait, True, Type, Unsafe, Use, Where, While, And` (matches §2 of spec)
  - [ ] **Verify:** `cargo build`; `grep -q 'pub enum TokenKind' src/lexer/token.rs`

- [x] Add literal variants to `TokenKind`
  - [ ] `IntLiteral(u64, IntSuffix)`, `FloatLiteral(f64, FloatSuffix)`, `CharLiteral(char)`, `StringLiteral(String)`, `RawStringLiteral(String)`
  - [ ] Define `IntSuffix` and `FloatSuffix` enums in same file (variants: `I8..I64, ISize, U8..U64, USize, Unsuffixed` and `F32, F64, Unsuffixed`)
  - [ ] **Verify:** `cargo build`; `grep -q 'IntLiteral' src/lexer/token.rs`

- [x] Add identifier and operator variants to `TokenKind`
  - [ ] `Ident(String)`
  - [ ] Operator variants: `Plus, Minus, Star, Slash, Percent, EqEq, BangEq, Lt, Gt, Le, Ge, Amp, Pipe, Caret, Tilde, Shl, Shr, Eq, PlusEq, MinusEq, StarEq, SlashEq, PercentEq`
  - [ ] Punctuation: `Dot, ColonColon, LBracket, RBracket, LParen, RParen, LBrace, RBrace, Question, DotDot, DotDotEq, Arrow, FatArrow, Semi, Comma, Colon, Underscore`
  - [ ] Special: `Eof`, `Error(String)`
  - [ ] **Verify:** `cargo build`

- [x] Define `Token` struct
  - [ ] Fields: `kind: TokenKind`, `span: Span`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub struct Token' src/lexer/token.rs`

---

## 4. Scanner

- [x] Implement `Scanner` struct in `src/lexer/scan.rs`
  - [ ] Fields: `src: &'a str`, `bytes: &'a [u8]`, `pos: usize`, `file_id: FileId`
  - [ ] Methods: `pub fn new(src, file_id)`, helpers `peek`, `peek_at`, `bump`, `eat_while`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub struct Scanner' src/lexer/scan.rs`

- [x] Implement decimal int literal scanning
  - [ ] Method `Scanner::scan_int_decimal` returning `(u64, IntSuffix, Span)`
  - [ ] Handle `_` separators; reject leading `_`
  - [ ] Handle suffix parse (i8/i16/i32/i64/isize/u8/.../usize)
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::decimal_int_with_underscores_and_suffix`

- [x] Implement hex (`0x`) and binary (`0b`) int literal scanning
  - [ ] Extend `scan_int_decimal` or add `scan_int_hex` / `scan_int_bin`
  - [ ] Handle `_` separators; reject empty digit run after prefix
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::hex_and_bin_literals`

- [x] Implement float literal scanning
  - [ ] Method `Scanner::scan_float` handling `1.0`, `1.0e10`, `1.0E-3`, `.5` (rejected per spec — must have leading digit), `f32`/`f64` suffix
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::float_literal_forms`

- [x] Implement char literal scanning
  - [ ] Method `Scanner::scan_char`
  - [ ] Escapes: `\n \t \r \\ \' \" \0 \xNN \u{NNNN}`
  - [ ] Reject multi-codepoint content; reject unterminated
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::char_literal_escapes`

- [x] Implement string literal scanning (regular)
  - [ ] Method `Scanner::scan_string`
  - [ ] Same escape set as chars; allow embedded newlines
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::string_literal_escapes`

- [x] Implement raw string literal scanning
  - [ ] Method `Scanner::scan_raw_string` handling `r"..."` and `r#"..."#` with arbitrary `#` count
  - [ ] Reject mismatched `#` counts; preserve content verbatim
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::raw_string_arbitrary_hashes`

- [x] Implement line and block comment scanning
  - [ ] Method `Scanner::skip_comments`
  - [ ] `// ... \n` and `/* ... */` with proper nesting depth counter
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::nested_block_comments`

- [x] Implement doc comment scanning
  - [ ] Recognize `/// ...` (outer) and `//! ...` (inner) — emit them as `TokenKind::DocComment(String, DocStyle)` instead of dropping
  - [ ] Define `DocStyle` enum in `token.rs`: `Outer, Inner`
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::doc_comments_preserved`

- [x] Implement operator scanning with maximal munch
  - [ ] Method `Scanner::scan_operator`
  - [ ] Order: `..=` before `..` before `.`; `<<=` before `<<` before `<`; `==` before `=`; `>=` before `>`; etc.
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::operator_maximal_munch`

- [x] Implement keyword vs identifier disambiguation
  - [ ] Method `Scanner::scan_ident_or_keyword`
  - [ ] Read identifier; map against the 29 keywords table; otherwise emit `Ident`
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::keywords_take_priority_over_idents`

- [x] Wire all scanners into `Scanner::next_token` driver
  - [ ] `pub fn next_token(&mut self) -> Token`
  - [ ] Skip whitespace + comments; dispatch on first char
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::tokenizes_full_program`

- [x] Verify every token carries a `Span`
  - [ ] Add an integration test that walks `Scanner::next_token` to EOF and asserts every `Token.span.start < Token.span.end`
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::all_tokens_have_nonzero_span`

---

## 5. Lexer error recovery

- [x] Invalid character recovery
  - [ ] Emit `TokenKind::Error("invalid character: <ch>".to_string())`; advance one codepoint; continue
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::invalid_char_recovers`

- [x] Unterminated string recovery
  - [ ] Emit error spanning open-quote → EOF; continue at EOF
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::unterminated_string_recovers`

- [x] Invalid numeric literal recovery
  - [ ] On parse failure, emit error and advance past the offending run
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::invalid_numeric_recovers`

- [x] Fuzz-style robustness test
  - [ ] In tests, run `Scanner::new` on 1000 random byte sequences (PRNG-seeded) and assert no panic
  - [ ] **Verify:** `cargo test --lib lexer::scan::tests::fuzz_random_bytes_no_panic`

---

## 6. Lexer test infrastructure

- [x] Snapshot-test helper macro in `src/lexer/test_util.rs`
  - [ ] `macro_rules! lex_eq { ($input:expr, $expected:expr) => { ... } }`
  - [ ] Compare token kind list (drop spans for snapshot brevity)
  - [ ] **Verify:** `cargo test --lib lexer::test_util::tests::macro_works`

- [x] Add 30+ snapshot tests for spec §2 examples
  - [ ] One test per example in `vertex_v1_spec.md` §2
  - [ ] **Verify:** `cargo test --lib lexer:: 2>&1 | grep -c 'test result: ok' >= 1` (full suite passes)

---

## 7. AST node taxonomy

- [x] Define `NodeId` newtype in `src/ast/mod.rs`
  - [ ] `pub struct NodeId(pub u32);` with `Copy, Clone, PartialEq, Eq, Hash, Debug`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub struct NodeId' src/ast/mod.rs`

- [x] Wire arena allocator into AST
  - [ ] Add `typed-arena = "2"` to `Cargo.toml`
  - [ ] Define `pub struct Arena { ... }` wrapping per-node-type arenas
  - [ ] **Verify:** `cargo build`; `grep -q '^typed-arena' Cargo.toml`

- [x] Define `Item` enum in `src/ast/item.rs`
  - [ ] Variants: `Fn(FnDef), Struct(StructDef), Enum(EnumDef), Impl(ImplDef), Trait(TraitDef), Mod(ModDef), Use(UseDef), Const(ConstDef), Static(StaticDef), TypeAlias(TypeAliasDef)`
  - [ ] Each variant references a struct stub (fields can be added later items)
  - [ ] Every node carries `id: NodeId, span: Span`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub enum Item' src/ast/item.rs`

- [x] Define `Stmt` enum in `src/ast/stmt.rs`
  - [ ] Variants: `Let { pattern, ty, init, span, id }`, `Expr(Expr, has_semi: bool)`, `Item(Item)`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub enum Stmt' src/ast/stmt.rs`

- [x] Define `Expr` enum in `src/ast/expr.rs` — literal + path variants
  - [ ] Variants: `IntLit, FloatLit, CharLit, StrLit, BoolLit, Path(Path)`
  - [ ] `Path { segments: Vec<PathSegment> }` with `PathSegment { ident, generic_args }`
  - [ ] Every variant carries `id: NodeId, span: Span`
  - [ ] **Verify:** `cargo build`; `grep -q 'IntLit' src/ast/expr.rs`

- [x] Add operator + control-flow variants to `Expr`
  - [ ] `Unary { op, operand }, Binary { op, lhs, rhs }, Call { callee, args }, MethodCall { receiver, method, args, generic_args }, Field { receiver, name }, TupleField { receiver, idx }, Index { receiver, idx }, Cast { expr, ty }, Try { expr }`
  - [ ] **Verify:** `cargo build`

- [x] Add aggregate + literal-construction variants to `Expr`
  - [ ] `Range { start, end, inclusive }, Closure { params, body, move_kw }, StructLit { path, fields, base }, TupleLit { elems }, ArrayLit { elems }, ArrayRepeat { value, count }, Block { stmts, tail }`
  - [ ] **Verify:** `cargo build`

- [x] Add control-flow variants to `Expr`
  - [ ] `If { cond, then, else_branch }, Loop { body }, While { cond, body }, For { pat, iter, body }, Match { scrutinee, arms }, Return { value }, Break { label, value }, Continue { label }`
  - [ ] Define `MatchArm { pattern, guard, body, span, id }`
  - [ ] **Verify:** `cargo build`

- [x] Define `Type` enum in `src/ast/ty.rs`
  - [ ] Variants: `Path(Path), Ref { mutable, ty, span, id }, Ptr { mutable, ty }, Array { elem, len }, Slice { elem }, Tuple(Vec<Type>), Fn { params, ret }, Infer`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub enum Type' src/ast/ty.rs`

- [x] Define `Pattern` enum in `src/ast/pat.rs`
  - [ ] Variants: `Wild, Ident { name, mutable, sub }, Literal(Lit), Range { start, end, inclusive }, Tuple(Vec<Pattern>), Struct { path, fields, rest }, TupleStruct { path, elems }, Ref { mutable, pattern }, Or(Vec<Pattern>)`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub enum Pattern' src/ast/pat.rs`

- [x] Define `Generics` and `WhereClause` in `src/ast/generics.rs`
  - [ ] `Generics { params: Vec<TypeParam>, where_clause: Option<WhereClause> }`
  - [ ] `TypeParam { name, bounds: Vec<TraitBound> }`
  - [ ] `WhereClause { predicates: Vec<WherePred> }`
  - [ ] `TraitBound { path, generic_args }`
  - [ ] **Verify:** `cargo build`; `grep -q 'pub struct Generics' src/ast/generics.rs`

---

## 8. Parser foundation

- [x] Implement `Parser` struct in `src/parser/mod.rs`
  - [ ] Fields: `tokens: Vec<Token>`, `pos: usize`, `errors: ErrorAccumulator`
  - [ ] Methods: `pub fn new(tokens)`, `peek`, `peek_at`, `bump`, `eat`, `expect`
  - [ ] `eat(kind) -> bool` (advance if match), `expect(kind) -> Result<Token, CompileError>`
  - [ ] **Verify:** `cargo test --lib parser::tests::peek_and_bump_basics`

- [x] Add error-recovery sync points
  - [ ] Method `Parser::recover_to_sync` that advances until next `;`, `}`, or item-start keyword
  - [ ] Used by `Parser::expected_token_error` flow
  - [ ] **Verify:** `cargo test --lib parser::tests::recovery_advances_past_garbage`

---

## 9. Expression parser

- [x] Parse literal expressions
  - [ ] Methods `Parser::parse_int_lit`, `parse_float_lit`, `parse_char_lit`, `parse_str_lit`, `parse_bool_lit`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::literal_expressions`

- [ ] Parse path expressions
  - [ ] Method `Parser::parse_path` handling `a::b::c` and `Type::<T>::method` turbofish
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::path_with_turbofish`

- [x] Parse parenthesized + tuple + unit
  - [ ] Disambiguate `(expr)` vs `(a, b)` vs `(x,)` vs `()`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::paren_tuple_unit`

- [x] Parse unary prefix expressions
  - [ ] `-`, `not`, `*` (deref), `&`, `&mut`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::unary_prefix`

- [x] Pratt parser for binary operators
  - [ ] Precedence table: `*, /, % > +, - > <<, >> > & > ^ > | > comparisons > and > or`
  - [ ] Comparison non-associative (reject `a < b < c`)
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::operator_precedence`; `cargo test --lib parser::expr::tests::comparison_non_associative_rejected`

- [x] Parse function call + method call + field access
  - [ ] `f(args)`, `x.method(args)`, `x.field`, `x.0`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::call_method_field`

- [x] Parse indexing + cast + try
  - [ ] `x[i]`, `x as T`, `x?`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::index_cast_try`

- [x] Parse range expressions
  - [ ] `a..b`, `a..=b`, `a..`, `..b`, `..`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::range_forms`

- [x] Parse closure expressions
  - [ ] `|params| body`, `move |params| body`, `|x: i32| -> i32 { body }`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::closure_forms`

- [ ] Parse struct literal expressions
  - [ ] `Path { field: val, .. }` with optional base expression
  - [ ] Disambiguate from block expressions in `if`/`while` heads (per Rust convention)
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::struct_literal`

- [x] Parse array literal expressions
  - [ ] `[a, b, c]` and `[value; count]`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::array_literal_and_repeat`

- [x] Parse block expressions
  - [ ] `{ stmts; tail_expr_optional }`
  - [ ] Block as last-stmt-without-semi → block-typed; otherwise unit
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::block_trailing_expr`

- [x] Parse if/else expressions
  - [ ] `if cond { a } else if cond2 { b } else { c }`
  - [ ] Non-block branches not allowed (must be `{ ... }`)
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::if_else_chain`

- [x] Parse loop / while / for expressions
  - [ ] `loop { body }`, `while cond { body }`, `for pat in iter { body }`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::loop_while_for`

- [x] Parse match expressions
  - [ ] `match scrut { pat if guard => expr, ... }`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::match_basic`

- [x] Parse return / break / continue
  - [ ] `return value?`, `break 'label value?`, `continue 'label?`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::return_break_continue`

---

## 10. Statement parser

- [ ] Parse `let` statements
  - [ ] `let pat: Type = expr;`, `let pat = expr;`, `let pat: Type;` (init-less)
  - [ ] **Verify:** `cargo test --lib parser::stmt::tests::let_forms`

- [x] Parse expression statements with semicolon-significance
  - [ ] Trailing `;` produces `Stmt::Expr(expr, true)`; without, `Stmt::Expr(expr, false)`
  - [ ] **Verify:** `cargo test --lib parser::stmt::tests::semicolon_significance`

- [ ] Parse item statements (nested fn / struct inside a block)
  - [ ] **Verify:** `cargo test --lib parser::stmt::tests::nested_item_in_block`

- [x] Block trailing-expression-as-value semantics
  - [ ] Last statement without `;` is the block's value; otherwise unit
  - [ ] **Verify:** `cargo test --lib parser::stmt::tests::block_value_semantics`

---

## 11. Parser error recovery

- [x] Insert placeholder `Expr::Error(NodeId, Span)` and continue
  - [ ] Add `Error` variant to `Expr` enum with `id, span`
  - [ ] On parse failure, push `CompileError`, return placeholder, sync to next stmt boundary
  - [ ] **Verify:** `cargo test --lib parser::tests::error_node_recovery`

- [x] "Expected one of: ..." messages
  - [ ] On `expect` mismatch, build message listing candidate follow-set
  - [ ] **Verify:** `cargo test --lib parser::tests::expected_message_lists_candidates`

- [ ] End-to-end recovery test
  - [ ] Input `let x = ; let y = 10;` produces one error and a valid AST for `let y`
  - [ ] **Verify:** `cargo test --lib parser::tests::recovery_let_garbage_let_valid`

---

## 12. Item parsers — functions

- [x] Parse plain function items
  - [ ] `fn name(params) -> ret_ty { body }`
  - [ ] Param: `name: Type` (no patterns yet)
  - [ ] **Verify:** `cargo test --lib parser::item::tests::plain_fn`

- [x] Add modifiers: `const`, `unsafe`, `extern "ABI"`
  - [ ] `const fn`, `unsafe fn`, `extern "C" fn`, combinations
  - [ ] **Verify:** `cargo test --lib parser::item::tests::fn_modifiers`

- [ ] Add visibility: `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`
  - [ ] `Visibility` enum in `src/ast/item.rs`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::fn_visibility`

- [ ] Add attribute parsing
  - [ ] `#[no_mangle]`, `#[inline]`, `#[derive(Clone, Debug)]`
  - [ ] Generic AST node `Attribute { path, args, span }`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::fn_attributes`

- [x] Add self parameters
  - [ ] `self`, `&self`, `&mut self`, `self: Box<Self>`, `self: Rc<Self>`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::self_params`

- [x] Add generics and where-clauses to function items
  - [ ] `fn foo<T, U>(x: T) -> U where T: Clone + Debug`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::fn_generics_and_where`

---

## 13. Item parsers — types

- [x] Parse normal struct items
  - [ ] `struct Name<T> { field: Ty, pub field2: Ty }`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::struct_normal`

- [x] Parse tuple + unit struct items
  - [ ] `struct Name<T>(T, T);`, `struct Unit;`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::struct_tuple_unit`

- [ ] Add `#[repr(...)]` parsing on structs
  - [ ] `#[repr(C)]`, `#[repr(transparent)]` (AST only — no semantic checks)
  - [ ] **Verify:** `cargo test --lib parser::item::tests::struct_repr`

- [x] Parse enum items with all variant kinds
  - [ ] Unit variants, tuple variants, struct variants
  - [ ] Explicit discriminants `Foo = 5,`
  - [ ] Generic enums `enum Result<T, E> { Ok(T), Err(E) }`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::enum_all_variant_kinds`

- [x] Parse trait items
  - [ ] `trait Name<T>: Super + Super2 { fn method(&self); type Item; const MAX: usize; }`
  - [ ] Default method bodies allowed
  - [ ] **Verify:** `cargo test --lib parser::item::tests::trait_with_assoc`

- [ ] Parse inherent and trait impls
  - [ ] `impl<T> Name<T> { ... }`
  - [ ] `impl<T> Clone for Name<T> where T: Clone { ... }`
  - [ ] Associated type/const bindings inside impl bodies
  - [ ] **Verify:** `cargo test --lib parser::item::tests::impl_inherent_and_trait`

---

## 14. Item parsers — modules + use

- [x] Parse `mod foo;` (file-loaded) vs `mod foo { ... }` (inline)
  - [ ] Both forms produce `ModDef` with `kind: ModKind::External | ModKind::Inline(items)`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::mod_external_vs_inline`

- [x] Parse `use` items — simple paths
  - [ ] `use foo::bar;`, `use foo::bar as baz;`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::use_simple_and_alias`

- [x] Parse `use` items — nested + glob
  - [ ] `use { a, b::c, d::{e, f} };`, `use foo::*;`, `pub use bar;`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::use_nested_glob_pub`

---

## 15. Item parsers — const/static/type-alias

- [x] Parse `const` items
  - [ ] `const NAME: T = expr;`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::const_item`

- [x] Parse `static` and `static mut` items
  - [ ] `static NAME: T = expr;`, `static mut NAME: T = expr;`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::static_item`

- [ ] Parse type-alias items
  - [ ] `type Alias<T> = ConcreteTy;`
  - [ ] **Verify:** `cargo test --lib parser::item::tests::type_alias`

---

## 16. Type parser

- [ ] Parse path types with generic args
  - [ ] `Vec<T>`, `HashMap<String, Vec<i32>>`
  - [ ] **Verify:** `cargo test --lib parser::ty::tests::path_with_generics`

- [ ] Parse reference types
  - [ ] `&T`, `&mut T`, `&'static str` (lifetime is parsed but ignored semantically — Stage 0 simplification)
  - [ ] **Verify:** `cargo test --lib parser::ty::tests::ref_types`

- [ ] Parse raw pointer types
  - [ ] `*const T`, `*mut T`
  - [ ] **Verify:** `cargo test --lib parser::ty::tests::raw_ptr_types`

- [ ] Parse slice + array types
  - [ ] `&[T]`, `[T; N]` (where `N` is a const expr)
  - [ ] **Verify:** `cargo test --lib parser::ty::tests::slice_and_array_types`

- [ ] Parse tuple types
  - [ ] `(T, U, V)`, `()` (unit)
  - [ ] **Verify:** `cargo test --lib parser::ty::tests::tuple_types`

- [ ] Parse function types
  - [ ] `fn(T, U) -> V`, `extern "C" fn(...)`
  - [ ] **Verify:** `cargo test --lib parser::ty::tests::fn_types`

- [ ] Parse associated-type projection
  - [ ] `<T as Iterator>::Item`
  - [ ] **Verify:** `cargo test --lib parser::ty::tests::assoc_projection`

- [ ] Parse infer placeholder
  - [ ] `_` as a type
  - [ ] **Verify:** `cargo test --lib parser::ty::tests::infer_placeholder`

---

## 17. Pattern parser

- [ ] Parse literal patterns
  - [ ] Int, float, char, string, bool literals
  - [ ] **Verify:** `cargo test --lib parser::pat::tests::literal_patterns`

- [ ] Parse ident patterns + `mut` + `@` sub-binding
  - [ ] `x`, `mut x`, `name @ Some(_)`
  - [ ] **Verify:** `cargo test --lib parser::pat::tests::ident_patterns`

- [ ] Parse tuple patterns
  - [ ] `(a, b, _)`
  - [ ] **Verify:** `cargo test --lib parser::pat::tests::tuple_patterns`

- [ ] Parse struct patterns
  - [ ] `Point { x, y: b, .. }`
  - [ ] **Verify:** `cargo test --lib parser::pat::tests::struct_patterns`

- [ ] Parse tuple-struct + enum patterns
  - [ ] `Some(x)`, `Color(r, g, b)`
  - [ ] **Verify:** `cargo test --lib parser::pat::tests::tuple_struct_patterns`

- [ ] Parse range patterns
  - [ ] `0..=100`, `'a'..='z'`
  - [ ] **Verify:** `cargo test --lib parser::pat::tests::range_patterns`

- [ ] Parse reference patterns
  - [ ] `&x`, `&mut x`, `ref x`, `ref mut x`
  - [ ] **Verify:** `cargo test --lib parser::pat::tests::ref_patterns`

- [ ] Parse or-patterns and wildcard
  - [ ] `Some(x) | None`, `_`
  - [ ] **Verify:** `cargo test --lib parser::pat::tests::or_and_wildcard`

---

## 18. Built-in syntax recognition

- [ ] Add `vec!` macro recognition
  - [ ] Parse `vec![a, b, c]` → `Expr::VecLiteral(Vec<Expr>)`
  - [ ] Parse `vec![x; n]` → `Expr::VecRepeat(Box<Expr>, Box<Expr>)`
  - [ ] Reject any other `<ident>!` form with "user macros not supported in v1"
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::vec_macro`; `cargo test --lib parser::expr::tests::user_macro_rejected`

- [ ] Add `print` / `println` / `eprint` / `eprintln` recognition
  - [ ] Parse as `Expr::BuiltinCall { func: BuiltinFn::Print|Println|EPrint|EPrintln, args }`
  - [ ] Define `BuiltinFn` enum in `src/ast/expr.rs`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::print_family`

- [ ] Add `format` recognition
  - [ ] `format("...", args)` → `Expr::BuiltinCall { func: BuiltinFn::Format, args }`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::format_call`

- [ ] Add `assert` / `debug_assert` / `panic` recognition
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::assert_family`

- [ ] Add `size_of` / `align_of` recognition
  - [ ] `size_of::<T>()`, `align_of::<T>()` → `Expr::BuiltinCall { func, generic_args }`
  - [ ] **Verify:** `cargo test --lib parser::expr::tests::size_align_of`

- [ ] Validate `#[derive(...)]` allow-list
  - [ ] Allowed: `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default`
  - [ ] Any other derive name → error E0XYZ "user-defined derives not supported in v1"
  - [ ] **Verify:** `cargo test --lib parser::item::tests::derive_allow_list`

---

## 19. Test infrastructure expansion

- [ ] Parser snapshot helper macro in `src/parser/test_util.rs`
  - [ ] `macro_rules! parse_eq { ($input:expr, $expected_debug:expr) => { ... } }`
  - [ ] **Verify:** `cargo test --lib parser::test_util::tests::macro_works`

- [ ] Error golden-file harness in `tests/errors/`
  - [ ] `tests/errors/inputs/*.vx` source files; `tests/errors/expected/*.txt` golden output; runner compares stripped of trailing whitespace
  - [ ] Initial corpus: 5 examples covering parse errors
  - [ ] **Verify:** `cargo test --test errors`

- [ ] Add 100+ parser tests covering every form in spec §7-§8
  - [ ] One per construct; can be a single file `tests/parser_corpus.rs` with one #[test] per construct
  - [ ] **Verify:** `cargo test --test parser_corpus 2>&1 | grep -q 'test result: ok\. \([1-9][0-9][0-9]\|[2-9][0-9]\) passed'`

- [ ] Set CI fmt + clippy gate to deny warnings
  - [ ] In `.github/workflows/ci.yml`, ensure `cargo clippy --all-targets -- -D warnings` is present
  - [ ] **Verify:** `grep -q -- '-D warnings' .github/workflows/ci.yml`

---

## 20. Phase 1.5 wrap-up

- [ ] AST pretty-printer in `src/ast/printer.rs`
  - [ ] `pub fn print(node: &impl PrintNode) -> String`
  - [ ] Round-trip: parse → print → parse → ast-equal (modulo NodeIds)
  - [ ] **Verify:** `cargo test --lib ast::printer::tests::round_trip_preserves_structure`

- [ ] Document Phase 1.5 boundary
  - [ ] Append a section to `CHANGELOG.md` listing what Phase 1.5 covers
  - [ ] **Verify:** `grep -q 'Phase 1.5' CHANGELOG.md`

- [ ] Smoke test: parse the spec's largest example end-to-end
  - [ ] Add `tests/integration/parse_full_example.rs` parsing a 200-line synthetic Vertex program (covers fn, struct, enum, impl, trait, generics, patterns, expressions)
  - [ ] Asserts: 0 errors emitted; ast tree size > 100 nodes
  - [ ] **Verify:** `cargo test --test parse_full_example`

---

<!--
================================================================================
Out of scope for autonomous runner
================================================================================

Everything below is Stage 0 Phase 2+ and beyond. Each item requires design
judgment, cross-module reasoning, or research the runner cannot perform
safely. They remain visible as a roadmap but are HTML-commented so
Get-TodoItems will not queue them. To promote one back into scope: copy it
above this comment block, decompose into single-commit deliverables with
concrete Verify gates, and trim aspirational phrasing.

## A. Stage 0 Phase 2 — Type System

- File-system-based module loader (resolve `mod foo;` → file path)
- Scope hierarchy with arena-allocated Scope nodes
- Import resolution including prelude injection, glob, re-export, alias
- Visibility checking across module boundaries
- Internal type IR (`Ty` enum, `IntTy`/`UintTy`/`FloatTy`, `AdtDef`, `FnSig`, `Region`, `InferVar`)
- Hindley-Milner constraint generation walking HIR
- Unification with union-find, occurs check, generic-arg recursion
- Numeric-literal defaulting (int → i32, float → f64)
- Reference and deref coercions
- Generic type instantiation (substitution + inference + turbofish)
- Trait definition table with super-trait collection + impl coherence
- Method resolution with auto-ref/auto-deref + ambiguity errors
- Trait bound checking with E0277-style messages
- Associated-type resolution + projection normalization
- Derive expansion (Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)
- String-indexing prohibition (E0608) with help suggestion + UTF-8 note
- Lifetime constraint generation (Vertex simplified rules)
- Lifetime constraint solver (transitive closure + contradiction detection)
- Const evaluation: `ConstValue` IR, compile-time interpreter, recursion + heap rejection

## B. Stage 0 Phase 3 — Closures and Iterators

- Free-variable analysis classifying each capture as ImmBorrow / MutBorrow / Move
- Closure type assignment (Fn / FnMut / FnOnce hierarchy)
- Closure type checking with bound propagation
- `Iterator` trait + `Range<...>` impls + `for` loop desugaring
- `IntoIterator` trait + blanket + `Vec` / `&Vec` / `&mut Vec` impls
- Iterator combinators (map, filter, fold, take, skip, zip, chain, ...)
- `FromIterator` trait + `Vec` / `String` / `HashMap` impls
- HIR lowering pass with full desugaring (for-loop, if-let, ?, vec!, format, assert)

## C. Stage 0 Phase 4 — Safety Analysis

- MIR data structures (BasicBlock, Statement, Place, Rvalue, Terminator)
- HIR → MIR lowering for all expression and statement forms
- Drop elaboration with scope tracking and reverse-declaration order
- Drop flags for conditional initialization
- Unwind paths with cleanup BBs + Resume terminator
- Liveness analysis (backward data-flow)
- Borrow tracking with `BorrowSet` and projection-aware overlap
- Borrow rule enforcement: E0499, E0502, E0503, E0505 with multi-label diagnostics
- Move checking with partial-move support
- Borrow-check error messages on par with Rust quality (20 canonical examples)

## D. Stage 0 Phase 5 — Code Generation

- Codegen driver emitting one C translation unit per crate
- Name mangling: `vertex_<crate>_<module>_<local>_<generics>`
- Type translation: primitives, refs, raw ptrs, tuples, arrays, slices, &str, String, ADTs, Vec, fn types
- Function codegen from MIR
- Debug-mode overflow checks via `__builtin_*_overflow`
- Release-mode wrapping arithmetic with `-fwrapv`
- Explicit checked/saturating/wrapping methods on integer types
- Mono-item collection (worklist-based with dedup)
- Mono-item codegen with cache and unused-instantiation elimination
- Closure struct + call generation (Fn / FnMut / FnOnce)
- Static dispatch for trait methods (no vtable in v1)

## E. Stage 0 Phase 6 — Runtime + Linker

- `vertex_runtime.h` / `vertex_runtime.c` (panic, alloc, dealloc, realloc)
- String runtime helpers (UTF-8 invariant enforcement)
- Vec runtime helpers (push, pop, capacity growth)
- HashMap runtime helpers (FNV1a or SipHash, open-addressing)
- Linker integration (cc-rs or direct invocation)
- Symbol-uniqueness stress test (10k mangled names)

## F. Stage 0 Phase 7 — Optimization Pipeline

- Constant folding pass over MIR
- Dead-code elimination
- Inlining of small functions
- Dead-store elimination
- Common-subexpression elimination
- LLVM backend (alternative to C backend; out-of-scope for v1)

## G. Stage 1+ Language Features (post-bootstrap)

- if-let, while-let with full semantics
- Macros (declarative `macro_rules!`-style)
- Async / await
- Const generics
- Generic associated types
- Trait objects (`dyn Trait`)
- Specialization
- Higher-ranked trait bounds
- Variadic generics

## H. Cross-cutting design / documentation

- Threat model
- Stability + backward-compatibility policy
- Editions strategy
- Stdlib design proposals beyond Stage 0 prelude
- Compiler-internal architecture diagrams
- Performance benchmarks vs rustc / gcc
- Security audit of generated C code

================================================================================
End of out-of-scope appendix
================================================================================
-->

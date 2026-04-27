# Plan: implement-keyword-vs-identifier-disambiguation

## Goal
Add a `Scanner::scan_ident_or_keyword` method that reads an identifier-shaped run and emits the matching keyword `TokenKind` if the lexeme is in the reserved-word table, otherwise emits `TokenKind::Ident(String)`, with a unit test that pins keyword-priority for every existing keyword variant.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs` (which already imports `TokenKind` and `Span`), add a public method:
   `pub fn scan_ident_or_keyword(&mut self) -> Option<(TokenKind, Span)>`
2. Inside the method:
   - Snapshot `start = self.pos`.
   - Peek the first byte. If it is not ASCII `a..=z` or `A..=Z`, return `None` (do not advance). A bare `_` or `_foo` is out of scope for this method — `Underscore` and any future `_`-prefixed identifier handling live in the `next_token` driver; this is consistent with the existing module's "scanner returns `None` if it isn't its turn" contract.
   - Advance one byte, then `eat_while(|b| b.is_ascii_alphanumeric() || b == b'_')` to consume the rest of the identifier run.
   - Slice the lexeme: `let lex = &self.src[start..self.pos];`
   - Match `lex` against a `&[(&str, TokenKind)]` keyword table covering every keyword variant currently defined in `TokenKind` (32 entries):
     `("and", And)`, `("break", Break)`, `("const", Const)`, `("continue", Continue)`, `("else", Else)`, `("enum", Enum)`, `("extern", Extern)`, `("false", False)`, `("fn", Fn)`, `("for", For)`, `("if", If)`, `("impl", Impl)`, `("in", In)`, `("let", Let)`, `("loop", Loop)`, `("match", Match)`, `("mod", Mod)`, `("mut", Mut)`, `("not", Not)`, `("or", Or)`, `("pub", Pub)`, `("return", Return)`, `("self", SelfLower)`, `("Self", SelfUpper)`, `("struct", Struct)`, `("trait", Trait)`, `("true", True)`, `("type", Type)`, `("unsafe", Unsafe)`, `("use", Use)`, `("where", Where)`, `("while", While)`. Use a `match lex { ... }` over string literals (or a small linear scan over a const slice) — both are O(k) and trivial; a `match` reads cleanest.
   - If matched, return `Some((kw_kind, Span::new(self.file_id, start as u32, self.pos as u32)))`.
   - Otherwise return `Some((TokenKind::Ident(lex.to_string()), Span::new(self.file_id, start as u32, self.pos as u32)))`.
3. The keyword-table `match` returns owned `TokenKind` values directly (each keyword variant is a unit variant — no `.clone()` cost). Ident allocation is a single `String::from(lex)`.
4. Add `#[test] fn keywords_take_priority_over_idents` inside the existing `mod tests` block in `vertex_stage0/src/lexer/scan.rs`. Drive a table of `(input, expected_kind)` covering:
   - Every one of the 32 keyword strings → its matching keyword variant (e.g. `("fn", TokenKind::Fn)`, `("self", TokenKind::SelfLower)`, `("Self", TokenKind::SelfUpper)`, `("true", TokenKind::True)`, `("and", TokenKind::And)`).
   - Plain identifiers that are NOT keywords: `("foo", Ident("foo"))`, `("Foo", Ident("Foo"))`, `("foo_bar", Ident("foo_bar"))`, `("x1", Ident("x1"))`, `("FOO", Ident("FOO"))`, `("fnord", Ident("fnord"))` (a keyword prefix but not the whole lexeme — proves the longest run is consumed before lookup), `("self_", Ident("self_"))`, `("Self2", Ident("Self2"))`, `("returnn", Ident("returnn"))` (extra trailing char defeats the keyword match).
   - For each happy case assert the returned kind equals the expected variant, `span.start == 0`, `span.end as usize == input.len()`, `s.pos == input.len()`, and `span.file_id` matches the constructor argument.
   - Boundary: feed `"fn x"` and assert the call returns `TokenKind::Fn` with `span.end == 2` and `s.pos == 2` (whitespace and the trailing `x` are NOT consumed — driver's job to call again).
   - Negative inputs that must return `None` and leave `pos == 0`: `"_"`, `"_foo"`, `"1abc"`, `""`, `" foo"`, `"123"`, `"!"`. (Leading underscore is rejected per spec §40, which defines `identifier = letter { letter | digit | "_" }`.)
5. Do not wire `scan_ident_or_keyword` into a `next_token` driver; that's the separate `wire-all-scanners-into-scanner-next-token-driver` item.

## Files
- `vertex_stage0/src/lexer/scan.rs` — add `pub fn scan_ident_or_keyword` method on `Scanner<'a>`, add `#[test] fn keywords_take_priority_over_idents` in the existing `mod tests`.

## Risks
- **Keyword-table drift.** If a future todo adds a new keyword variant to `TokenKind` (e.g. `Defer`, `Static` from the spec BNF) but forgets to update this table, that lexeme will silently become an `Ident`. Mitigated by using a `match` (not `HashMap`) so the table is immediately greppable, and by the test enumerating every keyword variant — adding a variant without test coverage will be obvious in later snapshot tests.
- **Leading-underscore policy.** Spec §40 says identifiers start with a letter only. Rust convention allows `_foo`. This plan follows the spec literally — `_foo` returns `None` from this method. If the parser later needs `_foo` to be an identifier, the `next_token` driver can either (a) call a separate `scan_underscore_ident` or (b) extend the start-byte predicate here. Pinning the strict-spec behavior in a test now lets a future change be deliberate.
- **String allocation per identifier.** `Ident(String)` forces a heap allocation for every identifier token. This matches the existing `TokenKind` shape (already `Ident(String)`), so it's not a regression — but it will need to become a `Symbol`/intern handle before parser perf matters. Out of scope here.
- **UTF-8 identifiers.** The spec's `letter` rule is ASCII-only, and `is_ascii_alphanumeric` enforces that. A leading non-ASCII byte (e.g. `é`) will fail the start check and return `None`, leaving `pos == 0`. Safe.
- **Keyword as prefix of longer ident** (e.g. `fnord`, `returnn`). The maximal-munch loop consumes the whole run *before* the table lookup, so the lookup sees `"fnord"` (not `"fn"`) and correctly returns `Ident`. The test pins this.

## Prereqs
- add-identifier-and-operator-variants-to-tokenkind
- define-tokenkind-enum-in-src-lexer-token-rs-keyword-variants
- implement-scanner-struct-in-src-lexer-scan-rs

(All three already appear committed — `TokenKind` has the 32 keyword variants plus `Ident(String)`, and `Scanner` exists in `src/lexer/scan.rs`. Listed for completeness so the runner's dependency graph is correct.)

## Verify
```
cargo test --lib -p vertex_stage0 lexer::scan::tests::keywords_take_priority_over_idents
cargo build -p vertex_stage0
```

## Assumptions
- Method signature is `pub fn scan_ident_or_keyword(&mut self) -> Option<(TokenKind, Span)>`, matching the `Option<(value, Span)>` shape every other scanner method in this file already uses (`scan_char`, `scan_string`, `scan_operator`, etc.).
- The "29 keywords" wording in the bundled todo is taken from `plan.md`'s prose, but `TokenKind` actually defines 32 keyword variants today (it includes the word-operators `and`/`or`/`not` and both `self`/`Self`). The implementation maps every one of those 32 — fewer would leave gaps; more would not compile.
- `defer` and `static` from the spec BNF (`vertex_v1_spec.md` line 3170) are NOT mapped here because no `Defer`/`Static` `TokenKind` variants exist yet. They will become `Ident("defer")` / `Ident("static")` until a future item adds the variants. This is the correct conservative behavior for now.
- Identifier start-byte is strict ASCII letter (`a-zA-Z`) per spec §40. Leading `_` is rejected and the standalone `_` token is `Underscore` — produced by a different scanner path (the future `next_token` driver).
- Identifier continuation bytes are `a-zA-Z0-9_` per spec §40.
- The keyword table is implemented as a `match` on `&str` rather than a `phf` map or `HashMap`; with 32 entries the linear `match` is faster and has no dependencies.
- `Ident` payload is `String` (owned). Interning is a future-phase concern.
- Verify uses `-p vertex_stage0` because the workspace root has no library; this matches the existing convention used by the operator-scanning plan (`.claude/plans/implement-operator-scanning-with-maximal-munch.md`).

## Blockers
Blockers: none

## Summary
Adds keyword-vs-identifier disambiguation to `Scanner` via a longest-run scan plus a 32-entry keyword table, with a table-driven unit test that pins keyword priority over identifiers and the strict-ASCII-letter-start identifier rule from the spec.

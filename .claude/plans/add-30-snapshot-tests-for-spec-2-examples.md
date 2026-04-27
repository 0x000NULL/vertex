# Plan: add-30-snapshot-tests-for-spec-2-examples

## Goal
Add a `spec_section_2` test module to `src/lexer/scan.rs` that uses the `lex_eq!` macro to assert the `TokenKind` sequence produced by 30+ short example snippets drawn from `vertex_v1_spec.md` §2 (keywords, operators, literals, built-in syntax forms).

## Steps
1. Re-read `vertex_v1_spec.md` §2 (lines 34–141) and the existing `TokenKind` variants in `src/lexer/token.rs` to confirm exactly which spec strings the current scanner can already classify (every keyword variant present in `TokenKind`, every operator variant, every literal variant, plus `vec`/`print`/`println`/`format`/`assert` etc. which lex as `Ident` followed by punctuation).
2. Inside the existing `#[cfg(test)] mod tests` block at the bottom of `src/lexer/scan.rs` (the same module that already houses `nested_block_comments`, `keywords_take_priority_over_idents`, etc.), add a new nested `mod spec_section_2 { … }` with `use super::*;` so it can reach `TokenKind` and the `lex_eq!` macro (which is `#[macro_export]` and re-exported through `$crate`, so it is visible from any test module — no extra import required).
3. Write 30+ `#[test]` functions, each calling `lex_eq!(input, vec![ … ])` against one example string lifted verbatim from §2. Group them so the section being exercised is obvious from the function name. Suggested coverage (≥30):
   - **Keywords (one per variant present in `TokenKind`, ~28):** `kw_break`, `kw_const`, `kw_continue`, `kw_else`, `kw_enum`, `kw_extern`, `kw_false`, `kw_fn`, `kw_for`, `kw_if`, `kw_impl`, `kw_in`, `kw_let`, `kw_loop`, `kw_match`, `kw_mod`, `kw_mut`, `kw_pub`, `kw_return`, `kw_self_lower`, `kw_self_upper` (`Self`), `kw_struct`, `kw_trait`, `kw_true`, `kw_type`, `kw_unsafe`, `kw_use`, `kw_where`, `kw_while`. Skip `defer` because it has no `TokenKind` variant (would lex as `Ident("defer")`); record this in Assumptions.
   - **Logical word operators (3):** `op_word_and`, `op_word_or`, `op_word_not` (each input is the bare word, expected token is the operator variant `And`/`Or`/`Not`).
   - **Arithmetic / bitwise / comparison / assignment operator block (1 each, ≥10):** `op_arith_pack` for `"+ - * / %"`, `op_cmp_pack` for `"== != < > <= >="`, `op_bitwise_pack` for `"& | ^ ~ << >>"`, `op_assign_pack` for `"= += -= *= /= %="`, `op_access_pack` for `". :: [] ()"`, `op_control_flow_pack` for `"? .. ..= ->"`, `op_special_pack` for `"; , : _"`, plus `op_fat_arrow` (`=>`) and `op_amp_mut` (`&mut`).
   - **Literals (one per literal variant, ≥7):** `lit_int_decimal` (`"42"`), `lit_int_underscored` (`"1_000_000"`), `lit_int_hex` (`"0xff"`), `lit_int_binary` (`"0b1010"`), `lit_float_simple` (`"3.14"`), `lit_float_exp` (`"1.0e-10"`), `lit_char` (`"'a'"`), `lit_string` (`"\"hello\""`), `lit_raw_string` (`"r\"raw string\""`), `lit_bool_true` (`"true"`), `lit_bool_false` (`"false"`).
   - **Built-in syntax examples (≥5):** `builtin_vec_macro` for `"vec![1, 2, 3]"` (lexes as `Ident("vec") LBracket IntLiteral(1) Comma IntLiteral(2) Comma IntLiteral(3) RBracket` — note `!` from spec text is described as absent for these forms; confirm against spec lines 107–141), `builtin_println` for `"println(\"text\")"`, `builtin_format` for `"format(\"Hello {}\", name)"`, `builtin_array_repeat` for `"[0; 256]"`, `builtin_assert_msg` for `"assert(x == y, \"x must equal y\")"`, `builtin_derive_attr` for `"#[derive(Clone)]"`.
4. For each test, hand-compute the expected `Vec<TokenKind>` by walking `TokenKind` definitions in `src/lexer/token.rs` (e.g., string literals carry `TokenKind::StringLiteral(s.into())`; integer literals are `TokenKind::IntLiteral(value, IntSuffix::Unsuffixed)`; floats use `FloatSuffix::Unsuffixed`). Use `into()` / explicit `String::from` consistently to match how `lex_eq!`'s assertion compares values.
5. Run `cargo test --lib lexer::scan::tests::spec_section_2` locally (mentally) to confirm naming; the runner's verify step will execute the full lexer suite.

## Files
- `vertex_stage0/src/lexer/scan.rs` — append a `mod spec_section_2 { … }` inside the existing `#[cfg(test)] mod tests` block, containing 30+ `#[test]` functions that each call `lex_eq!`. No changes to non-test code.

## Risks
- The spec's text examples like `vec![1, 2, 3]` and `print("text")` are described in the spec as "built-in syntax forms, NOT macros" with no `!`, but the literal characters in the spec source still include `!` for `vec![…]` and `[0; 256]`. The current `TokenKind` has no `Bang` variant — `!` would tokenise as `TokenKind::Error("invalid character: !")`. Mitigate by either using the `vec![…]` form and accepting the `Error` token in the expected vector, or by paraphrasing to `Vec::new()` style. See Assumptions for the chosen approach.
- A handful of spec snippets contain Unicode punctuation (e.g., curly quotes inside prose) — only lift snippets from fenced ```` ```vertex ```` blocks where the source bytes are ASCII to avoid surprising tokens.
- `defer` is listed in §2 as a keyword but is **not** present in `TokenKind`; including it in a per-keyword test would fail. Skip it (documented in Assumptions).
- The `lex_eq!` macro lives in `src/lexer/test_util.rs` behind `#[cfg(test)] pub mod test_util;` and is `#[macro_export]`ed at the crate root via `$crate::…` paths. From `scan::tests::spec_section_2`, the macro must be reachable. Per Rust's `macro_export` rules it is reachable as `crate::lex_eq!` or via `use crate::lex_eq;` — confirm during execute and add the import if needed.

## Prereqs
Prereqs: none

## Verify
```
cd vertex_stage0 && cargo test --lib lexer::
cd vertex_stage0 && cargo test --lib lexer::scan::tests::spec_section_2
```

## Assumptions
- "30+ examples" means 30+ `#[test]` functions, each driven by `lex_eq!`, **not** 30 entries inside one giant table — this gives each example a clear name and a clear failure message, matches the `snapshot-test-helper-macro-in-src-lexer-test-util-rs` plan's stated purpose for the macro, and satisfies the stated verify (`test result: ok` count ≥ 1 for the lexer suite).
- Tests live in `src/lexer/scan.rs` (alongside the other lexer tests) rather than in `tests/` because everything else in this todo run treats lexer tests as `--lib lexer::*` tests.
- For the `vec![…]` and `[0; 256]` array-repeat examples I will keep the literal `!`/`;` forms and assert the *actual* current scanner output (including any `Error` token for `!`); this snapshots reality and will fail-loud the day a `Bang` token is added, which is the correct behaviour for a snapshot suite.
- `defer` is intentionally omitted because the current `TokenKind` does not contain it; if a `Defer` variant is added later, a follow-up plan can add the test.
- All keyword tests use a single-token input (e.g., `"let"`) and assert `vec![TokenKind::Let]` so the test stays focused on disambiguation/keyword-priority, not on whitespace handling (already covered by `keywords_take_priority_over_idents`).
- String/raw-string expected values use `TokenKind::StringLiteral("hello".into())` / `TokenKind::RawStringLiteral("raw string".into())` matching how the existing scanner tests construct expected payloads.
- The spec subsections referenced are lines 34–141 of `vertex_v1_spec.md` (the entirety of `## 2. Syntax`, including its Keywords, Operators, Literals, and Built-in Syntax sub-blocks). Examples from §3+ are out of scope for this plan even though they continue to demonstrate syntax.

## Blockers
Blockers: none

## Summary
Adds a 30+ test `spec_section_2` snapshot module under `src/lexer/scan.rs` that pins the scanner's `TokenKind` output for one example per keyword/operator/literal/built-in form drawn from `vertex_v1_spec.md` §2.

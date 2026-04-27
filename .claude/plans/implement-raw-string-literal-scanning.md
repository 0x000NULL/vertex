# Plan: implement-raw-string-literal-scanning

## Goal
Add `Scanner::scan_raw_string` that recognizes `r"..."` and `r#"..."#` (with arbitrary `#` count), preserves the inner content verbatim (no escape processing), rewinds on mismatched/unterminated forms, and returns the literal text plus a `Span`.

## Steps
1. In `vertex_stage0/src/lexer/scan.rs`, add a new public method `scan_raw_string(&mut self) -> Option<(String, Span)>` on `impl<'a> Scanner<'a>`, placed alongside the existing `scan_string` method.
2. Implementation outline (mirroring the rewind-on-failure pattern used by `scan_char` / `scan_string` / `scan_int_hex`):
   - Save `start = self.pos`.
   - Require `peek() == Some(b'r')`; otherwise return `None`.
   - Advance past `r`. Count consecutive `#` bytes into `hash_count: usize` via a small loop (no upper bound enforced beyond what fits in the source).
   - Require the next byte to be `b'"'`; if not, restore `self.pos = start` and return `None`. This ensures bare identifier `r`, `r#`, `r##ident`, etc. are left to other scanners.
   - Advance past the opening `"`. Mark `content_start = self.pos`.
   - Scan forward byte-by-byte looking for a closing `"` followed by exactly `hash_count` `#` bytes:
     - On EOF (`peek() == None`) before a valid terminator, restore `self.pos = start` and return `None` (unterminated / mismatched).
     - When `peek() == Some(b'"')`, look ahead `hash_count` bytes (`peek_at(1..=hash_count)`) and confirm each is `b'#'`. If yes, capture `content_end = self.pos`, advance past the `"` and the `hash_count` `#`s, build `String::from(&self.src[content_start..content_end])`, build `Span::new(self.file_id, start as u32, self.pos as u32)`, and return `Some((content, span))`.
     - Otherwise advance one UTF-8 char (use `self.src[self.pos..].chars().next()` + `len_utf8()`, matching the pattern in `scan_string`) so that multi-byte content is handled correctly and a stray `"` followed by too few `#` does not falsely match.
   - Verbatim preservation: do NOT call `scan_escape_char`; backslashes, newlines, quotes-with-fewer-hashes, etc. are all literal content.
3. Rewind semantics: any failure path (non-`r` lead, missing opening quote, EOF before terminator, mismatched `#` count where content runs out) sets `self.pos = start` before returning `None`, identical to the convention used by `scan_string`.
4. Add a `#[test] fn raw_string_arbitrary_hashes()` in the existing `mod tests` block in `scan.rs`. Cover, at minimum:
   - Happy path: `r""` -> `""`, `r"hello"` -> `"hello"`, `r#"a"b"#` -> `a"b`, `r##"x"#y"##` -> `x"#y`, `r###"contains "## inside"###` -> `contains "## inside`, `r"\n"` -> literal `\n` (two chars, no escape), `r"line1\nline2"` -> literal backslash-n preserved, embedded actual newline `r"a\nb"` where the source contains `\n` byte preserved verbatim.
   - Span/pos: assert `span.file_id`, `span.start == 0`, `span.end as usize == input.len()`, and `s.pos == input.len()` for each happy case (same shape as `string_literal_escapes`).
   - Rejections (each must return `None` AND leave `s.pos == 0`): `r` alone (no quote), `r#` (no quote), `r"abc` (unterminated, no closing quote), `r#"abc"` (closing quote present but missing `#`), `r##"abc"#` (only one `#` at close), `abc` (no leading `r`).
5. Do NOT wire `scan_raw_string` into the `next_token` driver -- that is the job of `wire-all-scanners-into-scanner-next-token-driver`. Only the standalone method + unit test land here.

## Files
- `vertex_stage0/src/lexer/scan.rs` -- add `scan_raw_string` method on `impl<'a> Scanner<'a>` (placed near `scan_string`), and add `raw_string_arbitrary_hashes` unit test in the existing `#[cfg(test)] mod tests` block.

## Risks
- `peek_at(offset)` for large `hash_count` is O(hash_count) per inner-loop iteration, making the inner scan O(n * hash_count). Acceptable for stage0 (raw strings rarely have many hashes) but worth noting; using a slice-prefix check on `&self.bytes[self.pos+1..]` is a cheap optimization if needed.
- Mistakenly consuming the `r` of an identifier like `raw_value` if the method is later called unconditionally; the `peek() == Some(b'r')` check plus required `"`/`#` lookahead with rewind prevents that, but the driver wiring (separate item) must be careful too.
- For the closing-quote scan, advancing one ASCII byte at a time is wrong when the content has multi-byte UTF-8 (would split a codepoint and `&self.src[content_start..content_end]` would panic). Using `chars().next().len_utf8()` for non-`"` bytes avoids this.
- The spec test name in the verify line is `raw_string_arbitrary_hashes`; the test function name MUST match exactly or the verify command will fail.
- Hash count of zero (`r"..."`) must work: the lookahead loop for `#` bytes must yield 0 cleanly and the close check must accept a bare `"` with no trailing `#`.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib --manifest-path vertex_stage0/Cargo.toml lexer::scan::tests::raw_string_arbitrary_hashes
```

## Assumptions
- The crate root is `vertex_stage0/` (confirmed by `vertex_stage0/src/lexer/scan.rs` existing); `cargo test --lib` therefore needs `--manifest-path vertex_stage0/Cargo.toml` to run from the repo root used by the harness.
- The required test path `lexer::scan::tests::raw_string_arbitrary_hashes` matches the existing module structure (`src/lexer/scan.rs` with `#[cfg(test)] mod tests`), so the test must live in that same `mod tests` block.
- Returned content type is `String` (not `&str`), matching `scan_string`'s signature, since the spec sub-step describes "preserve content verbatim" without prescribing a borrow.
- Raw-string content is preserved as raw source bytes -- no escape processing whatsoever, including no `\u{...}`, no `\n`, no `\\`. A literal newline byte in the source becomes a literal newline byte in the returned string.
- No length cap on `#` count is enforced (Rust uses 255; stage0 needs only "arbitrary").
- The closing terminator is the FIRST `"` followed by exactly `hash_count` `#`s (greedy on left, exact on right). For `r#"a"##b"#`, the terminator is the first `"#` -- content is `a`, and the trailing `#b"#` is left for the driver. (This matches Rust raw-string semantics.) The test cases above respect this.
- Failure modes all rewind `self.pos` to the original start, matching the convention of every other `scan_*` method in the file.
- The `simplify` skill / `fewer-permission-prompts` skill are not relevant to this planning task.

## Blockers
Blockers: none

## Summary
Add a rewind-on-failure `Scanner::scan_raw_string` that handles `r"..."` / `r#"..."#` with arbitrary `#` counts, preserves content verbatim, and is covered by the `raw_string_arbitrary_hashes` unit test the spec verifies against.

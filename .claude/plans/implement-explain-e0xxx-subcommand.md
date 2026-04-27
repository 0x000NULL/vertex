# Plan: implement-explain-e0xxx-subcommand

## Goal
Add a `vertex_stage0::explain` module with a `explain(code) -> Option<&'static str>` registry of error explanations, expose it via a `vertexc --explain E0XXX` subcommand, and ship a passing unit test for E0308.

## Steps
1. Create `vertex_stage0/src/explain.rs` containing `pub fn explain(code: &str) -> Option<&'static str>`. Implementation: normalize `code` to uppercase (so both `e0308` and `E0308` work), `match` on the normalized string, return `Some(&'static str)` for each registered code, `None` otherwise.
2. Populate stub entries for E0080, E0133, E0277, E0308, E0369, E0382, E0425, E0433, E0499, E0502, E0503, E0505, E0599, E0608. Each entry is a `&'static str` with three blank-line-separated paragraphs: (1) one-paragraph plain-English explanation of the error, (2) a minimal Vertex-flavored code example that triggers it (fenced as a plain text block, no markdown fences inside the rust string — just indented or labelled lines), (3) a one-paragraph "how to fix" note. Bind each long string to a `const` (e.g. `const E0308_TEXT: &str = "...";`) above the `match` so the function body stays readable.
3. Add `pub mod explain;` to `vertex_stage0/src/lib.rs` (alphabetical position between `error` and `lexer`).
4. Replace the no-op `pub fn run()` in `lib.rs` with an arg-aware version: read `std::env::args()`, look for `--explain <CODE>` (and the joined `--explain=CODE` form), call `explain::explain(code)`. On hit: print to stdout and `return`. On miss: print `unknown error code: {code}` to stderr and `std::process::exit(1)`. With no `--explain` flag, behave as before (no-op return). `main.rs` continues to call `vertex_stage0::run()` unchanged — the wiring happens in `run()` so the lib remains testable.
5. Add a `#[cfg(test)] mod tests` block at the bottom of `explain.rs` with `#[test] fn explain_e0308_returns_text()` asserting `explain("E0308").is_some()` and that the returned text is non-empty / contains a recognizable substring (e.g. `"mismatched types"` or `"E0308"`). Add a companion `explain_unknown_returns_none` test asserting `explain("E9999").is_none()` so the lookup miss path is exercised too.
6. Run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` mentally against the new module (no comments, no unused imports) so the existing CI gate from `add-ci-workflow` stays green.

## Files
- `vertex_stage0/src/explain.rs` -- new file: 14 const explanation strings, `explain()` lookup function, two unit tests under `mod tests`.
- `vertex_stage0/src/lib.rs` -- declare `pub mod explain;`; rewrite `pub fn run()` to parse `--explain CODE` / `--explain=CODE` from `std::env::args()` and dispatch to `explain::explain`, printing to stdout (hit) or stderr + `exit(1)` (miss).
- `vertex_stage0/src/main.rs` -- no change needed (already delegates to `vertex_stage0::run()`); leave as-is.

## Risks
- The verify test name `explain_e0308_returns_text` is a unit test inside the lib (`--lib`), so the module must be declared `pub mod explain;` in `lib.rs` and the test must live in `mod tests` inside `explain.rs`. Getting the path wrong (e.g. putting it in `tests/`) makes the verify command fail to find the test.
- Putting non-trivial work inside `pub fn run()` may collide with future plans that rewrite `run()` to be the real compiler driver. Mitigation: keep the `--explain` branch small and at the top of `run()` so it's easy to extract later. (Other pending items don't appear to overlap directly.)
- Long multi-paragraph string literals can trip rustfmt's `format_strings` if ever enabled, but the repo's current rustfmt config is default, so plain `"..."` literals with `\n\n` separators are safe.
- Clippy may flag the 14-arm `match` as `too_many_lines`; default clippy lints don't, so this should be fine under `-D warnings`, but if it does, splitting per-code into a `&[(&str, &str)]` table is the fallback.

## Prereqs
Prereqs: none

## Verify
```
cargo test --lib explain::tests::explain_e0308_returns_text
cargo build -p vertex_stage0
cargo clippy --all-targets -- -D warnings
test -f vertex_stage0/src/explain.rs
grep -q "pub fn explain" vertex_stage0/src/explain.rs
grep -q "E0308" vertex_stage0/src/explain.rs
grep -q "pub mod explain" vertex_stage0/src/lib.rs
grep -q "\-\-explain" vertex_stage0/src/lib.rs
```

## Assumptions
- The bare `src/explain.rs` path in the spec refers to `vertex_stage0/src/explain.rs`, since that crate hosts every other module in the workspace and `lib.rs` lives there. The top-level `Cargo.toml` is a workspace manifest, not a separate crate.
- The verify command `cargo test --lib explain::tests::explain_e0308_returns_text` runs from the workspace root and resolves to the `vertex_stage0` lib crate. Adding a `-p vertex_stage0` qualifier is unnecessary because `vertex_stage0` is the only library crate.
- E0080 (evaluation of constant value failed) and E0133 (call to unsafe function) are not in the existing `ErrorCode` registry in `error/mod.rs`, but the task explicitly requires them as `--explain` entries. The explain registry is a documentation lookup — it does **not** require corresponding `ErrorCode` constants to exist. Treating it as a free-standing string table keeps this plan independent of the error-code list.
- "3-paragraph string" means three paragraphs separated by blank lines (`\n\n`) inside one `&'static str`, not three separate strings. This matches how `rustc --explain` formats its output.
- Wiring into `main.rs` arg parsing is satisfied by extending `vertex_stage0::run()` (which `main.rs` already calls) rather than moving arg-parsing into `main.rs`. This keeps the logic unit-testable from the lib and matches the existing structure.
- No `clap`/`structopt` dependency is added; a hand-rolled scan of `std::env::args()` for `--explain` / `--explain=…` is sufficient for one flag and avoids touching `Cargo.toml`.
- The example snippets inside each explanation string are illustrative pseudo-Vertex code (e.g. `let x: i32 = "hello";`) — Vertex's surface syntax is close enough to Rust's that rustc-style examples remain meaningful, and the parser/typechecker that would actually evaluate them isn't built yet.
- Output destination on a hit is stdout (so the user can pipe to `less`), and on a miss is stderr + non-zero exit, mirroring `rustc --explain` behavior.

## Blockers
Blockers: none

## Summary
Adds a static error-explanation registry covering the 14 required codes plus a `vertexc --explain E0XXX` CLI surface, verified by a lib unit test on E0308.

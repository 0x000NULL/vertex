# Plan: add-tests-integration-directory-with-a-smoke-test

## Goal
Add a Cargo integration test at `vertex_stage0/tests/integration/smoke.rs` that calls `vertex_stage0::run()`, registered as the `smoke` test binary so `cargo test --test smoke crate_runs` exercises it.

## Steps
1. Create `vertex_stage0/tests/integration/smoke.rs` containing exactly:
   ```rust
   #[test]
   fn crate_runs() {
       vertex_stage0::run();
   }
   ```
2. Register the test binary in `vertex_stage0/Cargo.toml` by appending a `[[test]]` table (`name = "smoke"`, `path = "tests/integration/smoke.rs"`). This is required because Cargo's default integration-test discovery only picks up files directly inside `tests/` — files in `tests/integration/` need an explicit entry to surface as the `smoke` test binary expected by the verify command.
3. Run `cargo test --manifest-path vertex_stage0/Cargo.toml --test smoke crate_runs` to confirm the test compiles, runs, and passes.

## Files
- `vertex_stage0/tests/integration/smoke.rs` -- new file; integration test calling `vertex_stage0::run()`.
- `vertex_stage0/Cargo.toml` -- add a `[[test]]` entry naming `smoke` with `path = "tests/integration/smoke.rs"`.

## Risks
- Without the `[[test]]` entry, Cargo will not produce a test binary named `smoke` for a file nested under `tests/integration/`, and `cargo test --test smoke` would fail with "no test target named `smoke`". The Cargo.toml addition is the load-bearing piece.
- The repo has no workspace `Cargo.toml` at the root, so `cargo` must be pointed at `vertex_stage0/Cargo.toml` (via `--manifest-path`) when invoked from the repo root.
- `vertex_stage0::run()` is currently an empty `pub fn` (`src/lib.rs:11`); the test will pass trivially today and is intentionally a smoke test for crate linkage rather than behavior.

## Verify
```
cargo test --manifest-path vertex_stage0/Cargo.toml --test smoke crate_runs
test -f vertex_stage0/tests/integration/smoke.rs
grep -q "fn crate_runs" vertex_stage0/tests/integration/smoke.rs
grep -q "vertex_stage0::run" vertex_stage0/tests/integration/smoke.rs
```

## Assumptions
- "tests/integration/" is interpreted as relative to the `vertex_stage0` crate root (`vertex_stage0/tests/integration/`), since that is the only crate in the repo and Cargo integration tests are per-package. There is no top-level workspace.
- The `cargo test --test smoke` form in the spec implies the test binary must be named `smoke`, which requires an explicit `[[test]]` entry in `Cargo.toml` because the file lives in a subdirectory of `tests/`.
- The verify command is run from the repo root, so `--manifest-path vertex_stage0/Cargo.toml` is added; the spec's literal `cargo test --test smoke crate_runs` would only work from inside `vertex_stage0/`, but the manifest-path form is equivalent and safer for the runner's working directory.
- The `#[test]` attribute is applied directly to `crate_runs` (no `mod` wrapper), matching the spec verbatim.
- No additional dev-dependencies are needed; the test only references the crate's own public API.

## Blockers
Blockers: none

## Summary
Adds a registered `smoke` integration test that calls `vertex_stage0::run()`, giving the crate its first end-to-end test wiring.

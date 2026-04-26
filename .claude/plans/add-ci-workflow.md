# Plan: add-ci-workflow

## Goal
Add a GitHub Actions CI workflow that builds, tests, lints, and format-checks the `vertex_stage0` Rust crate on every push and pull request.

## Steps
1. Create `.github/workflows/ci.yml` with a single `ci` job running on `ubuntu-latest`, triggered on `push` and `pull_request`.
2. In that job, check out the repo, install the stable Rust toolchain with the `rustfmt` and `clippy` components, and add a Cargo build cache step (or rely on the toolchain action's default cache) keyed on `vertex_stage0/Cargo.lock`.
3. Run, in this order, against the `vertex_stage0` crate (using `working-directory: vertex_stage0` since there is no workspace `Cargo.toml` at the repo root):
   - `cargo fmt --check`
   - `cargo build`
   - `cargo test`
   - `cargo clippy --all-targets -- -D warnings`
4. Keep the workflow minimal -- no matrix, no nightly, no extra jobs -- so future items (e.g., adding a workspace, adding more crates) can extend it without churn.

## Files
- `.github/workflows/ci.yml` -- new GitHub Actions workflow with the four cargo commands listed in the spec, run from `vertex_stage0/`.

## Risks
- The repo has no top-level `Cargo.toml` (only `vertex_stage0/Cargo.toml`), so cargo commands must run from inside `vertex_stage0/` or fail with "could not find Cargo.toml". Mitigation: set `working-directory: vertex_stage0` on each cargo step.
- `cargo fmt --check` may fail on existing source if it isn't already formatted; this is a real signal, not a flaw in CI -- if it surfaces drift, that's the workflow doing its job. Acceptable.
- Pinning to `stable` toolchain means a future Rust release that introduces new clippy lints could break CI on a green-then-red flip. Acceptable for stage0; can revisit by pinning to a specific toolchain later.
- Using a third-party action (`dtolnay/rust-toolchain@stable`) introduces a small supply-chain surface; it's the de-facto standard and pinned by tag. Acceptable.

## Verify
```
test -f .github/workflows/ci.yml
grep -q 'cargo build' .github/workflows/ci.yml
grep -q 'cargo test' .github/workflows/ci.yml
grep -q 'cargo clippy --all-targets -- -D warnings' .github/workflows/ci.yml
grep -q 'cargo fmt --check' .github/workflows/ci.yml
cargo fmt --check --manifest-path vertex_stage0/Cargo.toml
cargo build --manifest-path vertex_stage0/Cargo.toml
cargo test --manifest-path vertex_stage0/Cargo.toml
cargo clippy --manifest-path vertex_stage0/Cargo.toml --all-targets -- -D warnings
```

## Assumptions
- The crate lives only at `vertex_stage0/` and there is no root workspace `Cargo.toml`, so the workflow steps use `working-directory: vertex_stage0` rather than running cargo from the repo root.
- Stable Rust toolchain is acceptable (no `rust-toolchain.toml` exists in the repo, so no pin to honor).
- Linux runner (`ubuntu-latest`) is sufficient; no cross-platform matrix is requested by the spec.
- Trigger on both `push` and `pull_request` to all branches; this is the conventional default and not constrained by the spec.
- `dtolnay/rust-toolchain@stable` (or `actions-rust-lang/setup-rust-toolchain`) is acceptable for installing the toolchain with `rustfmt` and `clippy` components -- standard community practice.
- Order chosen as `fmt -> build -> test -> clippy`: cheap formatting check first to fail fast, then build, then tests, then clippy as the most expensive lint pass. Order is not specified by the spec.
- The exact strings `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` will appear verbatim in the workflow YAML so the verify greps match.
- No caching action is strictly required to satisfy the spec; if added, it must not change the literal cargo command strings.

## Blockers
Blockers: none

## Summary
Adds a minimal GitHub Actions CI workflow that runs `cargo fmt --check`, `cargo build`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` against the `vertex_stage0` crate on push and pull request.

# Plan: add-ci-workflow

## Goal
Create `.github/workflows/ci.yml` that runs `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` on push/PR so future regressions are caught automatically.

## Steps
1. Create the directory `.github/workflows/` (it does not yet exist).
2. Author `.github/workflows/ci.yml` with one job (`ci`) on `ubuntu-latest`, triggered on `push` and `pull_request` to any branch. The job:
   - checks out the repo (`actions/checkout@v4`),
   - installs the stable Rust toolchain with `rustfmt` and `clippy` components (`dtolnay/rust-toolchain@stable`),
   - caches cargo registry/build artifacts (`Swatinem/rust-cache@v2`),
   - runs the four required commands in order so any failure blocks the green check:
     1. `cargo fmt --all -- --check`
     2. `cargo clippy --all-targets -- -D warnings`
     3. `cargo build --all-targets --verbose`
     4. `cargo test --all-targets --verbose`
3. Use the workspace root as the working directory (the `Cargo.toml` at repo root declares `members = ["vertex_stage0"]`, so all four cargo commands operate on the whole workspace from there — no `--manifest-path` needed).
4. Do NOT touch `vertex_stage0/` source, `Cargo.toml`, or anything else: this item is purely additive.

## Files
- `.github/workflows/ci.yml` -- new file; the GitHub Actions workflow described above.

## Risks
- `cargo fmt --check` and `cargo clippy -D warnings` will only stay green if the in-tree code is already clean. The most recent commit (`cargo fmt across vertex_stage0`) addressed the prior fmt drift that previously broke verify, so the file at HEAD is fmt-clean. Clippy output cannot be confirmed locally from this read-only session; if it is dirty on the real runner, that is a code problem for a later item (`set-ci-fmt-clippy-gate-to-deny-warnings`), not a CI-file problem.
- Verify spec is text-matching only (`test -f` + `grep -q 'cargo clippy'`), so a syntactically invalid YAML would still pass verify; we mitigate by using the standard, well-known action versions and a minimal job shape.
- `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` are pinned to stable major tags; that is the conventional Rust-CI choice and is acceptable for stage-0 bootstrap.

## Prereqs
Prereqs: none

## Verify
```
test -f .github/workflows/ci.yml
grep -q 'cargo clippy' .github/workflows/ci.yml
grep -q 'cargo fmt' .github/workflows/ci.yml
grep -q 'cargo build' .github/workflows/ci.yml
grep -q 'cargo test' .github/workflows/ci.yml
cargo fmt --manifest-path vertex_stage0/Cargo.toml --all -- --check
```

## Assumptions
- "CI workflow" means a GitHub Actions workflow (the repo lives on GitHub; `.github/` is the conventional location and matches how `set-ci-fmt-clippy-gate-to-deny-warnings` is later phrased).
- One job on `ubuntu-latest` with the stable toolchain is sufficient for stage-0; no Windows/macOS matrix, no MSRV pinning, no nightly. Keeps the file small and deterministic.
- Triggers are `push` and `pull_request` to all branches (no branch filter), matching the most common Rust template.
- Using third-party actions `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` is acceptable; they are de-facto standards in the Rust ecosystem and keep the workflow short.
- All cargo commands run from the workspace root (`Cargo.toml` is a virtual workspace pointing at `vertex_stage0`); no per-crate `--manifest-path` flags are needed.
- `--all-targets` is added to `build`/`test`/`clippy` so doc-tests/examples/integration tests are exercised; this is consistent with the explicit `--all-targets` already required for clippy.
- Verify includes a local `cargo fmt --check` to catch the same failure mode that broke the previous attempt of this item, ensuring the workspace is fmt-clean at the moment the workflow file is committed.

## Blockers
Blockers: none

## Summary
Adds a stable, minimal GitHub Actions CI workflow at `.github/workflows/ci.yml` enforcing fmt, clippy (`-D warnings`), build, and test on every push/PR.

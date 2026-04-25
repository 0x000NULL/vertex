# Plan: initialize-vertex-stage0-cargo-crate-at-the-repo-root

## Goal
Create a minimal `vertex_stage0/` Cargo crate at the repo root that compiles cleanly with both a `vertexc` binary target and a library target.

## Steps
1. Create directory `vertex_stage0/src/`.
2. Write `vertex_stage0/Cargo.toml` declaring package `vertex_stage0` with `edition = "2021"`, a `[lib]` table pointing at `src/lib.rs`, and a `[[bin]]` table named `vertexc` pointing at `src/main.rs`.
3. Write `vertex_stage0/src/lib.rs` containing `pub fn run() {}`.
4. Write `vertex_stage0/src/main.rs` containing `fn main() { vertex_stage0::run(); }`.
5. Confirm `cargo build` succeeds against `vertex_stage0/Cargo.toml`.

## Files
- `vertex_stage0/Cargo.toml` -- new manifest with `[package]` (name `vertex_stage0`, version `0.1.0`, edition `2021`), `[lib]` with `path = "src/lib.rs"`, `[[bin]]` with `name = "vertexc"` and `path = "src/main.rs"`. No dependencies.
- `vertex_stage0/src/lib.rs` -- new file containing only `pub fn run() {}`.
- `vertex_stage0/src/main.rs` -- new file containing only `fn main() { vertex_stage0::run(); }`.

## Risks
- A workspace `Cargo.toml` may later be added at the repo root; this crate is a standalone package for now and would need to be listed under `[workspace.members]` then. Not in scope here.
- `cargo` must be on PATH in the runner's environment; if absent the verify step fails for environmental rather than code reasons.
- Build artifacts (`vertex_stage0/target/`) are produced by `cargo build`; a `.gitignore` is not part of this todo and is left for a later item.

## Verify
```
test -f vertex_stage0/Cargo.toml
test -f vertex_stage0/src/lib.rs
test -f vertex_stage0/src/main.rs
cargo build --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- Package name is `vertex_stage0` (matches the directory name and the `vertex_stage0::run()` call required in `main.rs`).
- Package version is `0.1.0` (Cargo requires a version field; nothing in the spec dictates otherwise).
- No `[dependencies]` are added at this stage -- the todo specifies an empty/minimal scaffold.
- No `[workspace]` table is created at the repo root; this is a standalone crate until a later todo introduces a workspace.
- No `.gitignore` for `target/` is added in this commit (out of scope for this todo).
- `cargo build` in verify is invoked with `--manifest-path vertex_stage0/Cargo.toml` because the runner executes verify lines from the repo root, not from inside the crate.
- Both `src/main.rs` and `src/lib.rs` contain exactly the bodies dictated by the sub-steps; "empty" in the todo refers to no further logic beyond those one-liners.

## Blockers
Blockers: none

## Summary
Scaffold a minimal `vertex_stage0` Cargo crate (lib + `vertexc` bin) at the repo root that builds cleanly with no dependencies.

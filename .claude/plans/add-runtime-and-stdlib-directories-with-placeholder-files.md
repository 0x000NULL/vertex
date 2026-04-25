# Plan: add-runtime-and-stdlib-directories-with-placeholder-files

## Goal
Create the top-level `runtime/` and `stdlib/` directories with placeholder files so future runtime (C support library) and stdlib work has a home in the repo.

## Steps
1. Create `runtime/vertex_runtime.h` with header guards (`#ifndef VERTEX_RUNTIME_H` / `#define VERTEX_RUNTIME_H` / `#endif`) and a single `// TODO:` comment placeholder inside the guards.
2. Create `runtime/vertex_runtime.c` with a `#include "vertex_runtime.h"` line and a single `// TODO:` comment placeholder (no symbols defined).
3. Create `stdlib/.gitkeep` as an empty file so the otherwise-empty `stdlib/` directory is tracked by git.

## Files
- `runtime/vertex_runtime.h` -- new; header guards + TODO comment, no declarations.
- `runtime/vertex_runtime.c` -- new; includes its header + TODO comment, no definitions.
- `stdlib/.gitkeep` -- new; empty placeholder so the directory is committable.

## Risks
- None of significance. These are inert placeholder files outside the existing `vertex_stage0` Cargo crate, so they do not affect the build. The only risk is path/casing: the runner verify uses lowercase `runtime/` and `stdlib/` at repo root, which matches the spec.

## Verify
```
test -f runtime/vertex_runtime.h
test -f runtime/vertex_runtime.c
test -f stdlib/.gitkeep
grep -q VERTEX_RUNTIME_H runtime/vertex_runtime.h
grep -q vertex_runtime.h runtime/vertex_runtime.c
cargo check --manifest-path vertex_stage0/Cargo.toml
```

## Assumptions
- Directories `runtime/` and `stdlib/` live at the repo root (siblings of `vertex_stage0/`), matching the design docs' convention of treating runtime/stdlib as separate top-level components from the stage0 compiler crate.
- The header guard macro name should be `VERTEX_RUNTIME_H` (uppercase, file-name-derived) -- standard C convention.
- "TODO comment" means a literal `// TODO:` line; no specific TODO text is mandated by the spec, so a generic placeholder ("// TODO: implement Vertex runtime") suffices.
- `stdlib/.gitkeep` is an empty file (the conventional use of `.gitkeep`); no content required.
- The `runtime/` C files are placeholders only and are NOT yet wired into any build system (no Makefile, no `cc` crate, no `build.rs` changes). Wiring will come in a later todo.
- Running `cargo check` on the existing crate is included in verify only as a smoke test that the new top-level dirs do not somehow interfere with the Rust build; the deliverables themselves are the three files.

## Blockers
Blockers: none

## Summary
Adds the empty `runtime/` (with stub `vertex_runtime.{h,c}`) and `stdlib/` (with `.gitkeep`) directories so future work has a place to land without affecting the current build.

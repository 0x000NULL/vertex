# Plan: add-stub-changelog-md-and-contributing-md

## Goal
Add minimal one-line stub `CHANGELOG.md` and `CONTRIBUTING.md` files at the repo root as placeholders for future content.

## Steps
1. Create `CHANGELOG.md` at the repo root with a single-line stub indicating the changelog is not yet populated.
2. Create `CONTRIBUTING.md` at the repo root with a single-line stub indicating contribution guidelines are not yet populated.
3. Commit both files together as one coherent change.

## Files
- `CHANGELOG.md` -- new file, one-line stub placeholder (e.g., `# Changelog` heading plus a single sentence noting it is a placeholder).
- `CONTRIBUTING.md` -- new file, one-line stub placeholder (e.g., `# Contributing` heading plus a single sentence noting it is a placeholder).

## Risks
- Negligible: stub files cannot break the build, tests, or existing tooling. Only risk is path collision if a file with either name already exists, which would be overwritten -- mitigated by checking before write.

## Verify
```
test -f CHANGELOG.md
test -f CONTRIBUTING.md
```

## Assumptions
- "One-line content each" permits a short markdown heading plus a placeholder sentence (effectively one logical line of meaningful content); the runner can flesh these out later.
- Files belong at the repository root (`C:\Users\Ethan\vertex\`), not inside `vertex_stage0/`, since these are standard project-level meta files.
- No existing `CHANGELOG.md` or `CONTRIBUTING.md` is present; if one exists it will be left untouched (verify only checks existence).
- No license/heading conventions are enforced by the project, so plain markdown is acceptable.
- The commit should bundle both files together in a single commit per the "single coherent commit" instruction.

## Blockers
Blockers: none

## Summary
Adds two root-level stub markdown files (`CHANGELOG.md`, `CONTRIBUTING.md`) as placeholders for future project meta-documentation.

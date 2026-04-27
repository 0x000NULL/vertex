
## add-ci-workflow
- Item: Add CI workflow
- Reason: verify failed
- Timestamp: 2026-04-26T01:32:30.0994521Z

### Detail
```
+ test -f .github/workflows/ci.yml
+ grep -q 'cargo build' .github/workflows/ci.yml
+ grep -q 'cargo test' .github/workflows/ci.yml
+ grep -q 'cargo clippy --all-targets -- -D warnings' .github/workflows/ci.yml
+ grep -q 'cargo fmt --check' .github/workflows/ci.yml
+ cargo fmt --check --manifest-path vertex_stage0/Cargo.toml
Diff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\codegen\mod.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\error.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\lexer\mod.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\lib.rs:1:
[32m+pub mod codegen;
[0m[32m+pub mod error;
[0m pub mod lexer;
[32m+pub mod mir;
[0m pub mod parser;
 pub mod resolve;
[31m-pub mod typecheck;
[0m[31m-pub mod mir;
[0m[31m-pub mod codegen;
[0m[31m-pub mod error;
[0m pub mod span;
[32m+pub mod typecheck;
[0m pub mod util;
 
 pub fn run() {}
Diff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\mir\mod.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\parser\mod.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\resolve\mod.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\span.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\typecheck\mod.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\util.rs:1:
[32m+
[0m[32m+
[0mDiff in \\?\C:\Users\Ethan\vertex\vertex_stage0\src\main.rs:1:
[31m-fn main() { vertex_stage0::run(); }
[0m[32m+fn main() {
[0m[32m+    vertex_stage0::run();
[0m[32m+}
[0m
```

---


<!-- 3 entries removed 2026-04-26 after workspace Cargo.toml added at repo root:
     implement-span-struct-in-src-span-rs, define-errorcode-and-errorkind-in-src-error-rs,
     define-compileerror-struct-in-src-error-rs. All three failed verify with
     "could not find Cargo.toml in C:\Users\Ethan\vertex". With workspace now
     present, removing the slug entries lets the runner re-queue them on the
     next iteration. -->


## implement-erroraccumulator-in-src-error-rs
- Item: Implement `ErrorAccumulator` in `src/error.rs`
- Reason: blockers
- Timestamp: 2026-04-26T03:13:58.5753971Z

### Blocker: CompileError / ErrorCode / ErrorKind do not yet exist in error.rs
- severity: cross-item
- affects: define-compileerror-struct-in-src-error-rs, define-errorcode-and-errorkind, error-pretty-printer, parser eat/expect, parse-failure-recovery
- question: Should this item bootstrap the missing prereq types (ErrorCode, ErrorKind, CompileError) so it can compile, or wait for the earlier two TODO items to be re-attempted and merged first?
- default_assumption: Bootstrap them inline, using the field/method shapes already specified in `compiler_architecture.md` §6 and TODO lines 79–90 so a re-run of the earlier items will be a no-op reconcile rather than a conflict.
- Resolution: workspace `Cargo.toml` added 2026-04-26 — items `define-errorcode-and-errorkind-in-src-error-rs` and `define-compileerror-struct-in-src-error-rs` will be re-queued first in TODO order, so prereq types will exist by the time this item re-runs. Wait for those, then proceed normally (no inline bootstrap needed).

---


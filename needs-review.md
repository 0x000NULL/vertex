
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


## define-compileerror-struct-in-src-error-rs
- Item: Define `CompileError` struct in `src/error.rs`
- Reason: verify failed
- Timestamp: 2026-04-27T02:56:32.2216795Z

### Detail
```
+ cargo test --lib -p vertex_stage0 error::tests::compile_error_builder_chains
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running unittests src\lib.rs (target\debug\deps\vertex_stage0-98f9861405ee0bf7.exe)

running 1 test
test error::tests::compile_error_builder_chains ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

+ cargo fmt --all -- --check
+ cargo clippy --all-targets -- -D warnings
    Checking vertex_stage0 v0.1.0 (C:\Users\Ethan\vertex\vertex_stage0)
error: struct `Span` has a public `len` method, but no `is_empty` method
  --> vertex_stage0\src\span.rs:22:5
   |
22 |     pub fn len(&self) -> u32 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.95.0/index.html#len_without_is_empty
   = note: `-D clippy::len-without-is-empty` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::len_without_is_empty)]`

error: could not compile `vertex_stage0` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `vertex_stage0` (lib test) due to 1 previous error
```

---


## define-stmt-enum-in-src-ast-stmt-rs
- Item: Define `Stmt` enum in `src/ast/stmt.rs`
- Reason: blockers
- Timestamp: 2026-04-27T04:16:06.5377123Z

### Blocker: prereq enums absent at execute time
- severity: cross-item
- affects: define-expr-enum-in-src-ast-expr-rs-literal-path-variants, define-type-enum-in-src-ast-ty-rs, define-pattern-enum-in-src-ast-pat-rs
- question: Will the runner honor the Prereqs section and ensure `Expr`, `Ty`, and `Pattern` modules exist before this item executes?
- default_assumption: If a prereq is missing at execute time, land a minimal placeholder module (e.g. `pub enum Pattern {}` in `pat.rs`, same for `expr.rs` and `ty.rs`) inline as part of this commit, so `Stmt` compiles. Accept that the dedicated prereq items may need to merge into / overwrite those placeholders later.
- Resolution: 

### Blocker: Pattern vs Pat naming
- severity: local
- affects: define-pattern-enum-in-src-ast-pat-rs
- question: Is the type named `Pattern` (per the spec wording) or `Pat` (a common shorter convention also hinted at by `pat.rs`)?
- default_assumption: Use `Pattern` since the task spec writes `pattern: Pattern` explicitly; if the prereq lands `Pat`, follow that and update the field type to `Pat`.
- Resolution: 

---


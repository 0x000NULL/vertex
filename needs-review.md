
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


## implement-span-struct-in-src-span-rs
- Item: Implement `Span` struct in `src/span.rs`
- Reason: verify failed
- Timestamp: 2026-04-26T02:04:51.9692446Z

### Detail
```
+ cargo test -p vertex_stage0 --lib span::tests::span_merge_takes_outer_bounds
error: could not find `Cargo.toml` in `C:\Users\Ethan\vertex` or any parent directory
```

---


## define-errorcode-and-errorkind-in-src-error-rs
- Item: Define `ErrorCode` and `ErrorKind` in `src/error.rs`
- Reason: verify failed
- Timestamp: 2026-04-26T02:53:50.9676456Z

### Detail
```
+ cargo build -p vertex_stage0
error: could not find `Cargo.toml` in `C:\Users\Ethan\vertex` or any parent directory
```

---


## define-compileerror-struct-in-src-error-rs
- Item: Define `CompileError` struct in `src/error.rs`
- Reason: verify failed
- Timestamp: 2026-04-26T03:12:06.9990456Z

### Detail
```
+ cargo test --lib --manifest-path vertex_stage0/Cargo.toml error::tests::compile_error_builder_chains
   Compiling vertex_stage0 v0.1.0 (C:\Users\Ethan\vertex\vertex_stage0)
error[E0425]: cannot find type `ErrorCode` in this scope
  --> src\error.rs:12:15
   |
12 |     pub code: ErrorCode,
   |               ^^^^^^^^^ not found in this scope

error[E0425]: cannot find type `ErrorKind` in this scope
  --> src\error.rs:13:15
   |
13 |     pub kind: ErrorKind,
   |               ^^^^^^^^^ not found in this scope
   |
help: consider importing this enum
   |
 1 + use std::io::ErrorKind;
   |

error[E0425]: cannot find type `ErrorCode` in this scope
  --> src\error.rs:22:15
   |
22 |         code: ErrorCode,
   |               ^^^^^^^^^ not found in this scope

error[E0425]: cannot find type `ErrorKind` in this scope
  --> src\error.rs:23:15
   |
23 |         kind: ErrorKind,
   |               ^^^^^^^^^ not found in this scope
   |
help: consider importing this enum
   |
 1 + use std::io::ErrorKind;
   |

error[E0433]: cannot find type `ErrorKind` in this scope
  --> src\error.rs:56:51
   |
56 |         let err = CompileError::new(ErrorCode(1), ErrorKind::Other, span, "boom")
   |                                                   ^^^^^^^^^ use of undeclared type `ErrorKind`
   |
help: consider importing this enum
   |
50 +     use std::io::ErrorKind;
   |

error[E0433]: cannot find type `ErrorKind` in this scope
  --> src\error.rs:65:36
   |
65 |         assert!(matches!(err.kind, ErrorKind::Other));
   |                                    ^^^^^^^^^ use of undeclared type `ErrorKind`
   |
help: consider importing this enum
   |
50 +     use std::io::ErrorKind;
   |

warning: unused import: `Path`
 --> src\span.rs:1:17
  |
1 | use std::path::{Path, PathBuf};
  |                 ^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0425]: cannot find function, tuple struct or tuple variant `ErrorCode` in this scope
  --> src\error.rs:56:37
   |
56 |         let err = CompileError::new(ErrorCode(1), ErrorKind::Other, span, "boom")
   |                                     ^^^^^^^^^ not found in this scope

error[E0425]: cannot find function, tuple struct or tuple variant `ErrorCode` in this scope
  --> src\error.rs:64:30
   |
64 |         assert_eq!(err.code, ErrorCode(1));
   |                              ^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0433.
For more information about an error, try `rustc --explain E0425`.
warning: `vertex_stage0` (lib test) generated 1 warning
error: could not compile `vertex_stage0` (lib test) due to 8 previous errors; 1 warning emitted
```

---


## implement-erroraccumulator-in-src-error-rs
- Item: Implement `ErrorAccumulator` in `src/error.rs`
- Reason: blockers
- Timestamp: 2026-04-26T03:13:58.5753971Z

### Blocker: CompileError / ErrorCode / ErrorKind do not yet exist in error.rs
- severity: cross-item
- affects: define-compileerror-struct-in-src-error-rs, define-errorcode-and-errorkind, error-pretty-printer, parser eat/expect, parse-failure-recovery
- question: Should this item bootstrap the missing prereq types (ErrorCode, ErrorKind, CompileError) so it can compile, or wait for the earlier two TODO items to be re-attempted and merged first?
- default_assumption: Bootstrap them inline, using the field/method shapes already specified in `compiler_architecture.md` §6 and TODO lines 79–90 so a re-run of the earlier items will be a no-op reconcile rather than a conflict.
- Resolution: 

---


# Vertex Compiler Bootstrap Plan

**Version**: 1.0.0
**Status**: Implementation Roadmap
**Date**: December 2024

## Executive Summary

This document outlines the three-stage bootstrap process for the Vertex compiler, from initial implementation in a host language to full self-hosting. The goal is to create a production-ready, self-hosted Vertex compiler through incremental development.

## 1. Bootstrap Overview

```
┌────────────────────────────────────────────────────────┐
│  Stage 0: Compiler with Essential Features            │
│  - Written in Rust/C++                                 │
│  - Targets C code generation                           │
│  - Implements generics, traits, closures               │
│  Duration: 9-12 months                                 │
└───────────────────────┬────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────┐
│  Stage 1: Vertex Compiler in Vertex Subset             │
│  - Rewritten in compilable Vertex subset               │
│  - Compiled by Stage 0 compiler                        │
│  - Full feature set                                    │
│  Duration: 6-12 months                                 │
└───────────────────────┬────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────┐
│  Stage 2: Full Self-Hosted Compiler                    │
│  - Uses all Vertex features                            │
│  - Compiles itself                                     │
│  - Production ready                                    │
│  Duration: 3-6 months                                  │
└────────────────────────────────────────────────────────┘
```

## 2. Stage 0: Minimal Compiler in Host Language

### 2.1 Goals

**Primary Objective**: Create a working compiler that can compile a useful subset of Vertex to C code.

**Success Criteria**:
- Compiles simple Vertex programs to C
- Passes basic test suite (100+ tests)
- Can compile a non-trivial program (e.g., simple CLI tool)
- Provides reasonable error messages

### 2.2 Implementation Language Choice

**Recommended: Rust**

Advantages:
- Similar type system to Vertex
- Excellent for compiler development
- Rich ecosystem (parser libraries, etc.)
- Memory safety during development

Alternative: C++
- More widely known
- Mature tooling
- Good performance

### 2.3 Stage 0 Feature Set

#### Supported Features (Minimal Viable)

**Type System**:
- ✓ Primitive types (i32, i64, u32, u64, f32, f64, bool, char)
- ✓ String and &str
- ✓ Arrays (fixed size)
- ✓ Slices (&[T])
- ✓ Tuples
- ✓ Structs (including generic structs)
- ✓ Enums (including generic enums)
- ✓ References (&T, &mut T)
- ✓ Generic types (structs and enums with type parameters)
- ✓ Basic generic instantiation and monomorphization
- ✗ Advanced generic features (const generics, GATs, etc.)

**Control Flow**:
- ✓ if/else
- ✓ loop, while, for
- ✓ match (basic patterns)
- ✓ break, continue, return

**Functions**:
- ✓ Basic functions
- ✓ Methods (impl blocks)
- ✓ Generic functions with type parameters
- ✓ Closures (basic capture semantics)
- ✓ Closure traits (Fn, FnMut, FnOnce)
- ✗ Complex closure coercion edge cases

**Memory Management**:
- ✓ Ownership and moves
- ✓ Borrowing (&, &mut)
- ✓ Basic lifetime inference
- ✓ Drop trait
- ✗ Complex lifetime scenarios
- ✗ Explicit lifetime parameters (Vertex v1 limitation)

**Traits**:
- ✓ Trait definitions
- ✓ Trait implementations
- ✓ Trait bounds in function signatures
- ✓ Basic trait method dispatch
- ✓ Standard traits (Clone, Copy, Debug, Display, Eq, Ord, Hash)
- ✓ Iterator trait and IntoIterator
- ✓ Associated types (REQUIRED for Iterator trait)
- ✗ Higher-ranked trait bounds (deferred)
- ✗ Trait objects (dynamic dispatch - NOT in v1.0 spec)

**Modules**:
- ✓ Module system (mod, use)
- ✓ Basic visibility (pub)
- ✓ File-based modules

**Error Handling**:
- ✓ Result<T, E>
- ✓ ? operator
- ✓ panic

**Standard Library** (Essential for Self-Hosting):
- ✓ Vec<T>
- ✓ String
- ✓ HashMap<K, V> and HashSet<T>
- ✓ Option<T> and Result<T, E>
- ✓ Iterator trait and combinators (map, filter, fold, etc.)
- ✓ Basic I/O (print, println, Read, Write traits)
- ✓ File I/O
- ✓ Box<T> (heap allocation)
- ✓ Rc<T> and Arc<T> (in prelude per spec)
- ✗ Advanced collections (BTreeMap, etc.)

**Arithmetic Operations**:
- ✓ Overflow checking in debug mode (panic on overflow)
- ✓ Wrapping arithmetic in release mode (two's complement)
- ✓ Checked methods (checked_add, checked_sub, etc.)
- ✓ Saturating methods (saturating_add, etc.)
- ✓ Wrapping methods (wrapping_add, etc.)

**Not Supported in Stage 0**:
- Trait objects (dynamic dispatch - NOT in v1.0)
- Advanced pattern matching (or-patterns, @ bindings with ranges)
- Macros (Vertex has none - only built-in syntax)
- Unsafe code
- Full FFI (only C runtime linking)
- Inline assembly
- Advanced lifetime features

### 2.4 Stage 0 Architecture

```rust
// Project structure for Stage 0 compiler
vertex_stage0/
├── Cargo.toml
├── src/
│   ├── main.rs              // Driver
│   ├── lib.rs               // Library root
│   ├── lexer/
│   │   ├── mod.rs
│   │   └── token.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── ast.rs
│   │   └── grammar.rs
│   ├── resolve/
│   │   ├── mod.rs
│   │   └── scope.rs
│   ├── typecheck/
│   │   ├── mod.rs
│   │   ├── infer.rs
│   │   └── unify.rs
│   ├── mir/
│   │   ├── mod.rs
│   │   ├── build.rs
│   │   └── borrow_check.rs
│   ├── codegen/
│   │   ├── mod.rs
│   │   └── c_backend.rs
│   ├── error.rs
│   └── util.rs
├── runtime/
│   ├── vertex_runtime.c     // Minimal runtime
│   └── vertex_runtime.h
└── tests/
    ├── lexer_tests.rs
    ├── parser_tests.rs
    └── compile_tests/
        ├── simple.vx
        ├── structs.vx
        └── ...
```

### 2.5 Stage 0 Development Timeline

**Month 1-2: Foundation**
- [ ] Lexer implementation
  - Tokenization
  - Source location tracking
  - Error reporting infrastructure
- [ ] Basic parser
  - Expression parsing
  - Statement parsing
  - Item parsing (structs, functions, traits)
- [ ] AST definition (including generics)
- [ ] Test harness

**Month 3-5: Type System and Generics**
- [ ] Name resolution
  - Module system
  - Scope handling
  - Import resolution
  - Trait resolution
- [ ] Type checking
  - Type inference with generics
  - Unification algorithm
  - Generic instantiation
  - Trait bound checking
- [ ] Monomorphization infrastructure
- [ ] Error message formatting

**Month 6-7: Traits and Closures**
- [ ] Trait system implementation
  - Trait method dispatch
  - Standard trait implementations
  - Iterator trait infrastructure
- [ ] Closure support
  - Closure capture analysis
  - Fn/FnMut/FnOnce trait hierarchy
  - Closure code generation

**Month 8-9: MIR and Borrow Checking**
- [ ] MIR generation
  - Control flow graph
  - Basic blocks
  - Closure and generic handling
- [ ] Borrow checker
  - Move checking
  - Borrow validation
  - Lifetime inference

**Month 10-11: Code Generation**
- [ ] C code generator
  - Function generation with generics
  - Trait method calls
  - Closure generation
  - Struct layout
  - Runtime integration
- [ ] Monomorphization pass

**Month 12: Standard Library and Testing**
- [ ] Standard library implementation
  - Vec<T>, HashMap<K,V>, Option<T>, Result<T,E>
  - Iterator trait and combinators
  - String and I/O
- [ ] Comprehensive testing
  - Unit tests
  - Integration tests
  - Real programs (1000+ lines)
- [ ] Bug fixes and polish
- [ ] Documentation

### 2.6 Stage 0 Test Programs

**Test 1: Hello World**
```vertex
fn main() {
    println("Hello, Vertex!")
}
```

**Test 2: Fibonacci**
```vertex
fn fib(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() {
    let result = fib(10)
    println("fib(10) = {}", result)
}
```

**Test 3: Struct and Methods**
```vertex
struct Point {
    x: f64,
    y: f64
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

fn main() {
    let p = Point::new(3.0, 4.0)
    println("Distance: {}", p.distance())
}
```

**Test 4: Ownership and Borrowing**
```vertex
fn takes_ownership(s: String) {
    println("{}", s)
}

fn borrows(s: &String) {
    println("{}", s)
}

fn main() {
    let s1 = String::from("hello")
    borrows(&s1)
    takes_ownership(s1)
    // s1 no longer valid here
}
```

**Test 5: File I/O**
```vertex
use std::fs

fn main() -> Result<(), std::io::Error> {
    let contents = fs::read_to_string("input.txt")?
    fs::write("output.txt", contents)?
    Ok(())
}
```

### 2.7 Stage 0 Deliverables

1. **Compiler Binary**: `vertex0`
   - Compiles .vx files to C
   - Invokes C compiler for final binary

2. **Runtime Library**: `libvertex_runtime.a`
   - Panic handler
   - Memory allocator wrappers
   - Basic I/O functions

3. **Standard Library Subset**: `stdlib_minimal/`
   - Core types (Vec, String)
   - I/O module
   - Result/Option

4. **Test Suite**: 100+ passing tests

5. **Documentation**:
   - Language subset guide
   - Compiler usage
   - Known limitations

## 3. Stage 1: Vertex Compiler in Vertex Subset

### 3.1 Goals

**Primary Objective**: Rewrite the Stage 0 compiler in Vertex itself, using only features that Stage 0 supports.

**Success Criteria**:
- Stage 0 can compile Stage 1 source to C
- Stage 1 binary has same functionality as Stage 0
- Stage 1 passes all Stage 0 tests
- Stage 1 can compile itself (via Stage 0 first)

### 3.2 Strategy

**Incremental Rewrite**:
1. Port modules one at a time
2. Maintain parallel Rust and Vertex versions
3. Test equivalence at each step
4. Switch over once complete module works

**Module Porting Order**:
1. Error handling and utilities
2. Lexer
3. AST definition
4. Parser
5. Name resolution
6. Type checking
7. MIR generation
8. Borrow checker
9. Code generator

### 3.3 Stage 1 Feature Additions

During Stage 1 development, add features beyond Stage 0:

**Added Features**:
- ✓ Advanced pattern matching (or-patterns, @ bindings with ranges)
- ✓ RefCell<T> and interior mutability (Cell<T>, RefCell<T>)
- ✓ Advanced iterator adapters
- ✓ LLVM backend (optional - C backend still primary)
- ✓ Complete standard library (all prelude items)

**Still Deferred to Stage 2 or Future**:
- Trait objects (dynamic dispatch - NOT in v1.0 spec)
- Unsafe code
- Full FFI (C ABI, #[repr(C)], extern blocks)
- Inline assembly
- Advanced optimizations

### 3.4 Stage 1 Architecture

```
vertex_stage1/  (Written in Vertex)
├── vertex.toml
├── src/
│   ├── main.vx
│   ├── lib.vx
│   ├── lexer/
│   │   ├── mod.vx
│   │   └── token.vx
│   ├── parser/
│   │   ├── mod.vx
│   │   ├── ast.vx
│   │   └── grammar.vx
│   ├── resolve/
│   │   ├── mod.vx
│   │   └── scope.vx
│   ├── typecheck/
│   │   ├── mod.vx
│   │   ├── infer.vx
│   │   └── unify.vx
│   ├── mir/
│   │   ├── mod.vx
│   │   ├── build.vx
│   │   └── borrow_check.vx
│   ├── codegen/
│   │   ├── mod.vx
│   │   ├── c_backend.vx
│   │   └── llvm_backend.vx  // NEW: LLVM support
│   ├── error.vx
│   └── util.vx
└── tests/
    └── ... (same test suite)
```

### 3.5 Stage 1 Development Timeline

**Month 1-3: Core Compiler Port**
- [ ] Port lexer and parser
- [ ] Port AST and basic structures
- [ ] Port using Stage 0's generics and traits
- [ ] Verify correctness against Stage 0

**Month 4-6: Semantics Port**
- [ ] Port name resolution (using HashMap<K,V>)
- [ ] Port type checker
- [ ] Port generic instantiation
- [ ] Port trait resolution

**Month 7-9: Backend Port**
- [ ] Port MIR generation
- [ ] Port borrow checker
- [ ] Port C backend
- [ ] Optional: Add LLVM backend

**Month 10-12: Self-Hosting**
- [ ] Compile Stage 1 with Stage 0
- [ ] Test Stage 1 output
- [ ] Bootstrap: Stage 1 compiles Stage 1
- [ ] Performance tuning
- [ ] Bug fixes

### 3.6 Self-Hosting Verification

```bash
# Step 1: Compile Stage 1 with Stage 0 (Rust)
$ vertex0 vertex_stage1/src/main.vx -o vertex1

# Step 2: Compile Stage 1 with itself
$ vertex1 vertex_stage1/src/main.vx -o vertex1_self

# Step 3: Verify binaries are equivalent
$ ./vertex1 test.vx -o test1
$ ./vertex1_self test.vx -o test2
$ diff test1 test2  # Should be identical
```

### 3.7 Stage 1 Deliverables

1. **Self-Hosted Compiler**: `vertex1`
   - Written in Vertex
   - Compiles Vertex code
   - Can compile itself

2. **Full Standard Library**: `stdlib/`
   - Collections (Vec, HashMap, HashSet)
   - String handling
   - I/O
   - Error types
   - Iterators

3. **Build System**: `vertexbuild`
   - Parses vertex.toml
   - Manages dependencies
   - Builds projects

4. **Extended Test Suite**: 500+ tests

5. **Language Documentation**:
   - Complete language reference
   - Standard library docs
   - Compiler internals guide

## 4. Stage 2: Full Production Compiler

### 4.1 Goals

**Primary Objective**: Create a production-quality, fully-featured Vertex compiler.

**Success Criteria**:
- Fast compilation (10,000+ lines/sec)
- Excellent error messages
- Stable ABI
- Complete standard library
- Production use cases validated

### 4.2 Feature Completions

**Added in Stage 2**:
- ✓ Unsafe code support
- ✓ Full FFI (C interop)
- ✓ Inline assembly (platform-specific)
- ✓ Optimization passes
- ✓ Debug information (DWARF)
- ✓ Incremental compilation
- ✓ Parallel compilation
- ✓ Profile-guided optimization

### 4.3 Stage 2 Enhancements

**Compiler Improvements**:
- Advanced optimizations
- Better error recovery
- Faster type checking
- Incremental compilation
- IDE integration (LSP)

**Standard Library**:
- Networking
- Threading
- Async I/O (future)
- Regular expressions
- Serialization
- Crypto

**Tooling**:
- Package manager
- Documentation generator
- Code formatter
- Linter
- Test framework

### 4.4 Stage 2 Timeline

**Month 1-3: Performance**
- [ ] Optimization passes
- [ ] Parallel compilation
- [ ] Benchmarking suite
- [ ] Performance tuning

**Month 4-6: Completeness**
- [ ] Unsafe code support
- [ ] Full FFI implementation
- [ ] Platform support (Linux, macOS, Windows)
- [ ] Complete standard library

**Month 7-9: Ecosystem**
- [ ] Package manager
- [ ] Build system enhancements
- [ ] IDE support (LSP server)
- [ ] Documentation tools

**Month 10-12: Production Readiness**
- [ ] Stability testing
- [ ] Real-world projects
- [ ] Performance validation
- [ ] 1.0 release preparation

### 4.5 Stage 2 Deliverables

1. **Production Compiler**: `vertex`
   - Fast, optimized compilation
   - All features supported
   - Stable ABI

2. **Complete Standard Library**
   - All planned modules
   - Well-documented
   - Well-tested

3. **Toolchain**:
   - `vertex` - Compiler
   - `vertexbuild` - Build system
   - `vertexpkg` - Package manager
   - `vertexfmt` - Code formatter
   - `vertexdoc` - Documentation generator
   - `vertex-lsp` - Language server

4. **Documentation**:
   - Language specification
   - Standard library reference
   - The Vertex Book (guide)
   - Compiler internals
   - Contributing guide

5. **Website and Community**:
   - Official website
   - Package repository
   - Forum/Discord
   - GitHub organization

## 5. Risk Mitigation

### 5.1 Technical Risks

**Risk**: Stage 0 compiler is too limited

**Mitigation**:
- Start with generous subset
- Add features incrementally
- Test with progressively complex programs

**Risk**: Performance issues in self-hosted compiler

**Mitigation**:
- Profile early and often
- Implement key optimizations in Stage 0
- Use LLVM backend (better codegen)

**Risk**: Borrow checker too complex to implement

**Mitigation**:
- Start with simplified version
- Add complexity gradually
- Learn from Rust's approach (Polonius)

### 5.2 Schedule Risks

**Risk**: Development takes longer than planned

**Mitigation**:
- Build in 20% schedule buffer
- Prioritize ruthlessly
- Release early, iterate

**Risk**: Feature creep

**Mitigation**:
- Stick to v1.0 spec strictly
- Defer non-essential features
- Separate "nice to have" from "must have"

### 5.3 Resource Risks

**Risk**: Limited developer time

**Mitigation**:
- Clear milestones
- Good documentation for contributors
- Automated testing

## 6. Quality Assurance

### 6.1 Testing Strategy

**Unit Tests**:
- Each compiler phase
- Edge cases and error conditions
- Regression tests

**Integration Tests**:
- End-to-end compilation
- Real programs
- Standard library tests

**Fuzz Testing**:
- Parser fuzzing
- Type checker fuzzing
- Find crashes and hangs

**Performance Tests**:
- Compilation speed benchmarks
- Memory usage tracking
- Generated code performance

### 6.2 Continuous Integration

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test-stage0:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build Stage 0
        run: cargo build --release
      - name: Run tests
        run: cargo test
      - name: Integration tests
        run: ./test_suite.sh

  test-stage1:
    needs: test-stage0
    runs-on: ubuntu-latest
    steps:
      - name: Build Stage 1 with Stage 0
        run: ./vertex0 build stage1/
      - name: Test Stage 1
        run: ./vertex1 test
```

### 6.3 Code Review Process

- All changes require review
- Maintain high code quality
- Document complex algorithms
- Test coverage requirements

## 7. Community Building

### 7.1 Open Source Strategy

**License**: MIT or Apache 2.0 (permissive)

**Repository**:
- GitHub organization
- Clear contributing guidelines
- Issue templates
- PR templates

### 7.2 Documentation

**Target Audiences**:
1. Users: How to use Vertex
2. Contributors: How to hack on compiler
3. Library authors: How to write libraries

**Documentation Types**:
- Tutorial (The Vertex Book)
- Reference (Language spec)
- API docs (Standard library)
- Internals (Compiler architecture)

### 7.3 Release Strategy

**Versioning**: Semantic versioning (semver)

**Release Cadence**:
- Stage 0: Single release when complete
- Stage 1: Monthly releases (0.x.0)
- Stage 2: 6-week release cycle (stable)

**Stability**:
- No breaking changes in minor versions
- Deprecation warnings before removal
- Long-term support (LTS) releases

## 8. Success Metrics

### 8.1 Stage 0 Success Criteria

- [ ] Fully supports generics (types and functions)
- [ ] Fully supports traits (definitions, implementations, bounds, associated types)
- [ ] Fully supports closures (Fn/FnMut/FnOnce)
- [ ] Can compile HashMap<K,V>, Vec<T>, Iterator trait with associated types
- [ ] Overflow checking works in debug mode, wrapping in release
- [ ] String indexing prohibition enforced (no Index<usize> for String/str)
- [ ] 150+ passing tests (including generic/trait/overflow tests)
- [ ] Can compile a 1500+ line program using generics
- [ ] Reasonable error messages with trait bound failures
- [ ] Compilation speed > 3000 lines/sec (acceptable for Stage 0)

### 8.2 Stage 1 Success Criteria

- [ ] Self-hosting (compiles itself)
- [ ] All Stage 0 tests pass
- [ ] 500+ passing tests
- [ ] Can compile 5000+ line programs
- [ ] Performance within 2x of Stage 0

### 8.3 Stage 2 Success Criteria

- [ ] Production-ready
- [ ] 10,000+ lines/sec compilation
- [ ] Complete standard library
- [ ] 3+ real-world projects using Vertex
- [ ] Active community (100+ contributors)
- [ ] Comprehensive documentation

## 9. Post-Bootstrap Roadmap

After achieving self-hosting, future directions:

**Language Evolution**:
- Async/await (v2.0)
- Const generics
- Advanced type system features

**Tooling**:
- Debugger integration
- Profiler
- Memory sanitizers

**Ecosystem**:
- Package ecosystem growth
- Web framework
- System libraries

**Platform Support**:
- Embedded systems
- WebAssembly
- Mobile platforms

## 10. Unified Feature Staging Matrix

The following table clarifies exactly which features are implemented in each stage:

| Feature Category | Feature | Stage 0 | Stage 1 | Stage 2 |
|-----------------|---------|---------|---------|---------|
| **Core Types** | Primitives (i32, f64, bool, char, etc.) | ✓ | ✓ | ✓ |
| | String, &str | ✓ | ✓ | ✓ |
| | Arrays [T; N] | ✓ | ✓ | ✓ |
| | Slices &[T] | ✓ | ✓ | ✓ |
| | Tuples | ✓ | ✓ | ✓ |
| | Tuple field access (.0, .1) | ✓ | ✓ | ✓ |
| **User Types** | Structs (non-generic) | ✓ | ✓ | ✓ |
| | Structs (generic) | ✓ | ✓ | ✓ |
| | Enums (non-generic) | ✓ | ✓ | ✓ |
| | Enums (generic) | ✓ | ✓ | ✓ |
| | Methods (impl blocks) | ✓ | ✓ | ✓ |
| **Generics** | Generic type parameters | ✓ | ✓ | ✓ |
| | Generic function parameters | ✓ | ✓ | ✓ |
| | Monomorphization | ✓ | ✓ | ✓ |
| | Const generics | ✗ | ✗ | Future |
| **Traits** | Trait definitions | ✓ | ✓ | ✓ |
| | Trait implementations | ✓ | ✓ | ✓ |
| | Trait bounds | ✓ | ✓ | ✓ |
| | Standard traits (Clone, Debug, etc.) | ✓ | ✓ | ✓ |
| | Iterator trait | ✓ | ✓ | ✓ |
| | Associated types | ✓ | ✓ | ✓ |
| | Trait objects (dyn Trait) | ✗ | ✗ | NOT IN v1.0 |
| | Higher-ranked trait bounds | ✗ | ✗ | Maybe |
| **Functions** | Basic functions | ✓ | ✓ | ✓ |
| | Closures | ✓ | ✓ | ✓ |
| | Fn/FnMut/FnOnce traits | ✓ | ✓ | ✓ |
| | Const functions | ✓ | ✓ | ✓ |
| **Control Flow** | if/else | ✓ | ✓ | ✓ |
| | loop/while | ✓ | ✓ | ✓ |
| | for (with IntoIterator) | ✓ | ✓ | ✓ |
| | match (basic patterns) | ✓ | ✓ | ✓ |
| | match (or-patterns, @) | ✗ | ✓ | ✓ |
| | break/continue/return | ✓ | ✓ | ✓ |
| **Memory** | Ownership & moves | ✓ | ✓ | ✓ |
| | Borrowing (&, &mut) | ✓ | ✓ | ✓ |
| | Lifetime inference | ✓ | ✓ | ✓ |
| | Drop trait | ✓ | ✓ | ✓ |
| | Box<T> | ✓ | ✓ | ✓ |
| | Rc<T> | ✓ | ✓ | ✓ |
| | Arc<T> | ✓ | ✓ | ✓ |
| | RefCell<T> | ✗ | ✓ | ✓ |
| **Collections** | Vec<T> | ✓ | ✓ | ✓ |
| | HashMap<K,V> | ✓ | ✓ | ✓ |
| | HashSet<T> | ✓ | ✓ | ✓ |
| | BTreeMap/BTreeSet | ✗ | ✓ | ✓ |
| **Error Handling** | Result<T, E> | ✓ | ✓ | ✓ |
| | Option<T> | ✓ | ✓ | ✓ |
| | ? operator | ✓ | ✓ | ✓ |
| | panic/assert | ✓ | ✓ | ✓ |
| | catch_unwind | ✗ | ✗ | ✓ |
| **I/O** | print/println | ✓ | ✓ | ✓ |
| | Read/Write traits | ✓ | ✓ | ✓ |
| | File I/O | ✓ | ✓ | ✓ |
| | Networking | ✗ | Basic | ✓ |
| **Modules** | mod/use | ✓ | ✓ | ✓ |
| | pub visibility | ✓ | ✓ | ✓ |
| | pub(crate), pub(super) | ✓ | ✓ | ✓ |
| **Advanced** | Unsafe code | ✗ | ✗ | ✓ |
| | FFI (C ABI) | ✗ | Basic | ✓ |
| | Inline assembly | ✗ | ✗ | ✓ |
| | LLVM backend | ✗ | Optional | ✓ |

**Key Insights**:
- **Stage 0 is substantial**: Generics, traits (with associated types), closures are essential
- **Stage 1 adds polish**: Advanced patterns, RefCell, complete standard library
- **Stage 2 adds safety escapes**: Unsafe code, full FFI, platform-specific features
- **Trait objects**: NOT in v1.0 spec - deferred to future versions

## 11. Conclusion

The three-stage bootstrap plan provides a clear path from initial implementation to a production-ready, self-hosted Vertex compiler. By following this plan:

1. **Stage 0** (9-12 months): Compiler with generics, traits, closures in Rust
2. **Stage 1** (6-12 months): Full compiler rewritten in Vertex
3. **Stage 2** (3-6 months): Production polish and ecosystem

**Total Timeline**: 18-30 months to production-ready compiler

**Note**: Stage 0 is more complex than initially envisioned due to the need for generics, traits (including associated types), and closures. These features are fundamental requirements for a self-hosting compiler and cannot be deferred. The Iterator trait requires associated types, making them essential for Stage 0.

**Keys to Success**:
- Incremental development
- Comprehensive testing
- Community engagement
- Realistic scope management
- Focus on core value proposition

The bootstrap journey transforms Vertex from a specification into a living, self-sustaining programming language.

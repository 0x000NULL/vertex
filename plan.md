# Vertex Language Implementation Plan

**Version**: 1.0.0
**Created**: December 2024
**Status**: Implementation Roadmap

---

## Executive Summary

### Overview of Vertex Language

Vertex is a memory-safe systems programming language that aims to provide **80% of Rust's safety with 50% of the complexity**. It targets the same domain as Rust but with:

- **Simplified lifetime system**: Most lifetimes inferred, no lifetime polymorphism
- **No macro system**: Only built-in syntax (vec!, print, derive)
- **Cleaner syntax**: Fewer symbols, more keywords
- **Clear error handling**: Result<T, E> for errors, Option<T> for optional values

**Core Value Proposition**: A gentler learning curve while maintaining memory safety without garbage collection.

### Key Goals

1. **Stage 0** (Months 1-12): Bootstrap compiler in Rust/C++ with full generics, traits, and closures
2. **Stage 1** (Months 13-24): Self-hosted compiler rewritten in Vertex
3. **Stage 2** (Months 25-30): Production-ready with optimization and complete toolchain

**Total Timeline**: 24-30 months to production readiness

### Success Metrics

**Stage 0 Completion Criteria**:
- Compiles programs using generics (types and functions with type parameters)
- Supports trait system with associated types (essential for Iterator)
- Implements closures with Fn/FnMut/FnOnce traits
- Can compile HashMap<K,V>, Vec<T>, String with full standard library
- Enforces string indexing prohibition at compile time
- Overflow checking in debug mode, wrapping in release
- 150+ passing tests including complex generic/trait programs
- Reasonable error messages with trait bound diagnostics

**Stage 1 Completion Criteria**:
- Self-hosting: compiler compiles itself
- All Stage 0 tests pass plus 350+ additional tests
- Can compile 5000+ line programs
- Build system (vertex.toml support)

**Stage 2 Completion Criteria**:
- Production-ready compiler
- Compilation speed: 10,000+ lines/second
- Complete standard library with all prelude items
- Toolchain: formatter, documentation generator, LSP server
- 3+ real-world projects using Vertex

---

## Stage 0: Bootstrap Compiler (Months 1-12)

### Overview

**Objective**: Create a working compiler in Rust (or C++) that can compile a useful subset of Vertex to C code, including full generic support, trait system with associated types, and closures.

**Implementation Language**: **Rust** (recommended)
- Similar type system makes porting easier
- Excellent ecosystem for compiler development
- Memory safety during development
- Rich parsing libraries (LALRPOP, pest, nom, or hand-written)

**Target Backend**: C code generation (primary), optional LLVM

**Critical Requirements for Stage 0**:
- MUST include full generic type system (required for self-hosting)
- MUST include trait system with associated types (required for Iterator)
- MUST include closures with Fn/FnMut/FnOnce (required for functional programming)
- These are NOT optional - they're fundamental to the language

---

### Phase 1: Foundation (Months 1-2)

#### Month 1: Lexer and Basic Parser

**Milestones**:
- ✅ Complete lexer with all token types
- ✅ Source location tracking infrastructure
- ✅ Error reporting system
- ✅ Basic expression parser
- ✅ Test infrastructure

**Technical Tasks**:

1. **Lexer Implementation** (Week 1)
   - Tokenize all Vertex keywords (29 total: break, const, continue, else, enum, extern, false, fn, for, if, impl, in, let, loop, match, mod, mut, pub, return, self, struct, trait, true, type, unsafe, use, where, while)
   - Handle numeric literals (decimal, hex, binary, float) with type suffixes
   - String literals (regular "" and raw r"")
   - Character literals with Unicode support
   - Operators: arithmetic (+, -, *, /, %), comparison (==, !=, <, >, <=, >=), logical (and, or, not), bitwise (&, |, ^, ~, <<, >>)
   - Source span tracking (file, line, column, byte offset)
   - Comment handling (// and /* */)

   **Acceptance Criteria**:
   - Tokenizes all valid Vertex syntax
   - Preserves exact source locations for error reporting
   - Handles UTF-8 correctly
   - Reports lexical errors without panicking

   **Test Strategy**:
   - Unit tests for each token type
   - Edge case tests (empty file, only comments, Unicode)
   - Error recovery tests

2. **Error Reporting Infrastructure** (Week 1)
   - Design error struct with code, span, message, suggestions
   - Implement error accumulator (collect multiple errors)
   - Pretty-print errors with source snippets
   - Color-coded terminal output (using termcolor or similar)

   **Acceptance Criteria**:
   - Beautiful error messages similar to Rust
   - Multiple errors collected before stopping
   - Suggestions for common mistakes

3. **Basic Parser - Expressions** (Week 2)
   - Pratt parser or recursive descent for expressions
   - Binary operators with correct precedence
   - Unary operators (-, not, *, &, &mut)
   - Literals (int, float, bool, char, string)
   - Parenthesized expressions
   - Function calls
   - Method calls
   - Field access
   - Tuple field access (.0, .1, etc.)
   - Array/slice indexing

   **Acceptance Criteria**:
   - Correct operator precedence
   - Error recovery at expression boundaries
   - Preserves source spans for all nodes

4. **Basic Parser - Statements** (Week 3)
   - let bindings with pattern matching
   - Expression statements
   - if/else
   - loop, while, for
   - return, break, continue
   - match expressions

   **Acceptance Criteria**:
   - Parses all control flow constructs
   - Error recovery at statement boundaries

5. **Test Infrastructure** (Week 4)
   - Test harness for lexer
   - Test harness for parser
   - Snapshot testing for AST
   - Golden file tests for error messages
   - CI setup (GitHub Actions or similar)

   **Acceptance Criteria**:
   - Automated test suite runs on every commit
   - Easy to add new tests
   - Fast test execution (<5 seconds for unit tests)

**Dependencies**: None

**Risks**:
- Parser complexity may take longer than expected
- **Mitigation**: Use a parser generator library (LALRPOP) if hand-written parser is too complex

#### Month 2: Full Parser and AST

**Milestones**:
- ✅ Complete AST definition
- ✅ Parse all item types (functions, structs, enums, traits, impls, mods)
- ✅ Parse generic type parameters
- ✅ Parse trait bounds
- ✅ Pattern matching in let, match, function parameters
- ✅ Built-in syntax parsing (vec!, print, derive)

**Technical Tasks**:

1. **AST Design** (Week 1)
   - Define all AST node types
   - Include generic parameters in all relevant nodes
   - Span information on every node
   - Use arena allocation for AST nodes (saves memory)

   **Key AST Types**:
   ```rust
   enum Item {
       Fn(FnItem),        // Functions with generics
       Struct(StructItem), // Structs with generics
       Enum(EnumItem),     // Enums with generics
       Impl(ImplItem),     // Impl blocks (trait impls)
       Trait(TraitItem),   // Trait definitions
       Mod(ModItem),       // Modules
       Use(UseItem),       // Imports
       Const(ConstItem),   // Constants
       TypeAlias(TypeAliasItem),
   }

   struct Generics {
       params: Vec<TypeParam>,  // <T, U>
       where_clause: Option<WhereClause>,  // where T: Clone
   }

   struct TypeParam {
       name: Ident,
       bounds: Vec<TraitBound>,  // T: Clone + Debug
   }
   ```

   **Acceptance Criteria**:
   - AST can represent all Vertex syntax
   - Includes generic type parameters everywhere needed
   - Memory-efficient representation

2. **Item Parsing** (Week 2)
   - Function items with generics: `fn foo<T>(x: T) -> T`
   - Struct items with generics: `struct Vec<T> { ... }`
   - Enum items with generics: `enum Result<T, E> { Ok(T), Err(E) }`
   - Trait definitions with associated types: `trait Iterator { type Item; ... }`
   - Impl blocks: `impl<T> Clone for Vec<T> where T: Clone { ... }`
   - Module declarations
   - Use statements (imports)

   **Acceptance Criteria**:
   - Parses all item types
   - Handles generic parameters correctly
   - Error recovery at item boundaries

3. **Type Parsing** (Week 3)
   - Generic types: `Vec<i32>`, `HashMap<K, V>`
   - References: `&T`, `&mut T`
   - Slices: `&[T]`
   - Arrays: `[T; N]`
   - Tuples: `(T1, T2, T3)`
   - Function types: `fn(T) -> U`
   - Trait bounds in types: `T: Clone + Debug`

   **Acceptance Criteria**:
   - Handles nested generics: `Vec<Vec<T>>`
   - Parses complex trait bounds: `T: Iterator<Item=i32>`

4. **Pattern Parsing** (Week 4)
   - Literal patterns
   - Identifier patterns with @ binding
   - Tuple patterns
   - Struct patterns (both named and tuple)
   - Enum patterns
   - Or-patterns: `Some(x) | None`
   - Range patterns: `0..=100`
   - Reference patterns: `ref x`, `ref mut x`

   **Acceptance Criteria**:
   - Parses all pattern types
   - Handles nested patterns
   - Error recovery on malformed patterns

5. **Built-in Syntax Parsing** (Week 4)
   - `vec![...]` and `vec![x; n]` - IMPORTANT: Despite the `!`, this is NOT a macro, it's built-in syntax
   - `print()`, `println()`, `format()` - built-in functions with format string validation
   - `#[derive(...)]` attributes
   - Array repeat syntax: `[0; 256]`

   **Acceptance Criteria**:
   - Parses all built-in syntax as special AST nodes
   - Format strings validated during parsing

**Dependencies**: Month 1 completion

**Risks**:
- Generic syntax is complex and error-prone
- **Mitigation**: Extensive test suite with complex generic examples

---

### Phase 2: Type System (Months 3-5)

#### Month 3: Name Resolution

**Milestones**:
- ✅ Module system implementation
- ✅ File-based module loading
- ✅ Scope hierarchy
- ✅ Name binding and resolution
- ✅ Import resolution (use statements)
- ✅ Visibility checking

**Technical Tasks**:

1. **Module Discovery** (Week 1)
   - File-system-based module loading
   - Resolve `mod foo;` to `foo.vx` or `foo/mod.vx`
   - Build module tree from crate root (main.vx or lib.vx)
   - Detect circular module dependencies
   - Handle inline modules: `mod foo { ... }`

   **Algorithm**:
   ```rust
   fn load_module(parent_path: &Path, mod_name: &str) -> Result<Module> {
       let file_path = parent_path.join(format!("{}.vx", mod_name));
       let dir_path = parent_path.join(mod_name).join("mod.vx");

       match (file_path.exists(), dir_path.exists()) {
           (true, true) => Err(Error::AmbiguousModule),
           (true, false) => parse_file(file_path),
           (false, true) => parse_file(dir_path),
           (false, false) => Err(Error::ModuleNotFound),
       }
   }
   ```

   **Acceptance Criteria**:
   - Loads all modules correctly
   - Detects circular dependencies
   - Clear error messages for missing modules

2. **Scope Management** (Week 2)
   - Build scope hierarchy (module, function, block, loop)
   - Track definitions in each scope
   - Handle shadowing correctly
   - Support forward references within modules

   **Data Structures**:
   ```rust
   struct Scope {
       parent: Option<ScopeId>,
       defs: HashMap<String, DefId>,
       kind: ScopeKind,  // Module, Function, Block, Loop
   }

   struct DefId {
       crate_id: CrateId,
       module: ModuleId,
       local_id: LocalDefId,
   }
   ```

   **Acceptance Criteria**:
   - Correctly resolves names to definitions
   - Handles shadowing properly
   - Forward references work

3. **Import Resolution** (Week 3)
   - Resolve `use` statements
   - Handle glob imports: `use foo::*;`
   - Re-exports: `pub use internal::public_api;`
   - Detect ambiguous imports
   - Build import table

   **Acceptance Criteria**:
   - All imports resolve correctly
   - Ambiguous imports detected
   - Visibility rules enforced

4. **Visibility Checking** (Week 4)
   - `pub` vs private items
   - `pub(crate)` and `pub(super)` support
   - Visibility across modules
   - Error on private access

   **Acceptance Criteria**:
   - Visibility rules enforced
   - Clear error messages for visibility violations

**Dependencies**: Month 2 completion

**Risks**:
- Module system complexity with file I/O errors
- **Mitigation**: Comprehensive test suite with various module layouts

#### Month 4: Type Checking Foundation

**Milestones**:
- ✅ Type representation
- ✅ Type inference engine
- ✅ Unification algorithm
- ✅ Basic trait resolution
- ✅ Method resolution

**Technical Tasks**:

1. **Type Representation** (Week 1)
   - Design internal type representation
   - Handle generic types: `Vec<T>`, `HashMap<K, V>`
   - Type inference variables (TyVar)
   - Never type (!)

   **Type System**:
   ```rust
   enum Ty {
       Bool, Char, Int(IntTy), Uint(UintTy), Float(FloatTy),
       Str, Never,
       Tuple(Vec<Ty>),
       Array(Box<Ty>, u64),
       Slice(Box<Ty>),
       Ref(Region, Box<Ty>, Mutability),
       Fn(FnSig),
       Adt(AdtDef, Vec<Ty>),  // Struct/Enum with generic args
       Param(ParamId),         // Generic parameter T
       Projection(TraitRef, Ident),  // Associated type T::Item
       Infer(InferVar),        // Type inference variable ?T
       Error,                  // Type error placeholder
   }
   ```

   **Acceptance Criteria**:
   - Can represent all Vertex types
   - Handles generic type substitution
   - Efficient memory layout

2. **Hindley-Milner Inference** (Week 2-3)
   - Generate type constraints from expressions
   - Fresh inference variables for unknowns
   - Collect equality constraints
   - Unification algorithm

   **Algorithm**:
   ```rust
   fn infer_expr(&mut self, expr: &Expr) -> Ty {
       match expr {
           Expr::Literal(lit) => self.infer_literal(lit),
           Expr::Binary { op, lhs, rhs } => {
               let lhs_ty = self.infer_expr(lhs);
               let rhs_ty = self.infer_expr(rhs);
               self.unify(lhs_ty, rhs_ty);
               // Return result type based on operator
           }
           Expr::Call { func, args } => {
               let func_ty = self.infer_expr(func);
               // Extract function signature, check args
           }
           // ... other cases
       }
   }

   fn unify(&mut self, ty1: Ty, ty2: Ty) -> Result<(), TypeError> {
       match (ty1, ty2) {
           (Ty::Infer(v1), ty2) => self.bind_var(v1, ty2),
           (ty1, Ty::Infer(v2)) => self.bind_var(v2, ty1),
           (Ty::Int(i1), Ty::Int(i2)) if i1 == i2 => Ok(()),
           (Ty::Adt(def1, args1), Ty::Adt(def2, args2)) => {
               if def1 == def2 {
                   for (a1, a2) in args1.iter().zip(args2.iter()) {
                       self.unify(a1.clone(), a2.clone())?;
                   }
                   Ok(())
               } else {
                   Err(TypeError::Mismatch)
               }
           }
           _ => Err(TypeError::Mismatch),
       }
   }
   ```

   **Acceptance Criteria**:
   - Infers types correctly for simple expressions
   - Unification handles substitution
   - Type errors reported clearly

3. **Basic Trait Resolution** (Week 4)
   - Collect trait definitions
   - Build impl table: which types implement which traits
   - Resolve method calls to trait impls
   - Check trait bounds on generic parameters

   **Acceptance Criteria**:
   - Method calls resolve correctly
   - Trait bounds checked
   - Clear errors for missing impls

**Dependencies**: Month 3 completion

**Risks**:
- Type inference is complex and bug-prone
- **Mitigation**: Test-driven development with extensive type inference tests

#### Month 5: Generic Type System and Trait System with Associated Types

**Milestones**:
- ✅ Generic type instantiation
- ✅ Trait system with associated types (CRITICAL)
- ✅ Iterator trait implementation
- ✅ Generic function monomorphization (early version)

**Technical Tasks**:

1. **Generic Type Instantiation** (Week 1-2)
   - Substitute type parameters with concrete types
   - Handle nested generics: `Vec<Vec<i32>>`
   - Generic function calls: `identity::<i32>(42)`
   - Type parameter inference from arguments

   **Algorithm**:
   ```rust
   fn instantiate_generic(&mut self, def_id: DefId, type_args: Vec<Ty>) -> Ty {
       let generics = self.get_generics(def_id);
       assert_eq!(generics.params.len(), type_args.len());

       let subst = Substitution::new(
           generics.params.iter().map(|p| p.id).collect(),
           type_args
       );

       self.apply_subst(def_id, subst)
   }
   ```

   **Acceptance Criteria**:
   - Can instantiate `Vec<i32>`, `HashMap<String, Vec<i32>>`
   - Type parameter inference works: `vec![1, 2, 3]` infers `Vec<i32>`
   - Error on wrong number of type arguments

2. **Trait System with Associated Types** (Week 2-3) - **CRITICAL FOR STAGE 0**
   - Parse associated type declarations in traits
   - Parse associated type bindings in impls
   - Resolve associated types in type checking
   - Implement projection types: `<T as Iterator>::Item`

   **Key Trait: Iterator**:
   ```rust
   trait Iterator {
       type Item;  // Associated type
       fn next(&mut self) -> Result<Self::Item, ()>;
   }

   impl Iterator for Range<i32> {
       type Item = i32;  // Concrete associated type
       fn next(&mut self) -> Result<i32, ()> { ... }
   }
   ```

   **Type Checking with Associated Types**:
   ```rust
   fn resolve_associated_type(&mut self, base_ty: Ty, trait_ref: TraitRef, assoc: Ident) -> Ty {
       // Find the impl of trait_ref for base_ty
       let impl_id = self.find_impl(base_ty, trait_ref)?;

       // Get the associated type binding from impl
       let assoc_ty = self.get_assoc_type(impl_id, assoc)?;

       assoc_ty
   }
   ```

   **Acceptance Criteria**:
   - Can define traits with associated types
   - Can implement traits with associated type bindings
   - Iterator trait works: `fn sum<I: Iterator<Item=i32>>(iter: I)`
   - Type checking resolves `T::Item` correctly
   - Clear errors for missing associated type bindings

3. **Standard Trait Implementations** (Week 4)
   - Clone, Copy (marker traits)
   - Debug, Display
   - PartialEq, Eq, PartialOrd, Ord
   - Hash
   - Iterator, IntoIterator (with associated types)
   - From, Into

   **Acceptance Criteria**:
   - All standard traits defined
   - Example implementations for primitive types
   - Derive macro stubs (implement in next phase)

**Dependencies**: Month 4 completion

**Risks**:
- Associated types add significant complexity
- **Mitigation**: Study Rust's implementation, incremental testing
- **CRITICAL**: This cannot be skipped or deferred - Iterator requires associated types

---

### Phase 3: Advanced Features (Months 6-7)

#### Month 6: Closure Support

**Milestones**:
- ✅ Closure parsing and AST representation
- ✅ Closure capture analysis
- ✅ Fn/FnMut/FnOnce trait hierarchy
- ✅ Closure type checking

**Technical Tasks**:

1. **Closure Capture Analysis** (Week 1-2)
   - Determine which variables are captured
   - Classify captures: immutable borrow, mutable borrow, or move
   - Handle `move` closures
   - Detect closure escape requirements

   **Algorithm**:
   ```rust
   fn analyze_captures(&mut self, closure: &Closure) -> CaptureSet {
       let mut captures = CaptureSet::new();

       for var in self.free_variables(closure.body) {
           let mode = if self.is_mutated(var, closure.body) {
               CaptureMode::MutBorrow
           } else if self.needs_move(var, closure) {
               CaptureMode::Move
           } else {
               CaptureMode::ImmBorrow
           };

           captures.insert(var, mode);
       }

       if closure.is_move {
           // Force all captures to Move
           captures.make_all_move();
       }

       captures
   }
   ```

   **Acceptance Criteria**:
   - Correctly identifies all captured variables
   - Classifies capture modes accurately
   - Handles `move` keyword

2. **Closure Traits** (Week 2-3)
   - Implement Fn, FnMut, FnOnce trait hierarchy
   - Assign correct trait to each closure
   - Type check closure calls

   **Trait Hierarchy**:
   ```rust
   trait FnOnce<Args> {
       type Output;
       fn call_once(self, args: Args) -> Self::Output;
   }

   trait FnMut<Args>: FnOnce<Args> {
       fn call_mut(&mut self, args: Args) -> Self::Output;
   }

   trait Fn<Args>: FnMut<Args> {
       fn call(&self, args: Args) -> Self::Output;
   }
   ```

   **Closure Trait Assignment**:
   - Immutable captures only → Fn
   - Mutable captures → FnMut
   - Move captures / consumes captured values → FnOnce

   **Acceptance Criteria**:
   - Closures assigned correct trait
   - Type checking enforces trait bounds
   - Error messages for incorrect closure usage

3. **Closure Type Checking** (Week 4)
   - Infer closure types from context
   - Handle generic closure parameters
   - Check closure trait bounds in function signatures

   **Example**:
   ```vertex
   fn map<T, U, F>(slice: &[T], f: F) -> Vec<U>
       where F: Fn(&T) -> U
   {
       // ...
   }
   ```

   **Acceptance Criteria**:
   - Closures type check correctly in generic contexts
   - Trait bound checking works
   - Infers closure return types

**Dependencies**: Month 5 completion

**Risks**:
- Closure capture analysis is complex
- **Mitigation**: Test with various closure examples from Rust

#### Month 7: Iterator Trait and Standard Library Foundation

**Milestones**:
- ✅ Iterator trait fully implemented with associated types
- ✅ IntoIterator trait
- ✅ Basic iterator combinators (map, filter, fold)
- ✅ for-loop desugaring

**Technical Tasks**:

1. **Iterator Trait Implementation** (Week 1)
   - Complete Iterator trait with `type Item` associated type
   - Implement for primitive ranges (Range<i32>, etc.)
   - Implement next() method returning `Result<Self::Item, ()>`

   **Acceptance Criteria**:
   - Iterator trait fully functional
   - Range types iterate correctly
   - next() returns Result correctly

2. **Iterator Combinators** (Week 2)
   - map, filter, fold, collect
   - Chain, zip, enumerate
   - take, skip, take_while, skip_while

   **Example Implementation**:
   ```rust
   struct Map<I, F> {
       iter: I,
       f: F,
   }

   impl<I, F, B> Iterator for Map<I, F>
   where
       I: Iterator,
       F: FnMut(I::Item) -> B,
   {
       type Item = B;

       fn next(&mut self) -> Result<B, ()> {
           match self.iter.next() {
               Ok(item) => Ok((self.f)(item)),
               Err(()) => Err(()),
           }
       }
   }
   ```

   **Acceptance Criteria**:
   - Common combinators work
   - Chain iterator adapters: `iter.map(f).filter(p).collect()`
   - Type checking works with complex iterator chains

3. **IntoIterator Trait** (Week 3)
   - Define IntoIterator with associated types
   - Implement for Vec, slice, array, range
   - Use in for-loop desugaring

   **Trait Definition**:
   ```rust
   trait IntoIterator {
       type Item;
       type IntoIter: Iterator<Item=Self::Item>;
       fn into_iter(self) -> Self::IntoIter;
   }
   ```

   **Acceptance Criteria**:
   - IntoIterator works for all collection types
   - for-loops desugar correctly

4. **For-Loop Desugaring** (Week 4)
   - Transform `for x in iter { body }` to while-loop with Iterator::next()
   - Handle break/continue correctly

   **Desugaring**:
   ```vertex
   // for x in iter { body }
   // Becomes:
   {
       let mut __iter = IntoIterator::into_iter(iter);
       loop {
           match __iter.next() {
               Ok(x) => { body }
               Err(()) => break
           }
       }
   }
   ```

   **Acceptance Criteria**:
   - for-loops work over all iterable types
   - break/continue work correctly
   - Error messages point to original source

**Dependencies**: Month 6 completion

**Risks**:
- Iterator combinators are complex with many edge cases
- **Mitigation**: Extensive testing, refer to Rust's implementation

---

### Phase 4: Safety Analysis (Months 8-9)

#### Month 8: MIR Generation

**Milestones**:
- ✅ MIR definition (control flow graph with basic blocks)
- ✅ Lower HIR to MIR
- ✅ Generate explicit control flow
- ✅ Insert drop statements

**Technical Tasks**:

1. **MIR Design** (Week 1)
   - Define MIR data structures
   - Basic blocks with statements and terminators
   - Explicit control flow (no nesting)

   **MIR Structure**:
   ```rust
   struct Mir {
       basic_blocks: IndexVec<BasicBlock, BasicBlockData>,
       local_decls: IndexVec<Local, LocalDecl>,
       arg_count: usize,
       return_ty: Ty,
   }

   struct BasicBlockData {
       statements: Vec<Statement>,
       terminator: Terminator,
   }

   enum Statement {
       Assign(Place, Rvalue),
       StorageLive(Local),
       StorageDead(Local),
       Nop,
   }

   enum Terminator {
       Goto { target: BasicBlock },
       SwitchInt { discr: Operand, targets: SwitchTargets },
       Return,
       Unreachable,
       Drop { place: Place, target: BasicBlock, unwind: Option<BasicBlock> },
       Call { func: Operand, args: Vec<Operand>, destination: Place, target: BasicBlock },
   }
   ```

   **Acceptance Criteria**:
   - MIR can represent all Vertex control flow
   - Basic blocks have single entry, single exit

2. **HIR to MIR Lowering** (Week 2-3)
   - Convert typed HIR to MIR
   - Build control flow graph
   - Handle loops, branches, match
   - Insert temporary variables

   **Acceptance Criteria**:
   - All HIR constructs lower to MIR
   - Control flow correct

3. **Drop Elaboration** (Week 4)
   - Identify values that need dropping
   - Insert explicit Drop terminators
   - Handle drop order (reverse declaration order)
   - Generate drop flags for conditional initialization
   - Handle unwinding (drop on panic)

   **Drop Order Rules**:
   - Local variables: reverse declaration order
   - Struct fields: declaration order
   - Tuple elements: left to right
   - Function arguments: reverse order

   **Acceptance Criteria**:
   - Drop statements inserted correctly
   - Drop order matches specification
   - Unwinding drops all initialized values

**Dependencies**: Month 7 completion

**Risks**:
- MIR generation is complex
- **Mitigation**: Start with simple functions, gradually add complexity

#### Month 9: Borrow Checker

**Milestones**:
- ✅ Polonius-inspired borrow checking
- ✅ Move semantics checking
- ✅ Lifetime inference (simplified)
- ✅ Clear borrow check error messages

**Technical Tasks**:

1. **Borrow Check Algorithm** (Week 1-3)
   - Data-flow analysis on MIR
   - Track borrows across basic blocks
   - Check aliasing rules (no simultaneous &mut, or &mut + &)
   - Validate reference lifetimes

   **Algorithm** (Polonius-inspired):
   ```rust
   fn check_borrows(&mut self, mir: &Mir) {
       // Compute liveness information
       let liveness = self.compute_liveness(mir);

       // Track active borrows at each program point
       let mut active_borrows = BorrowSet::new();

       for (bb, data) in mir.basic_blocks.iter_enumerated() {
           for (idx, stmt) in data.statements.iter().enumerate() {
               let location = Location { block: bb, statement_index: idx };

               // Check statement doesn't violate borrow rules
               self.check_statement(stmt, &active_borrows, location)?;

               // Update active borrows
               active_borrows = self.update_borrows(active_borrows, stmt);
           }
       }
   }

   fn check_statement(&mut self, stmt: &Statement, borrows: &BorrowSet, loc: Location) -> Result<()> {
       match stmt {
           Statement::Assign(place, rvalue) => {
               // Check place is not borrowed mutably
               if borrows.has_active_mut_borrow(place) {
                   return Err(BorrowError::MutBorrowConflict { ... });
               }

               // Check rvalue doesn't violate borrows
               match rvalue {
                   Rvalue::Ref(mutability, borrowed_place) => {
                       if *mutability == Mutability::Mut {
                           // Check no other borrows active
                           if borrows.has_any_borrow(borrowed_place) {
                               return Err(BorrowError::CannotBorrowAsMutable { ... });
                           }
                       }
                   }
                   _ => {}
               }
           }
           _ => {}
       }
       Ok(())
   }
   ```

   **Acceptance Criteria**:
   - Detects use-after-move
   - Detects simultaneous &mut borrows
   - Detects &mut + & conflicts
   - Allows multiple immutable borrows

2. **Move Checking** (Week 3)
   - Track moved values
   - Error on use-after-move
   - Handle partial moves (struct fields)

   **Acceptance Criteria**:
   - Detects all use-after-move errors
   - Handles partial moves correctly

3. **Lifetime Inference** (Week 4)
   - Infer function return lifetimes
   - Simple inference rules (no explicit lifetimes)
   - Error when inference is ambiguous

   **Inference Rules**:
   - Single input reference → output tied to that input
   - Multiple inputs → output is shortest lifetime
   - Methods with &self → return borrows from self

   **Acceptance Criteria**:
   - Lifetime inference works for common patterns
   - Clear errors when inference fails

4. **Error Messages** (Week 4)
   - Beautiful error messages with source snippets
   - Explain borrow conflicts clearly
   - Suggest fixes (e.g., use clone())

   **Acceptance Criteria**:
   - Error messages similar to Rust quality
   - Users can understand and fix borrow errors

**Dependencies**: Month 8 completion

**Risks**:
- Borrow checking is the hardest part of the compiler
- **Mitigation**: Study Rust's Polonius, implement incrementally, extensive testing

---

### Phase 5: Code Generation (Months 10-11)

#### Month 10: C Backend Foundation

**Milestones**:
- ✅ MIR to C translation
- ✅ Generate C functions
- ✅ Handle basic types
- ✅ Struct and enum layout

**Technical Tasks**:

1. **C Code Generator Structure** (Week 1)
   - Design C codegen architecture
   - Handle name mangling for generics
   - Generate C header declarations

   **Name Mangling**:
   ```rust
   fn mangle_name(def_id: DefId, generic_args: &[Ty]) -> String {
       let mut name = format!("vertex_{}", def_id);
       if !generic_args.is_empty() {
           name.push_str("_");
           for ty in generic_args {
               name.push_str(&mangle_type(ty));
           }
       }
       name
   }
   ```

   **Acceptance Criteria**:
   - Generated C code compiles
   - Names are unique and valid C identifiers

2. **Type Translation** (Week 1)
   - Map Vertex types to C types
   - Generate struct definitions
   - Generate enum definitions (tagged unions)

   **Type Mapping**:
   - i32 → int32_t
   - u64 → uint64_t
   - bool → bool (C99)
   - &T → T* (const)
   - &mut T → T*
   - Vec<T> → vertex_Vec_T struct

   **Acceptance Criteria**:
   - All Vertex types map to C types
   - Struct layout matches C ABI

3. **Function Translation** (Week 2)
   - Generate C function definitions
   - Translate function calls
   - Handle return values

   **Acceptance Criteria**:
   - Simple functions compile and run correctly

4. **Expression Translation** (Week 3)
   - Arithmetic operations with overflow checking (debug mode)
   - Wrapping arithmetic (release mode)
   - Boolean operations
   - Comparisons

   **Overflow Checking** (Debug Mode):
   ```c
   // a + b with overflow check
   {
       int32_t __result;
       if (__builtin_add_overflow(a, b, &__result)) {
           vertex_panic("attempt to add with overflow", "main.vx", 42);
       }
       __result  // evaluates to result
   }
   ```

   **Wrapping Arithmetic** (Release Mode):
   ```c
   (a + b)  // No checks, wrapping behavior
   ```

   **Acceptance Criteria**:
   - Debug mode panics on overflow
   - Release mode wraps (two's complement)
   - Checked methods (checked_add, etc.) always return Result

5. **Control Flow Translation** (Week 4)
   - If/else to C if
   - Loop to C while
   - Match to C switch or if-chain
   - Break/continue

   **Acceptance Criteria**:
   - All control flow constructs work
   - Nested control flow handled correctly

**Dependencies**: Month 9 completion

**Risks**:
- C code generation bugs are hard to debug
- **Mitigation**: Generate readable C code, test with simple programs first

#### Month 11: Monomorphization and Advanced Codegen

**Milestones**:
- ✅ Generic instantiation (monomorphization)
- ✅ Closure code generation
- ✅ Trait method dispatch
- ✅ Runtime library integration

**Technical Tasks**:

1. **Monomorphization Pass** (Week 1-2)
   - Collect all generic instantiations used in program
   - Generate specialized versions for each concrete type set
   - Eliminate unused instantiations

   **Algorithm**:
   ```rust
   struct MonoCollector {
       items: Vec<MonoItem>,
       visited: HashSet<MonoItem>,
   }

   struct MonoItem {
       def_id: DefId,
       substs: Vec<Ty>,  // Type arguments
   }

   fn collect_mono_items(&mut self, mir: &Mir, substs: &[Ty]) {
       for bb in &mir.basic_blocks {
           for stmt in &bb.statements {
               match stmt {
                   Statement::Assign(_, Rvalue::Call(func, args)) => {
                       if let Some(func_def) = self.resolve_func(func) {
                           let func_substs = self.infer_type_args(func_def, args);
                           let mono_item = MonoItem { def_id: func_def, substs: func_substs };
                           if self.visited.insert(mono_item.clone()) {
                               // Recursively process this function
                               self.items.push(mono_item);
                           }
                       }
                   }
                   _ => {}
               }
           }
       }
   }
   ```

   **Acceptance Criteria**:
   - Generates specialized versions of all used generics
   - No duplicate instantiations
   - Unused generics not instantiated

2. **Closure Code Generation** (Week 2)
   - Generate closure structs with captured variables
   - Generate closure call functions
   - Handle different closure traits (Fn/FnMut/FnOnce)

   **Closure Translation**:
   ```vertex
   let x = 10;
   let closure = |y| x + y;
   ```

   Generated C:
   ```c
   // Closure struct
   struct closure_123 {
       int32_t captured_x;
   };

   // Closure call function
   int32_t closure_123_call(const struct closure_123* self, int32_t y) {
       return self->captured_x + y;
   }

   // Usage
   struct closure_123 closure = { .captured_x = x };
   int32_t result = closure_123_call(&closure, 5);
   ```

   **Acceptance Criteria**:
   - Closures compile to C structs
   - Capture variables correctly
   - Different closure traits work

3. **Trait Method Dispatch** (Week 3)
   - Static dispatch for monomorphized trait methods
   - Generate trait impl calls

   **Example**:
   ```vertex
   fn print_debug<T: Debug>(x: T) {
       println("{:?}", x);
   }
   ```

   Monomorphized:
   ```c
   void print_debug_i32(int32_t x) {
       vertex_println(i32_debug(x));
   }

   void print_debug_String(vertex_String x) {
       vertex_println(String_debug(&x));
   }
   ```

   **Acceptance Criteria**:
   - Trait method calls resolve to correct impls
   - Generic trait bounds work

4. **Runtime Library** (Week 4)
   - Implement panic handler
   - Memory allocator wrappers
   - Basic I/O functions
   - Atomic operations

   **Runtime Functions**:
   ```c
   // vertex_runtime.c
   void vertex_panic(const char* msg, const char* file, uint32_t line);
   void* vertex_alloc(size_t size, size_t align);
   void vertex_dealloc(void* ptr, size_t size, size_t align);
   void vertex_print(const char* str);
   void vertex_println(const char* str);
   ```

   **Acceptance Criteria**:
   - Panic works (prints message, exits)
   - Memory allocation works
   - Print functions work

**Dependencies**: Month 10 completion

**Risks**:
- Monomorphization can generate large code
- **Mitigation**: Profile code size, optimize later

---

### Phase 6: Standard Library & Testing (Month 12)

#### Month 12: Standard Library and Comprehensive Testing

**Milestones**:
- ✅ Core types (Result, Option, Box, Rc, Arc)
- ✅ Collections (Vec, HashMap, HashSet, String)
- ✅ Iterator trait and combinators
- ✅ I/O traits and file system
- ✅ Comprehensive test suite (150+ tests)
- ✅ Self-hosting preparation

**Technical Tasks**:

1. **Core Types** (Week 1)
   - Result<T, E> with full API
   - Option<T> with full API
   - Box<T> for heap allocation
   - Rc<T> for reference counting
   - Arc<T> for atomic reference counting

   **Acceptance Criteria**:
   - All core types work correctly
   - Derive macros work (Clone, Debug, etc.)

2. **Collections** (Week 1-2)
   - Vec<T> with push, pop, indexing, iteration
   - HashMap<K, V> with hash implementation
   - HashSet<T>
   - String with UTF-8 support (no indexing!)
   - Enforce string indexing prohibition

   **String Indexing Error**:
   ```vertex
   let s = String::from("hello");
   // let c = s[0];  // ERROR E0608: Cannot index String with usize
   ```

   Error message:
   ```
   error[E0608]: cannot index String with usize
     --> main.vx:2:13
      |
    2 |     let c = s[0];
      |             ^^^^
      |
      = help: use .chars().nth(n) or .as_bytes()[n] instead
      = note: String uses UTF-8 encoding where characters can be 1-4 bytes
   ```

   **Acceptance Criteria**:
   - All collections work correctly
   - HashMap performance is reasonable
   - String indexing blocked at compile time with helpful error

3. **Iterator Trait and Combinators** (Week 2)
   - Full Iterator implementation with associated types
   - map, filter, fold, collect
   - Chain, zip, enumerate, take, skip
   - IntoIterator for all collection types

   **Acceptance Criteria**:
   - Iterator combinators chain correctly
   - Type inference works with iterators
   - for-loops use IntoIterator

4. **I/O and File System** (Week 3)
   - Read and Write traits
   - File open/read/write
   - Standard input/output
   - Error handling with Result

   **Acceptance Criteria**:
   - Can read and write files
   - Error handling works correctly

5. **Test Suite** (Week 4)
   - Unit tests for each compiler phase
   - Integration tests (end-to-end compilation)
   - Test programs covering all features
   - Fuzz testing for parser

   **Test Categories**:
   - Lexer tests (50 tests)
   - Parser tests (100 tests)
   - Type checker tests (200 tests)
   - Borrow checker tests (100 tests)
   - Codegen tests (100 tests)
   - Standard library tests (200 tests)
   - Integration tests (50 programs)

   **Test Programs**:
   - Hello world
   - Fibonacci (recursive and iterative)
   - Generic data structures (Vec, HashMap)
   - Trait usage (Iterator, Display)
   - Closures and higher-order functions
   - File I/O
   - Error handling with Result
   - Complex program (1500+ lines)

   **Acceptance Criteria**:
   - 750+ tests passing
   - Test coverage >80%
   - CI runs all tests on every commit
   - Can compile and run complex programs

6. **Performance Tuning** (Week 4)
   - Profile compiler performance
   - Optimize hot paths
   - Target: 3000+ lines/second

   **Acceptance Criteria**:
   - Compilation speed acceptable for Stage 0

**Dependencies**: Month 11 completion

**Risks**:
- Standard library is large and complex
- **Mitigation**: Prioritize essential types, defer advanced features

---

## Stage 1: Self-Hosting (Months 13-24)

### Overview

**Objective**: Rewrite the compiler in Vertex itself, using only features that Stage 0 supports.

**Strategy**: Incremental module-by-module rewrite, maintaining Stage 0 (Rust) and Stage 1 (Vertex) versions in parallel.

---

### Phase 1: Core Modules Port (Months 13-15)

#### Month 13-15: Lexer and Parser Port

**Milestones**:
- ✅ Port lexer to Vertex
- ✅ Port parser to Vertex
- ✅ Port AST definitions to Vertex
- ✅ Verify equivalence with Stage 0

**Technical Tasks**:

1. **Lexer Port** (Month 13)
   - Rewrite lexer using Vertex's String, Vec, Result
   - Use pattern matching for tokenization
   - Port error handling to Vertex's Result

   **Example**:
   ```vertex
   struct Lexer {
       source: String,
       position: usize,
       tokens: Vec<Token>
   }

   impl Lexer {
       fn next_token(&mut self) -> Result<Token, LexError> {
           // Port logic from Rust version
       }
   }
   ```

   **Acceptance Criteria**:
   - Vertex lexer produces identical output to Stage 0
   - All tests pass

2. **Parser Port** (Month 14-15)
   - Port parser to Vertex
   - Use Vec, HashMap for data structures
   - Use match for parsing logic
   - Handle error recovery

   **Challenges**:
   - No macros, so error handling is more verbose
   - Vertex's simpler lifetimes may require owned types instead of references

   **Acceptance Criteria**:
   - Vertex parser produces identical AST to Stage 0
   - All tests pass

**Dependencies**: Stage 0 completion

**Risks**:
- Performance may be slower in Vertex than Rust
- **Mitigation**: Profile and optimize hot paths

---

### Phase 2: Semantic Analysis Port (Months 16-18)

#### Month 16-18: Name Resolution and Type Checking Port

**Milestones**:
- ✅ Port name resolution
- ✅ Port type checking
- ✅ Port trait resolution
- ✅ Port generic instantiation

**Technical Tasks**:

1. **Name Resolution Port** (Month 16)
   - Port scope management using HashMap
   - Port module system
   - Port import resolution

   **Acceptance Criteria**:
   - Name resolution works identically to Stage 0

2. **Type Checking Port** (Month 17-18)
   - Port type inference
   - Port unification
   - Port trait resolution
   - Port generic instantiation

   **Acceptance Criteria**:
   - Type checking produces same results as Stage 0
   - Error messages are equivalent

**Dependencies**: Month 15 completion

**Risks**:
- Type checker is complex, bugs are likely
- **Mitigation**: Extensive testing, compare outputs with Stage 0

---

### Phase 3: MIR and Borrow Checker Port (Months 19-21)

#### Month 19-21: MIR Generation and Borrow Checking Port

**Milestones**:
- ✅ Port MIR generation
- ✅ Port borrow checker
- ✅ Port drop elaboration

**Technical Tasks**:

1. **MIR Generation Port** (Month 19-20)
   - Port HIR to MIR lowering
   - Port control flow graph construction
   - Port drop elaboration

   **Acceptance Criteria**:
   - MIR identical to Stage 0

2. **Borrow Checker Port** (Month 21)
   - Port borrow checking algorithm
   - Port data-flow analysis
   - Port error reporting

   **Acceptance Criteria**:
   - Borrow checker produces same errors as Stage 0

**Dependencies**: Month 18 completion

**Risks**:
- Borrow checker is the most complex component
- **Mitigation**: Incremental testing, compare with Stage 0

---

### Phase 4: Code Generation Port (Months 22-23)

#### Month 22-23: C Backend Port

**Milestones**:
- ✅ Port C code generator
- ✅ Port monomorphization
- ✅ Add LLVM backend (optional)

**Technical Tasks**:

1. **C Backend Port** (Month 22)
   - Port MIR to C translation
   - Port monomorphization

   **Acceptance Criteria**:
   - Generated C code identical to Stage 0

2. **Optional: LLVM Backend** (Month 23)
   - Add LLVM IR generation (if time permits)
   - Use llvm-sys Rust bindings (called from Vertex via FFI)

   **Acceptance Criteria**:
   - LLVM backend generates working code
   - Performance better than C backend

**Dependencies**: Month 21 completion

**Risks**:
- LLVM backend is complex
- **Mitigation**: LLVM backend is optional, focus on C backend

---

### Phase 5: Self-Hosting Verification (Month 24)

#### Month 24: Self-Hosting and Testing

**Milestones**:
- ✅ Compile Stage 1 with Stage 0
- ✅ Compile Stage 1 with Stage 1 (self-hosting!)
- ✅ Verify binary equivalence
- ✅ 500+ tests passing

**Technical Tasks**:

1. **Self-Hosting Test** (Week 1-2)
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

   **Acceptance Criteria**:
   - Stage 1 successfully compiles itself
   - Output is identical to Stage 0 compilation

2. **Build System** (Week 3)
   - Implement vertex.toml parser
   - Dependency resolution
   - Build orchestration

   **Acceptance Criteria**:
   - `vertex build` command works
   - Can build multi-crate projects

3. **Extended Testing** (Week 4)
   - 500+ tests passing
   - Performance benchmarks
   - Bug fixes

   **Acceptance Criteria**:
   - All tests pass
   - Compilation speed within 2x of Stage 0

**Dependencies**: Month 23 completion

**Risks**:
- Self-hosting bugs can be hard to diagnose
- **Mitigation**: Compare outputs at each stage, extensive logging

---

## Stage 2: Production Polish (Months 25-30)

### Overview

**Objective**: Create a production-quality compiler with optimizations, complete standard library, and toolchain.

---

### Phase 1: Optimization (Months 25-26)

#### Month 25-26: Performance Optimization

**Milestones**:
- ✅ MIR optimization passes
- ✅ Parallel compilation
- ✅ Incremental compilation foundation
- ✅ 10,000+ lines/second compilation speed

**Technical Tasks**:

1. **MIR Optimizations** (Month 25)
   - Dead code elimination
   - Constant folding
   - Inlining (small functions)
   - Copy propagation

   **Acceptance Criteria**:
   - Generated code is faster
   - Compilation time increase is acceptable

2. **Parallel Compilation** (Month 26)
   - Parse modules in parallel (Rayon)
   - Type-check independent modules in parallel
   - Codegen functions in parallel

   **Acceptance Criteria**:
   - Compilation speed significantly improved (2-4x on multi-core)

3. **Incremental Compilation** (Month 26)
   - Dependency tracking
   - Cached compilation units
   - Change detection

   **Acceptance Criteria**:
   - Incremental builds are fast (<100ms for small changes)

**Dependencies**: Month 24 completion

**Risks**:
- Optimization bugs can cause incorrect code generation
- **Mitigation**: Extensive testing, compare with unoptimized version

---

### Phase 2: Standard Library Completion (Months 27-28)

#### Month 27-28: Complete Standard Library

**Milestones**:
- ✅ All prelude items implemented
- ✅ RefCell, Cell for interior mutability
- ✅ Additional collections (BTreeMap, BTreeSet)
- ✅ Advanced iterator adapters
- ✅ Networking module (basic)
- ✅ Regular expressions
- ✅ Serialization support

**Technical Tasks**:

1. **Interior Mutability** (Week 1)
   - Cell<T> for Copy types
   - RefCell<T> with runtime borrow checking

   **Acceptance Criteria**:
   - RefCell works correctly
   - Panics on borrow violations at runtime

2. **Complete Collections** (Week 2-3)
   - BTreeMap and BTreeSet
   - LinkedList
   - VecDeque
   - BinaryHeap

   **Acceptance Criteria**:
   - All collections tested and working

3. **Networking** (Week 4)
   - TcpStream, TcpListener
   - UdpSocket
   - Basic HTTP client (maybe)

   **Acceptance Criteria**:
   - Can create TCP server and client
   - Basic networking examples work

4. **Additional Libraries** (Week 5-8)
   - Regular expressions
   - Serialization (JSON, maybe)
   - Date/time handling

   **Acceptance Criteria**:
   - Libraries documented and tested

**Dependencies**: Month 26 completion

**Risks**:
- Standard library is large
- **Mitigation**: Prioritize essential modules

---

### Phase 3: Toolchain Development (Month 29)

#### Month 29: Toolchain and IDE Support

**Milestones**:
- ✅ Code formatter (vertexfmt)
- ✅ Documentation generator (vertexdoc)
- ✅ Language Server Protocol (LSP) server
- ✅ Package manager foundation

**Technical Tasks**:

1. **Code Formatter** (Week 1-2)
   - Parse and reformat code
   - Configurable style
   - Integrate with editor plugins

   **Acceptance Criteria**:
   - Formats code consistently
   - Fast (<1 second for 10k lines)

2. **Documentation Generator** (Week 2-3)
   - Extract doc comments
   - Generate HTML documentation
   - Cross-reference links

   **Acceptance Criteria**:
   - Generates readable docs
   - Similar to Rust's rustdoc

3. **LSP Server** (Week 3-4)
   - Autocomplete
   - Go-to-definition
   - Hover information
   - Diagnostics

   **Acceptance Criteria**:
   - Works with VS Code and other LSP clients
   - Fast response times

4. **Package Manager** (Week 4)
   - Basic package registry support
   - Dependency resolution
   - Git dependencies

   **Acceptance Criteria**:
   - Can install packages from git
   - Resolves dependencies

**Dependencies**: Month 28 completion

**Risks**:
- LSP server is complex
- **Mitigation**: Use rust-analyzer as reference

---

### Phase 4: Production Readiness (Month 30)

#### Month 30: Final Polish and Release

**Milestones**:
- ✅ Stable ABI
- ✅ Complete documentation
- ✅ Real-world test projects
- ✅ 1.0 release

**Technical Tasks**:

1. **Stability Testing** (Week 1-2)
   - Run fuzzer on compiler
   - Test with large codebases
   - Fix all critical bugs

   **Acceptance Criteria**:
   - No compiler crashes
   - Handles large projects (10k+ lines)

2. **Documentation** (Week 2-3)
   - Language specification (complete)
   - The Vertex Book (tutorial)
   - Standard library reference
   - Compiler internals guide

   **Acceptance Criteria**:
   - Documentation is comprehensive
   - Easy for beginners to learn

3. **Real-World Projects** (Week 3-4)
   - Build 3+ real projects in Vertex
   - Web server
   - CLI tool
   - Library

   **Acceptance Criteria**:
   - Projects work correctly
   - Validate language design

4. **Release Preparation** (Week 4)
   - Version 1.0 release
   - Website launch
   - Announcement
   - Community building

   **Acceptance Criteria**:
   - Release is stable
   - Documentation complete
   - Community excited

**Dependencies**: Month 29 completion

**Risks**:
- Real-world usage may reveal design flaws
- **Mitigation**: Beta testing, feedback from users

---

## Technical Deep Dives

### 1. Lexer and Parser

**Architecture**:
- Hand-written recursive descent parser with operator precedence parsing for expressions
- Alternative: LALRPOP parser generator if complexity is too high
- Error recovery at synchronization points (semicolons, braces, item boundaries)

**Implementation Strategy**:
1. Start with simple tokenizer
2. Add operator precedence parser for expressions (Pratt parsing)
3. Add statement parsing
4. Add item parsing with generics
5. Add error recovery

**Testing Approach**:
- Unit tests for each token type
- Golden file tests for parser output
- Fuzz testing with random inputs
- Error recovery tests

**Common Pitfalls**:
- Operator precedence bugs
- Error recovery causing parser to lose track
- Generic parsing ambiguities (turbofish `::<T>`)

**Solutions**:
- Precedence table for operators
- Clear synchronization points
- Lookahead for generic disambiguation

---

### 2. Generic Type System with Monomorphization

**Architecture**:
- Type parameters in AST: `<T, U>`
- Type substitution during instantiation
- Monomorphization: generate specialized versions for each concrete type set

**Implementation Strategy**:
1. Parse generic parameters: `fn foo<T>(x: T)`
2. Type check with generic parameters as inference variables
3. Collect all generic instantiations during monomorphization pass
4. Generate specialized MIR for each instantiation
5. Generate specialized C code for each instantiation

**Example**:
```vertex
fn identity<T>(x: T) -> T { x }

fn main() {
    identity(42);       // Generates identity_i32
    identity("hello");  // Generates identity_str
}
```

Monomorphized:
```c
int32_t identity_i32(int32_t x) { return x; }
const char* identity_str(const char* x) { return x; }
```

**Testing Approach**:
- Test with simple generics first
- Test nested generics: `Vec<Vec<T>>`
- Test generic constraints: `fn foo<T: Clone>(x: T)`
- Test monomorphization with multiple instantiations

**Common Pitfalls**:
- Type parameter substitution bugs
- Infinite generic recursion
- Duplicate monomorphizations

**Solutions**:
- Careful substitution algorithm
- Recursion depth limit
- Deduplication during collection

---

### 3. Trait System with Associated Types

**Architecture**:
- Traits define interface with methods and associated types
- Impls provide concrete implementations
- Type checker resolves trait bounds
- Monomorphization generates code for each impl

**Implementation Strategy**:
1. Parse trait definitions with associated types:
   ```vertex
   trait Iterator {
       type Item;
       fn next(&mut self) -> Result<Self::Item, ()>;
   }
   ```

2. Parse trait implementations with associated type bindings:
   ```vertex
   impl Iterator for Range<i32> {
       type Item = i32;
       fn next(&mut self) -> Result<i32, ()> { ... }
   }
   ```

3. Type checking:
   - Resolve associated types: `<T as Iterator>::Item`
   - Check trait bounds on generic parameters
   - Validate associated type bindings in impls

4. Code generation:
   - Monomorphize trait methods for each impl
   - Generate static dispatch

**Example**:
```vertex
fn sum<I: Iterator<Item=i32>>(mut iter: I) -> i32 {
    let mut total = 0;
    loop {
        match iter.next() {
            Ok(n) => total += n,
            Err(()) => break,
        }
    }
    total
}
```

Type checking:
- `I` must implement `Iterator`
- `I::Item` must be `i32`

Monomorphization for `sum::<Range<i32>>`:
```c
int32_t sum_Range_i32(Range_i32 iter) {
    int32_t total = 0;
    while (1) {
        Result_i32_unit result = Range_i32_next(&iter);
        if (result.tag == Ok) {
            total += result.ok;
        } else {
            break;
        }
    }
    return total;
}
```

**Testing Approach**:
- Test Iterator trait with various types
- Test trait bounds in generic functions
- Test associated type resolution
- Test complex trait hierarchies

**Common Pitfalls**:
- Associated type resolution bugs
- Trait bound checking misses cases
- Impl overlap detection

**Solutions**:
- Careful resolution algorithm
- Thorough trait bound checking
- Coherence checking for impls

---

### 4. Closure Capture Analysis

**Architecture**:
- Analyze closure body to determine captured variables
- Classify captures: immutable borrow, mutable borrow, or move
- Generate closure struct with captured variables
- Implement Fn, FnMut, or FnOnce trait

**Implementation Strategy**:
1. Find free variables in closure body (not parameters)
2. Determine capture mode for each variable:
   - If mutated → mutable borrow (FnMut)
   - If moved or consumed → move (FnOnce)
   - Otherwise → immutable borrow (Fn)
3. If `move` keyword present, all captures become move
4. Generate closure struct:
   ```c
   struct closure_123 {
       int32_t captured_x;  // Move
       int32_t* captured_y; // Immutable borrow
   };
   ```
5. Generate closure call function:
   ```c
   int32_t closure_123_call(const struct closure_123* self, int32_t arg) {
       return self->captured_x + *self->captured_y + arg;
   }
   ```

**Example**:
```vertex
let x = 10;
let mut y = 20;
let closure = |z| x + y + z;  // Captures x and y by immutable borrow
```

Generated:
```c
struct closure_1 {
    const int32_t* captured_x;
    const int32_t* captured_y;
};

int32_t closure_1_call(const struct closure_1* self, int32_t z) {
    return *self->captured_x + *self->captured_y + z;
}
```

**Testing Approach**:
- Test immutable captures
- Test mutable captures
- Test move captures
- Test move keyword
- Test nested closures

**Common Pitfalls**:
- Incorrectly classifying capture modes
- Missing captured variables
- Lifetime issues with borrowed captures

**Solutions**:
- Thorough free variable analysis
- Conservative capture mode classification
- Borrow checker validates lifetimes

---

### 5. Borrow Checker (Polonius-Inspired)

**Architecture**:
- Data-flow analysis on MIR
- Track active borrows at each program point
- Check aliasing rules
- Validate reference lifetimes

**Implementation Strategy**:
1. Build control flow graph from MIR
2. Compute liveness information (which variables are live at each point)
3. Track borrows:
   - When a borrow is created, add to active borrow set
   - When a borrow is used, check no conflicting borrows
   - When a borrow ends (last use), remove from active set
4. Check rules:
   - No simultaneous mutable borrows
   - No mutable + immutable borrows simultaneously
   - References must outlive all uses

**Algorithm** (simplified):
```rust
fn check_borrows(mir: &Mir) -> Result<(), Vec<BorrowError>> {
    let mut errors = vec![];
    let liveness = compute_liveness(mir);

    for (bb, data) in mir.basic_blocks.iter_enumerated() {
        let mut active_borrows = BorrowSet::new();

        for (idx, stmt) in data.statements.iter().enumerate() {
            let location = Location { block: bb, statement_index: idx };

            match stmt {
                Statement::Assign(place, Rvalue::Ref(mutability, borrowed_place)) => {
                    // Creating a borrow
                    if *mutability == Mutability::Mut {
                        // Check no other borrows of this place
                        if active_borrows.has_any_borrow(borrowed_place) {
                            errors.push(BorrowError::CannotBorrowAsMutable { location });
                        }
                    } else {
                        // Check no mutable borrows of this place
                        if active_borrows.has_mut_borrow(borrowed_place) {
                            errors.push(BorrowError::CannotBorrowWhileMutablyBorrowed { location });
                        }
                    }

                    // Add to active borrows
                    let borrow_id = active_borrows.insert(borrowed_place, *mutability);
                }

                Statement::Assign(place, rvalue) => {
                    // Using a place
                    if active_borrows.has_mut_borrow(place) {
                        errors.push(BorrowError::CannotUseWhileMutablyBorrowed { location });
                    }
                }

                _ => {}
            }

            // Remove borrows that end here (last use according to liveness)
            active_borrows.retain(|borrow_id| {
                liveness.is_live_after(borrow_id.borrowed_place, location)
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

**Testing Approach**:
- Test simple borrow violations
- Test complex control flow (if, loop, match)
- Test move semantics
- Test lifetime inference
- Test error messages

**Common Pitfalls**:
- False positives (rejecting valid code)
- False negatives (accepting invalid code)
- Poor error messages

**Solutions**:
- Conservative analysis (better to reject than accept invalid code)
- Extensive testing with Rust-like examples
- Iterate on error messages based on user feedback

---

### 6. MIR Generation and Optimization

**Architecture**:
- Lower typed HIR to MIR (control flow graph)
- Explicit control flow (no nesting)
- Explicit drops
- Optimization passes

**Implementation Strategy**:
1. Build basic blocks from HIR expressions and statements
2. Generate explicit control flow (Goto, SwitchInt, Return)
3. Insert drops at scope exits
4. Optimization passes:
   - Dead code elimination
   - Constant folding
   - Inlining (small functions)
   - Copy propagation

**MIR Structure**:
```rust
struct Mir {
    basic_blocks: IndexVec<BasicBlock, BasicBlockData>,
    local_decls: IndexVec<Local, LocalDecl>,
    arg_count: usize,
    return_ty: Ty,
}

struct BasicBlockData {
    statements: Vec<Statement>,
    terminator: Terminator,
}
```

**Example HIR to MIR**:
```vertex
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
```

MIR:
```
bb0: {
    let _0: i32;  // return value
    let _1: i32 = n;  // parameter
    let _2: bool = _1 <= 1;
    switchInt(_2) -> [true: bb1, false: bb2];
}

bb1: {
    _0 = 1;
    goto -> bb3;
}

bb2: {
    let _3: i32 = _1 - 1;
    let _4: i32 = call factorial(_3);
    _0 = _1 * _4;
    goto -> bb3;
}

bb3: {
    return;
}
```

**Testing Approach**:
- Test MIR generation for all HIR constructs
- Test optimization passes preserve semantics
- Test drop insertion

**Common Pitfalls**:
- Control flow bugs
- Missing drops
- Incorrect optimization

**Solutions**:
- Validate MIR structure (CFG well-formed)
- Test optimizations against unoptimized version
- Careful drop elaboration

---

### 7. C Code Generation with Overflow Handling

**Architecture**:
- Translate MIR to C code
- Handle overflow checking (debug vs release)
- Name mangling for generics
- Integrate with runtime library

**Implementation Strategy**:
1. Generate C function for each MIR function
2. Translate MIR statements to C statements
3. Handle overflow checking:
   - Debug mode: use `__builtin_add_overflow` and friends, call panic on overflow
   - Release mode: plain arithmetic (wrapping behavior)
4. Generate main function that calls Vertex main
5. Link with runtime library

**Overflow Handling**:

Debug mode:
```c
// a + b
{
    int32_t __result;
    if (__builtin_add_overflow(a, b, &__result)) {
        vertex_panic("attempt to add with overflow", "main.vx", 42);
    }
    __result
}
```

Release mode:
```c
// a + b
(a + b)
```

**Name Mangling**:
```rust
fn mangle_name(def_id: DefId, generic_args: &[Ty]) -> String {
    let mut name = format!("vertex_{}", def_id.local_id);
    for ty in generic_args {
        name.push_str("_");
        name.push_str(&mangle_type(ty));
    }
    name
}

fn mangle_type(ty: &Ty) -> String {
    match ty {
        Ty::Int(IntTy::I32) => "i32".to_string(),
        Ty::Adt(def, args) => {
            let mut s = format!("{}", def.name);
            if !args.is_empty() {
                s.push_str("_");
                for arg in args {
                    s.push_str(&mangle_type(arg));
                }
            }
            s
        }
        _ => format!("{:?}", ty),  // Simple representation
    }
}
```

**Testing Approach**:
- Test code generation for all MIR constructs
- Test overflow checking in debug and release modes
- Test name mangling with generics
- Test generated C compiles with gcc/clang

**Common Pitfalls**:
- Name mangling collisions
- Incorrect overflow checking
- Generated C doesn't compile

**Solutions**:
- Use unique IDs in mangled names
- Test overflow checking thoroughly
- Generate readable C for debugging

---

### 8. Drop Elaboration and Panic Unwinding

**Architecture**:
- Insert explicit drop statements in MIR
- Handle drop order (reverse declaration order for locals)
- Generate unwinding paths for panic
- Handle drop flags for conditional initialization

**Implementation Strategy**:
1. Identify values that need dropping (non-Copy types)
2. Determine drop order:
   - Local variables: reverse declaration order
   - Struct fields: declaration order
   - Tuple elements: left to right
3. Insert Drop terminators at scope exits
4. Generate drop flags for conditional initialization
5. Generate unwinding paths:
   - Drop all initialized values on panic
   - Continue unwinding to caller

**Drop Order Example**:
```vertex
fn example() {
    let a = String::from("a");
    let b = String::from("b");
    let c = String::from("c");
}  // Drop order: c, b, a
```

MIR:
```
bb0: {
    StorageLive(a);
    a = String::from("a");
    StorageLive(b);
    b = String::from("b");
    StorageLive(c);
    c = String::from("c");
    goto -> bb1;
}

bb1: {
    drop(c) -> [return: bb2, unwind: bb_cleanup];
}

bb2: {
    StorageDead(c);
    drop(b) -> [return: bb3, unwind: bb_cleanup2];
}

bb3: {
    StorageDead(b);
    drop(a) -> [return: bb4, unwind: bb_cleanup3];
}

bb4: {
    StorageDead(a);
    return;
}

bb_cleanup: {
    drop(b);
    drop(a);
    resume;
}

bb_cleanup2: {
    drop(a);
    resume;
}

bb_cleanup3: {
    resume;
}
```

**Conditional Initialization with Drop Flags**:
```vertex
fn conditional(cond: bool) {
    let x: String;
    if cond {
        x = String::from("initialized");
    }
}  // Drop x only if initialized
```

MIR:
```
bb0: {
    let _drop_flag_x: bool = false;
    StorageLive(x);
    switchInt(cond) -> [true: bb1, false: bb3];
}

bb1: {
    x = String::from("initialized");
    _drop_flag_x = true;
    goto -> bb3;
}

bb3: {
    if _drop_flag_x {
        drop(x);
    }
    StorageDead(x);
    return;
}
```

**Panic Unwinding**:
- In unwind mode: Drop all initialized values, continue unwinding
- In abort mode: Immediately terminate process

**Testing Approach**:
- Test drop order for locals, struct fields, tuples
- Test conditional initialization with drop flags
- Test unwinding with panic
- Test double panic (should abort)

**Common Pitfalls**:
- Incorrect drop order
- Missing drops
- Double drops
- Missing drop flags

**Solutions**:
- Careful drop elaboration algorithm
- Test with Drop implementations that print
- Validate MIR structure

---

## Standard Library Implementation Priority

### Priority 1: Essential Types (Stage 0, Month 12)

1. **Result<T, E>**
   - Full API (unwrap, map, and_then, etc.)
   - ? operator support
   - From trait for error conversion

2. **Option<T>**
   - Full API (unwrap, map, and_then, etc.)
   - Conversions to/from Result

3. **Box<T>**
   - Heap allocation
   - Deref coercion
   - Drop implementation

4. **Rc<T>** and **Arc<T>**
   - Reference counting
   - Weak references
   - Thread-safe (Arc) vs single-threaded (Rc)

### Priority 2: Core Collections (Stage 0, Month 12)

1. **Vec<T>**
   - Push, pop, indexing
   - Iteration
   - Slicing
   - Capacity management
   - Built-in vec![] syntax

2. **HashMap<K, V>** and **HashSet<T>**
   - Hash-based lookup
   - Entry API
   - Iteration
   - Hash trait

### Priority 3: String Types (Stage 0, Month 12)

1. **String**
   - Owned, heap-allocated
   - UTF-8 encoding
   - **No indexing by usize** (enforced at compile time)
   - Methods: push_str, chars, split, etc.

2. **&str**
   - String slice (borrowed)
   - UTF-8 encoded
   - **No indexing by usize** (enforced at compile time)
   - Byte slicing (&str[0..5] slices bytes, not characters)

**String Indexing Prohibition**:
```vertex
let s = String::from("hello");
// let c = s[0];  // ❌ ERROR E0608: Cannot index String with usize

// Instead, use:
let chars: Vec<char> = s.chars().collect();  // Get characters
let bytes: &[u8] = s.as_bytes();             // Get bytes
let first = s.chars().next();                // Get first character
```

Error message:
```
error[E0608]: cannot index into a value of type `String`
  --> main.vx:2:13
   |
 2 |     let c = s[0];
   |             ^^^^
   |
   = help: the trait `Index<usize>` is not implemented for `String`
   = note: you can use `.chars().nth(n)` to get the nth character
   = note: or use `.as_bytes()[n]` to get the nth byte (not character!)
   = note: String uses UTF-8 encoding where characters can be 1-4 bytes
```

### Priority 4: Iterator Trait (Stage 0, Month 12)

1. **Iterator Trait** with **Associated Type**
   ```vertex
   trait Iterator {
       type Item;
       fn next(&mut self) -> Result<Self::Item, ()>;
   }
   ```

2. **IntoIterator Trait**
   ```vertex
   trait IntoIterator {
       type Item;
       type IntoIter: Iterator<Item=Self::Item>;
       fn into_iter(self) -> Self::IntoIter;
   }
   ```

3. **Iterator Combinators**
   - map, filter, fold, collect
   - take, skip, enumerate
   - chain, zip

### Priority 5: I/O Traits (Stage 0, Month 12)

1. **Read and Write Traits**
   - Read: read, read_to_string
   - Write: write, write_all, flush

2. **File I/O**
   - File::open, File::create
   - Read and Write implementations for File

3. **Standard I/O**
   - stdin, stdout, stderr
   - print, println (built-in)

### Priority 6: Smart Pointers (Stage 1, Month 27-28)

1. **RefCell<T>** and **Cell<T>**
   - Interior mutability
   - Runtime borrow checking (RefCell)
   - Copy types only (Cell)

2. **Weak<T>**
   - Weak references for Rc/Arc
   - Break reference cycles

### Priority 7: Additional Collections (Stage 1, Month 27-28)

1. **BTreeMap<K, V>** and **BTreeSet<T>**
   - Sorted collections
   - Range queries

2. **LinkedList<T>**, **VecDeque<T>**, **BinaryHeap<T>**
   - Specialized collections

---

## Testing Strategy

### Unit Testing (Throughout Development)

**Lexer Tests** (~50 tests):
- Each token type
- Numeric literals (decimal, hex, binary, float)
- String literals (regular, raw, with escapes)
- Character literals (ASCII, Unicode)
- Error cases (invalid tokens)

**Parser Tests** (~100 tests):
- Expressions with operator precedence
- Statements (let, if, loop, match, etc.)
- Items (functions, structs, enums, traits, impls)
- Generics (type parameters, trait bounds, where clauses)
- Patterns (literals, bindings, destructuring, or-patterns)
- Error recovery

**Name Resolution Tests** (~50 tests):
- Module system (file-based, inline)
- Imports (use statements)
- Scoping rules
- Visibility (pub, pub(crate), private)
- Error cases (undefined names, circular imports)

**Type Checker Tests** (~200 tests):
- Type inference (simple, complex)
- Generic types (instantiation, bounds)
- Trait resolution (method calls, trait bounds)
- Associated types (Iterator, etc.)
- String indexing prohibition (compile-time error)
- Error messages

**Borrow Checker Tests** (~100 tests):
- Aliasing rules (no &mut + &, no multiple &mut)
- Move semantics (use-after-move)
- Lifetime inference
- Error cases with clear messages

**MIR Tests** (~50 tests):
- MIR generation from HIR
- Control flow (if, loop, match)
- Drop elaboration (locals, structs, conditionals)
- Optimization passes (dead code, constant folding)

**Code Generation Tests** (~100 tests):
- Simple functions
- Generics (monomorphization)
- Closures
- Trait methods
- Overflow checking (debug vs release)
- Generated C compiles and runs correctly

**Standard Library Tests** (~200 tests):
- Result and Option APIs
- Collections (Vec, HashMap, String)
- Iterator combinators
- I/O (files, stdin/stdout)
- Smart pointers (Box, Rc, Arc)

### Integration Testing

**End-to-End Compilation** (~50 programs):
- Hello world
- Fibonacci (recursive, iterative)
- Struct and methods
- Generics (Vec, HashMap)
- Traits (Iterator, Display)
- Closures
- File I/O
- Error handling with Result
- Complex program (1500+ lines)

**Test Criteria**:
- Compiles without errors
- Runs correctly (output matches expected)
- Generated C is readable

### Compiler Test Suite Structure

**Directory Layout**:
```
tests/
├── lexer/
│   ├── tokens.rs
│   ├── literals.rs
│   └── errors.rs
├── parser/
│   ├── expressions.rs
│   ├── statements.rs
│   ├── items.rs
│   ├── generics.rs
│   └── errors.rs
├── resolve/
│   ├── modules.rs
│   ├── imports.rs
│   └── scopes.rs
├── typecheck/
│   ├── inference.rs
│   ├── generics.rs
│   ├── traits.rs
│   ├── associated_types.rs
│   └── errors.rs
├── borrow_check/
│   ├── aliasing.rs
│   ├── moves.rs
│   └── lifetimes.rs
├── mir/
│   ├── generation.rs
│   ├── drops.rs
│   └── optimizations.rs
├── codegen/
│   ├── functions.rs
│   ├── generics.rs
│   ├── closures.rs
│   ├── traits.rs
│   └── overflow.rs
├── stdlib/
│   ├── result_option.rs
│   ├── collections.rs
│   ├── iterators.rs
│   ├── io.rs
│   └── smart_pointers.rs
└── integration/
    ├── hello_world.vx
    ├── fibonacci.vx
    ├── generics.vx
    ├── traits.vx
    ├── closures.vx
    ├── file_io.vx
    └── complex.vx
```

**Test Harness**:
```rust
// tests/test_harness.rs

fn compile_test(source: &str, expected_output: &str) {
    let compiler = Compiler::new();
    let result = compiler.compile_string(source);

    match result {
        Ok(program) => {
            let output = run_program(program);
            assert_eq!(output, expected_output);
        }
        Err(errors) => {
            panic!("Compilation failed: {:?}", errors);
        }
    }
}

fn compile_fail_test(source: &str, expected_error: &str) {
    let compiler = Compiler::new();
    let result = compiler.compile_string(source);

    match result {
        Ok(_) => {
            panic!("Expected compilation to fail");
        }
        Err(errors) => {
            let error_messages = errors.iter().map(|e| e.message()).collect::<Vec<_>>();
            assert!(error_messages.iter().any(|msg| msg.contains(expected_error)));
        }
    }
}
```

### Fuzz Testing

**Parser Fuzzing**:
- Generate random token sequences
- Parser should never panic
- Report errors gracefully

**Type Checker Fuzzing**:
- Generate random well-formed programs
- Type checker should never panic
- May accept or reject, but must be sound

**Tools**:
- cargo-fuzz or afl for Rust
- Run continuously in CI

### Performance Testing

**Compilation Speed Benchmarks**:
- Measure lines/second for various program sizes
- Target: 3000+ lines/second for Stage 0, 10,000+ for Stage 2
- Track performance over time

**Memory Usage**:
- Track peak memory usage during compilation
- Target: <100MB for typical projects

**Generated Code Performance**:
- Benchmark generated code vs hand-written C
- Target: within 10% of hand-written performance

---

## Risk Management

### Technical Risks

**Risk 1: Borrow Checker Complexity**
- **Likelihood**: High
- **Impact**: High (core feature)
- **Mitigation**:
  - Study Rust's Polonius algorithm thoroughly
  - Implement incrementally with extensive testing
  - Start with simple cases, add complexity gradually
  - Budget extra time (2-3 weeks buffer)

**Risk 2: Generic Type System Bugs**
- **Likelihood**: High
- **Impact**: High (required for Stage 0)
- **Mitigation**:
  - Test with complex generic examples from Rust
  - Fuzz testing
  - Refer to Rust's implementation
  - Careful substitution algorithm

**Risk 3: Associated Types Complexity**
- **Likelihood**: Medium
- **Impact**: High (required for Iterator)
- **Mitigation**:
  - Study Rust's associated types implementation
  - Start with simple examples (Iterator)
  - Extensive testing
  - This is CRITICAL and cannot be deferred

**Risk 4: Closure Capture Analysis Bugs**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**:
  - Conservative capture analysis
  - Test with various closure examples
  - Refer to Rust's implementation

**Risk 5: C Code Generation Issues**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**:
  - Generate readable C for debugging
  - Test generated C compiles with gcc/clang
  - Use Valgrind to detect memory errors

**Risk 6: Performance Issues in Self-Hosted Compiler**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**:
  - Profile early and often
  - Optimize hot paths
  - Consider LLVM backend for better performance

### Schedule Risks

**Risk 1: Development Takes Longer Than Planned**
- **Likelihood**: High (software projects often do)
- **Impact**: Medium
- **Mitigation**:
  - 20% schedule buffer built in
  - Prioritize ruthlessly (Stage 0 features are non-negotiable)
  - Regular progress reviews
  - Adjust scope if necessary (defer Stage 2 features)

**Risk 2: Feature Creep**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**:
  - Stick to v1.0 spec strictly
  - Defer non-essential features to future versions
  - Clear "must have" vs "nice to have" distinction
  - Document deferred features for v2.0

**Risk 3: Testing Takes Longer Than Expected**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**:
  - Write tests alongside implementation (TDD)
  - Automate testing in CI
  - Budget dedicated testing time in each phase

### Scope Creep Prevention

**Must Have (Stage 0)**:
- Full generic type system
- Trait system with associated types
- Closures with Fn/FnMut/FnOnce
- Borrow checker
- C code generation
- Core standard library (Vec, HashMap, String, Iterator, I/O)
- String indexing prohibition
- Overflow checking (debug/release modes)

**Should Have (Stage 1)**:
- Self-hosting
- Build system (vertex.toml)
- Extended standard library (RefCell, BTreeMap, networking)
- Advanced pattern matching

**Nice to Have (Stage 2)**:
- LLVM backend
- Optimization passes
- Incremental compilation
- LSP server
- Package manager

**Deferred to Future Versions**:
- Async/await
- Const generics
- Trait objects (dynamic dispatch)
- Macros (beyond built-ins)
- Unsafe code (beyond minimal FFI)

---

## Milestone Checklist

### Stage 0 Milestones

**Month 1-2: Foundation**
- [ ] Lexer implementation complete
- [ ] Parser implementation complete
- [ ] AST definition complete (with generics)
- [ ] Test infrastructure set up
- [ ] 50+ lexer tests passing
- [ ] 100+ parser tests passing

**Month 3-5: Type System**
- [ ] Name resolution with module system
- [ ] Type checking with inference
- [ ] Generic type instantiation
- [ ] Trait system with associated types
- [ ] String indexing prohibition enforced
- [ ] 200+ type checker tests passing

**Month 6-7: Advanced Features**
- [ ] Closure capture analysis
- [ ] Fn/FnMut/FnOnce trait hierarchy
- [ ] Iterator trait with associated types fully functional
- [ ] Iterator combinators (map, filter, fold, collect)
- [ ] for-loop desugaring
- [ ] 100+ closure/iterator tests passing

**Month 8-9: Safety Analysis**
- [ ] MIR generation from HIR
- [ ] Drop elaboration with correct ordering
- [ ] Borrow checker (Polonius-inspired)
- [ ] Move checking
- [ ] Lifetime inference (simplified)
- [ ] 100+ borrow checker tests passing

**Month 10-11: Code Generation**
- [ ] C backend implementation
- [ ] Monomorphization pass
- [ ] Overflow checking (debug/release modes)
- [ ] Closure code generation
- [ ] Trait method dispatch
- [ ] Runtime library (panic, allocation)
- [ ] 100+ codegen tests passing

**Month 12: Standard Library & Testing**
- [ ] Result, Option, Box, Rc, Arc
- [ ] Vec, HashMap, HashSet, String
- [ ] Iterator trait and combinators
- [ ] I/O traits and file system
- [ ] String indexing blocked with helpful error
- [ ] Overflow checking works correctly
- [ ] 750+ total tests passing
- [ ] Can compile 1500+ line programs
- [ ] Compilation speed: 3000+ lines/second

### Stage 1 Milestones

**Month 13-15: Core Modules Port**
- [ ] Lexer ported to Vertex
- [ ] Parser ported to Vertex
- [ ] AST definitions ported to Vertex
- [ ] All Stage 0 tests still passing

**Month 16-18: Semantic Analysis Port**
- [ ] Name resolution ported to Vertex
- [ ] Type checking ported to Vertex
- [ ] Trait resolution ported to Vertex
- [ ] Generic instantiation ported to Vertex

**Month 19-21: MIR and Borrow Checker Port**
- [ ] MIR generation ported to Vertex
- [ ] Borrow checker ported to Vertex
- [ ] Drop elaboration ported to Vertex

**Month 22-23: Code Generation Port**
- [ ] C backend ported to Vertex
- [ ] Monomorphization ported to Vertex
- [ ] Optional: LLVM backend

**Month 24: Self-Hosting Verification**
- [ ] Stage 0 compiles Stage 1 source
- [ ] Stage 1 compiles itself (self-hosting!)
- [ ] Binary output identical to Stage 0
- [ ] Build system implemented (vertex.toml)
- [ ] 500+ tests passing (Stage 0 tests + Stage 1 additions)
- [ ] Can compile 5000+ line programs
- [ ] Performance within 2x of Stage 0

### Stage 2 Milestones

**Month 25-26: Optimization**
- [ ] MIR optimization passes
- [ ] Parallel compilation (2-4x speedup on multi-core)
- [ ] Incremental compilation foundation
- [ ] Compilation speed: 10,000+ lines/second

**Month 27-28: Standard Library Completion**
- [ ] RefCell, Cell for interior mutability
- [ ] BTreeMap, BTreeSet
- [ ] Additional collections (LinkedList, VecDeque, BinaryHeap)
- [ ] Networking module (TcpStream, TcpListener, UdpSocket)
- [ ] Regular expressions
- [ ] All prelude items implemented

**Month 29: Toolchain**
- [ ] Code formatter (vertexfmt)
- [ ] Documentation generator (vertexdoc)
- [ ] LSP server (autocomplete, go-to-def, hover)
- [ ] Package manager foundation

**Month 30: Production Readiness**
- [ ] Stability testing (no crashes)
- [ ] Complete documentation (spec, book, stdlib reference)
- [ ] 3+ real-world test projects
- [ ] Version 1.0 release

---

## Implementation Guidelines

### Code Organization

**Directory Structure** (Stage 0 in Rust):
```
vertex_stage0/
├── Cargo.toml
├── src/
│   ├── main.rs                   # Compiler driver
│   ├── lib.rs                    # Library root
│   ├── lexer/
│   │   ├── mod.rs
│   │   ├── token.rs
│   │   └── tests.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── ast.rs
│   │   ├── grammar.rs
│   │   └── tests.rs
│   ├── resolve/
│   │   ├── mod.rs
│   │   ├── scope.rs
│   │   └── tests.rs
│   ├── typecheck/
│   │   ├── mod.rs
│   │   ├── infer.rs
│   │   ├── unify.rs
│   │   ├── traits.rs
│   │   └── tests.rs
│   ├── mir/
│   │   ├── mod.rs
│   │   ├── build.rs
│   │   ├── borrow_check.rs
│   │   └── tests.rs
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── c_backend.rs
│   │   └── tests.rs
│   ├── error.rs                  # Error types
│   ├── span.rs                   # Source locations
│   └── util.rs                   # Utilities
├── runtime/
│   ├── vertex_runtime.c          # Minimal runtime
│   └── vertex_runtime.h
├── stdlib/                       # Vertex standard library (written in Vertex)
│   ├── core.vx
│   ├── result.vx
│   ├── option.vx
│   ├── vec.vx
│   ├── hashmap.vx
│   ├── string.vx
│   ├── iterator.vx
│   └── io.vx
└── tests/
    ├── lexer_tests.rs
    ├── parser_tests.rs
    ├── typecheck_tests.rs
    ├── borrow_check_tests.rs
    ├── codegen_tests.rs
    └── integration/
        ├── hello_world.vx
        ├── fibonacci.vx
        └── ...
```

### Naming Conventions

**Rust Code** (Stage 0):
- snake_case for functions, variables, modules
- CamelCase for types (structs, enums, traits)
- SCREAMING_SNAKE_CASE for constants
- Private by default, explicit pub for public items

**Vertex Code** (Stage 1+):
- Same conventions as Rust (Vertex uses similar style)
- Clear, descriptive names
- Avoid abbreviations unless very common

**File Names**:
- snake_case.rs for Rust
- snake_case.vx for Vertex
- One module per file (generally)

### Documentation Standards

**Code Comments**:
- Doc comments for all public items (///)
- Explain WHY, not WHAT (code should be self-documenting for WHAT)
- Examples in doc comments

**Module Documentation**:
- Module-level doc comment (//! ...) explaining purpose
- List of main types/functions
- Usage examples

**Architecture Documentation**:
- High-level design documents for each major component
- Algorithms explained (e.g., borrow checker, type inference)
- Diagrams for complex systems (control flow, data structures)

**Example**:
```rust
/// Performs type unification between two types.
///
/// This is a key part of the type inference algorithm. Given two types,
/// this function attempts to make them equal by substituting inference
/// variables. If successful, it returns Ok(()); otherwise, it returns
/// a type error describing why the types cannot be unified.
///
/// # Examples
///
/// ```
/// let ty1 = Ty::Infer(InferVar::new(0));
/// let ty2 = Ty::Int(IntTy::I32);
/// unify(&mut ctx, ty1, ty2)?; // ty1 is now i32
/// ```
fn unify(ctx: &mut InferCtx, ty1: Ty, ty2: Ty) -> Result<(), TypeError> {
    // ...
}
```

### Git Workflow

**Branching Strategy**:
- `main` branch: stable, always compiles
- Feature branches: `feature/lexer`, `feature/borrow-checker`, etc.
- Merge to main when feature is complete and tested

**Commit Messages**:
- Clear, descriptive messages
- Format: `[component] description`
- Examples:
  - `[lexer] Add support for raw string literals`
  - `[typecheck] Fix generic instantiation bug`
  - `[tests] Add borrow checker tests for closures`

**Pull Requests**:
- Each feature in a separate PR
- Code review required before merge
- All tests must pass
- CI checks:
  - cargo test (all tests pass)
  - cargo clippy (no warnings)
  - cargo fmt --check (code formatted)

**Version Control**:
- Tag releases: v0.1.0, v0.2.0, v1.0.0
- Keep CHANGELOG.md updated

---

## Conclusion

This implementation plan provides a detailed, actionable roadmap for building the Vertex compiler from scratch. The plan is structured into three stages:

1. **Stage 0** (Months 1-12): Bootstrap compiler in Rust with full generic support, trait system with associated types, and closures
2. **Stage 1** (Months 13-24): Self-hosted compiler rewritten in Vertex
3. **Stage 2** (Months 25-30): Production polish with optimizations, complete standard library, and toolchain

**Key Success Factors**:
- Incremental development with frequent testing
- Prioritize essential features (generics, traits, closures are non-negotiable for Stage 0)
- Realistic scope management (defer advanced features to later stages)
- Learn from Rust's implementation (borrow checker, trait system)
- Comprehensive testing at every stage
- Clear documentation for maintainability

**Critical Requirements for Stage 0**:
- Generic type system (MUST be included)
- Trait system with associated types (REQUIRED for Iterator)
- Closures with capture semantics (REQUIRED for functional programming)
- String indexing prohibition (enforced at compile time)
- Overflow checking (debug mode panics, release mode wraps)

**Timeline**: 24-30 months to production-ready compiler

By following this plan step-by-step, you can successfully implement Vertex and achieve the goal of a memory-safe systems language with a gentler learning curve than Rust.

# Vertex Compiler Architecture Specification

**Version**: 1.0.0
**Status**: Design Document
**Date**: December 2024

## Executive Summary

This document defines the implementation architecture for the Vertex compiler, a memory-safe systems programming language compiler. The compiler follows a traditional multi-pass architecture with frontend, middle-end, and backend phases.

## 1. High-Level Architecture

```
Source Code (.vx files)
    ↓
┌─────────────────────────────────────────────────────────┐
│                    FRONTEND                              │
├─────────────────────────────────────────────────────────┤
│  Lexer → Parser → AST → Name Resolution → Type Check    │
└─────────────────────────────────────────────────────────┘
    ↓ (Typed AST)
┌─────────────────────────────────────────────────────────┐
│                   MIDDLE-END                             │
├─────────────────────────────────────────────────────────┤
│  HIR Lowering → MIR Generation → Borrow Checking        │
│  → MIR Optimizations → Monomorphization                 │
└─────────────────────────────────────────────────────────┘
    ↓ (Monomorphized MIR)
┌─────────────────────────────────────────────────────────┐
│                    BACKEND                               │
├─────────────────────────────────────────────────────────┤
│  Code Generation (LLVM IR or C) → Linking               │
└─────────────────────────────────────────────────────────┘
    ↓
Executable Binary
```

## 2. Frontend Phases

### 2.1 Lexical Analysis (Lexer)

**Input**: Source text (.vx files)
**Output**: Token stream
**Error Recovery**: Continue scanning after invalid tokens

#### Responsibilities
- Tokenize source code into lexical tokens
- Handle string literals (regular and raw)
- Process numeric literals (decimal, hex, binary)
- Recognize keywords, identifiers, operators, and punctuation
- Track source locations (file, line, column) for error reporting
- Skip whitespace and comments

#### Token Types
```rust
enum TokenKind {
    // Literals
    IntLiteral(u64, IntSuffix),      // Value, type suffix
    FloatLiteral(f64, FloatSuffix),
    CharLiteral(char),
    StringLiteral(String),
    RawStringLiteral(String),

    // Keywords (29 total - 'defer' removed from v1.0)
    Break, Const, Continue, Else, Enum, Extern, False,
    Fn, For, If, Impl, In, Let, Loop, Match, Mod, Mut,
    Pub, Return, Self_, Struct, Trait, True, Type, Unsafe,
    Use, Where, While,

    // Identifiers
    Ident(String),

    // Operators and Punctuation
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or, Not,
    BitAnd, BitOr, BitXor, BitNot, Shl, Shr,
    Assign, PlusEq, MinusEq, StarEq, SlashEq, PercentEq,
    Dot, ColonColon, LBracket, RBracket, LParen, RParen,
    LBrace, RBrace,
    Question, DotDot, DotDotEq, Arrow, FatArrow,
    Semi, Comma, Colon, Underscore,

    // Special
    Eof,
    Error(String),
}

struct Token {
    kind: TokenKind,
    span: Span,
}

struct Span {
    file_id: FileId,
    start: u32,  // Byte offset
    end: u32,
}
```

#### Error Recovery Strategy
- Report lexical errors (invalid characters, malformed literals)
- Insert error token and continue scanning
- Track all errors in error accumulator
- Never panic during lexing

### 2.2 Syntactic Analysis (Parser)

**Input**: Token stream
**Output**: Abstract Syntax Tree (AST)
**Error Recovery**: Panic mode with synchronization points

#### Responsibilities
- Parse token stream into AST
- Validate syntactic structure
- Preserve source location information
- Implement error recovery to continue parsing after errors
- Build untyped, unresolved AST

#### AST Node Types
```rust
// Top-level items
enum Item {
    Fn(FnItem),
    Struct(StructItem),
    Enum(EnumItem),
    Impl(ImplItem),
    Trait(TraitItem),
    Mod(ModItem),
    Use(UseItem),
    Const(ConstItem),
    Static(StaticItem),
    TypeAlias(TypeAliasItem),
}

struct FnItem {
    visibility: Visibility,
    sig: FnSignature,
    body: Option<Block>,
    span: Span,
}

struct FnSignature {
    name: Ident,
    generics: Generics,
    params: Vec<Param>,
    return_ty: Option<Type>,
    is_const: bool,
    is_unsafe: bool,
}

// Statements
enum Stmt {
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        init: Option<Expr>,
        span: Span,
    },
    Expr(Expr, bool), // expression, has_semicolon
    Item(Item),
}

// Expressions
enum Expr {
    Literal(Literal),
    Ident(Path),
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Unary { op: UnOp, expr: Box<Expr> },
    Call { func: Box<Expr>, args: Vec<Expr> },
    MethodCall { receiver: Box<Expr>, method: Ident, args: Vec<Expr> },
    Index { base: Box<Expr>, index: Box<Expr> },
    Field { base: Box<Expr>, field: Ident },
    TupleField { base: Box<Expr>, index: u32 },
    If { cond: Box<Expr>, then: Block, else_: Option<Box<Expr>> },
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm> },
    Loop { body: Block },
    While { cond: Box<Expr>, body: Block },
    For { pattern: Pattern, iter: Box<Expr>, body: Block },
    Block(Block),
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue,
    Struct { path: Path, fields: Vec<FieldInit> },
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    ArrayRepeat { value: Box<Expr>, count: Box<Expr> },
    Range { start: Option<Box<Expr>>, end: Option<Box<Expr>>, inclusive: bool },
    Ref { mutable: bool, expr: Box<Expr> },
    Deref(Box<Expr>),
    Try(Box<Expr>),  // ? operator
    Cast { expr: Box<Expr>, ty: Type },
    Closure { params: Vec<Param>, body: Box<Expr>, is_move: bool },
}

// Types
enum Type {
    Path(Path),
    Ref { mutable: bool, ty: Box<Type> },
    Ptr { mutable: bool, ty: Box<Type> },
    Array { elem: Box<Type>, len: Expr },
    Slice(Box<Type>),
    Tuple(Vec<Type>),
    Fn { params: Vec<Type>, return_ty: Box<Type> },
    Infer,  // _ placeholder
}

// Patterns
enum Pattern {
    Wild,
    Ident { name: Ident, mutable: bool, subpattern: Option<Box<Pattern>> },
    Literal(Literal),
    Range { start: Literal, end: Literal, inclusive: bool },
    Tuple(Vec<Pattern>),
    Struct { path: Path, fields: Vec<FieldPattern> },
    TupleStruct { path: Path, fields: Vec<Pattern> },
    Ref { mutable: bool, pattern: Box<Pattern> },
    Or(Vec<Pattern>),
}
```

#### Error Recovery
**Synchronization Points**:
- Statement boundaries (semicolons)
- Block boundaries (braces)
- Item boundaries
- Top-level declarations

**Recovery Strategy**:
1. On parse error, enter panic mode
2. Skip tokens until synchronization point
3. Report error with expected tokens
4. Continue parsing from synchronization point
5. Insert placeholder AST nodes if needed

**Example**:
```vertex
fn foo() {
    let x = ;  // ERROR: expected expression
    // Recovery: skip to semicolon, report error, continue
    let y = 10;  // Continue parsing normally
}
```

### 2.2.5 Built-in Syntax Handling

**IMPORTANT**: Vertex has NO macro system. Certain constructs that look like macros are built directly into the compiler's parser and are NOT extensible.

#### Built-in Constructs

**1. vec! Syntax**
```rust
// Parser recognizes vec![...] as special syntax
enum Expr {
    // ...
    VecLiteral(Vec<Expr>),           // vec![1, 2, 3]
    VecRepeat(Box<Expr>, Box<Expr>), // vec![0; 100]
}

// Parsing rules:
// vec![elem1, elem2, ...] → VecLiteral
// vec![value; count] → VecRepeat
```

**2. Print Functions**
```rust
// Built-in functions (like keywords, not macros)
// Parsed as special CallBuiltin nodes
enum BuiltinFunction {
    Print,      // print("text")
    Println,    // println("text")
    Eprint,     // eprint("text")
    Eprintln,   // eprintln("text")
    Format,     // format("template {}", args)
    Assert,     // assert(condition)
    DebugAssert,// debug_assert(condition)
    Panic,      // panic("message")
}

// Format string validation:
// - Check {} placeholder count matches arguments
// - Ensure Display trait bounds for arguments
// - Perform at compile time (type checking phase)
```

**3. Derive Attributes**
```rust
// Compiler-provided only, no custom derives in v1
#[derive(Clone)]      // Generates Clone::clone implementation
#[derive(Copy)]       // Marks type as Copy (no code gen)
#[derive(Debug)]      // Generates Debug::fmt_debug implementation
#[derive(PartialEq)]  // Generates field-wise equality
#[derive(Eq)]         // Marker (requires PartialEq)
#[derive(PartialOrd)] // Generates comparison
#[derive(Ord)]        // Total ordering (requires Eq, PartialOrd)
#[derive(Hash)]       // Generates Hash::hash implementation
#[derive(Default)]    // Generates Default::default implementation

// Code generation occurs during type checking phase
// after trait resolution
```

**4. Array Repeat Syntax**
```rust
// Parser recognizes [value; count] specially
enum Expr {
    Array(Vec<Expr>),                    // [1, 2, 3]
    ArrayRepeat(Box<Expr>, Box<Expr>),  // [0; 256]
}

// Type checking ensures count is const evaluable
```

#### Implementation Notes

- All built-in syntax is **hardcoded** in the parser
- No extension mechanism (deliberate v1.0 limitation)
- Print functions are compiler intrinsics with format string checking
- Derive attributes invoke code generators for specific traits
- Future versions may add procedural macros, but not in v1.0

**IMPORTANT NOTE**: Despite the `vec!` syntax using `!`, this is NOT a macro. Vertex v1.0 has NO macro system. The `!` is purely syntactic sugar for familiarity with Rust, but `vec!` is a built-in compiler construct hardcoded in the parser.

### 2.3 Name Resolution

**Input**: AST
**Output**: Resolved AST with definition IDs
**Error Recovery**: Continue resolution, accumulate errors

#### Responsibilities
- Build module hierarchy from file system
- Resolve imports and use statements
- Create scope hierarchy
- Resolve identifiers to definitions
- Detect name conflicts and shadowing
- Validate visibility rules
- Build def-use chains

#### Data Structures
```rust
struct DefId {
    crate_id: CrateId,
    module: ModuleId,
    local_id: LocalDefId,
}

struct Scope {
    parent: Option<ScopeId>,
    defs: HashMap<String, DefId>,
    kind: ScopeKind,
}

enum ScopeKind {
    Module,
    Function,
    Block,
    Loop,
}

struct NameResolutionContext {
    scopes: Arena<Scope>,
    current_scope: ScopeId,
    def_table: HashMap<DefId, DefInfo>,
    use_resolutions: HashMap<NodeId, DefId>,
}

struct DefInfo {
    kind: DefKind,
    visibility: Visibility,
    span: Span,
}

enum DefKind {
    Mod,
    Fn,
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Const,
    Static,
    Local,
    Field,
    Variant,
}
```

#### Resolution Algorithm

**2.3.1 Module Discovery and File Loading**

Vertex uses a file-system-based module system similar to Rust:

```
myproject/
├── vertex.toml           // Project manifest
├── src/
│   ├── main.vx          // Binary entry point (has fn main)
│   ├── lib.vx           // Library entry point (optional)
│   ├── utils.vx         // Module 'utils'
│   ├── parser/
│   │   ├── mod.vx       // Module 'parser' (directory module)
│   │   ├── lexer.vx     // Module 'parser::lexer'
│   │   ├── ast.vx       // Module 'parser::ast'
│   │   └── expr.vx      // Module 'parser::expr'
│   └── codegen/
│       └── mod.vx       // Module 'codegen'
└── tests/
    └── integration.vx   // Test files
```

**Module Resolution Rules**:

1. **Crate Root**: Start from `src/main.vx` (binary) or `src/lib.vx` (library)

2. **mod Declaration**: When encountering `mod foo`:
   - Look for `foo.vx` in same directory as parent module
   - OR look for `foo/mod.vx` as directory module
   - ERROR if both exist or neither exists
   - ERROR if file unreadable or invalid UTF-8

3. **Module Hierarchy**:
   ```vertex
   // In src/main.vx:
   mod parser   // Resolves to src/parser/mod.vx
   mod utils    // Resolves to src/utils.vx

   // In src/parser/mod.vx:
   pub mod lexer  // Resolves to src/parser/lexer.vx
   pub mod ast    // Resolves to src/parser/ast.vx
   ```

4. **Inline vs File Modules**:
   ```vertex
   // Inline module (no file)
   mod inline {
       pub fn foo() { }
   }

   // File module (loads from file)
   mod utils;  // Loads utils.vx or utils/mod.vx
   ```

**Module Loading Algorithm**:

```rust
fn load_module(parent_path: &Path, mod_name: &str) -> Result<Module, Error> {
    let file_path = parent_path.join(format!("{}.vx", mod_name));
    let dir_path = parent_path.join(mod_name).join("mod.vx");

    match (file_path.exists(), dir_path.exists()) {
        (true, true) => Err(Error::AmbiguousModule {
            name: mod_name,
            file: file_path,
            dir: dir_path,
        }),
        (true, false) => parse_file(file_path),
        (false, true) => parse_file(dir_path),
        (false, false) => Err(Error::ModuleNotFound {
            name: mod_name,
            searched: vec![file_path, dir_path],
        }),
    }
}
```

**Visibility and Re-exports**:

```vertex
// In src/lib.vx
mod internal;           // Private module
pub mod public_api;     // Public module

pub use internal::useful_function;  // Re-export
```

**Module Discovery Order**:
1. Parse crate root (main.vx or lib.vx)
2. Build queue of `mod` declarations
3. Process queue (breadth-first or depth-first)
4. Recursively discover submodules
5. Build complete module tree
6. Check for circular module dependencies

2. **Import Resolution**
   - Process `use` statements in dependency order
   - Detect circular imports
   - Build import table

3. **Name Binding**
   - Two-pass approach:
     - Pass 1: Collect all definitions
     - Pass 2: Resolve references
   - Handle forward references within modules
   - Validate visibility

4. **Scope Rules**
   - Items: visible to entire module
   - Variables: visible from declaration to scope end
   - Shadowing: allowed across scopes, not within scope

#### Error Cases
- Undefined name
- Duplicate definition
- Circular imports
- Visibility violation
- Ambiguous import

### 2.4 Type Checking

**Input**: Resolved AST
**Output**: Typed AST with type information
**Error Recovery**: Assign error type, continue checking

#### Responsibilities
- Infer types for expressions
- Check type compatibility
- Validate trait bounds
- Resolve method calls
- Check pattern exhaustiveness
- Validate const expressions
- Perform lifetime inference
- **Enforce string indexing prohibition** (no Index<usize> for String/str)
- Validate overflow checking requirements (debug vs release)

#### Type Representation
```rust
enum Ty {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
    Str,
    Never,  // !
    Tuple(Vec<Ty>),
    Array(Box<Ty>, u64),
    Slice(Box<Ty>),
    Ref(Region, Box<Ty>, Mutability),
    RawPtr(Box<Ty>, Mutability),
    Fn(FnSig),
    Adt(AdtDef, Vec<Ty>),  // Struct/Enum with generic args
    Param(ParamId),        // Generic parameter
    Projection(TraitRef, Ident),  // Associated type
    Infer(InferVar),       // Type inference variable
    Error,                 // Type error placeholder
}

struct FnSig {
    params: Vec<Ty>,
    return_ty: Box<Ty>,
}

struct AdtDef {
    def_id: DefId,
    kind: AdtKind,
    generics: Generics,
    fields: Vec<FieldDef>,
}

enum AdtKind {
    Struct,
    Enum,
}
```

#### Type Inference Algorithm
Uses **Hindley-Milner** style inference with extensions:

1. **Constraint Generation**
   - Assign fresh inference variables to unknown types
   - Generate equality constraints from:
     - Variable assignments
     - Function calls
     - Binary operations
     - Return statements

2. **Constraint Solving**
   - Unification algorithm
   - Propagate type information
   - Resolve inference variables
   - Handle generic instantiation

3. **Trait Resolution**
   - Collect trait bounds
   - Check trait implementations
   - Resolve associated types
   - Validate where clauses

4. **Derive Macro Expansion**
   - Process #[derive(...)] attributes after trait resolution
   - Generate impl blocks for each derived trait:
     - Clone: Deep copy implementation
     - Copy: Marker trait (validates all fields are Copy)
     - Debug: Generate Debug::fmt_debug implementation
     - PartialEq/Eq: Field-wise equality
     - PartialOrd/Ord: Lexicographic comparison
     - Hash: Combine field hashes
     - Default: Call Default::default on all fields
   - Add generated impls to type's impl table
   - Validate trait requirements (e.g., Copy requires Clone)

#### Lifetime Inference Algorithm (Vertex Simplified)

**IMPORTANT**: Vertex v1.0 uses a simplified lifetime system compared to Rust.

**Core Principle**: Each reference has exactly one lifetime region. No explicit lifetime parameters are allowed in source code.

**Inference Rules**:

1. **Single Input Reference** → Output tied to that input
   ```vertex
   fn get_first(data: &Vec<i32>) -> &i32 {
       &data[0]  // Return lifetime = lifetime of 'data'
   }
   ```

2. **Multiple Input References** → Output is shortest input lifetime
   ```vertex
   fn choose_first(x: &str, y: &str) -> &str {
       x  // Return lifetime = min(lifetime(x), lifetime(y))
   }
   ```

3. **Method with &self** → Return borrows from self
   ```vertex
   impl Container {
       fn get_data(&self) -> &String {
           &self.data  // Return lifetime = lifetime of self
       }
   }
   ```

4. **No Input References** → Return must be 'static or owned
   ```vertex
   fn make_string() -> String {
       String::from("hello")  // OK: owned, not borrowed
   }

   fn get_static() -> &'static str {
       "static string"  // OK: 'static lifetime
   }
   ```

**Limitations (by design in v1.0)**:

1. **No Explicit Lifetime Parameters**
   ```vertex
   // INVALID in Vertex v1:
   // fn foo<'a>(x: &'a str) -> &'a str  // ERROR: explicit lifetimes not allowed

   // VALID:
   fn foo(x: &str) -> &str {  // Lifetime inferred automatically
       x
   }
   ```

2. **Cannot Express Different Lifetimes**
   ```vertex
   // IMPOSSIBLE in Vertex v1:
   // Return value with different lifetime than inputs
   // fn complex<'a, 'b>(x: &'a str, y: &'b str) -> &'a str

   // If you need this, restructure using owned types or indices
   ```

3. **Structs Cannot Store Non-Static References**
   ```vertex
   // INVALID:
   struct Container {
       data: &str  // ERROR: non-static reference in struct
   }

   // VALID alternatives:
   struct Container {
       data: String  // Use owned type
   }

   struct IndexedView {
       data: Vec<String>,
       current: usize  // Use index instead of reference
   }
   ```

4. **No Lifetime Polymorphism**
   - Cannot abstract over lifetimes
   - No lifetime bounds in trait definitions
   - Simplifies the type system significantly

**Lifetime Constraint Solving**:

```rust
struct LifetimeConstraint {
    sub: Region,      // Subregion (must outlive)
    sup: Region,      // Superregion (outlived by)
    span: Span,       // For error reporting
    reason: String,   // Why this constraint exists
}

// Inference algorithm:
// 1. Generate constraints from function signatures and expressions
// 2. Solve constraints using transitive closure
// 3. Check for contradictions (lifetime errors)
// 4. Assign concrete lifetimes to all references

// Example constraints:
// - &data[0] requires: lifetime(return) ⊆ lifetime(data)
// - x = y requires: lifetime(x) = lifetime(y) (for references)
```

**Workarounds for Complex Lifetime Needs**:

```vertex
// Instead of lifetime parameters, use indices:
struct Parser {
    source: String,       // Owned data
    position: usize       // Index into source
}

impl Parser {
    fn current_char(&self) -> Option<char> {
        self.source.chars().nth(self.position)  // Compute on demand
    }
}

// Or use owned types:
struct Node {
    data: String,         // Owned instead of &str
    children: Vec<Node>   // Owned tree
}
```

**Benefits of Simplified Lifetime System**:
- Easier to learn and use
- Faster compilation (simpler inference)
- Covers 90% of common use cases
- Can always fall back to owned types

**Tradeoffs**:
- Some Rust patterns impossible
- More copying/cloning in some cases
- May need to restructure data structures

#### Error Reporting
- Type mismatch with expected/found
- Missing trait implementation
- Invalid method receiver
- Argument count mismatch
- Pattern type mismatch
- Non-exhaustive match
- **String indexing error** - "cannot index String with usize; use .chars().nth(n) or .as_bytes()[n]"

## 3. Middle-End Phases

### 3.1 HIR (High-Level Intermediate Representation)

**Purpose**: Simplified AST for analysis

**Transformations from AST**:
- Desugar for loops to while + iterator
- Desugar if-let to match
- Desugar closure sugar
- Expand built-in syntax (vec!, print, etc.)
- Normalize paths

```rust
// Example desugaring:
// for x in iter { body }
// becomes:
// {
//     let mut __iter = IntoIterator::into_iter(iter);
//     loop {
//         match __iter.next() {
//             Ok(x) => { body }
//             Err(()) => break
//         }
//     }
// }
```

### 3.2 MIR (Mid-Level Intermediate Representation)

**Purpose**: Control-flow graph for borrow checking and optimization

#### Structure
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

struct Place {
    local: Local,
    projection: Vec<PlaceElem>,
}

enum PlaceElem {
    Deref,
    Field(FieldIdx),
    Index(Local),
    Downcast(VariantIdx),
}

enum Rvalue {
    Use(Operand),
    Ref(Mutability, Place),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    Discriminant(Place),
    Aggregate(AggKind, Vec<Operand>),
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

#### MIR Generation
1. Build control-flow graph from HIR
2. Convert expressions to assignments
3. Make control flow explicit
4. Insert drops at scope exits
5. Make panics explicit

### 3.3 Borrow Checking

**Input**: MIR
**Output**: MIR with borrow validation
**Errors**: Borrow check violations

#### Algorithm: Polonius-inspired

```rust
struct BorrowCheckContext {
    body: &Mir,
    dominators: Dominators,
    region_inference: RegionInferenceContext,
}

// Data-flow analysis
struct BorrowSet {
    borrows: Vec<BorrowData>,
    location_map: FxHashMap<Location, Vec<BorrowIndex>>,
}

struct BorrowData {
    region: Region,
    place: Place,
    kind: BorrowKind,
}

enum BorrowKind {
    Shared,
    Mut,
}
```

#### Checks Performed
1. **Loan Validity**
   - Ensure borrowed reference lifetime is valid
   - Check reference outlives all uses

2. **Aliasing Rules**
   - No simultaneous mutable borrows
   - No mutable + immutable borrows simultaneously

3. **Move Semantics**
   - Detect use-after-move
   - Validate Copy types
   - Ensure moves happen once

4. **Initialization**
   - Variables initialized before use
   - All struct fields initialized

5. **Drop Order and Semantics**
   - Track which values need dropping
   - Insert drops at correct scope exits
   - Handle conditional initialization with drop flags

#### Drop Order Rules

**IMPORTANT**: Drops occur in a deterministic order to ensure predictable cleanup:

**1. Local Variables** - Dropped in **reverse order of declaration**:
```vertex
fn example() {
    let a = String::from("first");
    let b = String::from("second");
    let c = String::from("third");
}  // Drop order: c, then b, then a
```

**2. Struct Fields** - Dropped in **declaration order**:
```vertex
struct Container {
    first: String,
    second: Vec<i32>,
    third: Box<Data>,
}  // Drop order: first, then second, then third
```

**3. Tuple Elements** - Dropped **left to right**:
```vertex
let t = (String::from("a"), String::from("b"), String::from("c"));
// Drop order: t.0, then t.1, then t.2
```

**4. Function Arguments** - Dropped in **reverse order**:
```vertex
fn consume(a: String, b: String, c: String) {
    // ...
}  // Drop order: c, then b, then a
```

**5. Nested Scopes** - Inner scope drops complete before outer:
```vertex
fn nested() {
    let outer = String::from("outer");
    {
        let inner = String::from("inner");
    }  // inner dropped here
}  // outer dropped here
```

#### Drop Flags for Conditional Initialization

For conditionally initialized values, the compiler tracks whether a drop is needed:

```vertex
fn conditional(cond: bool) {
    let x: String;
    if cond {
        x = String::from("initialized");
    }
    // Compiler inserts drop flag to track if x was initialized
}  // Drop x only if it was initialized
```

**MIR Representation of Drop Flags**:
```rust
// Generated MIR with drop flag
bb0: {
    _drop_flag_x = false;              // Initially not initialized
    switchInt(cond) -> [true: bb1, false: bb3];
}

bb1: {
    x = String::from("initialized");
    _drop_flag_x = true;               // Mark as initialized
    goto -> bb3;
}

bb3: {
    if _drop_flag_x {                  // Conditional drop
        drop(x);
    }
    return;
}
```

#### MIR Drop Statements

```rust
enum Statement {
    // ...
    StorageLive(Local),    // Allocate stack space
    StorageDead(Local),    // Deallocate stack space (after drop)
}

enum Terminator {
    // ...
    Drop {
        place: Place,              // What to drop
        target: BasicBlock,        // Where to go after drop
        unwind: Option<BasicBlock>, // Where to go if drop panics
    },
}
```

**Drop Elaboration** - Compiler pass that:
1. Identifies all values that need dropping (non-Copy types)
2. Determines drop order based on rules above
3. Inserts explicit Drop terminators in MIR
4. Generates drop flags for conditional initialization
5. Handles unwinding (drop all initialized values on panic)

**Example MIR with Drops**:
```rust
fn example() {
    let a = String::from("a");
    let b = String::from("b");
}

// Generated MIR:
bb0: {
    StorageLive(a);
    a = String::from("a");
    StorageLive(b);
    b = String::from("b");
    goto -> bb1;
}

bb1: {
    drop(b) -> [return: bb2, unwind: bb3];
}

bb2: {
    StorageDead(b);
    drop(a) -> [return: bb4, unwind: bb3];
}

bb3: {  // Unwind path
    drop(a);
    resume;
}

bb4: {
    StorageDead(a);
    return;
}
```

#### Error Messages
```
error[E0502]: cannot borrow `x` as mutable because it is also borrowed as immutable
  --> src/main.vx:5:10
   |
 4 |     let r1 = &x;
   |              -- immutable borrow occurs here
 5 |     let r2 = &mut x;
   |              ^^^^^^ mutable borrow occurs here
 6 |     println("{}", r1);
   |                   -- immutable borrow later used here
```

### 3.4 Const Evaluation

**Purpose**: Evaluate const expressions and const functions at compile time

**Input**: Typed AST with const expressions
**Output**: Evaluated constant values

#### Const Evaluation Engine

Vertex uses a **compile-time interpreter** for const contexts:

```rust
enum ConstValue {
    Int(i128),
    Uint(u128),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
    Array(Vec<ConstValue>),
    Tuple(Vec<ConstValue>),
    Struct { fields: Vec<(String, ConstValue)> },
}

struct ConstEvalContext {
    values: HashMap<DefId, ConstValue>,
    call_stack: Vec<DefId>,  // For detecting recursion
}
```

#### Supported in Const Contexts

**Allowed**:
- Arithmetic operations (+, -, *, /, %)
- Logical operations (and, or, not)
- Comparisons (==, !=, <, >, <=, >=)
- Bitwise operations (&, |, ^, <<, >>)
- if, match, loop, while (no for - requires Iterator)
- const function calls
- Array/tuple indexing
- Struct field access

**NOT Allowed**:
- Heap allocation (Vec, Box, HashMap, etc.)
- I/O operations
- Unsafe operations
- Trait method calls (no dynamic dispatch)
- for loops (require Iterator trait - Stage 1+)
- Mutable references (&mut)

#### Const Function Evaluation

```vertex
const fn factorial(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)  // Recursive const fn OK
    }
}

const FACT_10: u32 = factorial(10);  // Evaluated at compile time

const fn make_table() -> [u8; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = (i * 2) as u8;
        i += 1;
    }
    table
}

const LOOKUP: [u8; 256] = make_table();  // Runs at compile time
```

#### Const Evaluation Strategy

1. **Eager Evaluation**: Evaluate all const items immediately
2. **Lazy Evaluation**: Evaluate const fn calls on demand
3. **Caching**: Cache evaluation results to avoid recomputation
4. **Error Handling**: Report compile-time errors with full backtraces

```rust
impl ConstEvaluator {
    fn eval_const_item(&mut self, def_id: DefId) -> Result<ConstValue, ConstEvalError> {
        // Check cache
        if let Some(value) = self.cache.get(&def_id) {
            return Ok(value.clone());
        }

        // Evaluate
        let body = self.get_const_body(def_id);
        let value = self.eval_expr(body)?;

        // Cache result
        self.cache.insert(def_id, value.clone());
        Ok(value)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<ConstValue, ConstEvalError> {
        match expr {
            Expr::Literal(lit) => Ok(self.eval_literal(lit)),
            Expr::Binary { op, lhs, rhs } => {
                let lhs_val = self.eval_expr(lhs)?;
                let rhs_val = self.eval_expr(rhs)?;
                self.eval_binary_op(*op, lhs_val, rhs_val)
            }
            Expr::If { cond, then, else_ } => {
                let cond_val = self.eval_expr(cond)?;
                if cond_val.as_bool()? {
                    self.eval_block(then)
                } else if let Some(else_block) = else_ {
                    self.eval_expr(else_block)
                } else {
                    Ok(ConstValue::Unit)
                }
            }
            Expr::Call { func, args } => self.eval_const_fn_call(func, args),
            // ... other cases
            _ => Err(ConstEvalError::NotConstEvaluable),
        }
    }
}
```

#### Error Reporting

```
error[E0080]: evaluation of constant value failed
  --> src/lib.vx:5:18
   |
5  | const BAD: u32 = panic("compile error");
   |                  ^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'compile error'
   |
note: inside `BAD` at src/lib.vx:5:18
```

### 3.5 MIR Optimizations

**Optional in v1, but framework provided**:

1. **Simplification**
   - Remove dead code
   - Simplify branches
   - Constant folding

2. **Inlining**
   - Inline small functions
   - Respect `#[inline]` attributes

3. **Copy Propagation**
   - Eliminate redundant copies
   - Simplify assignments

4. **Dead Store Elimination**
   - Remove unused assignments

### 3.6 Monomorphization

**Purpose**: Expand generic functions/types to concrete versions

```rust
struct MonoItem {
    def_id: DefId,
    substs: Vec<Ty>,  // Type arguments
}

struct MonoContext {
    items: Vec<MonoItem>,
    visited: HashSet<MonoItem>,
}
```

#### Algorithm
1. Start from main() or exported functions
2. Collect all generic instantiations used
3. For each instantiation:
   - Substitute type parameters
   - Generate specialized MIR
   - Recursively process called functions
4. Eliminate unused instantiations

#### Example
```vertex
fn identity<T>(x: T) -> T { x }

fn main() {
    identity(42);      // Generates identity::<i32>
    identity("hello"); // Generates identity::<&str>
}
```

Results in two monomorphized functions:
```vertex
fn identity_i32(x: i32) -> i32 { x }
fn identity_str(x: &str) -> &str { x }
```

## 4. Backend Phases

### 4.1 Code Generation Backend Options

#### Option A: LLVM Backend (Preferred)

**Advantages**:
- Production-grade optimizations
- Wide platform support
- Excellent performance
- Debugging support (DWARF)

**Process**:
1. MIR → LLVM IR translation
2. LLVM optimization passes
3. LLVM code generation
4. Object file output

```rust
struct LLVMCodegen {
    context: LLVMContext,
    module: LLVMModule,
    builder: LLVMBuilder,
    fn_cache: HashMap<DefId, LLVMValue>,
}

impl LLVMCodegen {
    fn codegen_mir(&mut self, mir: &Mir) -> LLVMValue {
        // Translate MIR to LLVM IR
    }
}
```

#### Option B: C Backend (Bootstrap/Fallback)

**Advantages**:
- Simpler implementation
- Portable
- Easier debugging during development
- No LLVM dependency

**Process**:
1. MIR → C code translation
2. Invoke C compiler (gcc/clang)
3. Link with runtime

```rust
struct CCodegen {
    output: String,
    indent: usize,
    temp_count: usize,
}

impl CCodegen {
    fn codegen_mir(&mut self, mir: &Mir) -> String {
        // Generate C code from MIR
    }
}
```

**Generated C Example**:
```c
// Vertex: fn add(a: i32, b: i32) -> i32 { a + b }
int32_t vertex_add(int32_t a, int32_t b) {
    return a + b;
}
```

#### Option B.1: Arithmetic Operation Codegen with Overflow Handling

**Integer Overflow Strategy** (per Vertex spec):

**Debug Mode** (`-C overflow-checks=on`):
- Insert overflow checks before all arithmetic operations
- Call panic handler on overflow detection
- Performance cost: ~10-20% for arithmetic-heavy code

**Release Mode** (`-C overflow-checks=off`):
- Wrapping arithmetic (two's complement behavior)
- No runtime checks, no panics
- Matches C/C++/LLVM default behavior
- Zero overhead

**Implementation**:

```rust
impl CCodegen {
    fn codegen_binary_op(&mut self, op: BinOp, lhs: &str, rhs: &str, ty: &Ty, debug_mode: bool) -> String {
        match (op, debug_mode) {
            (BinOp::Add, true) if ty.is_signed_int() => {
                // Debug mode: checked addition
                format!(r#"
                    {{
                        {ty} __result;
                        if (__builtin_add_overflow({lhs}, {rhs}, &__result)) {{
                            vertex_panic("attempt to add with overflow", __FILE__, __LINE__);
                        }}
                        __result
                    }}
                "#, ty = self.c_type(ty), lhs = lhs, rhs = rhs)
            }
            (BinOp::Add, false) => {
                // Release mode: wrapping addition
                format!("({} + {})", lhs, rhs)
            }
            // Similar for Sub, Mul, Div, Rem, Shl, Shr
            _ => format!("({} {} {})", lhs, self.op_symbol(op), rhs)
        }
    }
}
```

**LLVM Backend Overflow Handling**:

```rust
impl LLVMCodegen {
    fn codegen_checked_add(&mut self, lhs: LLVMValue, rhs: LLVMValue, ty: &Ty) -> LLVMValue {
        // Use LLVM intrinsic: llvm.sadd.with.overflow
        let overflow_result = self.builder.build_call(
            self.get_intrinsic("llvm.sadd.with.overflow.i32"),
            &[lhs, rhs],
            "add_overflow"
        );

        // Extract result and overflow flag
        let result = self.builder.build_extract_value(overflow_result, 0, "result");
        let overflow = self.builder.build_extract_value(overflow_result, 1, "overflow");

        // Branch on overflow
        let overflow_bb = self.append_bb("overflow");
        let continue_bb = self.append_bb("continue");

        self.builder.build_cond_br(overflow, overflow_bb, continue_bb);

        // Overflow block: call panic
        self.builder.position_at_end(overflow_bb);
        self.build_panic_call("attempt to add with overflow");
        self.builder.build_unreachable();

        // Continue block
        self.builder.position_at_end(continue_bb);
        result
    }
}
```

**Generated LLVM IR Example** (Debug Mode):
```llvm
; a + b with overflow check
%1 = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %a, i32 %b)
%result = extractvalue {i32, i1} %1, 0
%overflow = extractvalue {i32, i1} %1, 1
br i1 %overflow, label %overflow_panic, label %continue

overflow_panic:
  call void @vertex_panic(i8* getelementptr inbounds ([29 x i8], [29 x i8]* @.str.overflow, i32 0, i32 0), i8* @.file, i32 %line)
  unreachable

continue:
  ; use %result
```

**Generated C Example** (Debug Mode):
```c
// Debug: a + b
{
    int32_t __result;
    if (__builtin_add_overflow(a, b, &__result)) {
        vertex_panic("attempt to add with overflow", "main.vx", 42);
    }
    __result  // evaluates to result
}

// Release: a + b
(a + b)  // No checks, wrapping behavior
```

**Opt-in Checked Methods** (Always Available):

These methods are ALWAYS checked regardless of build mode:

```vertex
let result = a.checked_add(b);     // Returns Result<i32, ()>
let result = a.saturating_add(b);  // Clamps to i32::MIN/MAX
let result = a.wrapping_add(b);    // Always wraps (two's complement)
```

Generated code:
```c
// checked_add (always returns Result)
Result_i32_unit checked_add_i32(int32_t a, int32_t b) {
    int32_t result;
    if (__builtin_add_overflow(a, b, &result)) {
        return (Result_i32_unit){ .tag = Err, .err = {} };
    }
    return (Result_i32_unit){ .tag = Ok, .ok = result };
}

// saturating_add
int32_t saturating_add_i32(int32_t a, int32_t b) {
    int64_t result = (int64_t)a + (int64_t)b;
    if (result > INT32_MAX) return INT32_MAX;
    if (result < INT32_MIN) return INT32_MIN;
    return (int32_t)result;
}

// wrapping_add
int32_t wrapping_add_i32(int32_t a, int32_t b) {
    return a + b;  // Relies on C undefined behavior -> wrapping
}
```

### 4.2 Runtime Support and Panic Handling

**Minimal Runtime Required**:
- Panic handler with unwinding/abort modes
- Memory allocator interface
- Stack unwinding support (optional)
- Atomic operations

#### Panic Implementation

**Two Panic Modes**:

1. **Unwind Mode** (default): Stack unwinding with Drop execution
2. **Abort Mode**: Immediate process termination

**Runtime Interface**:
```c
// vertex_runtime.h

// Panic function (does not return)
_Noreturn void vertex_panic(
    const char* message,
    const char* file,
    uint32_t line,
    uint32_t column
);

// Memory allocator
void* vertex_alloc(size_t size, size_t align);
void vertex_dealloc(void* ptr, size_t size, size_t align);
void* vertex_realloc(void* ptr, size_t old_size, size_t new_size, size_t align);

// Unwinding support (unwind mode only)
void vertex_begin_unwind(void* payload);
void* vertex_catch_unwind(void (*f)(void* data), void* data);
```

**Panic Behavior**:

**Unwind Mode** (`-C panic=unwind`):
```c
// Implementation using platform unwinding (libunwind on Unix)
#include <unwind.h>

typedef struct {
    const char* message;
    const char* file;
    uint32_t line;
} PanicPayload;

_Noreturn void vertex_panic(const char* msg, const char* file, uint32_t line, uint32_t column) {
    // Print panic message to stderr
    fprintf(stderr, "thread 'main' panicked at '%s', %s:%u:%u\n", msg, file, line, column);
    fprintf(stderr, "note: run with `VERTEX_BACKTRACE=1` for a backtrace\n");

    // Create payload
    PanicPayload payload = { msg, file, line };

    // Begin unwinding (platform-specific)
    vertex_begin_unwind(&payload);

    // If unwinding fails, abort
    abort();
}

// Catch unwinding (for FFI boundaries)
void* vertex_catch_unwind(void (*f)(void* data), void* data) {
    // Set up unwinding handler
    // Call function
    // Catch any panics
    // Return NULL on success, panic payload on panic
}
```

**Abort Mode** (`-C panic=abort`):
```c
_Noreturn void vertex_panic(const char* msg, const char* file, uint32_t line, uint32_t column) {
    fprintf(stderr, "thread 'main' panicked at '%s', %s:%u:%u\n", msg, file, line, column);
    abort();  // Immediate termination, no unwinding
}
```

**Generated Panic Calls**:

```vertex
// Vertex code
assert(x > 0, "x must be positive");

// Generated C (unwind mode)
if (!(x > 0)) {
    vertex_panic("x must be positive", "main.vx", 42, 5);
}

// Generated C (abort mode - same, but vertex_panic calls abort())
```

#### catch_unwind for FFI Safety

**Purpose**: Prevent unwinding across FFI boundaries (which is undefined behavior)

```vertex
// Vertex wrapper for C-callable functions
#[no_mangle]
pub extern "C" fn safe_vertex_function(x: i32) -> i32 {
    std::panic::catch_unwind(|| {
        // Function that might panic
        potentially_panicking_function(x)
    }).unwrap_or(-1)  // Return error value instead of unwinding
}
```

**Generated C**:
```c
int32_t safe_vertex_function(int32_t x) {
    void* panic_payload = vertex_catch_unwind(
        potentially_panicking_function_wrapper,
        &x
    );

    if (panic_payload) {
        // Panic occurred, return error value
        return -1;
    }

    // Normal return
    return result;
}
```

#### Unwinding and Drop

**During Unwinding** (unwind mode):
1. Walk up the stack
2. For each function frame:
   - Execute drop glue for all live values
   - Continue unwinding
3. Stop at:
   - `catch_unwind` boundary
   - `main()` (terminate program)
   - Thread boundary

**Drop Glue During Unwind**:
```rust
// MIR for function with potential panic
bb0: {
    StorageLive(a);
    a = String::from("a");
    StorageLive(b);
    b = String::from("b");

    // Call that might panic
    call(might_panic) -> [return: bb1, unwind: bb_unwind];
}

bb1: {
    // Normal path: drop in reverse order
    drop(b) -> [return: bb2, unwind: bb_unwind];
}

bb2: {
    drop(a) -> [return: bb3, unwind: bb_unwind];
}

bb3: {
    return;
}

bb_unwind: {
    // Unwind path: drop all live values
    drop(b) -> [return: bb_unwind2, unwind: terminate];
}

bb_unwind2: {
    drop(a) -> [return: resume, unwind: terminate];
}

resume: {
    resume unwind;  // Continue unwinding to caller
}

terminate: {
    terminate;  // Double panic - abort process
}
```

**Performance Impact**:
- **Unwind mode**: ~5-10% overhead (landing pads, unwind tables)
- **Abort mode**: Zero overhead

**Compilation Flags**:
```bash
vertex build -C panic=unwind   # Default: enable unwinding
vertex build -C panic=abort    # Smaller binaries, faster, no unwinding
```

### 4.3 Linking

**Static Linking** (default):
- Link all dependencies into single binary
- Include standard library

**Dynamic Linking** (future):
- Shared libraries
- Plugin systems

## 5. Phase Dependencies

```
┌──────────┐
│  Lexer   │
└────┬─────┘
     │ Token Stream
┌────▼─────┐
│  Parser  │
└────┬─────┘
     │ AST
┌────▼──────────┐
│ Name Resolution│
└────┬──────────┘
     │ Resolved AST
┌────▼───────────┐
│ Type Checking  │
└────┬───────────┘
     │ Typed AST
┌────▼────────┐
│ HIR Lowering│
└────┬────────┘
     │ HIR
┌────▼──────────┐
│ MIR Generation│
└────┬──────────┘
     │ MIR
┌────▼─────────────┐
│ Borrow Checking  │
└────┬─────────────┘
     │ Validated MIR
┌────▼───────────────┐
│ MIR Optimizations  │
└────┬───────────────┘
     │ Optimized MIR
┌────▼────────────────┐
│ Monomorphization    │
└────┬────────────────┘
     │ Monomorphized MIR
┌────▼─────────────┐
│ Code Generation  │
└────┬─────────────┘
     │ LLVM IR / C
┌────▼──────┐
│  Linking  │
└────┬──────┘
     │
   Binary
```

## 6. Error Recovery Strategy

### Philosophy
- Never panic during compilation
- Collect all errors before stopping
- Continue compilation as far as possible
- Provide helpful error messages

### Error Representation

```rust
struct CompileError {
    code: ErrorCode,
    kind: ErrorKind,
    span: Span,
    message: String,
    suggestions: Vec<Suggestion>,
    notes: Vec<String>,
}

type ErrorCode = u16;

enum ErrorKind {
    Lexical,
    Syntax,
    NameResolution,
    Type,
    BorrowCheck,
    Other,
}

struct Suggestion {
    message: String,
    replacement: Option<String>,
    span: Span,
}
```

### Error Code System

**Format**: `E` + 4-digit number (e.g., `E0308`, `E0502`)

**Category Ranges**:

| Range | Category | Examples |
|-------|----------|----------|
| E0001-E0099 | Lexical errors | Invalid characters, malformed literals |
| E0100-E0299 | Syntax errors | Missing semicolons, unclosed braces |
| E0300-E0499 | Name resolution | Undefined names, duplicate definitions |
| E0500-E0799 | Type errors | Type mismatches, missing traits |
| E0800-E0999 | Borrow check | Use after move, invalid borrows |
| E1000-E1999 | Other errors | Const eval, FFI, unsafe |

**Common Error Codes**:

```rust
// Type errors
const E0308: u16 = 308;  // Type mismatch
const E0369: u16 = 369;  // Binary operation not supported
const E0425: u16 = 425;  // Unresolved name
const E0433: u16 = 433;  // Failed to resolve import

// Borrow check errors
const E0382: u16 = 382;  // Use of moved value
const E0499: u16 = 499;  // Cannot borrow as mutable more than once
const E0502: u16 = 502;  // Cannot borrow as mutable while immutably borrowed
const E0503: u16 = 503;  // Cannot use value while mutably borrowed
const E0505: u16 = 505;  // Cannot move while borrowed

// Trait errors
const E0277: u16 = 277;  // Trait bound not satisfied
const E0599: u16 = 599;  // Method not found

// Const eval errors
const E0080: u16 = 80;   // Const evaluation failed
const E0133: u16 = 133;  // Unsafe in const context

// String indexing error (NEW - specific to Vertex)
const E0608: u16 = 608;  // Cannot index String/str with integer
```

**Error Documentation**:

Each error code has detailed documentation accessible via:
```bash
vertexc --explain E0308
```

Output:
```
Error E0308: Type Mismatch

This error occurs when the compiler expected one type but found another.

Example of erroneous code:

    let x: i32 = "hello";  // error: expected i32, found &str

The type of the value must match the type annotation. To fix this error,
either change the type annotation or change the value to match.

Correct examples:

    let x: i32 = 42;           // Correct: i32 value
    let x: &str = "hello";     // Correct: &str value
    let x = "hello";           // Correct: type inferred as &str
```

**Online Error Index**:
- Website: `https://docs.vertex.org/errors/`
- Searchable by error code
- Includes examples and solutions
- Links to language reference

### Recovery by Phase

| Phase | Recovery Strategy |
|-------|------------------|
| Lexer | Insert error token, continue |
| Parser | Skip to synchronization point |
| Name Resolution | Mark as unresolved, continue |
| Type Checking | Assign error type, continue |
| Borrow Checking | Report error, continue |
| Codegen | Skip erroneous items |

### Error Limits
- Stop after 100 errors (configurable)
- Prevent error cascades
- Deduplicate similar errors

## 7. Compiler Driver

```rust
struct CompilerConfig {
    input_files: Vec<PathBuf>,
    output: PathBuf,
    crate_type: CrateType,
    optimization_level: OptLevel,
    debug_info: bool,
    codegen_backend: CodegenBackend,
}

enum CrateType {
    Binary,
    Library,
    StaticLib,
}

struct Compiler {
    config: CompilerConfig,
    session: Session,
}

impl Compiler {
    fn compile(&mut self) -> Result<(), Vec<CompileError>> {
        // 1. Parse all files
        let ast = self.parse_crate()?;

        // 2. Name resolution
        let resolved_ast = self.resolve_names(ast)?;

        // 3. Type checking
        let typed_ast = self.type_check(resolved_ast)?;

        // 4. Lower to HIR
        let hir = self.lower_to_hir(typed_ast)?;

        // 5. Generate MIR
        let mir = self.generate_mir(hir)?;

        // 6. Borrow checking
        self.borrow_check(&mir)?;

        // 7. Optimize MIR
        let optimized_mir = self.optimize_mir(mir);

        // 8. Monomorphization
        let mono_mir = self.monomorphize(optimized_mir)?;

        // 9. Code generation
        let code = self.codegen(mono_mir)?;

        // 10. Link
        self.link(code)?;

        Ok(())
    }
}
```

## 8. Incremental Compilation (Future)

**Not in v1, but architected for**:
- Dependency tracking
- Cached compilation units
- Change detection
- Partial recompilation

## 9. Parallel Compilation

**Module-level parallelism**:
- Parse files in parallel
- Type-check independent modules in parallel
- Codegen functions in parallel

```rust
use rayon::prelude::*;

fn parse_crate_parallel(files: &[PathBuf]) -> Vec<Module> {
    files.par_iter()
         .map(|file| parse_file(file))
         .collect()
}
```

## 10. Testing Strategy

### Unit Tests
- Each phase tested independently
- Mock inputs for isolation
- Test error recovery

### Integration Tests
- End-to-end compilation tests
- Test files with expected output
- Error message tests

### Fuzzing
- Fuzz parser with random inputs
- Ensure no panics

## 11. Performance Targets

**Compilation Speed** (v1):
- 10,000 lines/second on modern hardware
- Incremental builds < 100ms (future)

**Memory Usage**:
- < 100MB for typical projects
- Streaming for large files

## 12. Build System Integration (vertex.toml)

### Project Manifest Format

**File**: `vertex.toml` at project root

**Complete Specification**:

```toml
#############################################################################
# Package Metadata
#############################################################################

[package]
name = "myproject"              # Package name (required)
version = "0.1.0"               # Semantic versioning (required)
edition = "2024"                # Language edition (required for future compat)
authors = ["Name <email@example.com>"]
description = "A Vertex package"
license = "MIT OR Apache-2.0"
repository = "https://github.com/user/project"
homepage = "https://example.com"
documentation = "https://docs.example.com"
readme = "README.md"
keywords = ["keyword1", "keyword2"]
categories = ["category1"]

#############################################################################
# Dependencies
#############################################################################

[dependencies]
# Local path dependency
utils = { path = "../utils" }

# Git dependency
parser_lib = { git = "https://github.com/user/parser", branch = "main" }
lexer = { git = "https://github.com/user/lexer", tag = "v1.0.0" }
ast = { git = "https://github.com/user/ast", rev = "abc123" }

# Registry dependency (future - when package registry exists)
# serde = "1.0"
# tokio = { version = "1.20", features = ["full"] }

# Optional dependencies (feature-gated)
logging = { path = "../logging", optional = true }

[dev-dependencies]
# Test-only dependencies
test_utils = { path = "../test_utils" }

#############################################################################
# Build Configuration
#############################################################################

[build]
optimization = "2"              # 0, 1, 2, 3, "s" (size), "z" (aggressive size)
debug = true                    # Include debug information
overflow-checks = true          # Check arithmetic overflow (default: true in debug, false in release)
panic = "unwind"                # "unwind" or "abort"
codegen-backend = "c"           # "c" or "llvm"
target = "x86_64-unknown-linux" # Target triple

# Link-time optimization
lto = false                     # false, true, "thin", "fat"

# Code generation units (parallelism)
codegen-units = 16              # Number of parallel codegen units

#############################################################################
# Release Profile
#############################################################################

[profile.dev]
optimization = "0"
debug = true
overflow-checks = true
panic = "unwind"

[profile.release]
optimization = "3"
debug = false
overflow-checks = false
panic = "unwind"  # or "abort" for smaller binaries
lto = true

[profile.test]
inherits = "dev"
overflow-checks = true

[profile.bench]
inherits = "release"
debug = true  # Debug info for profiling

#############################################################################
# Features (Optional Functionality)
#############################################################################

[features]
default = ["std"]               # Features enabled by default
std = []                        # Standard library (could be disabled for embedded)
logging = ["dep:logging"]       # Enable logging dependency
experimental = []               # Experimental features

#############################################################################
# Build Scripts (Future)
#############################################################################

[build-script]
path = "build.vx"               # Custom build script (future feature)

#############################################################################
# Binaries
#############################################################################

[[bin]]
name = "myapp"                  # Binary name
path = "src/main.vx"            # Entry point (default: src/main.vx)

# Multiple binaries in one project
[[bin]]
name = "helper_tool"
path = "src/bin/helper.vx"

#############################################################################
# Library
#############################################################################

[lib]
name = "mylib"                  # Library name (default: package name)
path = "src/lib.vx"             # Library entry (default: src/lib.vx)
crate-type = ["lib"]            # lib, staticlib, cdylib, dylib

#############################################################################
# Target-specific Dependencies
#############################################################################

[target.'cfg(unix)'.dependencies]
unix_lib = { path = "../unix_lib" }

[target.'cfg(windows)'.dependencies]
windows_lib = { path = "../windows_lib" }

#############################################################################
# Workspace (Multi-crate Projects)
#############################################################################

[workspace]
members = [
    "crates/parser",
    "crates/codegen",
    "tools/*"
]
exclude = ["crates/old"]

#############################################################################
# Metadata (Arbitrary Data for Tools)
#############################################################################

[package.metadata.docs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### Build Profiles

**Default Profiles**:

```toml
# Debug profile (default for `vertex build`)
[profile.dev]
optimization = "0"      # No optimization
debug = true            # Full debug info
overflow-checks = true  # Check overflow
incremental = true      # Incremental compilation (future)

# Release profile (for `vertex build --release`)
[profile.release]
optimization = "3"      # Aggressive optimization
debug = false           # No debug info
overflow-checks = false # Wrapping arithmetic
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization, slower build

# Test profile
[profile.test]
inherits = "dev"
```

### Dependency Version Specification

```toml
[dependencies]
# Exact version
exact = "=1.2.3"

# Semver ranges
compatible = "1.2"      # >=1.2.0, <2.0.0
minor = "1.2.3"         # >=1.2.3, <1.3.0
patch = "~1.2.3"        # >=1.2.3, <1.3.0
wildcard = "1.*"        # >=1.0.0, <2.0.0

# Multiple requirements
range = ">=1.2, <1.5"
```

### Build Commands

```bash
# Build project
vertex build                    # Debug build
vertex build --release          # Release build
vertex build --profile=custom   # Custom profile

# Run binary
vertex run                      # Build and run
vertex run --release            # Release run
vertex run --bin helper_tool    # Run specific binary

# Test
vertex test                     # Run tests
vertex test --release           # Test release build

# Clean
vertex clean                    # Remove build artifacts

# Check (type-check without codegen)
vertex check                    # Fast syntax/type check
```

## 13. Diagnostics Quality

### Error Message Format
```
error[E0308]: type mismatch
  --> src/main.vx:10:5
   |
10 |     x + "hello"
   |     ^^^^^^^^^^^ expected i32, found &str
   |
   = note: cannot add integer and string
   = help: convert the string to a number with: x + "hello".parse()?
```

### Features
- Source code snippets
- Colored output (terminal support)
- Suggestions for fixes
- Error codes for documentation lookup
- Multiple error labels

## 13. Build System Integration

**vertex.toml** format:
```toml
[package]
name = "myproject"
version = "0.1.0"

[dependencies]
std = "1.0"

[build]
optimization = "2"
debug = true
```

## Appendix A: Comparison with Rust Compiler

| Aspect | Rustc | Vertex |
|--------|-------|--------|
| Lifetimes | Explicit | Mostly inferred |
| Macros | Powerful macro system | None (built-ins only) |
| Async | Full async/await | Not in v1 |
| Compile Speed | Moderate | Target: faster |
| Error Recovery | Good | Emphasis on recovery |

## Appendix B: References

- The Rust Reference
- Crafting Interpreters (Nystrom)
- Engineering a Compiler (Cooper & Torczon)
- LLVM Language Reference

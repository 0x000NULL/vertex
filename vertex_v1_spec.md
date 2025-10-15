# Vertex Language Specification

**Version**: 1.0.0  
**Status**: Release Candidate  
**Date**: December 2024  

## Executive Summary

Vertex is a systems programming language that provides memory safety without garbage collection. It targets the same domain as Rust but with a gentler learning curve and fewer concepts to master.

**Core Value Proposition**: 80% of Rust's safety with 50% of the complexity.

## 1. Design Philosophy

### What Vertex Is
- Memory-safe systems language without GC
- Ownership-based memory management
- Zero-cost abstractions
- Minimal runtime
- C-compatible FFI

### What Vertex Is Not (v1.0)
- Not async (no colored functions in v1)
- Not a macro language (minimal macros)
- Not trying to prevent all bugs (just memory/thread safety)
- Not a research language (proven concepts only)

### Key Differentiators from Rust
1. **Simpler lifetime system** - Most lifetimes inferred, no lifetime polymorphism
2. **No macro system** - Only built-in derives
3. **Single error type** - Result<T, E> only, no Option type needed
4. **Cleaner syntax** - Fewer symbols, more keywords

## 2. Syntax

### Keywords (30 total)
```
break const continue defer else enum extern false fn for
if impl in let loop match mod mut pub return self
struct trait true type unsafe use where while
```

### Operators
```
// Arithmetic
+ - * / %

// Comparison
== != < > <= >=

// Logical (words only, no symbols)
and or not

// Bitwise
& | ^ ~ << >>

// Assignment  
= += -= *= /= %=

// Access
.    // Field/method access
::   // Path separator
[]   // Indexing
()   // Function call/grouping

// Reference/Pointer
&    // Borrow/reference
*    // Dereference/multiply
&mut // Mutable borrow

// Control Flow
?    // Error propagation
..   // Range (exclusive)
..=  // Range (inclusive)
->   // Function return type / match arm

// Special
;    // Statement terminator (optional)
,    // Separator
:    // Type annotation
_    // Placeholder/ignore
```

### Literals
```vertex
// Integers
42
1_000_000
0xff      // hex
0b1010    // binary

// Floats  
3.14
1.0e-10

// Strings and chars
'a'
"hello"
r"raw string"

// Booleans
true false
```

### Built-in Syntax

```vertex
// These are built-in syntax forms, NOT macros (Vertex has no user-defined macros)
// Note: No ! symbol - these are functions, not macros

// Vector construction
vec![1, 2, 3]                    // Creates Vec<i32>
vec![0; 100]                      // Creates Vec with 100 zeros

// Print functions (built into compiler, no ! needed)
print("text")                     // Print to stdout
println("text")                   // Print with newline
eprint("text")                    // Print to stderr
eprintln("text")                  // Print to stderr with newline

// String formatting (compiler built-in, no ! needed)
format("Hello {}", name)          // Returns formatted String
print("Value: {}", x)             // {} is replaced with Display output
println("{} + {} = {}", a, b, c)  // Multiple replacements

// Derive attributes (compiler-provided only)
#[derive(Clone)]                  // Generates Clone implementation
#[derive(Copy)]                   // Marks type as Copy
#[derive(Debug)]                  // Generates Debug implementation
#[derive(PartialEq)]              // Generates equality
#[derive(Eq)]                     // Generates full equality

// Assertion (built-in function, no ! needed)
assert(condition)                 // Panics if false
assert(x == y, "x must equal y")  // With message
debug_assert(expensive_check())   // Only in debug builds

// Array repeat syntax
[0; 256]                          // Array of 256 zeros: [i32; 256]
[true; 10]                        // Array of 10 trues: [bool; 10]
```

## 3. Type System

### Primitive Types
```vertex
// Signed integers
i8 i16 i32 i64 isize

// Unsigned integers
u8 u16 u32 u64 usize

// Floating point
f32 f64

// Text
char    // Unicode scalar
str     // String slice (borrowed)
String  // Owned string

// Unit
()      // Zero-sized
```

### String Types

```vertex
// String literals are &'static str (borrowed, compile-time)
let literal: &'static str = "hello"  // String literal
let raw: &'static str = r"raw\string" // Raw string literal

// String (owned, heap-allocated)
let owned: String = String::from("hello")
let mut s = String::new()
s.push_str("hello")

// &str (borrowed string slice)
let slice: &str = &owned           // Borrow from String
let slice: &str = &owned[0..5]     // Substring
let slice: &str = "literal"        // From literal

// Conversions
let owned = slice.to_string()      // &str -> String (allocates)
let slice = &owned                 // String -> &str (borrows)
let slice = owned.as_str()         // String -> &str (method)

// String implements Deref<Target=str>
fn takes_str(s: &str) { }
let s = String::from("hello")
takes_str(&s)  // Auto-deref String to &str

// Common operations
s.len()              // Length in bytes
s.is_empty()         // Check if empty
s.contains("sub")    // Substring search
s.starts_with("pre") // Prefix check
s.split(",")         // Returns iterator

// STRING INDEXING IS NOT ALLOWED
let s = String::from("hello")
// let c = s[0]      // ERROR: Cannot index String with usize
// Why? UTF-8 means characters can be 1-4 bytes

// Instead, use:
let chars: Vec<char> = s.chars().collect()  // Iterate over chars
let bytes: &[u8] = s.as_bytes()             // Get raw bytes
let first_char = s.chars().next()           // Get first char
```

### Ranges

```vertex
// Range types (in std::ops)
Range<T>           // start..end (exclusive end)
RangeInclusive<T>  // start..=end (inclusive end)  
RangeFrom<T>       // start.. (unbounded end)
RangeTo<T>         // ..end (unbounded start)
RangeFull          // .. (unbounded both)

// Range literals create these types
0..10              // Range<i32> { start: 0, end: 10 }
0..=10             // RangeInclusive<i32> { start: 0, end: 10 }
5..                // RangeFrom<i32> { start: 5 }
..10               // RangeTo<i32> { end: 10 }
..                 // RangeFull

// Ranges implement Iterator (when T: Step)
for i in 0..10 {   // Iterates 0 through 9
    print("{}", i)
}

for i in 0..=10 {  // Iterates 0 through 10
    print("{}", i)
}

// Range indexing for slices
let arr = [1, 2, 3, 4, 5]
let slice = &arr[1..3]   // [2, 3]
let slice = &arr[1..=3]  // [2, 3, 4]
let slice = &arr[2..]    // [3, 4, 5]
let slice = &arr[..3]    // [1, 2, 3]
let slice = &arr[..]     // [1, 2, 3, 4, 5]

// Range bounds checking
let vec = vec![1, 2, 3]
let slice = &vec[1..5]   // Panics: out of bounds
```

### Compound Types
```vertex
// Arrays (fixed size)
[T; N]           // [i32; 10]

// Slices (borrowed view)
[T]              // Used as &[T]

// Vectors (dynamic)
Vec<T>           // Vec<i32>

// Tuples
(T1, T2, T3)     // (i32, f64, String)

// Tuple field access
let t = (42, 3.14, "hello")
let first = t.0   // Field access by index: i32
let second = t.1  // f64
let third = t.2   // &str
// let invalid = t.3  // ERROR: No field 3

// Tuple destructuring (preferred)
let (x, y, z) = t  // Destructure all fields
let (a, _, c) = t  // Ignore middle field

// References
&T               // Immutable reference
&mut T           // Mutable reference

// Raw pointers (unsafe)
*const T         // Immutable pointer
*mut T           // Mutable pointer

// Function pointers
fn(T) -> U       // Function type
```

### User Types
```vertex
// Structures
struct Point {
    x: f64,
    y: f64
}

// Tuple structs  
struct Color(u8, u8, u8)

// Enums
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String)
}

// Type aliases
type NodeId = u64
```

### The Result Type
```vertex
enum Result<T, E> {
    Ok(T),
    Err(E)
}

// For error handling - the primary error type
```

### The Option Type
```vertex
enum Option<T> {
    Some(T),
    None
}

// For optional values - cleaner than Result<T, ()>
// Methods on Option<T>:
impl<T> Option<T> {
    fn is_some(&self) -> bool
    fn is_none(&self) -> bool
    fn unwrap(self) -> T  // Panics if None
    fn unwrap_or(self, default: T) -> T
    fn map<U>(self, f: fn(T) -> U) -> Option<U>
    fn and_then<U>(self, f: fn(T) -> Option<U>) -> Option<U>
    fn ok_or<E>(self, err: E) -> Result<T, E>
}
```

### Compile-Time Constants and Statics
```vertex
// Constant values (evaluated at compile time, no memory address)
const BUFFER_SIZE: usize = 1024
const MAX_CONNECTIONS: i32 = 100
const PI: f64 = 3.14159265359

// Const functions (can be evaluated at compile time)
const fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

const fn compute_table() -> [u8; 256] {
    // Compile-time array computation
    let mut table = [0; 256]
    let mut i = 0
    while i < 256 {
        table[i] = (i * 2) as u8
        i += 1
    }
    table
}

const LOOKUP_TABLE: [u8; 256] = compute_table()

// Static variables (single memory location, program lifetime)
static COUNTER: AtomicI32 = AtomicI32::new(0)    // Thread-safe
static VERSION: &str = "1.0.0"                    // Immutable static
static mut BUFFER: [u8; 1024] = [0; 1024]        // Mutable (unsafe)

// Key differences:
// const: Inlined at each use, no address, compile-time only
// static: Fixed memory location, has address, program lifetime
// static mut: Requires unsafe block to access

// Example showing the difference:
const C: i32 = 5
static S: i32 = 5

fn example() {
    let pc = &C      // ERROR: Cannot take address of const
    let ps = &S      // OK: Static has an address
    
    unsafe {
        BUFFER[0] = 42   // Accessing static mut requires unsafe
    }
}
```
```

### Array, Slice, and Vector Relationships
```vertex
// Arrays are fixed-size, stack-allocated
let array: [i32; 5] = [1, 2, 3, 4, 5]

// Slices are borrowed views into contiguous data
let slice: &[i32] = &array        // Borrow whole array
let slice: &[i32] = &array[1..3]  // Borrow sub-range

// Vectors are heap-allocated, growable
let vec: Vec<i32> = Vec::new()
let vec = vec![1, 2, 3]           // Macro for convenience

// Conversions
let slice: &[i32] = &vec          // Vec to slice (borrow)
let vec: Vec<i32> = slice.to_vec() // Slice to Vec (allocates)
let array: [i32; 3] = [1, 2, 3]
let vec: Vec<i32> = array.to_vec() // Array to Vec

// Common operations work on slices
fn sum(values: &[i32]) -> i32 {
    // Works with arrays, vectors, or any slice
}
```

## 4. Prelude

```vertex
// These items are automatically imported into every module
// (No explicit 'use' statement needed)

// Types
Result<T, E>
Option<T>
String
Vec<T>
Box<T>
Rc<T>
Arc<T>

// Traits  
Clone
Copy
Debug
PartialEq
Eq
PartialOrd
Ord
Default
Display
Iterator
IntoIterator
Drop
From
Into
AsRef
AsMut

// Functions
print
println
eprint
eprintln
format
assert
debug_assert
panic

// Result constructors
Ok
Err

// Option constructors  
Some
None

// Utility functions
drop    // Explicitly drop a value
```

## 5. Program Entry

```vertex
// Valid main function signatures for binary crates

// Basic - no arguments, no return
fn main() {
    // Program starts here
}

// With error return (common patterns)
fn main() -> Result<(), std::io::Error> {
    // Can use ? operator for I/O operations
    let contents = std::fs::read_to_string("file.txt")?
    Ok(())
}

// Generic error type
fn main() -> Result<(), Box<dyn Error>> {
    // Can return any error type
    Ok(())
}

// Exit code return
fn main() -> i32 {
    // Return 0 for success, non-zero for error
    0
}

// Accessing command-line arguments
fn main() {
    let args: Vec<String> = std::env::args().collect()
    // args[0] is the program name
    // args[1..] are the arguments
}

// Process termination
std::process::exit(0)     // Exit with success
std::process::exit(1)     // Exit with error code
// Returning from main also exits
```

## 6. Memory Model

### Ownership Rules

1. Each value has exactly one owner
2. When owner goes out of scope, value is dropped
3. Assignment moves ownership (for non-Copy types)

```vertex
let s1 = String::from("hello")
let s2 = s1  // Move: s1 no longer valid
// print(s1)  // ERROR: use after move
```

### Borrowing Rules

1. Any number of immutable borrows OR exactly one mutable borrow
2. References must always be valid
3. No explicit lifetimes in function signatures (all inferred)

```vertex
fn calculate(data: &Vec<i32>) -> i32 {
    // Immutable borrow
}

fn modify(data: &mut Vec<i32>) {
    // Mutable borrow
}
```

### Copy Types

Types implementing Copy are copied instead of moved:
- All primitive types
- Tuples of Copy types
- Arrays of Copy types

```vertex
let x = 5
let y = x  // Copy: x still valid
print(x)   // OK
```

### Drop

```vertex
// Automatic cleanup
impl Drop for FileHandle {
    fn drop(&mut self) {
        // Close file
    }
}
```

### Lifetime Inference Rules

```vertex
// Vertex infers lifetimes in most cases, but has limitations

// Simple cases - fully inferred
fn get_ref(data: &Vec<i32>) -> &i32 {
    &data[0]  // Lifetime tied to input
}

// Multiple inputs - requires explicit disambiguation
fn choose_first(x: &String, y: &String) -> &String {
    x  // Compiler infers return lifetime from x
}

// LIMITATION: Cannot express different lifetimes
// This is not possible in Vertex v1:
// fn complex<'a, 'b>(x: &'a str, y: &'b str) -> &'a str

// Struct lifetimes - automatically inferred
struct Container {
    // References in structs must use owned types or static
    data: String,        // Owned - OK
    cache: Vec<i32>,     // Owned - OK
    // field: &str,      // ERROR: Cannot store non-static reference
}

// Static references allowed
struct StaticRef {
    message: &'static str  // OK - 'static lifetime explicit
}

// Workaround for complex lifetime needs: use indices
struct IndexedView {
    data: Vec<String>,
    current: usize  // Index instead of reference
}

// Rules for inference:
// 1. Single input reference -> output gets same lifetime
// 2. Multiple input references -> output lifetime is shortest
// 3. Methods: &self lifetime used for output references
// 4. Structs cannot store non-static references (use owned types)
// 5. When inference fails, restructure to avoid complex lifetimes
```

## 7. Functions

### Basic Functions
```vertex
fn add(x: i32, y: i32) -> i32 {
    return x + y
}

// Last expression is return value
fn multiply(x: i32, y: i32) -> i32 {
    x * y  // No return keyword needed
}

// No return value
fn print_sum(x: i32, y: i32) {
    print("{}", x + y)
}
```

### Methods
```vertex
struct Rectangle {
    width: f64,
    height: f64
}

impl Rectangle {
    // ASSOCIATED FUNCTION (no self) - called with ::
    fn new(width: f64, height: f64) -> Rectangle {
        Rectangle { width, height }
    }
    
    // Another associated function
    fn square(size: f64) -> Rectangle {
        Rectangle::new(size, size)
    }
    
    // METHOD (takes &self) - called with .
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    // METHOD (takes &mut self)
    fn double(&mut self) {
        self.width *= 2.0
        self.height *= 2.0
    }
    
    // METHOD (takes self - consumes)
    fn consume(self) -> f64 {
        self.width + self.height
        // self is moved, no longer usable
    }
}

// Usage differences:
// Associated functions use ::
let rect = Rectangle::new(10.0, 20.0)
let square = Rectangle::square(5.0)

// Methods use .
let area = rect.area()       // &self method
rect.double()                // &mut self method
let sum = rect.consume()     // self method (moves rect)
// rect.area()               // ERROR: rect was moved

// Self parameter variations:
impl Example {
    fn immutable_borrow(&self) { }        // Most common
    fn mutable_borrow(&mut self) { }      // For mutations
    fn consume_self(self) { }             // Takes ownership
    fn explicit_form(self: &Self) { }     // Same as &self
    fn box_self(self: Box<Self>) { }      // Requires Box
    fn rc_self(self: Rc<Self>) { }        // Requires Rc
}
```

### Closures and Capture Semantics

```vertex
// Basic closure syntax
let add_one = |x| x + 1
let sum = |a, b| a + b

// With type annotations
let multiply: fn(i32, i32) -> i32 = |a, b| a * b

// CAPTURE SEMANTICS - How closures capture variables

// 1. Default: Capture by reference (immutable borrow)
let x = 5
let print_x = || println("{}", x)  // Borrows x
print_x()
println("{}", x)  // x still accessible

// 2. Mutable capture (mutable borrow)
let mut count = 0
let mut increment = || {
    count += 1  // Requires mutable borrow
}
increment()
// Cannot use count here while increment exists

// 3. Move capture (takes ownership)
let data = vec![1, 2, 3]
let take_ownership = move || {
    println("{:?}", data)  // Owns data
}
take_ownership()
// data no longer accessible here

// CAPTURE RULES:
// - Closures capture the minimum rights needed
// - Read-only access -> immutable borrow
// - Write access -> mutable borrow
// - Need to outlive closure -> move
// - Thread spawn -> always requires move

// Examples showing different capture modes
fn demonstrate_captures() {
    // Immutable borrow - can still use original
    let s = String::from("hello")
    let use_s = || println("{}", s)
    use_s()
    println("{}", s)  // OK
    
    // Mutable borrow - exclusive access
    let mut vec = vec![1, 2, 3]
    let mut push_vec = || vec.push(4)
    push_vec()
    // vec.push(5)  // ERROR: vec is mutably borrowed
    drop(push_vec)   // End the borrow
    vec.push(5)      // OK now
    
    // Move - transfers ownership
    let s2 = String::from("world")
    let consume = move || s2
    let result = consume()
    // println("{}", s2)  // ERROR: s2 was moved
}

// Closures for threading (must be Send + 'static)
fn thread_closure() {
    let data = vec![1, 2, 3]
    
    thread::spawn(move || {
        // Must move data for thread safety
        println("{:?}", data)
    })
    // data not accessible here
}

// Fn trait hierarchy
// Fn: Can be called multiple times, immutable capture
// FnMut: Can be called multiple times, mutable capture  
// FnOnce: Can be called once, consumes captured values

fn accepts_fn<F: Fn()>(f: F) { f(); f(); }
fn accepts_fn_mut<F: FnMut()>(mut f: F) { f(); f(); }
fn accepts_fn_once<F: FnOnce()>(f: F) { f(); }
```

## 8. Control Flow

### If Expressions
```vertex
// If is an expression
let result = if x > 0 {
    "positive"
} else if x < 0 {
    "negative"  
} else {
    "zero"
}

// Traditional if statement
if condition {
    do_something()
}
```

### Loops
```vertex
// Infinite loop
loop {
    if condition { break }
}

// While loop
while condition {
    do_work()
}

// For loop (over iterators only)
for item in collection {
    process(item)
}

// Range iteration
for i in 0..10 {
    print("{}", i)
}
```

### Match
```vertex
// Exhaustive pattern matching
match value {
    0 => "zero",
    1 => "one",
    _ => "other"
}

// Destructuring
match message {
    Message::Quit => quit(),
    Message::Move { x, y } => move_to(x, y),
    Message::Write(text) => print("{}", text)
}

// With guards
match value {
    n if n < 0 => "negative",
    n if n > 0 => "positive",
    _ => "zero"
}

// ref patterns - borrow instead of move
match value {
    ref r => {
        // r is &T, value not moved
        println("{:?}", r)
    }
}

match mut_value {
    ref mut r => {
        // r is &mut T, can modify
        *r += 1
    }
}

// @ bindings - bind value while matching pattern
match value {
    n @ 0..=10 => {
        // n binds the actual value
        println("Small number: {}", n)
    },
    n @ 11..=100 => println("Medium: {}", n),
    n => println("Large: {}", n)
}

// Complex @ binding
match point {
    Point { x: 0, y: y_val @ 0..=5 } => {
        // y_val binds the y value if between 0 and 5
        println("On axis at y={}", y_val)
    },
    _ => {}
}

// Combining ref and @
match value {
    ref s @ "hello" | ref s @ "world" => {
        // s is &str, bound to matching value
        println("Greeting: {}", s)
    },
    _ => {}
}
```

## 9. Error Handling

### Result Type

```vertex
// The only error type in Vertex
enum Result<T, E> {
    Ok(T),
    Err(E)
}

// Standard Result methods
impl<T, E> Result<T, E> {
    // Check status
    fn is_ok(&self) -> bool
    fn is_err(&self) -> bool
    
    // Extract value (panics if wrong variant)
    fn unwrap(self) -> T             // Panics if Err
    fn unwrap_err(self) -> E         // Panics if Ok
    fn expect(self, msg: &str) -> T  // Panics with message if Err
    
    // Extract with default
    fn unwrap_or(self, default: T) -> T
    fn unwrap_or_else(self, f: fn(E) -> T) -> T
    fn unwrap_or_default(self) -> T where T: Default
    
    // Transform
    fn map<U>(self, f: fn(T) -> U) -> Result<U, E>
    fn map_err<F>(self, f: fn(E) -> F) -> Result<T, F>
    fn and_then<U>(self, f: fn(T) -> Result<U, E>) -> Result<U, E>
    fn or_else<F>(self, f: fn(E) -> Result<T, F>) -> Result<T, F>
    
    // Convert to Result<T, ()> for optional values
    fn ok(self) -> Result<T, ()>
}
```

### The ? Operator
```vertex
fn read_file(path: &str) -> Result<String, IoError> {
    let file = File::open(path)?  // Returns early on error
    let contents = file.read_to_string()?
    Ok(contents)
}

// Error conversion with ?
// The ? operator can convert error types via From trait

trait From<T> {
    fn from(value: T) -> Self
}

// If ErrorB implements From<ErrorA>, ? converts automatically
impl From<IoError> for MyError {
    fn from(err: IoError) -> MyError {
        MyError::Io(err)
    }
}

fn foo() -> Result<i32, IoError> {
    // Returns IoError
}

fn bar() -> Result<i32, MyError> {
    let value = foo()?  // IoError converts to MyError via From
    Ok(value)
}

// Common pattern: Box<dyn Error> for any error type
fn generic_error() -> Result<String, Box<dyn Error>> {
    let file = File::open("test.txt")?  // IoError -> Box<dyn Error>
    let parsed: i32 = file.read_to_string()?.parse()?  // ParseError -> Box<dyn Error>
    Ok(format!("Value: {}", parsed))
}

// Without From implementation, must convert manually
fn manual_conversion() -> Result<i32, MyError> {
    match foo() {
        Ok(v) => Ok(v),
        Err(e) => Err(MyError::Custom(e.to_string()))
    }
}
```

### Explicit Handling
```vertex
match do_something() {
    Ok(value) => process(value),
    Err(e) => handle_error(e)
}
```

### Panic (Unrecoverable)
```vertex
// For bugs, not errors
panic("This should never happen")
assert(condition, "Assertion failed")
```

## 10. Traits

### Defining Traits
```vertex
trait Display {
    fn fmt(&self) -> String
}

trait Clone {
    fn clone(&self) -> Self
}
```

### Implementing Traits
```vertex
impl Display for Point {
    fn fmt(&self) -> String {
        format("({}, {})", self.x, self.y)
    }
}
```

### Trait Bounds
```vertex
fn print_it<T: Display>(value: T) {
    print("{}", value.fmt())
}

fn complex<T>(value: T) 
where T: Display + Clone {
    // Use Display and Clone methods
}
```

### Standard Traits
```vertex
// Automatically derivable
#[derive(Clone)]    // Deep copy
#[derive(Copy)]     // Bitwise copy
#[derive(Debug)]    // Debug formatting
#[derive(Eq)]       // Equality

struct Point {
    x: i32,
    y: i32
}
```

## 11. Generics

### Generic Functions
```vertex
fn swap<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

fn first<T>(vec: &Vec<T>) -> &T {
    &vec[0]
}
```

### Generic Types
```vertex
struct Pair<T> {
    first: T,
    second: T
}

enum Result<T, E> {
    Ok(T),
    Err(E)
}
```

### Generic Implementations
```vertex
impl<T> Pair<T> {
    fn new(first: T, second: T) -> Pair<T> {
        Pair { first, second }
    }
}

impl<T: Display> Pair<T> {
    fn show(&self) {
        print("{} {}", self.first.fmt(), self.second.fmt())
    }
}
```

## 12. Module System

### File Structure and Naming

```vertex
// Project structure
myproject/
├── vertex.toml        // Project configuration
├── src/
│   ├── main.vx       // Binary entry point (fn main())
│   ├── lib.vx        // Library entry point (optional)
│   ├── foo.vx        // Module 'foo'
│   └── bar/
│       ├── mod.vx    // Module 'bar' 
│       └── baz.vx    // Submodule 'bar::baz'
└── tests/
    └── integration.vx // Integration tests

// Module resolution rules:
// 1. 'mod foo' looks for:
//    - foo.vx in same directory
//    - foo/mod.vx as directory module
// 2. Modules form a hierarchy matching directory structure
// 3. Binary crates have main.vx with fn main()
// 4. Library crates have lib.vx as root
```

### Module Declarations

```vertex
// Module-level documentation
//! This module provides utility functions
//! 
//! Use double slash bang at the top of file

/// Item documentation goes before the item
/// Supports **Markdown** formatting
pub fn documented_function() { }

// In lib.vx or main.vx
mod utils           // Loads utils.vx or utils/mod.vx
mod network {       // Inline module
    pub fn connect() { }
}

// Module with documentation
/// Network utilities module
/// Provides TCP and UDP networking
pub mod network {
    //! Internal module docs can go here
    //! These document the module from inside
    
    /// Connects to a server
    /// Returns a connection handle
    pub fn connect(addr: &str) -> Connection { }
}

// In utils.vx
pub fn helper() { }   // Public to parent module

// In bar/mod.vx
pub mod baz         // Exposes submodule

// Module paths
use crate::utils    // From crate root
use super::foo      // Parent module  
use self::baz       // Current module's submodule
```

### Visibility Rules

```vertex
// DEFAULT VISIBILITY: Private to module
// Items without visibility modifiers are private

fn private_function() { }        // Private (default)
struct PrivateStruct { }         // Private (default)
enum PrivateEnum { }             // Private (default)

// Visibility modifiers
pub           // Public to all
pub(crate)    // Public within crate
pub(super)    // Public to parent module
// no modifier // Private to module (default)

// Example visibility
pub struct PublicStruct {
    pub field1: i32,        // Public everywhere
    pub(crate) field2: i32, // Public in crate
    pub(super) field3: i32, // Public to parent
    field4: i32            // Private to module (default)
}

// Important: Struct fields default to private even if struct is public
pub struct Example {
    private_by_default: i32,  // Private even though struct is pub
    pub must_be_explicit: i32 // Must explicitly mark as pub
}

// Enum variants inherit enum visibility
pub enum PublicEnum {
    Variant1,                 // As visible as the enum
    Variant2 { x: i32 }       // Variant fields always accessible
}

// Re-exports
pub use self::internal::PublicApi  // Re-export as public
use external::Thing as InternalThing  // Import as private
```

### Imports and Use Statements

```vertex
// Import specific items
use std::fs::File
use std::collections::{HashMap, HashSet}

// Import all public items
use std::io::*

// Aliasing
use std::collections::HashMap as Map
use very::long::path as short

// Multiple imports
use std::{
    fs::File,
    io::{Read, Write},
    path::Path
}

// Import precedence (highest to lowest):
// 1. Local definitions
// 2. Explicit imports (use statements)
// 3. Glob imports (use module::*)
// 4. Standard library prelude
```

### Crate Structure

```vertex
// Binary crate (application)
// src/main.vx
fn main() {
    // Entry point
}

// Library crate  
// src/lib.vx
pub fn public_api() { }
mod internal { }

// Both binary and library
// src/lib.vx - Library code
// src/main.vx - Binary using library
use crate::public_api  // Binary can use library

// Tests directory
// tests/integration.vx
use myproject  // Import library being tested

#[test]
fn test_integration() {
    myproject::public_api()
}
```

### Module Initialization

```vertex
// Modules are initialized in dependency order
// 1. Dependencies (depth-first)
// 2. Current module
// 3. Parent module

// Static initialization
static INIT: AtomicBool = AtomicBool::new(false)

// Module-level code runs once at program start
// (Currently not supported - under consideration for v2)
```

## 13. Unsafe Code and Memory Safety

### Unsafe Operations

```vertex
// Unsafe block allows specific dangerous operations
unsafe {
    // 1. Dereferencing raw pointers
    let value = *raw_ptr
    *mut_ptr = 42
    
    // 2. Calling unsafe functions
    unsafe_function()
    
    // 3. Accessing mutable statics
    GLOBAL_COUNTER += 1
    
    // 4. Implementing unsafe traits
    // (Send/Sync - see below)
    
    // 5. Inline assembly
    asm("nop")
}
```

### Undefined Behavior

The following operations cause undefined behavior and must NEVER occur, even in unsafe code:

```vertex
// 1. Data races
// - Simultaneous access to memory where at least one is a write
// - Without proper synchronization (Mutex, Atomic, etc.)

// 2. Dereferencing invalid pointers
// - Null pointers
// - Dangling pointers (use-after-free)
// - Unaligned pointers (unless type allows)
// - Pointers to invalid memory

// 3. Breaking aliasing rules
// - Multiple mutable references to same memory
// - Mutable and immutable references to same memory simultaneously
unsafe {
    let ptr = &mut value as *mut i32
    let ref1 = &mut *ptr
    let ref2 = &mut *ptr  // UB: Two mutable refs
}

// 4. Invalid primitive values
// - bool that is not 0 or 1
// - enum with invalid discriminant
// - char outside valid Unicode range
// - null references (references must always be valid)

// 5. Uninitialized memory
// - Reading uninitialized memory
// - Partially initialized structs/enums
unsafe {
    let mut x: i32
    let y = x  // UB: Reading uninitialized
}

// 6. Violating type layout assumptions
// - Wrong size/alignment via transmute
// - Type punning without repr(C)

// 7. Unwinding through FFI
// - Panic across FFI boundary without catch_unwind

// 8. Producing invalid UTF-8 in strings
unsafe {
    let mut s = String::from("hello")
    let bytes = s.as_mut_vec()
    bytes[0] = 0xFF  // UB: Invalid UTF-8
}
```

### Memory Safety Rules

```vertex
// Safe Rust guarantees (maintained outside unsafe):
// 1. No null pointer dereferences
// 2. No data races
// 3. No buffer overflows
// 4. No use after free
// 5. No double free
// 6. No uninitialized memory access

// Unsafe code must maintain these invariants:

// Validity invariants (always true):
// - References are always valid, aligned, and point to valid data
// - Owned values are only accessed by owner
// - Shared references (&T) allow aliasing but no mutation
// - Unique references (&mut T) allow mutation but no aliasing

// Safety invariants (for safe abstraction):
struct SafeWrapper {
    // INVARIANT: ptr always points to valid memory
    ptr: *mut i32,
    // INVARIANT: len <= capacity
    len: usize,
    capacity: usize
}

impl SafeWrapper {
    pub fn new() -> Self {
        unsafe {
            let ptr = allocate(10)
            SafeWrapper { ptr, len: 0, capacity: 10 }
        }
    }
    
    pub fn push(&mut self, value: i32) {
        // Safe interface maintains invariants
        assert(self.len < self.capacity)
        unsafe {
            *self.ptr.add(self.len) = value
            self.len += 1
        }
    }
}
```

### Unsafe Traits

```vertex
// Send: Type can be transferred across thread boundaries
unsafe trait Send { }

// Auto-implemented for types that don't contain:
// - Rc<T> (non-atomic reference counting)
// - Raw pointers
// - Thread-local storage

// Sync: Type can be shared between threads (&T is Send)
unsafe trait Sync { }

// Auto-implemented for types that don't contain:
// - Cell/RefCell (interior mutability without synchronization)
// - Rc<T>
// - Non-Sync types

// Manual implementation (must guarantee thread safety)
struct MyType {
    data: *mut i32  // Raw pointer prevents auto Send/Sync
}

unsafe impl Send for MyType { }  // I guarantee this is thread-safe
unsafe impl Sync for MyType { }  // I guarantee concurrent access is safe
```

### FFI Safety

```vertex
// C functions
extern "C" {
    // Unsafe by default - C has no safety guarantees
    fn c_function(ptr: *const i32) -> i32
}

// Calling FFI
unsafe {
    // Caller must ensure:
    // 1. Pointer is valid for C function's requirements
    // 2. Any aliasing rules C expects
    // 3. Correct calling convention
    // 4. No unwinding into C
    let result = c_function(ptr)
}

// Export to C
#[no_mangle]
pub extern "C" fn vertex_function(x: i32) -> i32 {
    // Should not panic (unwinding through FFI is UB)
    // Use catch_unwind if might panic
    std::panic::catch_unwind(|| {
        // Function that might panic
        x * 2
    }).unwrap_or(-1)
}

// Type layout for FFI
#[repr(C)]  // Guarantees C-compatible layout
struct FfiStruct {
    x: i32,    // Fixed layout order
    y: f64     // Predictable padding
}

#[repr(transparent)]  // Same layout as inner type
struct Wrapper(i32)   // Can be passed where i32 expected
```

### Safe Abstraction Guidelines

```vertex
// Unsafe code should be wrapped in safe abstractions

// DON'T: Expose raw unsafe operations
pub fn bad_api() -> *mut i32 {
    unsafe { allocate() }  // Leaks unsafety
}

// DO: Provide safe interface
pub struct SafeBuffer {
    ptr: *mut i32,
    len: usize
}

impl SafeBuffer {
    pub fn new(size: usize) -> Self {
        unsafe {
            // Unsafe contained here
            SafeBuffer {
                ptr: allocate(size),
                len: size
            }
        }
    }
    
    pub fn get(&self, index: usize) -> i32 {
        assert(index < self.len)  // Bounds check
        unsafe {
            // Safe because we checked bounds
            *self.ptr.add(index)
        }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        unsafe {
            // Clean up in destructor
            deallocate(self.ptr, self.len)
        }
    }
}
```

## 14. Standard Library

### Core Type Methods

```vertex
// String methods
impl String {
    fn new() -> String
    fn from(s: &str) -> String
    fn with_capacity(capacity: usize) -> String
    
    // Mutation
    fn push(mut self, ch: char)
    fn push_str(mut self, s: &str) 
    fn pop(mut self) -> Result<char, ()>
    fn clear(mut self)
    fn truncate(mut self, new_len: usize)
    
    // Query
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn capacity(&self) -> usize
    fn as_str(&self) -> &str
    
    // Conversion
    fn into_bytes(self) -> Vec<u8>
    fn from_utf8(Vec<u8>) -> Result<String, Utf8Error>
}

// Vec<T> methods
impl<T> Vec<T> {
    fn new() -> Vec<T>
    fn with_capacity(capacity: usize) -> Vec<T>
    
    // Mutation
    fn push(mut self, value: T)
    fn pop(mut self) -> Result<T, ()>
    fn insert(mut self, index: usize, element: T)
    fn remove(mut self, index: usize) -> T
    fn clear(mut self)
    
    // Query
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn capacity(&self) -> usize
    fn get(&self, index: usize) -> Result<&T, ()>
    
    // Iteration
    fn iter(&self) -> Iter<T>
    fn iter_mut(mut self) -> IterMut<T>
    
    // Conversion
    fn as_slice(&self) -> &[T]
}

// HashMap<K, V> methods
use std::collections::HashMap

impl<K: Eq + Hash, V> HashMap<K, V> {
    fn new() -> HashMap<K, V>
    fn with_capacity(capacity: usize) -> HashMap<K, V>
    
    // Insertion and removal
    fn insert(&mut self, k: K, v: V) -> Result<V, ()>  // Returns old value
    fn remove(&mut self, k: &K) -> Result<V, ()>
    fn clear(&mut self)
    
    // Query
    fn get(&self, k: &K) -> Result<&V, ()>
    fn get_mut(&mut self, k: &K) -> Result<&mut V, ()>
    fn contains_key(&self, k: &K) -> bool
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    
    // Iteration
    fn iter(&self) -> Iter<K, V>        // (&K, &V) pairs
    fn keys(&self) -> Keys<K, V>        // &K iterator
    fn values(&self) -> Values<K, V>    // &V iterator
    
    // Entry API for efficient updates
    fn entry(&mut self, k: K) -> Entry<K, V>
}

// HashMap example
let mut map = HashMap::new()
map.insert("key", "value")
let value = map.get(&"key").unwrap()
map.remove(&"key")

// size_of and align_of - Built-in const functions
const fn size_of<T>() -> usize     // Size in bytes
const fn align_of<T>() -> usize    // Alignment requirement

// Usage
const SIZE: usize = size_of<i32>()      // 4
const ALIGN: usize = align_of<i32>()    // 4
let runtime_size = size_of<String>()    // 24 (on 64-bit)
```

### Core Types
```vertex
// Collections
Vec<T>           // Dynamic array
HashMap<K, V>    // Hash map
String           // Owned string

// Smart pointers
Box<T>           // Heap allocation
Rc<T>            // Reference counting
Arc<T>           // Atomic RC (thread-safe)

// Synchronization
Mutex<T>         // Mutual exclusion
RwLock<T>        // Read-write lock
AtomicI32        // Atomic integer

// I/O
File             // File handle
TcpStream        // TCP connection
```

### Smart Pointers

```vertex
// Box<T> - Single ownership heap allocation
impl<T> Box<T> {
    fn new(value: T) -> Box<T>  // Allocate on heap
}

// Box automatically dereferences
let boxed = Box::new(42)
let value: i32 = *boxed  // Deref to access value

// Box is Drop - automatically frees memory
fn example() {
    let b = Box::new(vec![1, 2, 3])
}  // b dropped here, memory freed

// Rc<T> - Reference counting (single-threaded)
use std::rc::Rc

impl<T> Rc<T> {
    fn new(value: T) -> Rc<T>
    fn clone(&self) -> Rc<T>     // Increments ref count
    fn strong_count(&self) -> usize
    fn weak_count(&self) -> usize
}

// Rc example
let rc1 = Rc::new(vec![1, 2, 3])
let rc2 = rc1.clone()  // Both point to same data
// Data dropped when last Rc is dropped

// Weak<T> - Weak references (break cycles)
impl<T> Rc<T> {
    fn downgrade(&self) -> Weak<T>
}

impl<T> Weak<T> {
    fn upgrade(&self) -> Result<Rc<T>, ()>  // May fail if dropped
}

// Arc<T> - Atomic reference counting (thread-safe)
use std::sync::Arc

impl<T> Arc<T> {
    fn new(value: T) -> Arc<T>
    fn clone(&self) -> Arc<T>    // Thread-safe increment
    fn strong_count(&self) -> usize
    fn weak_count(&self) -> usize
    fn downgrade(&self) -> Weak<T>
}

// Arc is Send + Sync if T is Send + Sync
fn share_between_threads() {
    let data = Arc::new(vec![1, 2, 3])
    let data2 = data.clone()
    
    thread::spawn(move || {
        println("{:?}", data2)  // OK: Arc is thread-safe
    })
}

// RefCell<T> - Interior mutability (single-threaded)
use std::cell::RefCell

impl<T> RefCell<T> {
    fn new(value: T) -> RefCell<T>
    fn borrow(&self) -> Ref<T>        // Runtime borrow check
    fn borrow_mut(&self) -> RefMut<T> // Runtime mutable borrow
    fn try_borrow(&self) -> Result<Ref<T>, BorrowError>
    fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError>
}

// RefCell allows mutation through shared reference
let cell = RefCell::new(5)
let shared = &cell  // Shared reference
*shared.borrow_mut() = 10  // But can mutate!

// Cell<T> - Interior mutability for Copy types
use std::cell::Cell

impl<T: Copy> Cell<T> {
    fn new(value: T) -> Cell<T>
    fn get(&self) -> T
    fn set(&self, value: T)
}

// Cell example
let cell = Cell::new(5)
cell.set(10)  // Can mutate through shared ref
let value = cell.get()

// Smart pointer traits
trait Deref {
    type Target
    fn deref(&self) -> &Self::Target
}

trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target
}

// All smart pointers implement Deref
impl<T> Deref for Box<T> {
    type Target = T
    fn deref(&self) -> &T { /* ... */ }
}

impl<T> Deref for Rc<T> {
    type Target = T
    fn deref(&self) -> &T { /* ... */ }
}

impl<T> Deref for Arc<T> {
    type Target = T
    fn deref(&self) -> &T { /* ... */ }
}
```

### Common Operations
```vertex
// Iterators
vec.iter()
   .filter(|x| x > 0)
   .map(|x| x * 2)
   .collect()

// String operations
string.len()
string.contains("text")
string.split(",")

// File I/O
let contents = std::fs::read_to_string("file.txt")?
std::fs::write("output.txt", data)?
```

## 15. Core Protocols

### Iterator Protocol

```vertex
// Core iteration traits
trait Iterator {
    type Item
    fn next(&mut self) -> Result<Self::Item, ()>
}

trait IntoIterator {
    type Item
    type IntoIter: Iterator<Item = Self::Item>
    fn into_iter(self) -> Self::IntoIter
}

// FOR LOOP DESUGARING
// This:
for x in collection {
    body
}

// Desugars to:
{
    let mut iter = IntoIterator::into_iter(collection)
    loop {
        match iter.next() {
            Ok(x) => { body },
            Err(()) => break
        }
    }
}

// Iterator implementations for Vec
impl<T> IntoIterator for Vec<T> {
    type Item = T
    type IntoIter = VecIntoIter<T>
    fn into_iter(self) -> VecIntoIter<T>  // Consumes vector
}

impl<T> IntoIterator for &Vec<T> {
    type Item = &T
    type IntoIter = VecIter<T>
    fn into_iter(self) -> VecIter<T>  // Borrows vector
}

impl<T> IntoIterator for &mut Vec<T> {
    type Item = &mut T
    type IntoIter = VecIterMut<T>
    fn into_iter(self) -> VecIterMut<T>  // Mutably borrows
}

// Iterator methods (provided by Iterator trait)
impl<I: Iterator> I {
    // Transformers
    fn map<B, F>(self, f: F) -> Map<Self, F> 
    where F: Fn(Self::Item) -> B
    
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where P: Fn(&Self::Item) -> bool
    
    fn flat_map<U, F>(self, f: F) -> FlatMap<Self, F>
    where F: Fn(Self::Item) -> U, U: IntoIterator
    
    // Consumers
    fn collect<C>(self) -> C
    where C: FromIterator<Self::Item>
    
    fn fold<B, F>(self, init: B, f: F) -> B
    where F: Fn(B, Self::Item) -> B
    
    fn sum<S>(self) -> S
    where S: Sum<Self::Item>
    
    // Queries
    fn count(self) -> usize
    fn last(self) -> Result<Self::Item, ()>
    fn nth(&mut self, n: usize) -> Result<Self::Item, ()>
    
    // Predicates
    fn any<F>(&mut self, f: F) -> bool
    where F: Fn(Self::Item) -> bool
    
    fn all<F>(&mut self, f: F) -> bool
    where F: Fn(Self::Item) -> bool
    
    // Combinators
    fn take(self, n: usize) -> Take<Self>
    fn skip(self, n: usize) -> Skip<Self>
    fn enumerate(self) -> Enumerate<Self>
    fn zip<U>(self, other: U) -> Zip<Self, U::IntoIter>
    where U: IntoIterator
}

// Range iteration
impl Iterator for Range<i32> {
    type Item = i32
    fn next(&mut self) -> Result<i32, ()> {
        if self.start < self.end {
            let n = self.start
            self.start += 1
            Ok(n)
        } else {
            Err(())
        }
    }
}

// Collection from iterator
trait FromIterator<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Vec<T> {
        let mut vec = Vec::new()
        for item in iter {
            vec.push(item)
        }
        vec
    }
}

// Type inference with collect example:
let numbers = vec![1, 2, 3, 4, 5]
let doubled: Vec<i32> = numbers.iter()
    .map(|x| x * 2)
    .collect()  // Type annotation on 'doubled' guides inference
```

### Display Protocol
```vertex
trait Display {
    fn fmt(&self) -> String
}

// All types can implement Display for string conversion
impl Display for i32 {
    fn fmt(&self) -> String {
        // Convert to string
    }
}
```

### String Formatting
```vertex
// Built-in formatting function
fn format(template: &str, args: ...) -> String

// Formatting syntax
format("Hello {}", name)           // Simple interpolation
format("Value: {}", 42)            // Any Display type
format("Point: ({}, {})", x, y)    // Multiple args

// Print functions (built-in)
fn print(template: &str, args: ...)     // To stdout
fn eprint(template: &str, args: ...)    // To stderr
fn println(template: &str, args: ...)   // With newline
```

### Numeric Conversions
```vertex
// Primitive casting with 'as' keyword
let x: i32 = 42
let y: i64 = x as i64      // Widening cast
let z: i16 = x as i16      // Narrowing cast (may truncate)
let f: f32 = x as f32      // Int to float

// Overflow behavior
// Debug mode: Panic on overflow
// Release mode: Wrapping arithmetic
let a: u8 = 255
let b = a + 1  // Debug: panic, Release: wraps to 0

// Checked arithmetic (never panics)
let result = a.checked_add(1)  // Returns Result<u8, ()>
let result = a.saturating_add(1)  // Saturates at MAX
let result = a.wrapping_add(1)    // Always wraps
```

### Error Trait
```vertex
trait Error {
    fn description(&self) -> &str
    fn source(&self) -> Result<&Error, ()> {
        Err(())  // Default: no source
    }
}

// Standard error types implement Error
impl Error for IoError {
    fn description(&self) -> &str {
        match self {
            IoError::NotFound => "File not found",
            IoError::PermissionDenied => "Permission denied",
            // ...
        }
    }
}
```

### Operator Overloading
```vertex
// Arithmetic operators
trait Add<Rhs = Self> {
    type Output
    fn add(self, rhs: Rhs) -> Self::Output
}

trait Sub<Rhs = Self> {
    type Output
    fn sub(self, rhs: Rhs) -> Self::Output
}

// Example implementation
impl Add for Point {
    type Output = Point
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

// Comparison operators
trait PartialEq {
    fn eq(&self, other: &Self) -> bool
    fn ne(&self, other: &Self) -> bool {
        not self.eq(other)
    }
}
```

### Standard Traits

```vertex
// Conversion traits
trait From<T> {
    fn from(value: T) -> Self
}

trait Into<T> {
    fn into(self) -> T
}

// Automatic implementation: From implies Into
impl<T, U> Into<U> for T where U: From<T> {
    fn into(self) -> U {
        U::from(self)
    }
}

// Default values
trait Default {
    fn default() -> Self
}

// Reference conversions
trait AsRef<T> {
    fn as_ref(&self) -> &T
}

trait AsMut<T> {
    fn as_mut(&mut self) -> &mut T
}

// Smart pointer dereferencing
trait Deref {
    type Target
    fn deref(&self) -> &Self::Target
}

trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target
}

// Indexing
trait Index<Idx> {
    type Output
    fn index(&self, index: Idx) -> &Self::Output
}

trait IndexMut<Idx>: Index<Idx> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output
}

// Collection traits
trait IntoIterator {
    type Item
    type IntoIter: Iterator<Item = Self::Item>
    fn into_iter(self) -> Self::IntoIter
}

trait FromIterator<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self
}

trait Extend<A> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T)
}

// Thread safety markers
unsafe trait Send { }  // Can be transferred across thread boundaries
unsafe trait Sync { }  // Can be shared between threads

// Automatically implemented for most types
// NOT Send: Rc, raw pointers
// NOT Sync: Cell, RefCell, Rc

// Formatting trait (simplified)
trait Debug {
    fn fmt_debug(&self) -> String
}

// Size trait
trait Sized { }  // Automatically implemented for sized types

// Copy semantics
trait Copy: Clone { }  // Marker trait for bitwise copy
trait Clone {
    fn clone(&self) -> Self
}

// Usage examples
impl Default for Point {
    fn default() -> Self {
        Point { x: 0.0, y: 0.0 }
    }
}

impl<T> AsRef<[T]> for Vec<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T
    fn index(&self, i: usize) -> &T {
        &self.data[i]
    }
}
```

## 16. Concurrency

### Concurrency Primitives

```vertex
// Mutex<T> - Mutual exclusion
use std::sync::Mutex

impl<T> Mutex<T> {
    fn new(value: T) -> Mutex<T>
    fn lock(&self) -> MutexGuard<T>     // Blocks until acquired
    fn try_lock(&self) -> Result<MutexGuard<T>, TryLockError>
}

// MutexGuard automatically releases lock when dropped
let mutex = Mutex::new(0)
{
    let mut guard = mutex.lock()
    *guard += 1  // Access through guard
}  // Lock automatically released

// RwLock<T> - Read-write lock
use std::sync::RwLock

impl<T> RwLock<T> {
    fn new(value: T) -> RwLock<T>
    fn read(&self) -> RwLockReadGuard<T>   // Multiple readers OK
    fn write(&self) -> RwLockWriteGuard<T> // Single writer
}

// Atomic types
use std::sync::atomic::*

// Available atomic types
AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64
AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize

// Atomic operations
impl AtomicI32 {
    fn new(v: i32) -> AtomicI32
    fn load(&self, order: Ordering) -> i32
    fn store(&self, val: i32, order: Ordering)
    fn compare_and_swap(&self, current: i32, new: i32, order: Ordering) -> i32
    fn fetch_add(&self, val: i32, order: Ordering) -> i32
    fn fetch_sub(&self, val: i32, order: Ordering) -> i32
}

// Memory ordering
enum Ordering {
    Relaxed,   // No synchronization
    Release,   // Write barrier
    Acquire,   // Read barrier  
    AcqRel,    // Both barriers
    SeqCst     // Sequential consistency (default)
}

// Usage
let counter = AtomicI32::new(0)
counter.fetch_add(1, Ordering::SeqCst)

// Condvar - Condition variable
use std::sync::Condvar

impl Condvar {
    fn new() -> Condvar
    fn wait<T>(&self, guard: MutexGuard<T>) -> MutexGuard<T>
    fn notify_one(&self)
    fn notify_all(&self)
}

// Barrier - Synchronization point for multiple threads
use std::sync::Barrier

impl Barrier {
    fn new(n: usize) -> Barrier
    fn wait(&self) -> BarrierWaitResult
}

// Once - One-time initialization
use std::sync::Once

static INIT: Once = Once::new()
INIT.call_once(|| {
    // Runs exactly once
    initialize()
})
```

### Threads
```vertex
use std::thread

// thread::spawn constraints:
// Closure must be: FnOnce + Send + 'static
// - FnOnce: Called once (can consume captured values)
// - Send: Safe to transfer across thread boundaries
// - 'static: No borrowed references (or only 'static refs)

fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static

// Valid: moves ownership
let data = vec![1, 2, 3]
let handle = thread::spawn(move || {
    println("{:?}", data)  // OK: owns data
})

// Invalid: borrows local data
let data = vec![1, 2, 3]
let handle = thread::spawn(|| {
    println("{:?}", data)  // ERROR: closure may outlive data
})

// Solution: use Arc for sharing
let data = Arc::new(vec![1, 2, 3])
let data2 = data.clone()
let handle = thread::spawn(move || {
    println("{:?}", data2)  // OK: Arc is Send + 'static
})

// Thread handle methods
impl<T> JoinHandle<T> {
    fn join(self) -> Result<T, Box<dyn Any + Send>>
    fn thread(&self) -> &Thread
}

handle.join()  // Wait for thread to finish
```

### Channels
```vertex
let (tx, rx) = channel()

thread::spawn(move || {
    tx.send(42)
})

let value = rx.recv()?
```

### Arc + Mutex Pattern
```vertex
let data = Arc::new(Mutex::new(0))
let data_clone = data.clone()

thread::spawn(move || {
    let mut guard = data_clone.lock()
    *guard += 1
})
```

## 17. Build System

### Project File (vertex.toml)
```toml
[package]
name = "myproject"
version = "1.0.0"

[dependencies]
serde = "1.0"

[profile.release]
opt-level = 3
```

### Commands
```bash
vertex new myproject    # Create project
vertex build           # Build
vertex run             # Build and run
vertex test            # Run tests
vertex fmt             # Format code
```

## 18. Testing

### Unit Tests
```vertex
#[test]
fn test_add() {
    assert(add(2, 2) == 4)
}

#[test]
fn test_panic() {
    // Test should panic
    #[should_panic]
    divide(1, 0)
}
```

## 19. Language Semantics

### Evaluation Order
- Left to right
- Eager evaluation
- Function arguments evaluated before call

### Type Inference
- Local type inference only
- No global type inference
- Must specify function parameter types
- Can omit return types if inferrable

### Method Resolution
1. Inherent methods (impl Type)
2. Trait methods in scope
3. Error if ambiguous

### Drop Order
1. Local variables in reverse declaration order
2. Struct fields in declaration order
3. Tuple elements left to right

// Drop during panic
// If a destructor panics during unwinding:
// - In debug: Double panic -> abort
// - In release: Typically abort (depends on panic setting)

struct A;
struct B;
impl Drop for A { 
    fn drop(&mut self) { 
        panic!("A panics")  // Panics during drop
    } 
}
impl Drop for B { 
    fn drop(&mut self) { 
        println("B drops") 
    } 
}

fn test() {
    let a = A;
    let b = B;
    panic!("Initial panic")
    // Drop order: b drops, then a drops
    // When a panics during drop -> abort (double panic)
}

// Safe drop patterns:
impl Drop for SafeResource {
    fn drop(&mut self) {
        // Never panic in destructors!
        // Use catch_unwind if calling code that might panic
        let _ = std::panic::catch_unwind(|| {
            potentially_panicking_cleanup()
        });
    }
}

// Drop order guarantees:
// - Values dropped in reverse order of construction
// - Temporaries dropped at end of statement
// - Function arguments dropped after function returns
// - Match arms drop temporaries when arm exits

### Type Coercion

```vertex
// AUTOMATIC TYPE COERCIONS (implicit conversions)

// 1. Deref coercion
// &T -> &U when T implements Deref<Target=U>

let s = String::from("hello")
let str_ref: &str = &s  // String derefs to str

fn takes_str(s: &str) { }
takes_str(&s)  // Automatic deref coercion

// 2. Array to slice coercion
let array: [i32; 5] = [1, 2, 3, 4, 5]
let slice: &[i32] = &array  // Automatic coercion

fn takes_slice(s: &[i32]) { }
takes_slice(&array)  // Array coerces to slice

// 3. Subtyping (lifetime shortening)
// &'a T -> &'b T when 'a outlives 'b
fn shorter_lifetime<'a>(r: &'a str) {
    let _: &str = r  // 'a coerces to anonymous lifetime
}

// 4. Function item to function pointer
fn my_function() { }
let f: fn() = my_function  // Function item coerces to fn pointer

// EXPLICIT TYPE CONVERSIONS

// 1. Numeric casting with 'as'
let x: i32 = 42
let y: i64 = x as i64        // Widening
let z: i16 = x as i16        // Narrowing (may truncate)
let f: f32 = x as f32        // Int to float
let b: u8 = 300u16 as u8     // Truncates to 44

// 2. Pointer casting
let x = 5
let ptr: *const i32 = &x as *const i32    // Reference to raw pointer
let addr: usize = ptr as usize            // Pointer to integer
let ptr2: *const u8 = addr as *const u8   // Integer to pointer

// 3. Deref coercion rules
// Types that implement Deref automatically coerce
impl<T> Deref for Box<T> {
    type Target = T
}
impl Deref for String {
    type Target = str
}
impl<T> Deref for Vec<T> {
    type Target = [T]
}
impl<T> Deref for Rc<T> {
    type Target = T
}
impl<T> Deref for Arc<T> {
    type Target = T
}

// Deref coercion chain
// &Box<String> -> &String -> &str
fn example(s: &Box<String>) {
    let _: &String = s  // Box<String> derefs to String
    let _: &str = s     // String derefs to str
}

// NO implicit numeric conversions
let x: i32 = 42
let y: i64 = x       // ERROR: No implicit conversion
let y: i64 = x as i64  // OK: Explicit cast

// NO implicit bool conversions
let x = 5
if x { }             // ERROR: i32 doesn't coerce to bool
if x != 0 { }        // OK: Explicit comparison

// Slice DST (Dynamically Sized Type) coercion
// Arrays know size at compile time, slices don't
fn print_slice(s: &[i32]) {
    for i in s { println("{}", i) }
}

let array = [1, 2, 3, 4, 5]
print_slice(&array)         // [i32; 5] -> &[i32]
print_slice(&array[1..3])   // Sub-slice

let vec = vec![1, 2, 3]
print_slice(&vec)           // Vec<i32> -> &[i32] via Deref

// Method resolution with deref coercion
let s = Box::new(String::from("hello"))
s.len()  // Automatically derefs: Box<String> -> String -> str
         // Finds len() method on str
```

### Name Resolution Rules
```vertex
// Resolution order:
// 1. Local variables and parameters
// 2. Items in current module
// 3. Imported items
// 4. Items in parent modules

use std::result::Result as StdResult

fn example() {
    struct Result;  // Local type shadows import
    
    let x: Result = Result;        // Uses local Result
    let y: StdResult<i32, ()> = Ok(42); // Uses imported type
}

// Method resolution
struct Foo;

impl Foo {
    fn method(&self) {}  // Inherent method
}

trait Bar {
    fn method(&self) {}  // Trait method
}

impl Bar for Foo {
    fn method(&self) {}
}

let foo = Foo;
foo.method()  // Calls inherent method (higher priority)
Bar::method(&foo)  // Explicitly call trait method
```

### Debug vs Release Semantics
```vertex
// Integer overflow behavior
// Debug mode: Panic on overflow
let x: u8 = 255
let y = x + 1  // Panic in debug mode

// Release mode: Wrapping arithmetic
let x: u8 = 255  
let y = x + 1  // Wraps to 0 in release

// Assertions
assert(condition)  // Always checked
debug_assert(condition)  // Only in debug builds

// Bounds checking
array[index]  // Always checked
unsafe { array.get_unchecked(index) }  // No checks

// Build configurations
#[cfg(debug)]
fn debug_only() {
    print("Debug mode")
}

#[cfg(release)]
fn release_only() {
    print("Release mode")
}
```

### Panic Behavior
```vertex
// Panic causes:
// 1. Explicit panic() call
// 2. Failed assertion
// 3. Integer overflow (debug mode)
// 4. Out of bounds access
// 5. Unwrap on Err

// Panic behavior is configurable:
// Default: Unwind (clean up and terminate thread)
// Alternative: Abort (immediate process termination)

// In vertex.toml:
[profile.release]
panic = "abort"  // or "unwind"

// Panic handler (for no_std environments)
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Custom panic behavior
    // Must never return (-> !)
}

// Catch panics (testing only)
#[test]
fn test_panic() {
    let result = std::panic::catch_unwind(|| {
        panic("test")
    })
    assert(result.is_err())
}
```

### Attribute System
```vertex
// Built-in attributes only (no custom attributes in v1)

// Derive attributes (automatic implementation)
#[derive(Clone, Copy, Debug, Eq)]
struct Point { x: i32, y: i32 }

// Function attributes
#[inline]           // Hint to inline
#[inline(always)]   // Force inline
#[inline(never)]    // Prevent inline
#[cold]            // Rarely called
#[must_use]        // Warn if return value ignored

// Test attributes
#[test]            // Mark as test
#[should_panic]    // Test should panic
#[ignore]          // Skip test

// FFI attributes
#[no_mangle]       // Don't mangle name
#[link(name="c")]  // Link library
#[repr(C)]         // C-compatible layout

// Conditional compilation
#[cfg(debug)]      // Debug only
#[cfg(target_os = "linux")]  // Platform specific
```

### Derive Macros

```vertex
// BUILT-IN DERIVE MACROS
// These are compiler-provided automatic implementations

// Clone - Deep copy
#[derive(Clone)]
struct Point { x: i32, y: i32 }
// Generates:
impl Clone for Point {
    fn clone(&self) -> Self {
        Point { x: self.x.clone(), y: self.y.clone() }
    }
}

// Copy - Bitwise copy (requires Clone)
#[derive(Clone, Copy)]
struct Point { x: i32, y: i32 }
// Requirements: All fields must be Copy
// Makes type Copy - assignment copies instead of moves

// Debug - Debug formatting
#[derive(Debug)]
struct User { name: String, age: i32 }
// Generates Debug::fmt implementation for {:?} formatting

// PartialEq - Equality comparison
#[derive(PartialEq)]
struct Point { x: i32, y: i32 }
// Generates:
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x and self.y == other.y
    }
}

// Eq - Full equality (marker trait, requires PartialEq)
#[derive(PartialEq, Eq)]
struct Point { x: i32, y: i32 }

// PartialOrd - Partial ordering
#[derive(PartialEq, PartialOrd)]
struct Version { major: i32, minor: i32 }
// Generates <, >, <=, >= based on field order

// Ord - Total ordering (requires PartialOrd + Eq)
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Version { major: i32, minor: i32 }

// Default - Default values
#[derive(Default)]
struct Config {
    timeout: i32,  // Uses i32::default() -> 0
    retries: i32,  // Uses i32::default() -> 0
}

// DERIVE REQUIREMENTS

// Clone requirements:
// - All fields must implement Clone
#[derive(Clone)]
struct Container<T> {
    data: T  // T must implement Clone
}
// Generates: impl<T: Clone> Clone for Container<T>

// Copy requirements:
// - Must also derive Clone
// - All fields must be Copy
// - Cannot have Drop implementation
#[derive(Clone, Copy)]
struct Point { x: i32, y: i32 }  // OK: i32 is Copy

// This won't compile:
// #[derive(Copy)]  // ERROR: Copy requires Clone
// #[derive(Clone, Copy)]
// struct Bad { s: String }  // ERROR: String is not Copy

// PartialEq requirements:
// - All fields must implement PartialEq
#[derive(PartialEq)]
struct Wrapper<T> {
    value: T  // T must implement PartialEq
}

// GENERIC TYPE CONSTRAINTS
// Derive automatically adds required bounds

#[derive(Clone, Debug)]
struct Container<T> {
    value: T
}
// Generates:
// impl<T: Clone> Clone for Container<T>
// impl<T: Debug> Debug for Container<T>

// ENUM DERIVES

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Color {
    Red,
    Green,
    Blue
}

#[derive(Clone, Debug, PartialEq)]
enum Option<T> {
    Some(T),
    None
}
// Generates: impl<T: Clone> Clone for Option<T>
//           impl<T: PartialEq> PartialEq for Option<T>

// COMMON PATTERNS

// Value type (copyable, comparable)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
struct Point { x: i32, y: i32 }

// Resource type (moveable, not copyable)
#[derive(Debug)]
struct File { handle: FileHandle }
// No Copy because File owns a resource

// Configuration type
#[derive(Clone, Debug, Default, PartialEq)]
struct Config {
    #[default = 30]  // Hypothetical field default syntax
    timeout: i32,
    #[default = 3]
    retries: i32
}

// LIMITATIONS
// - No custom derive macros in v1.0
// - Only built-in derives listed above
// - Cannot customize derive behavior
// - Field attributes not supported (like #[default])
```

### ABI Guarantees
```vertex
// Default Vertex ABI (unspecified, may change)
fn vertex_function() {}

// C ABI (stable, for FFI)
extern "C" fn c_function() {}

// Type layout
#[repr(C)]     // C-compatible layout
struct CStruct {
    field1: i32,   // Guaranteed layout
    field2: f64
}

#[repr(transparent)]  // Same layout as inner type
struct Wrapper(i32)

// Calling conventions
extern "C"        // C calling convention
extern "system"   // Platform default (Windows: stdcall, others: C)
extern "rust"     // Rust ABI (for Rust interop)

// Size and alignment
// size_of<T>() and align_of<T>() are stable for repr(C) types
```

### Resource Limits
```vertex
// Default limits (platform-dependent)
const DEFAULT_STACK_SIZE: usize = 2 * 1024 * 1024  // 2MB
const MAX_TYPE_SIZE: usize = isize::MAX as usize    // Half address space
const RECURSION_LIMIT: usize = 128                  // For type recursion

// Configurable in vertex.toml
[build]
stack_size = "8MB"
recursion_limit = 256

// Runtime checks
fn recursive(depth: usize) {
    if depth > RECURSION_LIMIT {
        panic("Recursion limit exceeded")
    }
    recursive(depth + 1)
}
```

## 20. Differences from Rust

### Simplifications
1. **No explicit lifetimes** - All inferred
2. **No macro system** - Only built-in derives
3. **No async/await** - Threads and channels only
4. **No const generics** - Runtime generics only
5. **No trait objects** - Static dispatch only
6. **No higher-ranked trait bounds**
7. **Simpler module system** - No crate vs mod distinction
8. **One string type pair** - String (owned) and &str (borrowed)

### Syntax Changes
1. **Logical operators** - `and`/`or`/`not` instead of `&&`/`||`/`!`
2. **No macro syntax** - `println()` not `println!()` 

### Same as Rust
1. Ownership and borrowing
2. Pattern matching
3. Traits and generics
4. Zero-cost abstractions
5. No garbage collection
6. Option and Result types

## 21. Example Programs

### Hello World
```vertex
// hello.vx - The simplest Vertex program
fn main() {
    println("Hello, World!")
}
```

### Comprehensive Example: Word Frequency Counter
```vertex
// word_freq.vx - Count word frequencies in a text file
use std::fs::File
use std::io::Result
use std::collections::HashMap

// Custom error type
struct WordCountError {
    message: String
}

impl Error for WordCountError {
    fn description(&self) -> &str {
        &self.message
    }
}

// Convert IO errors to our error type
impl From<std::io::Error> for WordCountError {
    fn from(error: std::io::Error) -> Self {
        WordCountError {
            message: format("IO error: {}", error.description())
        }
    }
}

// Word frequency counter
struct WordCounter {
    words: HashMap<String, usize>
}

impl WordCounter {
    fn new() -> WordCounter {
        WordCounter {
            words: HashMap::new()
        }
    }
    
    fn add_word(&mut self, word: String) {
        let count = self.words.get(&word).unwrap_or(&0)
        self.words.insert(word, count + 1)
    }
    
    fn process_text(&mut self, text: &str) {
        for word in text.split_whitespace() {
            // Clean and normalize word
            let cleaned = word
                .to_lowercase()
                .trim_matches(|c| not char::is_alphabetic(c))
                .to_string()
            
            if not cleaned.is_empty() {
                self.add_word(cleaned)
            }
        }
    }
    
    fn get_top_words(&self, n: usize) -> Vec<(&String, &usize)> {
        let mut words: Vec<(&String, &usize)> = self.words.iter().collect()
        words.sort_by(|a, b| b.1.cmp(a.1))
        words.truncate(n)
        words
    }
}

// File processing with proper resource management
fn process_file(path: &str) -> Result<WordCounter, WordCountError> {
    let contents = std::fs::read_to_string(path)?
    
    let mut counter = WordCounter::new()
    counter.process_text(&contents)
    
    Ok(counter)
}

// Parallel processing version using threads
fn process_files_parallel(paths: Vec<String>) -> Result<WordCounter, WordCountError> {
    let mut handles = Vec::new()
    
    // Spawn thread for each file
    for path in paths {
        let handle = std::thread::spawn(move || {
            process_file(&path)
        })
        handles.push(handle)
    }
    
    // Collect results
    let mut total_counter = WordCounter::new()
    for handle in handles {
        match handle.join() {
            Ok(Ok(counter)) => {
                // Merge results
                for (word, count) in counter.words {
                    let total = total_counter.words.get(&word).unwrap_or(&0)
                    total_counter.words.insert(word, total + count)
                }
            },
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(WordCountError {
                message: String::from("Thread panicked")
            })
        }
    }
    
    Ok(total_counter)
}

// Main program
fn main() -> Result<(), WordCountError> {
    let args: Vec<String> = std::env::args().collect()
    
    if args.len() < 2 {
        eprint("Usage: {} <file1> [file2 ...]", args[0])
        return Err(WordCountError {
            message: String::from("No input files provided")
        })
    }
    
    // Process files (use parallel version for multiple files)
    let counter = if args.len() == 2 {
        process_file(&args[1])?
    } else {
        let files = args[1..].to_vec()
        process_files_parallel(files)?
    }
    
    // Display results
    println("Top 10 most frequent words:")
    println("----------------------------")
    
    for (word, count) in counter.get_top_words(10) {
        println("{:15} : {}", word, count)
    }
    
    println("----------------------------")
    println("Total unique words: {}", counter.words.len())
    
    Ok(())
}

// Unit tests
#[test]
fn test_word_counter() {
    let mut counter = WordCounter::new()
    counter.process_text("hello world hello")
    
    assert(counter.words.get("hello") == Ok(&2))
    assert(counter.words.get("world") == Ok(&1))
}

#[test] 
fn test_word_cleaning() {
    let mut counter = WordCounter::new()
    counter.process_text("Hello, World! HELLO...")
    
    // Should normalize to lowercase and remove punctuation
    assert(counter.words.get("hello") == Ok(&2))
    assert(counter.words.get("world") == Ok(&1))
}
```

### System Programming Example: Memory Pool Allocator
```vertex
// mempool.vx - Custom memory pool for embedded systems
use std::mem

const POOL_SIZE: usize = 64 * 1024  // 64KB
const MIN_BLOCK_SIZE: usize = 16

// Block header for free list
#[repr(C)]
struct BlockHeader {
    size: usize,
    next: *mut BlockHeader
}

// Memory pool allocator
struct MemoryPool {
    buffer: [u8; POOL_SIZE],
    free_list: *mut BlockHeader
}

impl MemoryPool {
    const fn new() -> MemoryPool {
        MemoryPool {
            buffer: [0; POOL_SIZE],
            free_list: std::ptr::null_mut()
        }
    }
    
    // Initialize the pool (must be called before use)
    fn init(&mut self) {
        unsafe {
            // Create initial free block
            let header = self.buffer.as_mut_ptr() as *mut BlockHeader
            (*header).size = POOL_SIZE - mem::size_of::<BlockHeader>()
            (*header).next = std::ptr::null_mut()
            self.free_list = header
        }
    }
    
    // Allocate memory from pool
    fn alloc(&mut self, size: usize) -> Result<*mut u8, ()> {
        let aligned_size = (size + MIN_BLOCK_SIZE - 1) & !(MIN_BLOCK_SIZE - 1)
        
        unsafe {
            let mut prev: *mut BlockHeader = std::ptr::null_mut()
            let mut current = self.free_list
            
            // First-fit allocation
            while not current.is_null() {
                if (*current).size >= aligned_size {
                    // Found suitable block
                    let remaining = (*current).size - aligned_size
                    
                    if remaining >= MIN_BLOCK_SIZE + mem::size_of::<BlockHeader>() {
                        // Split block
                        let new_block = (current as *mut u8).add(
                            mem::size_of::<BlockHeader>() + aligned_size
                        ) as *mut BlockHeader
                        
                        (*new_block).size = remaining - mem::size_of::<BlockHeader>()
                        (*new_block).next = (*current).next
                        (*current).size = aligned_size
                        (*current).next = new_block
                    }
                    
                    // Remove from free list
                    if prev.is_null() {
                        self.free_list = (*current).next
                    } else {
                        (*prev).next = (*current).next
                    }
                    
                    // Return pointer to usable memory
                    return Ok((current as *mut u8).add(mem::size_of::<BlockHeader>()))
                }
                
                prev = current
                current = (*current).next
            }
            
            Err(())  // Out of memory
        }
    }
    
    // Free memory back to pool
    fn free(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return
        }
        
        unsafe {
            // Get block header
            let header = (ptr as *mut u8).sub(mem::size_of::<BlockHeader>()) 
                as *mut BlockHeader
            
            // Add to free list (sorted by address for coalescing)
            let mut prev: *mut BlockHeader = std::ptr::null_mut()
            let mut current = self.free_list
            
            while not current.is_null() and (current as usize) < (header as usize) {
                prev = current
                current = (*current).next
            }
            
            // Insert block
            (*header).next = current
            if prev.is_null() {
                self.free_list = header
            } else {
                (*prev).next = header
                
                // Try to coalesce with previous block
                if (prev as *mut u8).add(
                    mem::size_of::<BlockHeader>() + (*prev).size
                ) == header as *mut u8 {
                    (*prev).size += mem::size_of::<BlockHeader>() + (*header).size
                    (*prev).next = (*header).next
                    header = prev
                }
            }
            
            // Try to coalesce with next block
            if not current.is_null() and 
               (header as *mut u8).add(
                   mem::size_of::<BlockHeader>() + (*header).size
               ) == current as *mut u8 {
                (*header).size += mem::size_of::<BlockHeader>() + (*current).size
                (*header).next = (*current).next
            }
        }
    }
}

// Global allocator for no_std environment
static mut POOL: MemoryPool = MemoryPool::new()

// Custom allocator implementation
pub unsafe fn custom_alloc(size: usize) -> *mut u8 {
    POOL.alloc(size).unwrap_or(std::ptr::null_mut())
}

pub unsafe fn custom_free(ptr: *mut u8) {
    POOL.free(ptr)
}

// Example usage
fn main() {
    unsafe {
        POOL.init()
        
        // Allocate some memory
        let p1 = custom_alloc(100)
        let p2 = custom_alloc(200)
        let p3 = custom_alloc(150)
        
        // Use memory
        if not p1.is_null() {
            *p1 = 42
            println("Allocated and wrote to memory")
        }
        
        // Free in different order
        custom_free(p2)
        custom_free(p1)
        custom_free(p3)
        
        println("Memory pool test complete")
    }
}

#[test]
fn test_pool_allocation() {
    let mut pool = MemoryPool::new()
    pool.init()
    
    let p1 = pool.alloc(64).unwrap()
    assert(not p1.is_null())
    
    pool.free(p1)
}
```

## 22. Formal Grammar

```ebnf
// Vertex Formal Grammar in Extended Backus-Naur Form (EBNF)

// ============ Lexical Structure ============

// Identifiers
identifier = letter { letter | digit | "_" }
letter = "a".."z" | "A".."Z"
digit = "0".."9"

// Keywords (reserved)
keyword = "and" | "break" | "const" | "continue" | "defer" | "else" 
        | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl"
        | "in" | "let" | "loop" | "match" | "mod" | "mut" | "not"
        | "or" | "pub" | "return" | "self" | "Self" | "static" 
        | "struct" | "trait" | "true" | "type" | "unsafe" | "use"
        | "where" | "while"

// Literals
literal = integer_literal | float_literal | string_literal 
        | char_literal | bool_literal

integer_literal = decimal_literal | hex_literal | binary_literal
decimal_literal = digit { digit | "_" }
hex_literal = "0x" hex_digit { hex_digit | "_" }
binary_literal = "0b" binary_digit { binary_digit | "_" }
hex_digit = digit | "a".."f" | "A".."F"
binary_digit = "0" | "1"

float_literal = digit { digit } "." digit { digit } [ exponent ]
exponent = ("e" | "E") ["+" | "-"] digit { digit }

string_literal = '"' { string_char | escape_sequence } '"'
              | 'r' raw_string
raw_string = '"' { any_char } '"' | '#' '"' { any_char } '"' '#'
char_literal = "'" ( char | escape_sequence ) "'"
escape_sequence = "\\" ( "n" | "r" | "t" | "\\" | "0" | '"' | "'" )

bool_literal = "true" | "false"

// Comments
comment = line_comment | block_comment
line_comment = "//" { non_newline } newline
block_comment = "/*" { any_char } "*/"
doc_comment = "///" { non_newline } newline
            | "//!" { non_newline } newline

// ============ Types ============

type = primitive_type | compound_type | user_type | type_path

primitive_type = "bool" | "char" 
               | "i8" | "i16" | "i32" | "i64" | "isize"
               | "u8" | "u16" | "u32" | "u64" | "usize"
               | "f32" | "f64" | "str" | "()"

compound_type = array_type | slice_type | tuple_type | pointer_type
              | reference_type | function_type

array_type = "[" type ";" expression "]"
slice_type = "[" type "]"
tuple_type = "(" [ type { "," type } ] ")"
pointer_type = "*" ("const" | "mut") type
reference_type = "&" ["mut"] type
function_type = "fn" "(" [ type { "," type } ] ")" [ "->" type ]

type_path = ["::"] path_segment { "::" path_segment }
path_segment = identifier [ generic_args ]
generic_args = "<" type { "," type } ">"

// ============ Items ============

program = { item }

item = function | struct_def | enum_def | trait_def 
     | impl_block | type_alias | use_decl | const_decl
     | static_decl | mod_decl | extern_block

// Function definition
function = [ visibility ] "fn" identifier [ generics ] 
          "(" [ parameters ] ")" [ "->" type ] [ where_clause ]
          block

visibility = "pub" [ "(" visibility_scope ")" ]
visibility_scope = "crate" | "super" | "in" path

generics = "<" generic_param { "," generic_param } ">"
generic_param = identifier [ ":" bounds ]
bounds = bound { "+" bound }
bound = trait_bound | lifetime_bound

parameters = parameter { "," parameter }
parameter = pattern ":" type

where_clause = "where" where_predicate { "," where_predicate }
where_predicate = type ":" bounds

// Struct definition
struct_def = [ visibility ] "struct" identifier [ generics ] 
            ( struct_body | ";" )
struct_body = "{" [ struct_field { "," struct_field } [","] ] "}"
struct_field = [ visibility ] identifier ":" type

// Enum definition  
enum_def = [ visibility ] "enum" identifier [ generics ]
          "{" enum_variant { "," enum_variant } [","] "}"
enum_variant = identifier [ variant_body ]
variant_body = "(" [ type { "," type } ] ")"
             | "{" [ struct_field { "," struct_field } ] "}"

// Trait definition
trait_def = [ visibility ] "trait" identifier [ generics ]
           [ ":" bounds ] "{" { trait_item } "}"
trait_item = trait_method | trait_type
trait_method = [ visibility ] "fn" identifier [ generics ]
              "(" [ self_param [ "," ] [ parameters ] ] ")"
              [ "->" type ] [ ";" | block ]
self_param = ["&" ["mut"]] "self"

// Implementation block
impl_block = "impl" [ generics ] type_path [ "for" type ] 
            [ where_clause ] "{" { impl_item } "}"
impl_item = [ visibility ] function

// Type alias
type_alias = [ visibility ] "type" identifier [ generics ] 
            "=" type ";"

// Use declaration
use_decl = [ visibility ] "use" use_tree ";"
use_tree = [ "::" ] path [ "::" ( "*" | "{" use_list "}" | "as" identifier ) ]
use_list = use_tree { "," use_tree }

// Const and static
const_decl = [ visibility ] "const" identifier ":" type 
            "=" expression ";"
static_decl = [ visibility ] "static" ["mut"] identifier ":" type
             "=" expression ";"

// Module declaration
mod_decl = [ visibility ] "mod" identifier ( ";" | block )

// Extern block
extern_block = "extern" [ string_literal ] "{" { extern_item } "}"
extern_item = function_signature ";"

// ============ Statements ============

statement = let_stmt | expression_stmt | item

let_stmt = "let" ["mut"] pattern [ ":" type ] [ "=" expression ] ";"
expression_stmt = expression [ ";" ]

block = "{" { statement } [ expression ] "}"

// ============ Expressions ============

expression = literal_expr | path_expr | operator_expr 
           | call_expr | method_expr | field_expr | index_expr
           | tuple_expr | array_expr | struct_expr
           | block_expr | if_expr | match_expr | loop_expr
           | closure_expr | return_expr | break_expr
           | continue_expr | unsafe_expr

literal_expr = literal
path_expr = type_path

// Operators (precedence handled separately)
operator_expr = prefix_expr | infix_expr | postfix_expr
prefix_expr = prefix_op expression
prefix_op = "-" | "not" | "&" | "&mut" | "*"

infix_expr = expression infix_op expression
infix_op = "+" | "-" | "*" | "/" | "%" 
         | "and" | "or" | "&" | "|" | "^"
         | "<<" | ">>" | "==" | "!=" 
         | "<" | ">" | "<=" | ">="
         | "=" | "+=" | "-=" | "*=" | "/=" | "%="

postfix_expr = expression postfix_op
postfix_op = "?" | "as" type

// Function and method calls
call_expr = expression "(" [ arguments ] ")"
method_expr = expression "." identifier "(" [ arguments ] ")"
arguments = expression { "," expression }

// Field and index access
field_expr = expression "." identifier
index_expr = expression "[" expression "]"

// Compound expressions
tuple_expr = "(" [ expression { "," expression } ] ")"
array_expr = "[" array_elements "]"
array_elements = expression { "," expression }
                | expression ";" expression

struct_expr = path "{" [ field_init { "," field_init } ] "}"
field_init = identifier [ ":" expression ]

// Control flow
block_expr = block

if_expr = "if" expression block [ "else" ( if_expr | block ) ]

match_expr = "match" expression "{" match_arm { "," match_arm } "}"
match_arm = pattern [ "if" expression ] "=>" ( expression | block )

loop_expr = "loop" block
          | "while" expression block
          | "for" pattern "in" expression block

// Closures
closure_expr = [ "move" ] "|" [ parameters ] "|" 
              [ "->" type ] ( expression | block )

// Control flow expressions
return_expr = "return" [ expression ]
break_expr = "break" [ expression ]
continue_expr = "continue"

// Unsafe
unsafe_expr = "unsafe" block

// ============ Patterns ============

pattern = literal_pattern | identifier_pattern | wildcard_pattern
        | tuple_pattern | struct_pattern | enum_pattern
        | slice_pattern | reference_pattern | range_pattern
        | binding_pattern

literal_pattern = literal
identifier_pattern = identifier
wildcard_pattern = "_"

tuple_pattern = "(" [ pattern { "," pattern } ] ")"
struct_pattern = path "{" [ field_pattern { "," field_pattern } ] [ ".." ] "}"
field_pattern = identifier [ ":" pattern ]

enum_pattern = path [ "(" [ pattern { "," pattern } ] ")" 
                    | "{" [ field_pattern { "," field_pattern } ] "}" ]

slice_pattern = "[" [ pattern { "," pattern } ] "]"
reference_pattern = "&" ["mut"] pattern
range_pattern = expression ".." [ "=" ] expression

binding_pattern = identifier "@" pattern
                | "ref" ["mut"] identifier

// ============ Attributes ============

attribute = "#" "[" attribute_content "]"
attribute_content = identifier [ "(" token_tree ")" ]
token_tree = { token }

// Common attributes
derive_attr = "#" "[" "derive" "(" identifier { "," identifier } ")" "]"
cfg_attr = "#" "[" "cfg" "(" cfg_predicate ")" "]"

// ============ Operator Precedence ============
// (Highest to Lowest)
// 1. Field access, method calls: . ()
// 2. Unary: - not * & &mut
// 3. Cast: as
// 4. Multiplicative: * / %
// 5. Additive: + -
// 6. Shift: << >>
// 7. Relational: < > <= >=
// 8. Equality: == !=
// 9. Bitwise AND: &
// 10. Bitwise XOR: ^
// 11. Bitwise OR: |
// 12. Logical AND: and
// 13. Logical OR: or
// 14. Range: .. ..=
// 15. Assignment: = += -= *= /= %=
// 16. Return, Break: return break
```

## 23. Out of Scope

This language specification does NOT cover:

- **Package Registry**: How packages are published, versioned, or distributed
- **Build System Internals**: Implementation details of the build tool
- **Compiler Optimizations**: Which optimizations are performed and when
- **Platform APIs**: OS-specific functionality beyond standard library
- **IDE Features**: Language server protocol extensions, debugging protocol
- **Deployment**: How to distribute Vertex applications
- **Ecosystem Policies**: Code of conduct, contribution guidelines
- **Performance Guarantees**: Specific performance characteristics
- **Binary Compatibility**: ABI stability across versions

These topics are covered in separate documentation:
- Package Manager Guide
- Platform Support Documentation  
- Compiler Implementation Guide
- Standard Library API Reference

## 24. Stability Guarantees

### Stable (Will Not Change)
- **Language Syntax**: All syntax in this specification
- **Core Types**: Primitive types, Result, Option, String, Vec
- **Ownership Rules**: Borrow checker semantics
- **Trait System**: How traits work
- **Pattern Matching**: Match expressions and patterns
- **Module System**: How modules and visibility work

### May Expand (Backward Compatible)
- **Standard Library**: New functions and types may be added
- **Attributes**: New built-in attributes may be added
- **Platform Support**: New platforms may be added
- **Derive Macros**: New built-in derives may be added

### Not Stable
- **Compiler Internals**: May change between versions
- **Error Message Format**: May improve  
- **Compilation Speed**: No guarantees
- **Binary Size**: May vary with compiler versions
- **Memory Layout**: Unless explicitly specified with `#[repr(C)]`

## 25. Future Considerations

Features being considered for Vertex 2.0:

### Async/Await
```vertex
// Potential future syntax
async fn fetch_data() -> Result<String, Error> {
    let response = await http_get(url)?
    Ok(response.body)
}
```

### Const Generics
```vertex
// Arrays with const generic sizes
fn process<const N: usize>(data: [u8; N]) {
    // Process fixed-size array
}
```

### Custom Derive Macros
```vertex
// User-defined derive macros
#[derive(MySerialize)]
struct Data { }
```

### Trait Objects
```vertex
// Dynamic dispatch
let drawable: Box<dyn Draw> = Box::new(circle)
```

### Explicit Lifetimes (When Needed)
```vertex
// For complex cases only
fn complex<'a, 'b>(x: &'a str, y: &'b str) -> &'a str
```

These features will only be added if they can maintain Vertex's simplicity goals.

## 26. Conformance Requirements

An implementation is considered "conforming" if it:

### Must Accept
1. All syntactically valid programs according to the grammar
2. All programs that follow the type rules
3. All programs that satisfy the borrow checker rules
4. All safe programs that don't violate memory safety

### Must Reject
1. Programs with syntax errors
2. Programs with type errors
3. Programs that violate borrowing rules
4. Programs that would cause memory unsafety (in safe code)

### Must Provide
1. All types listed in the standard library section
2. All functions and methods specified for those types
3. The same semantics for all operations
4. Compatible error types and Result/Option types

### Must Match Semantics
1. Evaluation order as specified
2. Drop order as specified  
3. Type inference rules as specified
4. Method resolution order as specified
5. Pattern matching exhaustiveness checking

### Implementation Freedom
Implementations may differ in:
1. Error message wording (but not meaning)
2. Optimization strategies
3. Compilation speed
4. Debug information format
5. Internal representations (except `#[repr(C)]`)

### Test Suite
A conformance test suite will be provided to verify implementation correctness. Passing the test suite is required for an implementation to be considered conforming.

## Conclusion

This specification defines Vertex 1.0, a systems programming language that provides memory safety without garbage collection through ownership and borrowing. By focusing on essential features and removing complexity, Vertex achieves its goal of being significantly simpler than Rust while maintaining core safety guarantees.

The language is designed to be:
- **Learnable** in weeks rather than months
- **Suitable** for systems programming, embedded development, and performance-critical applications  
- **Interoperable** with existing C codebases
- **Safe** by default with explicit unsafe escape hatches
- **Predictable** in both performance and behavior

This specification provides sufficient detail for implementing a conforming Vertex compiler. Additional documentation covers the standard library API, platform-specific behavior, and ecosystem tooling.

For the latest updates and reference implementation, see: https://vertex-lang.org

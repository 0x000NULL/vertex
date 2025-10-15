# Vertex Core Library API Specification

**Version**: 1.0.0
**Status**: API Reference
**Date**: December 2024

## Executive Summary

This document provides detailed API specifications for all core types and standard library modules in Vertex v1.0. All items listed here are available in the standard library and most are included in the prelude.

## Table of Contents

1. [Core Types](#1-core-types)
2. [Result and Option](#2-result-and-option)
3. [Collections](#3-collections)
4. [Smart Pointers](#4-smart-pointers)
5. [String Types](#5-string-types)
6. [I/O Module](#6-io-module)
7. [File System](#7-file-system)
8. [Iterators](#8-iterators)
9. [Traits](#9-traits)
10. [Synchronization](#10-synchronization)
11. [Memory Module](#11-memory-module)
12. [Conversion Traits](#12-conversion-traits)

---

## 1. Core Types

### 1.1 Primitive Types

```vertex
// Integer types (signed)
type i8   // -128 to 127
type i16  // -32,768 to 32,767
type i32  // -2,147,483,648 to 2,147,483,647
type i64  // -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
type isize // Pointer-sized signed integer

// Integer types (unsigned)
type u8   // 0 to 255
type u16  // 0 to 65,535
type u32  // 0 to 4,294,967,295
type u64  // 0 to 18,446,744,073,709,551,615
type usize // Pointer-sized unsigned integer

// Floating point types
type f32  // IEEE 754 single precision
type f64  // IEEE 754 double precision

// Boolean type
type bool // true or false

// Character type
type char // Unicode scalar value (4 bytes)

// Unit type
type () // Zero-sized type
```

### 1.2 Integer Methods

```vertex
impl i32 {
    // Constants
    const MIN: i32 = -2147483648
    const MAX: i32 = 2147483647

    // Checked arithmetic (returns Result)
    fn checked_add(self, rhs: i32) -> Result<i32, ()>
    fn checked_sub(self, rhs: i32) -> Result<i32, ()>
    fn checked_mul(self, rhs: i32) -> Result<i32, ()>
    fn checked_div(self, rhs: i32) -> Result<i32, ()>
    fn checked_rem(self, rhs: i32) -> Result<i32, ()>
    fn checked_neg(self) -> Result<i32, ()>
    fn checked_shl(self, rhs: u32) -> Result<i32, ()>
    fn checked_shr(self, rhs: u32) -> Result<i32, ()>

    // Saturating arithmetic
    fn saturating_add(self, rhs: i32) -> i32
    fn saturating_sub(self, rhs: i32) -> i32
    fn saturating_mul(self, rhs: i32) -> i32

    // Wrapping arithmetic (always wraps)
    fn wrapping_add(self, rhs: i32) -> i32
    fn wrapping_sub(self, rhs: i32) -> i32
    fn wrapping_mul(self, rhs: i32) -> i32
    fn wrapping_div(self, rhs: i32) -> i32
    fn wrapping_rem(self, rhs: i32) -> i32
    fn wrapping_neg(self) -> i32
    fn wrapping_shl(self, rhs: u32) -> i32
    fn wrapping_shr(self, rhs: u32) -> i32

    // Absolute value
    fn abs(self) -> i32

    // Power
    fn pow(self, exp: u32) -> i32

    // Conversions
    fn to_string(self) -> String
    fn from_str(s: &str) -> Result<i32, ParseIntError>
}

// Similar implementations for i8, i16, i64, isize
// u8, u16, u32, u64, usize (without checked_neg)
```

### 1.3 Float Methods

```vertex
impl f64 {
    // Constants
    const INFINITY: f64
    const NEG_INFINITY: f64
    const NAN: f64
    const MIN: f64
    const MAX: f64
    const EPSILON: f64
    const PI: f64
    const E: f64

    // Classification
    fn is_nan(self) -> bool
    fn is_infinite(self) -> bool
    fn is_finite(self) -> bool
    fn is_normal(self) -> bool
    fn is_sign_positive(self) -> bool
    fn is_sign_negative(self) -> bool

    // Rounding
    fn floor(self) -> f64
    fn ceil(self) -> f64
    fn round(self) -> f64
    fn trunc(self) -> f64
    fn fract(self) -> f64

    // Arithmetic
    fn abs(self) -> f64
    fn sqrt(self) -> f64
    fn cbrt(self) -> f64
    fn powf(self, n: f64) -> f64
    fn powi(self, n: i32) -> f64
    fn exp(self) -> f64
    fn exp2(self) -> f64
    fn ln(self) -> f64
    fn log2(self) -> f64
    fn log10(self) -> f64

    // Trigonometry
    fn sin(self) -> f64
    fn cos(self) -> f64
    fn tan(self) -> f64
    fn asin(self) -> f64
    fn acos(self) -> f64
    fn atan(self) -> f64
    fn atan2(self, other: f64) -> f64

    // Hyperbolic
    fn sinh(self) -> f64
    fn cosh(self) -> f64
    fn tanh(self) -> f64

    // Comparison
    fn max(self, other: f64) -> f64
    fn min(self, other: f64) -> f64

    // Conversion
    fn to_string(self) -> String
    fn from_str(s: &str) -> Result<f64, ParseFloatError>
}

// Similar for f32
```

### 1.4 Boolean Methods

```vertex
impl bool {
    // Logical operations (provided by operators)
    // a and b, a or b, not a

    // Conversion
    fn to_string(self) -> String
}
```

### 1.5 Character Methods

```vertex
impl char {
    // Classification
    fn is_alphabetic(self) -> bool
    fn is_numeric(self) -> bool
    fn is_alphanumeric(self) -> bool
    fn is_lowercase(self) -> bool
    fn is_uppercase(self) -> bool
    fn is_whitespace(self) -> bool
    fn is_ascii(self) -> bool
    fn is_ascii_digit(self) -> bool
    fn is_ascii_hexdigit(self) -> bool

    // Case conversion
    fn to_lowercase(self) -> char
    fn to_uppercase(self) -> char

    // Conversions
    fn to_digit(self, radix: u32) -> Result<u32, ()>
    fn from_digit(num: u32, radix: u32) -> Result<char, ()>
    fn from_u32(i: u32) -> Result<char, ()>
}
```

### 1.6 Range Types

**Location**: `std::ops`

Ranges represent intervals and are used for iteration and slicing. They are created using range syntax literals.

```vertex
use std::ops::{Range, RangeInclusive, RangeFrom, RangeTo, RangeFull}

// Range<T> - Exclusive end (start..end)
struct Range<T> {
    start: T,
    end: T
}

impl<T> Range<T> {
    fn contains(&self, item: &T) -> bool
        where T: PartialOrd
    fn is_empty(&self) -> bool
        where T: PartialOrd
}

// Iterator implementation for numeric types
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

// Similar implementations for all integer types

// RangeInclusive<T> - Inclusive end (start..=end)
struct RangeInclusive<T> {
    start: T,
    end: T
}

impl<T> RangeInclusive<T> {
    fn contains(&self, item: &T) -> bool
        where T: PartialOrd
    fn is_empty(&self) -> bool
        where T: PartialOrd
    fn start(&self) -> &T
    fn end(&self) -> &T
}

impl Iterator for RangeInclusive<i32> {
    type Item = i32
    fn next(&mut self) -> Result<i32, ()>
}

// RangeFrom<T> - Unbounded end (start..)
struct RangeFrom<T> {
    start: T
}

impl<T> RangeFrom<T> {
    fn contains(&self, item: &T) -> bool
        where T: PartialOrd
}

impl Iterator for RangeFrom<i32> {
    type Item = i32
    fn next(&mut self) -> Result<i32, ()>  // Infinite iterator
}

// RangeTo<T> - Unbounded start (..end)
struct RangeTo<T> {
    end: T
}

impl<T> RangeTo<T> {
    fn contains(&self, item: &T) -> bool
        where T: PartialOrd
}

// Note: RangeTo does NOT implement Iterator (no starting point)

// RangeFull - Unbounded both (..)
struct RangeFull

impl RangeFull {
    fn contains<T>(&self, item: &T) -> bool {
        true  // Contains everything
    }
}

// Note: RangeFull does NOT implement Iterator (no starting point)
```

**Range Literal Syntax**:

```vertex
// Create ranges using syntax literals
let r1 = 0..10              // Range<i32> { start: 0, end: 10 }
let r2 = 0..=10             // RangeInclusive<i32> { start: 0, end: 10 }
let r3 = 5..                // RangeFrom<i32> { start: 5 }
let r4 = ..10               // RangeTo<i32> { end: 10 }
let r5 = ..                 // RangeFull
```

**Iteration Examples**:

```vertex
// Exclusive end: 0..10 iterates 0 through 9
for i in 0..10 {
    print("{}", i)  // Prints: 0 1 2 3 4 5 6 7 8 9
}

// Inclusive end: 0..=10 iterates 0 through 10
for i in 0..=10 {
    print("{}", i)  // Prints: 0 1 2 3 4 5 6 7 8 9 10
}

// Unbounded end: infinite iterator
for i in 5.. {
    if i > 10 { break }
    print("{}", i)  // Prints: 5 6 7 8 9 10
}

// Collect into Vec
let numbers: Vec<i32> = (1..6).collect()  // vec![1, 2, 3, 4, 5]
```

**Slice Indexing**:

Ranges are primarily used for indexing slices and strings:

```vertex
let arr = [1, 2, 3, 4, 5]

let slice1 = &arr[1..3]     // [2, 3] (exclusive end)
let slice2 = &arr[1..=3]    // [2, 3, 4] (inclusive end)
let slice3 = &arr[2..]      // [3, 4, 5] (from index 2 to end)
let slice4 = &arr[..3]      // [1, 2, 3] (from start to index 3)
let slice5 = &arr[..]       // [1, 2, 3, 4, 5] (full slice)

// String slicing (by byte, not character!)
let s = "hello world"
let slice = &s[0..5]        // "hello"
let slice = &s[6..]         // "world"

// Out of bounds panics
let vec = vec![1, 2, 3]
// let bad = &vec[1..5]     // ❌ PANIC: index out of bounds
```

**Range Bounds Checking**:

```vertex
// Slicing checks bounds at runtime and panics if invalid
let vec = vec![1, 2, 3]
let slice = &vec[1..10]  // Panics: end index 10 > length 3

// Safe alternative: use get() method
match vec.get(1..10) {
    Some(slice) => println("{:?}", slice),
    None => println("Out of bounds")  // This branch taken
}
```

**Contains Method**:

```vertex
let range = 1..10
println(range.contains(&5))   // true
println(range.contains(&10))  // false (exclusive end)
println(range.contains(&0))   // false

let range_inc = 1..=10
println(range_inc.contains(&10))  // true (inclusive end)

let range_from = 5..
println(range_from.contains(&100))  // true
println(range_from.contains(&3))    // false
```

**Type Summary**:

| Syntax | Type | Start | End | Iterable | Use Cases |
|--------|------|-------|-----|----------|-----------|
| `a..b` | `Range<T>` | Inclusive | Exclusive | ✓ | Most iteration and slicing |
| `a..=b` | `RangeInclusive<T>` | Inclusive | Inclusive | ✓ | When end value needed |
| `a..` | `RangeFrom<T>` | Inclusive | Unbounded | ✓ | Infinite iteration |
| `..b` | `RangeTo<T>` | Unbounded | Exclusive | ✗ | Slicing from start |
| `..` | `RangeFull` | Unbounded | Unbounded | ✗ | Full slice (`&arr[..]`) |

---

## 2. Result and Option

### 2.1 Result<T, E>

```vertex
enum Result<T, E> {
    Ok(T),
    Err(E)
}

impl<T, E> Result<T, E> {
    // Querying
    fn is_ok(&self) -> bool
    fn is_err(&self) -> bool

    // Extracting (panics on wrong variant)
    fn unwrap(self) -> T
    fn unwrap_err(self) -> E
    fn expect(self, msg: &str) -> T
    fn expect_err(self, msg: &str) -> E

    // Extracting with defaults
    fn unwrap_or(self, default: T) -> T
    fn unwrap_or_else<F>(self, f: F) -> T
        where F: FnOnce(E) -> T
    fn unwrap_or_default(self) -> T
        where T: Default

    // Transforming
    fn map<U, F>(self, f: F) -> Result<U, E>
        where F: FnOnce(T) -> U

    fn map_err<F, O>(self, f: O) -> Result<T, F>
        where O: FnOnce(E) -> F

    fn and<U>(self, res: Result<U, E>) -> Result<U, E>

    fn and_then<U, F>(self, f: F) -> Result<U, E>
        where F: FnOnce(T) -> Result<U, E>

    fn or<F>(self, res: Result<T, F>) -> Result<T, F>

    fn or_else<F, O>(self, f: O) -> Result<T, F>
        where O: FnOnce(E) -> Result<T, F>

    // Converting to Option
    fn ok(self) -> Option<T>
    fn err(self) -> Option<E>

    // Borrowing
    fn as_ref(&self) -> Result<&T, &E>
    fn as_mut(&mut self) -> Result<&mut T, &mut E>
}

// Prelude exports
fn Ok<T, E>(value: T) -> Result<T, E>
fn Err<T, E>(error: E) -> Result<T, E>
```

### 2.2 Option<T>

```vertex
enum Option<T> {
    Some(T),
    None
}

impl<T> Option<T> {
    // Querying
    fn is_some(&self) -> bool
    fn is_none(&self) -> bool

    // Extracting (panics on None)
    fn unwrap(self) -> T
    fn expect(self, msg: &str) -> T

    // Extracting with defaults
    fn unwrap_or(self, default: T) -> T
    fn unwrap_or_else<F>(self, f: F) -> T
        where F: FnOnce() -> T
    fn unwrap_or_default(self) -> T
        where T: Default

    // Transforming
    fn map<U, F>(self, f: F) -> Option<U>
        where F: FnOnce(T) -> U

    fn map_or<U, F>(self, default: U, f: F) -> U
        where F: FnOnce(T) -> U

    fn map_or_else<U, D, F>(self, default: D, f: F) -> U
        where D: FnOnce() -> U,
              F: FnOnce(T) -> U

    fn and<U>(self, optb: Option<U>) -> Option<U>

    fn and_then<U, F>(self, f: F) -> Option<U>
        where F: FnOnce(T) -> Option<U>

    fn or(self, optb: Option<T>) -> Option<T>

    fn or_else<F>(self, f: F) -> Option<T>
        where F: FnOnce() -> Option<T>

    fn filter<P>(self, predicate: P) -> Option<T>
        where P: FnOnce(&T) -> bool

    // Converting to Result
    fn ok_or<E>(self, err: E) -> Result<T, E>
    fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
        where F: FnOnce() -> E

    // Borrowing
    fn as_ref(&self) -> Option<&T>
    fn as_mut(&mut self) -> Option<&mut T>

    // Taking ownership
    fn take(&mut self) -> Option<T>
    fn replace(&mut self, value: T) -> Option<T>
}

// Prelude exports
fn Some<T>(value: T) -> Option<T>
const None: Option<T>
```

### 2.3 When to Use Result vs Option

**Design Principle**: The choice between `Result` and `Option` depends on the semantic meaning of the absence of a value:

**Use `Option<T>` when**:
- A value may or may not exist, with no error condition
- The absence of a value is a normal, expected state
- There's no additional information about *why* the value is absent

**Examples**:
```vertex
fn pop(&mut self) -> Option<T>           // Vec may be empty (normal state)
fn last(self) -> Option<Self::Item>      // Iterator may be empty
fn find<P>(&mut self, p: P) -> Option<T> // Item may not be found
fn get(&self, index: usize) -> Option<&T> // Index may be out of bounds
```

**Use `Result<T, E>` when**:
- An operation can fail with error information
- You need to distinguish between different failure modes
- Callers need context about what went wrong

**Examples**:
```vertex
fn parse(&self) -> Result<i32, ParseError>           // Can fail with specific error
fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError>  // I/O can fail
fn open(path: &str) -> Result<File, IoError>         // File may not exist (error)
```

**Special Case: `Result<T, ()>`**:
- Used when an operation can fail, but there's only one way it can fail
- The operation itself indicates the failure mode (no additional context needed)
- This is a middle ground between `Option` and full `Result<T, E>`

**Examples**:
```vertex
fn checked_add(self, rhs: i32) -> Result<i32, ()>    // Can overflow (only one error)
fn next(&mut self) -> Result<Self::Item, ()>         // Iterator exhausted (only one way)
```

**Why Iterator::next Returns Result**:
Iterator's `next()` method returns `Result<Self::Item, ()>` rather than `Option<Self::Item>` as a design choice to maintain consistency with Vertex's error handling philosophy. While the for-loop desugaring *could* use Option, Result provides:
1. Consistency with other operations that can "fail" (including reaching end-of-iteration)
2. A uniform pattern with the `?` operator
3. Clear distinction between "operation completed" (Err) vs "value may not exist" (Option in search methods)

Note that iterator methods like `find()`, `last()`, `nth()` correctly return `Option<T>` because they represent *search* operations where a value may not exist, which is semantically different from iteration completion.

**Consistency Guidelines**:
1. Container operations (pop, get, first, last) → `Option<T>`
2. Parsing and I/O operations → `Result<T, E>` with specific error type
3. Arithmetic overflow checks → `Result<T, ()>`
4. Search operations → `Option<T>`
5. Iteration control (next only) → `Result<T, ()>`

---

## 3. Collections

### 3.1 Vec<T>

```vertex
struct Vec<T> {
    // Private fields
}

impl<T> Vec<T> {
    // Construction
    fn new() -> Vec<T>
    fn with_capacity(capacity: usize) -> Vec<T>
    fn from_elem(elem: T, n: usize) -> Vec<T>
        where T: Clone

    // Capacity
    fn capacity(&self) -> usize
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn reserve(&mut self, additional: usize)
    fn reserve_exact(&mut self, additional: usize)
    fn shrink_to_fit(&mut self)
    fn truncate(&mut self, len: usize)

    // Adding elements
    fn push(&mut self, value: T)
    fn pop(&mut self) -> Option<T>
    fn insert(&mut self, index: usize, element: T)
    fn remove(&mut self, index: usize) -> T
    fn swap_remove(&mut self, index: usize) -> T
    fn append(&mut self, other: &mut Vec<T>)
    fn clear(&mut self)

    // Accessing elements
    fn get(&self, index: usize) -> Option<&T>
    fn get_mut(&mut self, index: usize) -> Option<&mut T>
    fn first(&self) -> Option<&T>
    fn last(&self) -> Option<&T>
    fn first_mut(&mut self) -> Option<&mut T>
    fn last_mut(&mut self) -> Option<&mut T>

    // Slicing
    fn as_slice(&self) -> &[T]
    fn as_mut_slice(&mut self) -> &mut [T]

    // Iteration
    fn iter(&self) -> slice::Iter<T>
    fn iter_mut(&mut self) -> slice::IterMut<T>

    // Searching
    fn contains(&self, x: &T) -> bool
        where T: PartialEq
    fn binary_search(&self, x: &T) -> Result<usize, usize>
        where T: Ord
    fn binary_search_by<F>(&self, f: F) -> Result<usize, usize>
        where F: FnMut(&T) -> Ordering

    // Sorting
    fn sort(&mut self)
        where T: Ord
    fn sort_by<F>(&mut self, compare: F)
        where F: FnMut(&T, &T) -> Ordering
    fn reverse(&mut self)

    // Deduplication
    fn dedup(&mut self)
        where T: PartialEq

    // Splitting
    fn split_off(&mut self, at: usize) -> Vec<T>

    // Resizing
    fn resize(&mut self, new_len: usize, value: T)
        where T: Clone
    fn resize_with<F>(&mut self, new_len: usize, f: F)
        where F: FnMut() -> T

    // Extending
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item = T>
}

// Indexing
impl<T> Index<usize> for Vec<T> {
    type Output = T
    fn index(&self, index: usize) -> &T
}

impl<T> IndexMut<usize> for Vec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T
}

// Iteration
impl<T> IntoIterator for Vec<T> {
    type Item = T
    type IntoIter = vec::IntoIter<T>
    fn into_iter(self) -> vec::IntoIter<T>
}

impl<T> IntoIterator for &Vec<T> {
    type Item = &T
    type IntoIter = slice::Iter<T>
    fn into_iter(self) -> slice::Iter<T>
}

impl<T> IntoIterator for &mut Vec<T> {
    type Item = &mut T
    type IntoIter = slice::IterMut<T>
    fn into_iter(self) -> slice::IterMut<T>
}

// Built-in syntax (NOT macros - hardcoded in compiler parser)
// Despite the `!`, these are NOT macros. Vertex has no macro system.
fn vec<T>(elements: ...T) -> Vec<T>  // vec![1, 2, 3]
fn vec_repeat<T>(value: T, count: usize) -> Vec<T>  // vec![0; 100]
    where T: Clone

// IMPORTANT: vec! is built-in compiler syntax, NOT a user-extensible macro
```

### 3.2 Slice [T]

```vertex
// Slices are unsized, always used as &[T] or &mut [T]

impl<T> [T] {
    // Length
    fn len(&self) -> usize
    fn is_empty(&self) -> bool

    // Accessing
    fn first(&self) -> Option<&T>
    fn last(&self) -> Option<&T>
    fn get(&self, index: usize) -> Option<&T>
    fn get_mut(&mut self, index: usize) -> Option<&mut T>

    // Slicing
    fn split_at(&self, mid: usize) -> (&[T], &[T])
    fn split_at_mut(&mut self, mid: usize) -> (&mut [T], &mut [T])

    // Iteration
    fn iter(&self) -> slice::Iter<T>
    fn iter_mut(&mut self) -> slice::IterMut<T>
    fn windows(&self, size: usize) -> Windows<T>
    fn chunks(&self, chunk_size: usize) -> Chunks<T>
    fn chunks_mut(&mut self, chunk_size: usize) -> ChunksMut<T>

    // Searching
    fn contains(&self, x: &T) -> bool
        where T: PartialEq
    fn starts_with(&self, needle: &[T]) -> bool
        where T: PartialEq
    fn ends_with(&self, needle: &[T]) -> bool
        where T: PartialEq
    fn binary_search(&self, x: &T) -> Result<usize, usize>
        where T: Ord

    // Sorting
    fn sort(&mut self)
        where T: Ord
    fn sort_by<F>(&mut self, compare: F)
        where F: FnMut(&T, &T) -> Ordering
    fn reverse(&mut self)

    // Rotation
    fn rotate_left(&mut self, mid: usize)
    fn rotate_right(&mut self, k: usize)

    // Copying
    fn copy_from_slice(&mut self, src: &[T])
        where T: Copy
    fn clone_from_slice(&mut self, src: &[T])
        where T: Clone

    // Conversion
    fn to_vec(&self) -> Vec<T>
        where T: Clone
}

// Indexing
impl<T> Index<usize> for [T] {
    type Output = T
    fn index(&self, index: usize) -> &T
}

impl<T> IndexMut<usize> for [T] {
    fn index_mut(&mut self, index: usize) -> &mut T
}

// Range indexing
impl<T> Index<Range<usize>> for [T] {
    type Output = [T]
    fn index(&self, range: Range<usize>) -> &[T]
}

// Similar for RangeInclusive, RangeFrom, RangeTo, RangeFull
```

### 3.3 HashMap<K, V>

```vertex
use std::collections::HashMap

struct HashMap<K, V> {
    // Private fields
}

impl<K, V> HashMap<K, V>
    where K: Eq + Hash
{
    // Construction
    fn new() -> HashMap<K, V>
    fn with_capacity(capacity: usize) -> HashMap<K, V>

    // Capacity
    fn capacity(&self) -> usize
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn reserve(&mut self, additional: usize)
    fn shrink_to_fit(&mut self)

    // Modification
    fn insert(&mut self, k: K, v: V) -> Option<V>
    fn remove(&mut self, k: &K) -> Option<V>
    fn remove_entry(&mut self, k: &K) -> Option<(K, V)>
    fn clear(&mut self)

    // Access
    fn get(&self, k: &K) -> Option<&V>
    fn get_mut(&mut self, k: &K) -> Option<&mut V>
    fn get_key_value(&self, k: &K) -> Option<(&K, &V)>
    fn contains_key(&self, k: &K) -> bool

    // Entry API
    fn entry(&mut self, key: K) -> Entry<K, V>

    // Iteration
    fn iter(&self) -> hash_map::Iter<K, V>
    fn iter_mut(&mut self) -> hash_map::IterMut<K, V>
    fn keys(&self) -> hash_map::Keys<K, V>
    fn values(&self) -> hash_map::Values<K, V>
    fn values_mut(&mut self) -> hash_map::ValuesMut<K, V>

    // Retain
    fn retain<F>(&mut self, f: F)
        where F: FnMut(&K, &mut V) -> bool
}

// Entry API
enum Entry<K, V> {
    Occupied(OccupiedEntry<K, V>),
    Vacant(VacantEntry<K, V>)
}

impl<K, V> Entry<K, V> {
    fn or_insert(self, default: V) -> &mut V
    fn or_insert_with<F>(self, default: F) -> &mut V
        where F: FnOnce() -> V
    fn or_default(self) -> &mut V
        where V: Default
    fn and_modify<F>(self, f: F) -> Entry<K, V>
        where F: FnOnce(&mut V)
}

// Indexing (panics if key not present)
impl<K, V> Index<&K> for HashMap<K, V>
    where K: Eq + Hash
{
    type Output = V
    fn index(&self, key: &K) -> &V
}

impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V)
    type IntoIter = hash_map::IntoIter<K, V>
    fn into_iter(self) -> hash_map::IntoIter<K, V>
}
```

### 3.4 HashSet<T>

```vertex
use std::collections::HashSet

struct HashSet<T> {
    // Private fields
}

impl<T> HashSet<T>
    where T: Eq + Hash
{
    // Construction
    fn new() -> HashSet<T>
    fn with_capacity(capacity: usize) -> HashSet<T>

    // Capacity
    fn capacity(&self) -> usize
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn reserve(&mut self, additional: usize)
    fn shrink_to_fit(&mut self)

    // Modification
    fn insert(&mut self, value: T) -> bool
    fn remove(&mut self, value: &T) -> bool
    fn take(&mut self, value: &T) -> Option<T>
    fn clear(&mut self)

    // Query
    fn contains(&self, value: &T) -> bool
    fn get(&self, value: &T) -> Option<&T>

    // Set operations
    fn union(&self, other: &HashSet<T>) -> Union<T>
    fn intersection(&self, other: &HashSet<T>) -> Intersection<T>
    fn difference(&self, other: &HashSet<T>) -> Difference<T>
    fn symmetric_difference(&self, other: &HashSet<T>) -> SymmetricDifference<T>

    fn is_subset(&self, other: &HashSet<T>) -> bool
    fn is_superset(&self, other: &HashSet<T>) -> bool
    fn is_disjoint(&self, other: &HashSet<T>) -> bool

    // Iteration
    fn iter(&self) -> hash_set::Iter<T>

    // Retain
    fn retain<F>(&mut self, f: F)
        where F: FnMut(&T) -> bool
}

impl<T> IntoIterator for HashSet<T> {
    type Item = T
    type IntoIter = hash_set::IntoIter<T>
    fn into_iter(self) -> hash_set::IntoIter<T>
}
```

---

## 4. Smart Pointers

### 4.1 Box<T>

```vertex
struct Box<T> {
    // Private pointer
}

impl<T> Box<T> {
    // Construction
    fn new(value: T) -> Box<T>

    // Conversion
    fn into_raw(b: Box<T>) -> *mut T
    unsafe fn from_raw(raw: *mut T) -> Box<T>

    // Leaking (prevent drop)
    fn leak(b: Box<T>) -> &'static mut T
}

// Deref
impl<T> Deref for Box<T> {
    type Target = T
    fn deref(&self) -> &T
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T
}

// Automatic drop
impl<T> Drop for Box<T> {
    fn drop(&mut self)
}
```

### 4.2 Rc<T>

```vertex
use std::rc::Rc

struct Rc<T> {
    // Private reference-counted pointer
}

impl<T> Rc<T> {
    // Construction
    fn new(value: T) -> Rc<T>

    // Cloning (increments reference count)
    fn clone(&self) -> Rc<T>

    // Reference counting
    fn strong_count(this: &Rc<T>) -> usize
    fn weak_count(this: &Rc<T>) -> usize

    // Weak references
    fn downgrade(this: &Rc<T>) -> Weak<T>

    // Getting mutable reference (if unique owner)
    fn get_mut(this: &mut Rc<T>) -> Option<&mut T>

    // Converting
    fn into_raw(this: Rc<T>) -> *const T
    unsafe fn from_raw(ptr: *const T) -> Rc<T>
}

impl<T> Deref for Rc<T> {
    type Target = T
    fn deref(&self) -> &T
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Rc<T>
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self)
}

// Weak references
struct Weak<T> {
    // Private weak pointer
}

impl<T> Weak<T> {
    fn new() -> Weak<T>
    fn upgrade(&self) -> Option<Rc<T>>
    fn strong_count(&self) -> usize
    fn weak_count(&self) -> usize
}
```

### 4.3 Arc<T>

```vertex
use std::sync::Arc

struct Arc<T> {
    // Private atomic reference-counted pointer
}

impl<T> Arc<T> {
    // Construction
    fn new(value: T) -> Arc<T>

    // Cloning (atomic increment)
    fn clone(&self) -> Arc<T>

    // Reference counting
    fn strong_count(this: &Arc<T>) -> usize
    fn weak_count(this: &Arc<T>) -> usize

    // Weak references
    fn downgrade(this: &Arc<T>) -> Weak<T>

    // Getting mutable reference (if unique owner)
    fn get_mut(this: &mut Arc<T>) -> Option<&mut T>

    // Converting
    fn into_raw(this: Arc<T>) -> *const T
    unsafe fn from_raw(ptr: *const T) -> Arc<T>
}

impl<T> Deref for Arc<T> {
    type Target = T
    fn deref(&self) -> &T
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Arc<T>
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self)
}

// Arc is Send + Sync if T is Send + Sync
unsafe impl<T: Send + Sync> Send for Arc<T> { }
unsafe impl<T: Send + Sync> Sync for Arc<T> { }
```

### 4.4 Cell<T> and RefCell<T>

```vertex
use std::cell::{Cell, RefCell}

// Cell - interior mutability for Copy types
struct Cell<T> {
    // Private
}

impl<T: Copy> Cell<T> {
    fn new(value: T) -> Cell<T>
    fn get(&self) -> T
    fn set(&self, value: T)
    fn replace(&self, value: T) -> T
    fn swap(&self, other: &Cell<T>)
    fn into_inner(self) -> T
}

// RefCell - interior mutability with runtime borrow checking
struct RefCell<T> {
    // Private
}

impl<T> RefCell<T> {
    fn new(value: T) -> RefCell<T>

    // Borrowing (panics if rules violated)
    fn borrow(&self) -> Ref<T>
    fn borrow_mut(&self) -> RefMut<T>

    // Try borrowing (returns Result)
    fn try_borrow(&self) -> Result<Ref<T>, BorrowError>
    fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError>

    // Direct access
    fn into_inner(self) -> T
    fn replace(&self, value: T) -> T
    fn swap(&self, other: &RefCell<T>)
}

// Borrowed reference
struct Ref<T> {
    // Private
}

impl<T> Deref for Ref<T> {
    type Target = T
    fn deref(&self) -> &T
}

// Mutably borrowed reference
struct RefMut<T> {
    // Private
}

impl<T> Deref for RefMut<T> {
    type Target = T
    fn deref(&self) -> &T
}

impl<T> DerefMut for RefMut<T> {
    fn deref_mut(&mut self) -> &mut T
}
```

---

## 5. String Types

### 5.1 String

```vertex
struct String {
    // Private Vec<u8> with UTF-8 invariant
}

impl String {
    // Construction
    fn new() -> String
    fn with_capacity(capacity: usize) -> String
    fn from(s: &str) -> String
    fn from_utf8(vec: Vec<u8>) -> Result<String, FromUtf8Error>
    fn from_utf8_lossy(vec: &[u8]) -> String

    // Capacity
    fn capacity(&self) -> usize
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn reserve(&mut self, additional: usize)
    fn shrink_to_fit(&mut self)
    fn truncate(&mut self, new_len: usize)

    // Adding text
    fn push(&mut self, ch: char)
    fn push_str(&mut self, string: &str)
    fn pop(&mut self) -> Option<char>
    fn insert(&mut self, idx: usize, ch: char)
    fn insert_str(&mut self, idx: usize, string: &str)
    fn clear(&mut self)

    // Conversion
    fn as_str(&self) -> &str
    fn as_mut_str(&mut self) -> &mut str
    fn into_bytes(self) -> Vec<u8>
    fn as_bytes(&self) -> &[u8]
    unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8>

    // Removing text
    fn remove(&mut self, idx: usize) -> char
    fn retain<F>(&mut self, f: F)
        where F: FnMut(char) -> bool
}

// Deref to str
impl Deref for String {
    type Target = str
    fn deref(&self) -> &str
}

impl DerefMut for String {
    fn deref_mut(&mut self) -> &mut str
}

// String addition
impl Add<&str> for String {
    type Output = String
    fn add(self, other: &str) -> String
}

impl AddAssign<&str> for String {
    fn add_assign(&mut self, other: &str)
}
```

### 5.2 &str

```vertex
// str is unsized, always used as &str

impl str {
    // Length (in bytes)
    fn len(&self) -> usize
    fn is_empty(&self) -> bool

    // Querying
    fn contains(&self, pat: &str) -> bool
    fn starts_with(&self, pat: &str) -> bool
    fn ends_with(&self, pat: &str) -> bool
    fn find(&self, pat: &str) -> Option<usize>
    fn rfind(&self, pat: &str) -> Option<usize>

    // Case
    fn to_lowercase(&self) -> String
    fn to_uppercase(&self) -> String

    // Trimming
    fn trim(&self) -> &str
    fn trim_start(&self) -> &str
    fn trim_end(&self) -> &str
    fn trim_matches(&self, pat: char) -> &str

    // Splitting
    fn split(&self, pat: &str) -> Split
    fn split_whitespace(&self) -> SplitWhitespace
    fn lines(&self) -> Lines
    fn split_once(&self, delimiter: &str) -> Option<(&str, &str)>
    fn rsplit_once(&self, delimiter: &str) -> Option<(&str, &str)>

    // Replacement
    fn replace(&self, from: &str, to: &str) -> String
    fn replacen(&self, from: &str, to: &str, count: usize) -> String

    // Iteration
    fn chars(&self) -> Chars
    fn char_indices(&self) -> CharIndices
    fn bytes(&self) -> Bytes

    // Slicing (by byte index)
    fn get(&self, i: usize) -> Option<&str>
    fn get_range(&self, range: Range<usize>) -> Option<&str>
    unsafe fn get_unchecked(&self, i: usize) -> &str

    // Conversion
    fn to_string(&self) -> String
    fn as_bytes(&self) -> &[u8]

    // Parsing
    fn parse<F>(&self) -> Result<F, F::Err>
        where F: FromStr

    // Repetition
    fn repeat(&self, n: usize) -> String
}

// Indexing by range
impl Index<Range<usize>> for str {
    type Output = str
    fn index(&self, index: Range<usize>) -> &str
}

// Similar for other range types
```

### 5.3 String Indexing Prohibition

**IMPORTANT**: Direct indexing of `String` and `str` by integer is **NOT ALLOWED** in Vertex.

```vertex
let s = String::from("hello")
// let c = s[0]      // ❌ ERROR: Cannot index String with usize
// let c = s[1]      // ❌ ERROR: Cannot index str with usize
```

**Why This Restriction Exists**:

Strings in Vertex are UTF-8 encoded, which means:
- Characters can be 1-4 bytes long (variable width encoding)
- Indexing by integer would give you a **byte**, not a **character**
- This leads to confusion and potential bugs with non-ASCII text
- Slicing in the middle of a multi-byte character causes panics

**Examples of Problems**:

```vertex
let s = String::from("hello")
let c = s[0]  // Would this be 'h' (the character) or 104 (the byte)?

let s = String::from("café")
// UTF-8 encoding: [99, 97, 102, 195, 169]
//                  c   a   f   é (2 bytes)
let c = s[3]  // Would give 195 (first byte of é), not 'é'!
let c = s[4]  // Would give 169 (second byte of é), INVALID on its own!

let emoji = String::from("👍")
// UTF-8: [240, 159, 145, 141] (4 bytes for one emoji!)
let c = emoji[0]  // Would give 240, meaningless byte
```

**Correct Alternatives**:

**1. Iterate over characters** (for reading):
```vertex
let s = String::from("café")

// Get all characters as a Vec
let chars: Vec<char> = s.chars().collect()
let first = chars[0]  // 'c'
let last = chars[3]   // 'é'

// Iterate directly
for ch in s.chars() {
    println("{}", ch)
}

// Get specific character by position
let third_char = s.chars().nth(2)  // Returns Option<char>
```

**2. Access raw bytes** (if you really need bytes):
```vertex
let s = String::from("hello")
let bytes: &[u8] = s.as_bytes()
let first_byte = bytes[0]  // 104 (ASCII 'h')

// ⚠️  Warning: Only safe for ASCII text!
// Non-ASCII characters will be split into multiple bytes
```

**3. Slice by byte range** (allowed, but dangerous):
```vertex
let s = String::from("hello")
let slice: &str = &s[0..2]  // "he" (2 bytes = 2 ASCII chars)

// ⚠️  Panics if you slice in the middle of a character!
let s = String::from("café")
// let slice = &s[0..4]  // ❌ PANIC: byte 4 is in the middle of 'é'
let slice = &s[0..5]     // ✓ OK: "café" (5 bytes total)

// Safer: Use get() which returns Option
match s.get(0..4) {
    Some(slice) => println("{}", slice),
    None => println("Invalid byte range")
}
```

**4. Use char_indices for byte positions**:
```vertex
let s = String::from("café")

for (byte_idx, ch) in s.char_indices() {
    println("char '{}' starts at byte {}", ch, byte_idx)
}
// Output:
// char 'c' starts at byte 0
// char 'a' starts at byte 1
// char 'f' starts at byte 2
// char 'é' starts at byte 3  (but takes 2 bytes: 3 and 4)
```

**Summary of Safe String Access**:

| Goal | Method | Example |
|------|--------|---------|
| Get nth character | `.chars().nth(n)` | `s.chars().nth(0)` → `Option<char>` |
| Iterate characters | `.chars()` | `for ch in s.chars() { ... }` |
| All characters as Vec | `.chars().collect()` | `let v: Vec<char> = s.chars().collect()` |
| Get byte | `.as_bytes()[n]` | `s.as_bytes()[0]` → `u8` |
| Byte range slice | `&s[start..end]` | `&s[0..5]` → `&str` (⚠️  can panic) |
| Safe byte range | `.get(start..end)` | `s.get(0..5)` → `Option<&str>` |
| Char indices | `.char_indices()` | `for (i, ch) in s.char_indices()` |

**Key Principle**: Vertex enforces that you **explicitly choose** between:
- **Character-level access** (what you usually want) via `.chars()`
- **Byte-level access** (for performance/FFI) via `.as_bytes()` or slicing

This prevents subtle bugs with Unicode text and makes code intent crystal clear.

---

## 6. I/O Module

### 6.1 std::io

```vertex
mod std::io {
    // Error type
    struct Error {
        // Private
    }

    enum ErrorKind {
        NotFound,
        PermissionDenied,
        ConnectionRefused,
        ConnectionReset,
        ConnectionAborted,
        NotConnected,
        AddrInUse,
        AddrNotAvailable,
        BrokenPipe,
        AlreadyExists,
        WouldBlock,
        InvalidInput,
        InvalidData,
        TimedOut,
        WriteZero,
        Interrupted,
        UnexpectedEof,
        Other,
    }

    impl Error {
        fn new(kind: ErrorKind, error: &str) -> Error
        fn kind(&self) -> ErrorKind
        fn to_string(&self) -> String
    }

    // Result type alias
    type Result<T> = Result<T, Error>

    // Traits
    trait Read {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize>

        fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize>
        fn read_to_string(&mut self, buf: &mut String) -> Result<usize>
        fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>
    }

    trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize>
        fn flush(&mut self) -> Result<()>

        fn write_all(&mut self, buf: &[u8]) -> Result<()>
        fn write_fmt(&mut self, fmt: Arguments) -> Result<()>
    }

    trait BufRead: Read {
        fn fill_buf(&mut self) -> Result<&[u8]>
        fn consume(&mut self, amt: usize)

        fn read_line(&mut self, buf: &mut String) -> Result<usize>
        fn lines(&mut self) -> Lines<Self>
    }

    // Buffered readers/writers
    struct BufReader<R> {
        // Private
    }

    impl<R: Read> BufReader<R> {
        fn new(inner: R) -> BufReader<R>
        fn with_capacity(capacity: usize, inner: R) -> BufReader<R>
    }

    struct BufWriter<W> {
        // Private
    }

    impl<W: Write> BufWriter<W> {
        fn new(inner: W) -> BufWriter<W>
        fn with_capacity(capacity: usize, inner: W) -> BufWriter<W>
    }

    // Standard streams
    fn stdin() -> Stdin
    fn stdout() -> Stdout
    fn stderr() -> Stderr

    struct Stdin {
        // Private
    }

    impl Stdin {
        fn read_line(&self, buf: &mut String) -> Result<usize>
        fn lock(&self) -> StdinLock
    }

    struct Stdout {
        // Private
    }

    impl Stdout {
        fn lock(&self) -> StdoutLock
    }

    struct Stderr {
        // Private
    }

    impl Stderr {
        fn lock(&self) -> StderrLock
    }

    // Cursor (in-memory reader/writer)
    struct Cursor<T> {
        // Private
    }

    impl<T> Cursor<T> {
        fn new(inner: T) -> Cursor<T>
        fn into_inner(self) -> T
        fn get_ref(&self) -> &T
        fn get_mut(&mut self) -> &mut T
        fn position(&self) -> u64
        fn set_position(&mut self, pos: u64)
    }
}
```

---

## 7. File System

### 7.1 std::fs

```vertex
mod std::fs {
    use std::io::{self, Read, Write}

    // File type
    struct File {
        // Private
    }

    impl File {
        // Opening
        fn open(path: &str) -> io::Result<File>
        fn create(path: &str) -> io::Result<File>
        fn options() -> OpenOptions

        // Metadata
        fn metadata(&self) -> io::Result<Metadata>
        fn sync_all(&self) -> io::Result<()>
        fn sync_data(&self) -> io::Result<()>
        fn set_len(&self, size: u64) -> io::Result<()>
    }

    impl Read for File {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
    }

    impl Write for File {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize>
        fn flush(&mut self) -> io::Result<()>
    }

    // Open options
    struct OpenOptions {
        // Private
    }

    impl OpenOptions {
        fn new() -> OpenOptions
        fn read(&mut self, read: bool) -> &mut OpenOptions
        fn write(&mut self, write: bool) -> &mut OpenOptions
        fn append(&mut self, append: bool) -> &mut OpenOptions
        fn truncate(&mut self, truncate: bool) -> &mut OpenOptions
        fn create(&mut self, create: bool) -> &mut OpenOptions
        fn create_new(&mut self, create_new: bool) -> &mut OpenOptions
        fn open(&self, path: &str) -> io::Result<File>
    }

    // Metadata
    struct Metadata {
        // Private
    }

    impl Metadata {
        fn file_type(&self) -> FileType
        fn is_dir(&self) -> bool
        fn is_file(&self) -> bool
        fn len(&self) -> u64
        fn permissions(&self) -> Permissions
        fn modified(&self) -> io::Result<SystemTime>
        fn accessed(&self) -> io::Result<SystemTime>
        fn created(&self) -> io::Result<SystemTime>
    }

    struct FileType {
        // Private
    }

    impl FileType {
        fn is_dir(&self) -> bool
        fn is_file(&self) -> bool
        fn is_symlink(&self) -> bool
    }

    struct Permissions {
        // Private
    }

    impl Permissions {
        fn readonly(&self) -> bool
        fn set_readonly(&mut self, readonly: bool)
    }

    // Directory operations
    fn read_dir(path: &str) -> io::Result<ReadDir>
    fn create_dir(path: &str) -> io::Result<()>
    fn create_dir_all(path: &str) -> io::Result<()>
    fn remove_dir(path: &str) -> io::Result<()>
    fn remove_dir_all(path: &str) -> io::Result<()>

    struct ReadDir {
        // Private
    }

    impl Iterator for ReadDir {
        type Item = io::Result<DirEntry>
        fn next(&mut self) -> Option<io::Result<DirEntry>>
    }

    struct DirEntry {
        // Private
    }

    impl DirEntry {
        fn path(&self) -> String
        fn file_name(&self) -> String
        fn metadata(&self) -> io::Result<Metadata>
        fn file_type(&self) -> io::Result<FileType>
    }

    // File operations
    fn metadata(path: &str) -> io::Result<Metadata>
    fn rename(from: &str, to: &str) -> io::Result<()>
    fn copy(from: &str, to: &str) -> io::Result<u64>
    fn remove_file(path: &str) -> io::Result<()>

    // Convenience functions
    fn read(path: &str) -> io::Result<Vec<u8>>
    fn read_to_string(path: &str) -> io::Result<String>
    fn write(path: &str, contents: &[u8]) -> io::Result<()>
}
```

---

## 8. Iterators

### 8.1 Iterator Trait

```vertex
trait Iterator {
    type Item

    // Required method
    fn next(&mut self) -> Result<Self::Item, ()>

    // Transformations
    fn map<B, F>(self, f: F) -> Map<Self, F>
        where F: FnMut(Self::Item) -> B

    fn filter<P>(self, predicate: P) -> Filter<Self, P>
        where P: FnMut(&Self::Item) -> bool

    fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F>
        where F: FnMut(Self::Item) -> Option<B>

    fn flat_map<U, F>(self, f: F) -> FlatMap<Self, F>
        where F: FnMut(Self::Item) -> U,
              U: IntoIterator

    fn enumerate(self) -> Enumerate<Self>

    fn skip(self, n: usize) -> Skip<Self>

    fn take(self, n: usize) -> Take<Self>

    fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
        where P: FnMut(&Self::Item) -> bool

    fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
        where P: FnMut(&Self::Item) -> bool

    fn step_by(self, step: usize) -> StepBy<Self>

    fn chain<U>(self, other: U) -> Chain<Self, U::IntoIter>
        where U: IntoIterator<Item = Self::Item>

    fn zip<U>(self, other: U) -> Zip<Self, U::IntoIter>
        where U: IntoIterator

    fn rev(self) -> Rev<Self>
        where Self: DoubleEndedIterator

    fn cycle(self) -> Cycle<Self>
        where Self: Clone

    // Consumers
    fn collect<B>(self) -> B
        where B: FromIterator<Self::Item>

    fn fold<B, F>(self, init: B, f: F) -> B
        where F: FnMut(B, Self::Item) -> B

    fn reduce<F>(self, f: F) -> Option<Self::Item>
        where F: FnMut(Self::Item, Self::Item) -> Self::Item

    fn sum<S>(self) -> S
        where S: Sum<Self::Item>

    fn product<P>(self) -> P
        where P: Product<Self::Item>

    fn count(self) -> usize

    fn last(self) -> Option<Self::Item>

    fn nth(&mut self, n: usize) -> Option<Self::Item>

    // Search
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
        where P: FnMut(&Self::Item) -> bool

    fn position<P>(&mut self, predicate: P) -> Option<usize>
        where P: FnMut(Self::Item) -> bool

    fn max(self) -> Option<Self::Item>
        where Self::Item: Ord

    fn min(self) -> Option<Self::Item>
        where Self::Item: Ord

    fn max_by<F>(self, compare: F) -> Option<Self::Item>
        where F: FnMut(&Self::Item, &Self::Item) -> Ordering

    fn min_by<F>(self, compare: F) -> Option<Self::Item>
        where F: FnMut(&Self::Item, &Self::Item) -> Ordering

    // Boolean tests
    fn all<F>(&mut self, f: F) -> bool
        where F: FnMut(Self::Item) -> bool

    fn any<F>(&mut self, f: F) -> bool
        where F: FnMut(Self::Item) -> bool

    // Size
    fn size_hint(&self) -> (usize, Option<usize>)
}
```

**Important Note on `next()` Return Type**:

The `next()` method returns `Result<Self::Item, ()>` rather than `Option<Self::Item>`. This design choice ensures that:
- `Ok(value)` signals the iterator has another item
- `Err(())` signals the iterator is exhausted

This is critical for `for` loop desugaring:
```vertex
// This for loop:
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
```

Other methods like `last()`, `nth()`, `find()`, `max()`, and `min()` return `Option` because they represent search operations that may not find a value, which is semantically different from iteration completion.

### 8.2 IntoIterator Trait

```vertex
trait IntoIterator {
    type Item
    type IntoIter: Iterator<Item = Self::Item>

    fn into_iter(self) -> Self::IntoIter
}

// Implemented for all iterators
impl<I: Iterator> IntoIterator for I {
    type Item = I::Item
    type IntoIter = I
    fn into_iter(self) -> I {
        self
    }
}
```

### 8.3 FromIterator Trait

```vertex
trait FromIterator<A> {
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item = A>
}

// Implemented for Vec, HashMap, HashSet, String, etc.
```

---

## 9. Traits

### 9.1 Core Traits

```vertex
// Clone - explicit copy
trait Clone {
    fn clone(&self) -> Self
}

// Copy - implicit bitwise copy (marker trait)
trait Copy: Clone { }

// Drop - destructor
trait Drop {
    fn drop(&mut self)
}

// Default - default value
trait Default {
    fn default() -> Self
}

// PartialEq - partial equality
trait PartialEq {
    fn eq(&self, other: &Self) -> bool
    fn ne(&self, other: &Self) -> bool {
        not self.eq(other)
    }
}

// Eq - full equality (marker trait)
trait Eq: PartialEq { }

// PartialOrd - partial ordering
enum Ordering {
    Less,
    Equal,
    Greater
}

trait PartialOrd: PartialEq {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    fn lt(&self, other: &Self) -> bool
    fn le(&self, other: &Self) -> bool
    fn gt(&self, other: &Self) -> bool
    fn ge(&self, other: &Self) -> bool
}

// Ord - total ordering
trait Ord: Eq + PartialOrd {
    fn cmp(&self, other: &Self) -> Ordering
}

// Hash - hashing
trait Hash {
    fn hash<H: Hasher>(&self, state: &mut H)
}

trait Hasher {
    fn finish(&self) -> u64
    fn write(&mut self, bytes: &[u8])
}
```

### 9.2 Display and Debug

```vertex
// Display - user-facing formatting
trait Display {
    fn fmt(&self) -> String
}

// Debug - programmer-facing formatting
trait Debug {
    fn fmt_debug(&self) -> String
}
```

### 9.3 Operator Traits

```vertex
// Arithmetic
trait Add<Rhs = Self> {
    type Output
    fn add(self, rhs: Rhs) -> Self::Output
}

trait Sub<Rhs = Self> {
    type Output
    fn sub(self, rhs: Rhs) -> Self::Output
}

trait Mul<Rhs = Self> {
    type Output
    fn mul(self, rhs: Rhs) -> Self::Output
}

trait Div<Rhs = Self> {
    type Output
    fn div(self, rhs: Rhs) -> Self::Output
}

trait Rem<Rhs = Self> {
    type Output
    fn rem(self, rhs: Rhs) -> Self::Output
}

// Assignment operators
trait AddAssign<Rhs = Self> {
    fn add_assign(&mut self, rhs: Rhs)
}

// Similar for SubAssign, MulAssign, DivAssign, RemAssign

// Bitwise
trait BitAnd<Rhs = Self> {
    type Output
    fn bitand(self, rhs: Rhs) -> Self::Output
}

// Similar for BitOr, BitXor

trait Shl<Rhs> {
    type Output
    fn shl(self, rhs: Rhs) -> Self::Output
}

trait Shr<Rhs> {
    type Output
    fn shr(self, rhs: Rhs) -> Self::Output
}

// Negation
trait Neg {
    type Output
    fn neg(self) -> Self::Output
}

trait Not {
    type Output
    fn not(self) -> Self::Output
}
```

### 9.4 Deref Traits

```vertex
trait Deref {
    type Target
    fn deref(&self) -> &Self::Target
}

trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target
}
```

### 9.5 Index Traits

```vertex
trait Index<Idx> {
    type Output
    fn index(&self, index: Idx) -> &Self::Output
}

trait IndexMut<Idx>: Index<Idx> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output
}
```

### 9.6 Closure Traits

**Closure Trait Hierarchy**: Every closure implements one or more of these traits based on how it captures variables:

```vertex
// FnOnce - Can be called once, consumes captured values
trait FnOnce<Args> {
    type Output
    fn call_once(self, args: Args) -> Self::Output
}

// FnMut - Can be called multiple times, mutable capture
trait FnMut<Args>: FnOnce<Args> {
    fn call_mut(&mut self, args: Args) -> Self::Output
}

// Fn - Can be called multiple times, immutable capture
trait Fn<Args>: FnMut<Args> {
    fn call(&self, args: Args) -> Self::Output
}
```

**Trait Relationship**:
- `Fn` is the most restrictive (immutable borrows only)
- `FnMut` allows mutable borrows
- `FnOnce` can consume captured values (least restrictive)
- Every `Fn` is also `FnMut` and `FnOnce`
- Every `FnMut` is also `FnOnce`

**Capture Semantics**:

| Capture Mode | Implements | Can Call | Example |
|--------------|------------|----------|---------|
| No captures or immutable borrows | `Fn + FnMut + FnOnce` | Multiple times | `\|x\| x + 1` |
| Mutable borrows | `FnMut + FnOnce` | Multiple times (with `mut`) | `\|x\| { count += 1; x }` |
| Moves/consumes values | `FnOnce` | Once only | `\|x\| { drop(data); x }` |

**Examples**:

```vertex
// Fn: Can be called multiple times, no mutation
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))  // Call f twice
}

let double = |x| x * 2
apply_twice(double, 5)  // Returns 20

// FnMut: Can be called multiple times, with mutation
fn call_three_times<F: FnMut()>(mut f: F) {
    f(); f(); f()
}

let mut count = 0
call_three_times(|| count += 1)
// count is now 3

// FnOnce: Can only be called once
fn consume_with<F: FnOnce(String)>(f: F) {
    f(String::from("hello"))
}

let data = vec![1, 2, 3]
consume_with(|s| {
    drop(data)  // Consumes data
    println(s)
})
```

**Function Pointer Compatibility**:

Regular functions and methods also implement the `Fn` traits:

```vertex
fn add_one(x: i32) -> i32 { x + 1 }

// Functions implement Fn (since they don't capture anything)
fn accepts_fn<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

accepts_fn(add_one, 42)        // Works with function
accepts_fn(|x| x + 1, 42)      // Works with closure
```

**Common Use Cases**:

```vertex
// Iterator combinators use FnMut
vec![1, 2, 3].map(|x| x * 2)           // Fn is fine (no capture)
vec![1, 2, 3].filter(|x| *x > threshold)  // Fn (immutable capture)

let mut sum = 0
vec![1, 2, 3].map(|x| { sum += x; x })  // Requires FnMut (mutable capture)

// Thread spawning requires FnOnce + Send + 'static
let data = vec![1, 2, 3]
thread::spawn(move || {
    process(data)  // Consumes data
})
```

**Compiler-Synthesized Implementation**:

The compiler automatically implements the appropriate traits for each closure based on its capture behavior:

```vertex
// This closure:
let y = 10
let f = |x| x + y

// Gets an implementation like:
struct ClosureEnv { y: i32 }

impl Fn<(i32,)> for ClosureEnv {
    type Output = i32
    fn call(&self, args: (i32,)) -> i32 {
        args.0 + self.y
    }
}
```

### 9.7 Thread Safety Traits (Send and Sync)

**Marker Traits for Concurrency Safety**: These are `unsafe` marker traits that indicate whether a type can be safely used across thread boundaries.

```vertex
// Send: Type can be transferred across thread boundaries
unsafe trait Send { }

// Sync: Type can be shared between threads (&T is Send)
unsafe trait Sync { }
```

**Relationship**:
- `Send`: A type is `Send` if ownership can be transferred to another thread
- `Sync`: A type is `Sync` if it's safe to share references (`&T`) between threads
- If `T` is `Sync`, then `&T` is `Send`
- Both are *auto-traits* — compiler automatically implements them when safe

**Auto-Implementation Rules**:

A type is automatically `Send` **unless** it contains:
- `Rc<T>` (non-atomic reference counting)
- Raw pointers (`*const T`, `*mut T`)
- Types explicitly marked `!Send`
- Thread-local storage

A type is automatically `Sync` **unless** it contains:
- `Cell<T>` or `RefCell<T>` (interior mutability without synchronization)
- `Rc<T>` (non-atomic reference counting)
- Types explicitly marked `!Sync`
- Any non-`Sync` field

**Common Types**:

| Type | Send | Sync | Notes |
|------|------|------|-------|
| `i32`, `String`, `Vec<T>` | ✓ | ✓ | Basic types (if `T: Send + Sync`) |
| `Box<T>`, `&T`, `&mut T` | ✓ | Depends on `T` | Follow `T`'s properties |
| `Rc<T>` | ✗ | ✗ | Not thread-safe (use `Arc<T>`) |
| `Arc<T>` | ✓ | ✓ | Atomic reference counting |
| `Cell<T>`, `RefCell<T>` | ✓ | ✗ | Interior mutability without locks |
| `Mutex<T>`, `RwLock<T>` | ✓ | ✓ | Synchronized interior mutability |
| `MutexGuard<T>` | ✗ | ✓ | Lock guard (cannot send across threads) |

**Examples**:

```vertex
use std::thread
use std::sync::Arc

// Send: Ownership transferred to thread
fn send_example() {
    let data = vec![1, 2, 3]  // Vec<i32> is Send

    thread::spawn(move || {
        // data moved into this thread
        println("{}", data.len())
    })
}

// Sync: Shared between threads via Arc
fn sync_example() {
    let data = Arc::new(vec![1, 2, 3])  // Arc<Vec<i32>> is Send + Sync
    let data_clone = data.clone()

    thread::spawn(move || {
        // data_clone shares ownership with main thread
        println("{}", data_clone.len())
    })

    println("{}", data.len())  // Main thread still has access
}

// Not Send: Rc<T> cannot cross thread boundaries
fn not_send_example() {
    let data = Rc::new(vec![1, 2, 3])

    // ERROR: Rc<Vec<i32>> is not Send
    // thread::spawn(move || {
    //     println("{}", data.len())
    // })
}

// Not Sync: RefCell<T> cannot be shared between threads
fn not_sync_example() {
    let data = Arc::new(RefCell::new(vec![1, 2, 3]))

    // ERROR: Arc<RefCell<Vec<i32>>> is not Sync
    // thread::spawn(move || {
    //     data.borrow_mut().push(4)
    // })

    // Solution: Use Mutex<T> instead
    let data = Arc::new(Mutex::new(vec![1, 2, 3]))
    let data_clone = data.clone()

    thread::spawn(move || {
        data_clone.lock().unwrap().push(4)  // OK
    })
}
```

**Manual Implementation** (Requires `unsafe`):

When auto-implementation is incorrect (e.g., your type uses raw pointers but guarantees thread safety):

```vertex
struct MyThreadSafeType {
    data: *mut i32  // Raw pointer prevents auto Send/Sync
}

// SAFETY: I guarantee this type is thread-safe because:
// - The raw pointer is only accessed through synchronized methods
// - No mutable aliasing can occur
unsafe impl Send for MyThreadSafeType { }
unsafe impl Sync for MyThreadSafeType { }
```

**Compiler Errors**:

```vertex
// Error: closure may outlive the current function
thread::spawn(|| {
    let local = 42
    println("{}", local)  // ERROR: captures non-'static variable
})

// Fix: Move ownership into closure
let local = 42
thread::spawn(move || {
    println("{}", local)  // OK: local moved into closure
})
```

**Usage in Generic Bounds**:

```vertex
// Function that sends data to another thread
fn send_to_thread<T: Send + 'static>(data: T) {
    thread::spawn(move || {
        process(data)
    })
}

// Function that shares data between threads
fn share_between_threads<T: Sync + 'static>(data: Arc<T>) {
    let data_clone = data.clone()
    thread::spawn(move || {
        read_from(data_clone.as_ref())
    })
}
```

**Why These Are Unsafe Traits**:

Implementing `Send` or `Sync` incorrectly can lead to data races, which are undefined behavior in Vertex (just like in Rust). The `unsafe impl` keyword forces you to acknowledge that you're making safety guarantees the compiler cannot verify.

---

## 10. Synchronization

### 10.1 Mutex<T>

```vertex
use std::sync::Mutex

struct Mutex<T> {
    // Private
}

impl<T> Mutex<T> {
    fn new(t: T) -> Mutex<T>

    fn lock(&self) -> Result<MutexGuard<T>, PoisonError<MutexGuard<T>>>

    fn try_lock(&self) -> Result<MutexGuard<T>, TryLockError<MutexGuard<T>>>

    fn into_inner(self) -> Result<T, PoisonError<T>>

    fn get_mut(&mut self) -> Result<&mut T, PoisonError<&mut T>>
}

struct MutexGuard<T> {
    // Private
}

impl<T> Deref for MutexGuard<T> {
    type Target = T
    fn deref(&self) -> &T
}

impl<T> DerefMut for MutexGuard<T> {
    fn deref_mut(&mut self) -> &mut T
}

impl<T> Drop for MutexGuard<T> {
    fn drop(&mut self)  // Unlocks mutex
}
```

### 10.2 RwLock<T>

```vertex
use std::sync::RwLock

struct RwLock<T> {
    // Private
}

impl<T> RwLock<T> {
    fn new(t: T) -> RwLock<T>

    fn read(&self) -> Result<RwLockReadGuard<T>, PoisonError<RwLockReadGuard<T>>>

    fn write(&self) -> Result<RwLockWriteGuard<T>, PoisonError<RwLockWriteGuard<T>>>

    fn try_read(&self) -> Result<RwLockReadGuard<T>, TryLockError<RwLockReadGuard<T>>>

    fn try_write(&self) -> Result<RwLockWriteGuard<T>, TryLockError<RwLockWriteGuard<T>>>

    fn into_inner(self) -> Result<T, PoisonError<T>>

    fn get_mut(&mut self) -> Result<&mut T, PoisonError<&mut T>>
}

struct RwLockReadGuard<T> {
    // Private
}

impl<T> Deref for RwLockReadGuard<T> {
    type Target = T
    fn deref(&self) -> &T
}

struct RwLockWriteGuard<T> {
    // Private
}

impl<T> Deref for RwLockWriteGuard<T> {
    type Target = T
    fn deref(&self) -> &T
}

impl<T> DerefMut for RwLockWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut T
}
```

### 10.3 Atomic Types

```vertex
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicI64, AtomicU64, AtomicUsize}

enum Ordering {
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst
}

struct AtomicI32 {
    // Private
}

impl AtomicI32 {
    fn new(v: i32) -> AtomicI32

    fn load(&self, order: Ordering) -> i32

    fn store(&self, val: i32, order: Ordering)

    fn swap(&self, val: i32, order: Ordering) -> i32

    fn compare_exchange(
        &self,
        current: i32,
        new: i32,
        success: Ordering,
        failure: Ordering
    ) -> Result<i32, i32>

    fn fetch_add(&self, val: i32, order: Ordering) -> i32

    fn fetch_sub(&self, val: i32, order: Ordering) -> i32

    fn fetch_and(&self, val: i32, order: Ordering) -> i32

    fn fetch_or(&self, val: i32, order: Ordering) -> i32

    fn fetch_xor(&self, val: i32, order: Ordering) -> i32
}

// Similar for other atomic types
```

---

## 11. Memory Module

### 11.1 std::mem

```vertex
mod std::mem {
    // Size and alignment
    const fn size_of<T>() -> usize
    const fn align_of<T>() -> usize
    const fn size_of_val<T>(val: &T) -> usize
    const fn align_of_val<T>(val: &T) -> usize

    // Moving and swapping
    fn swap<T>(x: &mut T, y: &mut T)
    fn replace<T>(dest: &mut T, src: T) -> T
    fn take<T>(dest: &mut T) -> T
        where T: Default

    // Preventing drop
    fn forget<T>(t: T)

    // Transmutation (unsafe)
    unsafe fn transmute<T, U>(e: T) -> U

    // Discriminant
    fn discriminant<T>(v: &T) -> Discriminant<T>

    struct Discriminant<T> {
        // Private
    }
}
```

### 11.2 std::ptr

```vertex
mod std::ptr {
    // Null pointers
    const fn null<T>() -> *const T
    const fn null_mut<T>() -> *mut T

    // Reading/writing (unsafe)
    unsafe fn read<T>(src: *const T) -> T
    unsafe fn write<T>(dst: *mut T, src: T)
    unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize)
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize)

    // Swapping (unsafe)
    unsafe fn swap<T>(x: *mut T, y: *mut T)
    unsafe fn replace<T>(dst: *mut T, src: T) -> T

    // Dropping (unsafe)
    unsafe fn drop_in_place<T>(to_drop: *mut T)
}
```

---

## 12. Conversion Traits

### 12.1 From and Into

```vertex
trait From<T> {
    fn from(value: T) -> Self
}

trait Into<T> {
    fn into(self) -> T
}

// Blanket implementation
impl<T, U> Into<U> for T
    where U: From<T>
{
    fn into(self) -> U {
        U::from(self)
    }
}

// Examples
impl From<i32> for i64 {
    fn from(x: i32) -> i64 {
        x as i64
    }
}

impl From<&str> for String {
    fn from(s: &str) -> String {
        s.to_string()
    }
}
```

### 12.2 AsRef and AsMut

```vertex
trait AsRef<T> {
    fn as_ref(&self) -> &T
}

trait AsMut<T> {
    fn as_mut(&mut self) -> &mut T
}

// Examples
impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[T]> for Vec<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
```

### 12.3 FromStr

```vertex
trait FromStr {
    type Err
    fn from_str(s: &str) -> Result<Self, Self::Err>
}

// Implemented for primitives, enabling parse()
let num: i32 = "42".parse().unwrap()
let float: f64 = "3.14".parse().unwrap()
```

---

## 13. Standard Library Module Organization

The Vertex standard library is organized into a hierarchical module structure. This section provides a comprehensive overview of all standard library modules and their organization.

### 13.1 Module Hierarchy

```vertex
std  // Root module
├── collections     // Data structures
│   ├── HashMap
│   ├── HashSet
│   ├── BTreeMap   (future)
│   └── BTreeSet   (future)
│
├── io             // Input/output
│   ├── Error
│   ├── ErrorKind
│   ├── Read trait
│   ├── Write trait
│   ├── BufRead trait
│   ├── BufReader
│   ├── BufWriter
│   ├── stdin()
│   ├── stdout()
│   └── stderr()
│
├── fs             // File system operations
│   ├── File
│   ├── OpenOptions
│   ├── DirEntry
│   ├── ReadDir
│   ├── Metadata
│   ├── read()
│   ├── write()
│   ├── read_to_string()
│   ├── create_dir()
│   ├── create_dir_all()
│   ├── remove_file()
│   └── remove_dir()
│
├── path           // Path manipulation
│   ├── Path
│   ├── PathBuf
│   └── Component
│
├── env            // Environment interaction
│   ├── args()
│   ├── var()
│   ├── set_var()
│   ├── current_dir()
│   └── set_current_dir()
│
├── thread         // Threading primitives
│   ├── spawn()
│   ├── sleep()
│   ├── yield_now()
│   ├── Thread
│   └── JoinHandle
│
├── sync           // Synchronization primitives
│   ├── Arc
│   ├── Mutex
│   ├── MutexGuard
│   ├── RwLock
│   ├── RwLockReadGuard
│   └── RwLockWriteGuard
│
├── mem            // Memory manipulation
│   ├── size_of()
│   ├── align_of()
│   ├── drop()
│   ├── swap()
│   └── replace()
│
├── ops            // Operator traits and ranges
│   ├── Add, Sub, Mul, Div, Rem
│   ├── AddAssign, SubAssign, etc.
│   ├── Deref, DerefMut
│   ├── Index, IndexMut
│   ├── Range
│   ├── RangeInclusive
│   ├── RangeFrom
│   ├── RangeTo
│   └── RangeFull
│
├── ptr            // Raw pointer operations (unsafe)
│   ├── null()
│   ├── null_mut()
│   ├── read()
│   ├── write()
│   └── swap()
│
├── slice          // Slice operations
│   └── (inherent methods on [T])
│
├── str            // String slice operations
│   └── (inherent methods on str)
│
├── string         // Owned string type
│   └── String
│
├── vec            // Dynamic array type
│   └── Vec
│
├── boxed          // Heap allocation
│   └── Box
│
├── rc             // Reference counting
│   └── Rc
│
├── cell           // Interior mutability
│   ├── Cell
│   └── RefCell
│
├── option         // Option type
│   └── Option (Some, None)
│
├── result         // Result type
│   └── Result (Ok, Err)
│
├── iter           // Iterator traits and adaptors
│   ├── Iterator trait
│   ├── IntoIterator trait
│   ├── FromIterator trait
│   ├── Map, Filter, FilterMap
│   ├── Enumerate, Zip, Chain
│   └── (other iterator adaptors)
│
├── fmt            // Formatting
│   ├── Display trait
│   ├── Debug trait
│   └── (internal formatting machinery)
│
├── cmp            // Comparison traits
│   ├── PartialEq, Eq
│   ├── PartialOrd, Ord
│   └── Ordering
│
├── clone          // Clone trait
│   └── Clone trait
│
├── default        // Default trait
│   └── Default trait
│
├── convert        // Conversion traits
│   ├── From, Into
│   ├── AsRef, AsMut
│   ├── TryFrom, TryInto
│   └── FromStr
│
├── hash           // Hashing
│   ├── Hash trait
│   ├── Hasher trait
│   └── (hash implementations)
│
├── marker         // Marker traits
│   ├── Copy
│   ├── Send
│   ├── Sync
│   └── Sized
│
├── panic          // Panic handling
│   └── catch_unwind() (future)
│
└── prelude        // Prelude imports
    └── (commonly used items)
```

### 13.2 Import Patterns

**Using the Prelude** (no explicit imports needed):
```vertex
// These are always available without importing
fn main() {
    let v = vec![1, 2, 3]           // vec! from prelude
    let opt: Option<i32> = Some(5)  // Option and Some from prelude
    let res: Result<i32, &str> = Ok(10)  // Result and Ok from prelude
    println("Hello")                 // println from prelude
}
```

**Explicit Module Imports**:
```vertex
// Import specific items
use std::collections::HashMap
use std::fs::File
use std::io::Write

// Import multiple items
use std::io::{Read, Write, BufRead}

// Import all items (discouraged in production code)
use std::collections::*

// Rename imports
use std::collections::HashMap as Map
```

**Nested Imports**:
```vertex
// Group related imports
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Write}
}
```

### 13.3 Module Re-exports

The following types are re-exported at the top level for convenience:

```vertex
// In std lib root
pub use self::option::Option;
pub use self::result::Result;
pub use self::string::String;
pub use self::vec::Vec;
pub use self::boxed::Box;

// So you can write:
use std::Option  // Instead of std::option::Option
use std::String  // Instead of std::string::String
```

### 13.4 Module Access by Feature Category

**Collections & Data Structures**:
- `std::vec::Vec` - Dynamic array
- `std::string::String` - Owned UTF-8 string
- `std::collections::HashMap` - Hash-based key-value map
- `std::collections::HashSet` - Hash-based set
- `std::boxed::Box` - Heap-allocated smart pointer
- `std::rc::Rc` - Reference-counted pointer
- `std::sync::Arc` - Thread-safe reference-counted pointer

**Error Handling**:
- `std::result::Result` - Fallible operation result
- `std::option::Option` - Optional value

**I/O Operations**:
- `std::io::{Read, Write, BufRead}` - I/O traits
- `std::io::{stdin, stdout, stderr}` - Standard streams
- `std::fs::File` - File operations
- `std::fs::{read, write, read_to_string}` - Convenience functions

**Concurrency**:
- `std::thread::spawn` - Create threads
- `std::sync::{Mutex, RwLock}` - Synchronization primitives
- `std::sync::Arc` - Thread-safe shared ownership
- `std::marker::{Send, Sync}` - Thread safety markers

**Iteration**:
- `std::iter::Iterator` - Core iteration trait
- `std::iter::IntoIterator` - Conversion to iterator
- Range types in `std::ops` - `Range`, `RangeInclusive`, etc.

**System Interaction**:
- `std::env` - Environment variables, command-line args
- `std::fs` - File system operations
- `std::path::{Path, PathBuf}` - Path manipulation

**Memory & Pointers**:
- `std::mem::{size_of, align_of, drop}` - Memory introspection
- `std::ptr` - Raw pointer operations (unsafe)
- `std::cell::{Cell, RefCell}` - Interior mutability

**Traits**:
- `std::clone::Clone` - Explicit duplication
- `std::cmp::{PartialEq, Eq, PartialOrd, Ord}` - Comparison
- `std::default::Default` - Default values
- `std::fmt::{Display, Debug}` - Formatting
- `std::convert::{From, Into, AsRef, AsMut}` - Conversions
- `std::ops::*` - Operator overloading

### 13.5 Core vs. Std

Vertex has a two-tier library structure:

**`core`** (minimal, no allocation):
- Available in `no_std` environments
- Contains: primitives, Option, Result, basic traits
- No heap allocation, no I/O, no threading
- Suitable for embedded systems

**`std`** (full standard library):
- Builds on top of `core`
- Adds: Vec, String, HashMap, File I/O, threads
- Requires operating system support
- Default for normal applications

```vertex
// Using core (embedded/no_std)
#![no_std]
use core::option::Option

// Using std (normal programs)
use std::vec::Vec  // Includes everything from core
```

### 13.6 Common Import Presets

**File I/O Program**:
```vertex
use std::fs::File
use std::io::{Read, Write, BufReader, BufRead}
use std::path::Path
```

**Collections Program**:
```vertex
use std::collections::{HashMap, HashSet}
use std::vec::Vec
```

**Concurrent Program**:
```vertex
use std::thread
use std::sync::{Arc, Mutex}
```

**Command-Line Tool**:
```vertex
use std::env
use std::fs
use std::io::{self, Write}
use std::path::PathBuf
```

---

## Appendix A: Built-in Functions

```vertex
// Panic and assertion
fn panic(msg: &str) -> !
fn assert(condition: bool)
fn assert(condition: bool, msg: &str)
fn debug_assert(condition: bool)
fn debug_assert(condition: bool, msg: &str)

// Printing (built into compiler)
fn print(fmt: &str, args: ...)
fn println(fmt: &str, args: ...)
fn eprint(fmt: &str, args: ...)
fn eprintln(fmt: &str, args: ...)
fn format(fmt: &str, args: ...) -> String

// Memory
fn drop<T>(value: T)
```

## Appendix B: Type Aliases

```vertex
// Common type aliases in prelude
type Result<T, E> = Result<T, E>
type Option<T> = Option<T>
type String = String
type Vec<T> = Vec<T>
type Box<T> = Box<T>
```

## Appendix C: Prelude Contents

All items automatically imported into every module:

```vertex
// Types
Result, Option, String, Vec, Box, Rc, Arc

// Traits
Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord
Default, Display, Iterator, IntoIterator, Drop
From, Into, AsRef, AsMut

// Functions
print, println, eprint, eprintln, format
assert, debug_assert, panic, drop

// Enum variants
Ok, Err, Some, None

// Macros (built-in syntax)
vec!
```

---

This comprehensive API specification provides all the necessary details for implementing and using the Vertex standard library v1.0.

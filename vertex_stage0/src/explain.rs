const E0080_TEXT: &str = "\
E0080: evaluation of a constant value failed.

A constant expression performed an operation that cannot be evaluated at compile time, such as dividing by zero, indexing out of bounds, or overflowing an integer type. Vertex evaluates constants while compiling, so any panic in that evaluation is reported as E0080 at the constant's definition site rather than at runtime.

example:
    const N: i32 = 1 / 0;
    const A: [i32; 1] = [0];
    const X: i32 = A[5];

how to fix:
Change the constant so its evaluation always succeeds: avoid division or remainder by zero, keep array and slice indices within bounds, and use checked or saturating arithmetic when an operation may overflow. If the value really must be computed at run time, move it out of a `const` and into a regular `let` binding or a function call.";

const E0133_TEXT: &str = "\
E0133: call to unsafe function or use of an unsafe operation requires an `unsafe` block.

Some operations in Vertex are unsafe because the compiler cannot verify their soundness on its own: dereferencing raw pointers, calling functions marked `unsafe fn`, accessing mutable statics, and reading from unions. The caller has to acknowledge that responsibility by wrapping the call in an `unsafe { ... }` block or by marking the surrounding function `unsafe fn`.

example:
    unsafe fn dangerous() {}

    fn main() {
        dangerous();
    }

how to fix:
Wrap the call in an `unsafe` block once you have audited the safety preconditions:

    fn main() {
        unsafe { dangerous(); }
    }

If the caller itself can only be invoked in an unsafe context, mark it `unsafe fn` so the obligation propagates to its callers.";

const E0277_TEXT: &str = "\
E0277: a required trait bound is not satisfied for the given type.

A function, method, or generic item required that some type implement a particular trait, but the type you supplied does not. The compiler reports the trait it expected and the type that failed the bound, so the error message tells you exactly which `impl` is missing.

example:
    fn print_it<T: std::fmt::Display>(x: T) {
        println!(\"{}\", x);
    }

    struct Point { x: i32, y: i32 }

    fn main() {
        print_it(Point { x: 1, y: 2 });
    }

how to fix:
Either implement the required trait for your type (`impl Display for Point { ... }`), pick a type that already satisfies the bound, or relax the bound on the generic if the operation does not really need it. When the trait comes from another crate, derive it where possible (`#[derive(Debug, Clone)]`).";

const E0308_TEXT: &str = "\
E0308: mismatched types.

The compiler found a value of one type where it expected a value of a different, incompatible type. Vertex does not perform implicit conversions between distinct types, so an `i32` is not interchangeable with an `i64`, a `&str` is not a `String`, and `()` is not `i32`. The error message shows the expected type and the actual type at the offending expression.

example:
    fn main() {
        let x: i32 = \"hello\";
    }

how to fix:
Make the value's type match the expected one. Convert between numeric types with `as`, between owned and borrowed strings with `.to_string()` or `&s`, and between option-like wrappers with `Some(..)`/`Ok(..)`. When the function signature is wrong, change the declared return or parameter type to match the value you actually produce.";

const E0369_TEXT: &str = "\
E0369: a binary operation is not supported for the given operand types.

Operators such as `+`, `-`, `*`, `==`, and `<` are defined through traits (`Add`, `Sub`, `Mul`, `PartialEq`, `PartialOrd`, ...). If neither operand's type implements the matching trait for the other operand's type, the operator cannot be used and E0369 is reported.

example:
    struct Point { x: i32, y: i32 }

    fn main() {
        let a = Point { x: 1, y: 2 };
        let b = Point { x: 3, y: 4 };
        let c = a + b;
    }

how to fix:
Implement the relevant operator trait for your type (e.g. `impl Add for Point`), or convert the operands to types that already support the operation. For comparisons, derive `PartialEq` / `PartialOrd` when the field-wise definition is what you want.";

const E0382_TEXT: &str = "\
E0382: use of moved value.

When a value of a non-`Copy` type is assigned, passed by value, or returned, ownership moves to the new binding. The original binding is no longer valid, and any later use of it is rejected with E0382. This is the rule that prevents double-free and use-after-free bugs.

example:
    fn take(s: String) {}

    fn main() {
        let s = String::from(\"hi\");
        take(s);
        println!(\"{}\", s);
    }

how to fix:
Borrow the value with `&` or `&mut` instead of moving it, clone it with `.clone()` if you need an independent copy, or restructure the code so the value is used in only one place. For small types where copying is cheap, deriving `Copy` (and `Clone`) lets the compiler duplicate the value implicitly.";

const E0425_TEXT: &str = "\
E0425: cannot find a value, function, or other name in this scope.

The compiler tried to resolve an identifier and could not find any matching binding visible at the use site. Common causes are typos, forgetting to bring a name into scope with `use`, calling a method as a free function, or referring to a binding declared in a different block.

example:
    fn main() {
        let x = 1;
        println!(\"{}\", y);
    }

how to fix:
Check the spelling, add a `use` for items defined in another module, or declare the binding before you reference it. The compiler's \"did you mean ...\" hint usually points at the closest in-scope name and is worth taking seriously.";

const E0433_TEXT: &str = "\
E0433: failed to resolve a path during import or use.

A path such as `foo::bar::Baz` could not be resolved: one of its segments names a module, type, or item that does not exist or is not reachable from the current crate. This is most often caused by a misspelled module name, a missing `pub` on a re-export, or a missing dependency in `Cargo.toml`.

example:
    use std::collectionz::HashMap;

    fn main() {}

how to fix:
Verify each segment of the path: that the crate is declared as a dependency, that the module is reachable (intermediate modules are `pub` or you are in the same crate), and that the spelling matches. When importing from your own crate, prefer `crate::...` for absolute paths and `super::...` for paths relative to the parent module.";

const E0499_TEXT: &str = "\
E0499: cannot borrow as mutable more than once at a time.

Vertex's borrow checker allows at most one active mutable borrow of a value. Holding two `&mut` references to the same place at the same time would let either side observe the other's writes, defeating the aliasing guarantees that make `&mut T` sound, so it is rejected.

example:
    fn main() {
        let mut v = vec![1, 2, 3];
        let a = &mut v;
        let b = &mut v;
        a.push(4);
        b.push(5);
    }

how to fix:
Use one mutable borrow at a time: finish using the first reference (let it go out of scope, or do not use it again) before creating the second. When you genuinely need split mutable access, use a method that hands out disjoint mutable references (such as `split_at_mut`) or refactor so each borrow targets a different field.";

const E0502_TEXT: &str = "\
E0502: cannot borrow as mutable because it is also borrowed as immutable.

While a value has an outstanding shared borrow (`&T`), the borrow checker forbids creating a mutable borrow (`&mut T`) of the same value. Allowing both would let the mutable side change data that the immutable side is still observing, breaking the read-stability guarantee shared references rely on.

example:
    fn main() {
        let mut v = vec![1, 2, 3];
        let r = &v[0];
        v.push(4);
        println!(\"{}\", r);
    }

how to fix:
Sequence the borrows: stop using the immutable reference (let it go out of scope or consume it) before taking the mutable one. If you really need to read and modify at once, copy the relevant data out (`let value = v[0];`) and then mutate, or rework the algorithm so the read and the write target different places.";

const E0503_TEXT: &str = "\
E0503: cannot use a value because it was mutably borrowed.

While a `&mut` borrow of a value is live, that value cannot be read, copied, or otherwise used through any other path: the mutable borrow is exclusive. Touching the original binding while a mutable reference still exists is rejected with E0503.

example:
    fn main() {
        let mut x = 1;
        let r = &mut x;
        let y = x;
        *r += 1;
    }

how to fix:
Use the mutable reference for reads as well (`let y = *r;`) until it is no longer needed, or reorder the code so the mutable borrow ends before the original binding is read again. Restricting the borrow's scope by introducing an inner block is often enough.";

const E0505_TEXT: &str = "\
E0505: cannot move out of a value because it is borrowed.

Moving a value (assigning it elsewhere, returning it, or passing it by value) ends its lifetime at the source location. Doing so while another reference still points at it would leave that reference dangling, so the borrow checker rejects the move while any borrow is live.

example:
    fn main() {
        let s = String::from(\"hi\");
        let r = &s;
        let t = s;
        println!(\"{}\", r);
    }

how to fix:
Delay the move until after the borrow ends, clone the value if both bindings need to own data, or pass the original by reference rather than moving it. Shrinking the borrow's scope (often with an inner block) frequently resolves the conflict without any other change.";

const E0599_TEXT: &str = "\
E0599: no method named `...` found for the receiver type.

The compiler looked for a method on the receiver's type (and its trait `impl`s in scope) and found none with the requested name. This typically means the method does not exist on that type, the trait that defines it has not been imported, or a generic bound is missing the trait that provides it.

example:
    fn main() {
        let v: Vec<i32> = vec![1, 2, 3];
        v.frobnicate();
    }

how to fix:
Check the method's spelling, bring the defining trait into scope with `use`, or add the trait as a bound on the generic parameter (`T: SomeTrait`). When the method belongs to a different type, convert the receiver first (e.g. `s.as_str().parse()`).";

const E0608_TEXT: &str = "\
E0608: cannot index into a value of this type.

Indexing with `value[idx]` is only available for types that implement the `Index` trait. Trying to index a value whose type does not, such as a bare `str`, a tuple, or a custom struct without an `impl Index`, is rejected with E0608.

example:
    fn main() {
        let s: &str = \"hello\";
        let c = s[0];
    }

how to fix:
Use a type that supports the operation: `Vec<T>`, arrays, slices, and `HashMap<K, V>` all implement `Index`. For strings, iterate with `.chars()` or slice by byte range (`&s[0..1]`) instead of indexing. For custom types, implement `Index` yourself when indexing makes semantic sense.";

pub fn explain(code: &str) -> Option<&'static str> {
    let upper = code.to_ascii_uppercase();
    match upper.as_str() {
        "E0080" => Some(E0080_TEXT),
        "E0133" => Some(E0133_TEXT),
        "E0277" => Some(E0277_TEXT),
        "E0308" => Some(E0308_TEXT),
        "E0369" => Some(E0369_TEXT),
        "E0382" => Some(E0382_TEXT),
        "E0425" => Some(E0425_TEXT),
        "E0433" => Some(E0433_TEXT),
        "E0499" => Some(E0499_TEXT),
        "E0502" => Some(E0502_TEXT),
        "E0503" => Some(E0503_TEXT),
        "E0505" => Some(E0505_TEXT),
        "E0599" => Some(E0599_TEXT),
        "E0608" => Some(E0608_TEXT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_e0308_returns_text() {
        let text = explain("E0308").expect("E0308 should be registered");
        assert!(!text.is_empty());
        assert!(text.contains("mismatched types"));
        assert!(text.contains("E0308"));

        let lower = explain("e0308").expect("lowercase should also work");
        assert_eq!(text, lower);
    }

    #[test]
    fn explain_unknown_returns_none() {
        assert!(explain("E9999").is_none());
    }
}

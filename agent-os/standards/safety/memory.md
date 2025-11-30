# Memory Safety

## Overview

This document defines memory safety guidelines for Rust applications following Microsoft's guidelines. Rust's ownership system provides memory safety without garbage collection, but understanding these principles is essential for writing correct code.

## Ownership Fundamentals

### Ownership Rules

1. Each value has exactly one owner
2. When the owner goes out of scope, the value is dropped
3. Values can be moved or borrowed

```rust
fn main() {
    let s1 = String::from("hello");  // s1 owns the string

    let s2 = s1;  // Ownership moved to s2, s1 is invalid

    // println!("{}", s1);  // ERROR: s1 no longer valid

    println!("{}", s2);  // OK: s2 owns the string
}  // s2 goes out of scope, string is dropped
```

### Borrowing

```rust
fn main() {
    let s = String::from("hello");

    // Immutable borrow
    let len = calculate_length(&s);
    println!("{} has length {}", s, len);  // s still valid

    // Mutable borrow
    let mut s2 = String::from("hello");
    change(&mut s2);
}

fn calculate_length(s: &str) -> usize {
    s.len()
}

fn change(s: &mut String) {
    s.push_str(", world");
}
```

### Borrowing Rules

```rust
fn main() {
    let mut s = String::from("hello");

    // Multiple immutable borrows OK
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);

    // Mutable borrow after immutable borrows end
    let r3 = &mut s;
    println!("{}", r3);

    // ERROR: Can't have mutable and immutable at same time
    // let r4 = &s;     // immutable borrow
    // let r5 = &mut s; // mutable borrow - ERROR
}
```

## Clone and Copy

### Copy Types

Small, stack-allocated types implement `Copy`:

```rust
// Copy types are implicitly copied
let x = 5;
let y = x;  // Copy, both valid
println!("{} {}", x, y);

// Types that implement Copy:
// - All integer types (i32, u64, etc.)
// - bool
// - char
// - Floating point types (f32, f64)
// - Tuples of Copy types
// - Arrays of Copy types
```

### Clone Types

Heap-allocated types must be explicitly cloned:

```rust
let s1 = String::from("hello");
let s2 = s1.clone();  // Explicit clone
println!("{} {}", s1, s2);  // Both valid

// Derive Clone for custom types
#[derive(Clone)]
struct Data {
    value: String,
    count: i32,
}
```

### When to Clone

```rust
// GOOD: Clone when you need ownership
fn process(data: Data) {
    // Takes ownership
}

let data = load_data();
process(data.clone());  // Keep original
process(data);          // Move original

// BAD: Unnecessary cloning
fn display(data: &Data) {  // Takes reference
    println!("{:?}", data);
}

display(&data);  // Don't clone for reads
```

## Unsafe Guidelines (M-UNSAFE)

### When Unsafe is Valid

1. **Novel abstractions** - Smart pointers, allocators
2. **Performance** - After benchmarking proves necessity
3. **System calls** - Platform-specific operations

```rust
// VALID: Novel abstraction (implementing a data structure)
pub struct RingBuffer<T> {
    buffer: *mut T,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    // SAFETY: Documented why this is safe
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        // SAFETY: Caller must ensure index < capacity
        &*self.buffer.add(index)
    }
}
```

### When Unsafe is Invalid

```rust
// INVALID: Shortening safe code
// BAD: Using unsafe to avoid bounds checks
let slice = &[1, 2, 3];
let value = unsafe { *slice.get_unchecked(index) };  // Don't do this!

// GOOD: Use safe code
let value = slice.get(index);

// INVALID: Bypassing Send bounds
// BAD: Using unsafe to share non-Send types across threads

// INVALID: Bypassing lifetime requirements
// BAD: Using unsafe to extend lifetimes
```

### Unsafe Requirements

Every unsafe block must have:

1. **SAFETY comment** explaining why it's safe
2. **Invariants** documenting what must be true
3. **Miri testing** validating correctness

```rust
impl<T> Vec<T> {
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: We just checked that index is within bounds,
            // and self.ptr is valid for reads up to self.len elements.
            Some(unsafe { &*self.ptr.add(index) })
        } else {
            None
        }
    }
}
```

## Soundness (M-UNSOUND)

### All Code Must Be Sound

Unsound code is "safe" code that can cause undefined behavior. **Unsound code is never acceptable.**

```rust
// UNSOUND: Safe function that can cause UB
pub fn bad_transmute<T, U>(t: T) -> U {
    // This is unsound - safe code shouldn't allow this!
    unsafe { std::mem::transmute_copy(&t) }
}

// SOUND: Properly expose unsafe interface
pub unsafe fn transmute<T, U>(t: T) -> U {
    std::mem::transmute(t)
}
```

### Soundness Boundaries

```rust
// Module boundary is soundness boundary
mod internal {
    pub struct SafeWrapper {
        // Unsafe internals
        ptr: *mut u8,
    }

    impl SafeWrapper {
        // Safe public interface
        pub fn new() -> Self {
            Self {
                ptr: Box::into_raw(Box::new(0u8)),
            }
        }

        pub fn get(&self) -> u8 {
            // SAFETY: ptr was allocated in new() and is always valid
            unsafe { *self.ptr }
        }
    }

    impl Drop for SafeWrapper {
        fn drop(&mut self) {
            // SAFETY: ptr was allocated in new() with Box
            unsafe {
                let _ = Box::from_raw(self.ptr);
            }
        }
    }
}
```

## Unsafe Implies UB (M-UNSAFE-IMPLIES-UB)

Only mark functions `unsafe` if misuse causes undefined behavior:

```rust
// CORRECT: Unsafe because dereferencing raw pointer can cause UB
pub unsafe fn read_ptr(ptr: *const i32) -> i32 {
    *ptr
}

// INCORRECT: Unsafe for non-UB reasons
// BAD: This is dangerous but not UB-causing
pub unsafe fn delete_database() {  // Don't do this!
    std::fs::remove_dir_all("/data").unwrap();
}

// CORRECT: Use normal error handling for dangerous operations
pub fn delete_database() -> Result<(), std::io::Error> {
    std::fs::remove_dir_all("/data")
}
```

## Common Safety Patterns

### Interior Mutability

```rust
use std::cell::{Cell, RefCell};
use std::sync::{Mutex, RwLock};

// Single-threaded interior mutability
struct Counter {
    value: Cell<i32>,      // Copy types
    data: RefCell<Vec<u8>>, // Non-Copy types
}

// Thread-safe interior mutability
struct SharedState {
    value: Mutex<i32>,     // Exclusive access
    data: RwLock<Vec<u8>>, // Read-write access
}
```

### RAII (Resource Acquisition Is Initialization)

```rust
struct Connection {
    handle: RawHandle,
}

impl Connection {
    pub fn open(path: &str) -> Result<Self, Error> {
        let handle = unsafe { open_connection(path) }?;
        Ok(Self { handle })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Always clean up
        unsafe { close_connection(self.handle); }
    }
}
```

### Preventing Use-After-Free

```rust
pub struct Owner {
    data: Box<Data>,
}

impl Owner {
    pub fn data(&self) -> &Data {
        &self.data
    }

    // Takes ownership, preventing use-after-free
    pub fn into_data(self) -> Box<Data> {
        self.data
    }
}
```

## Arc and Rc

### Single-Threaded Reference Counting

```rust
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);
let data2 = Rc::clone(&data);  // Increment reference count

// Both point to same data
assert_eq!(data.len(), data2.len());
```

### Thread-Safe Reference Counting

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);

let data_clone = Arc::clone(&data);
std::thread::spawn(move || {
    println!("{:?}", data_clone);
});
```

### When to Use Each

```rust
// Rc: Single-threaded shared ownership
// - GUI widgets sharing state
// - Graph structures

// Arc: Multi-threaded shared ownership
// - Shared state across threads
// - Thread pools
// - Async tasks
```

## Best Practices Checklist

**Ownership:**
- [ ] Understand ownership transfer vs borrowing
- [ ] Prefer references over cloning
- [ ] Clone only when ownership is needed
- [ ] Use `Cow<T>` for clone-on-write scenarios

**Unsafe:**
- [ ] Only use unsafe when necessary
- [ ] Document SAFETY comments
- [ ] Minimize unsafe scope
- [ ] Test with Miri

**Soundness:**
- [ ] Never write unsound safe code
- [ ] Mark UB-causing functions as unsafe
- [ ] Keep soundness boundaries at module level

**Patterns:**
- [ ] Use RAII for resource management
- [ ] Use `Arc`/`Mutex` for thread-safe sharing
- [ ] Use `Rc`/`RefCell` for single-threaded sharing

## References

- [Rust Book - Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Miri](https://github.com/rust-lang/miri)

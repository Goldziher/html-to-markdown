______________________________________________________________________

## priority: high

# Memory Safety & Optimization Patterns

**Zero-copy APIs · RAII principle · Lifetime optimization · Automatic memory management**

## Zero-Copy Patterns

- **References over ownership**: Use `&T` and `&mut T` to avoid transfers; let Rust's borrow checker enforce safety
- **Borrowed data**: Prefer `&str` over `String`, `&[T]` over `Vec<T>` in function signatures
- **Cow<T>** (Copy-on-Write): Use for conditional ownership; clone only if modification needed
  - `Cow::Borrowed` for immutable access (zero cost)
  - `Cow::Owned` for owned data; switch to owned only when needed
- **Arc<T>** for shared immutable data: Reference-counted, thread-safe sharing without copying
- **Lifetime parameters**: Explicit lifetimes prevent premature deallocation; use `'a` to tie references

## RAII (Resource Acquisition Is Initialization)

- **Guard pattern**: Resources automatically released on scope exit; use `Drop` trait
- **Mutex/RwLock guards**: Automatic unlock on scope exit; prevents deadlocks and manual cleanup
- **File handles**: Automatic close via `Drop`; no manual file.close() needed
- **Scoped borrowing**: Resources released immediately after scope; no lingering references

### RAII Example

```rust
{
    let file = std::fs::File::open("data.txt")?;  // Acquire
    let mut reader = std::io::BufReader::new(file);
    // Use resource
}  // Automatically closed/dropped here

// With explicit guards
{
    let mut data = vec![1, 2, 3];
    let guard = std::sync::Mutex::new(&mut data);
    {
        let mut locked = guard.lock().unwrap();
        locked.push(4);
    }  // Unlock here automatically
}
```

## String Handling

- **String::new()**: Empty, zero-capacity; reserve space with `with_capacity()`
- **Cow<str>**: Use for conditional String/&str ownership
  - Avoid needless allocations when data might be borrowed
- **Arc<str>**: Cheap cloning for shared immutable strings across threads
- **String interning**: For repeated string comparisons, intern into static strings

### String Example

```rust
use std::borrow::Cow;

fn normalize_path(path: &str) -> Cow<'_, str> {
    if path.contains("//") {
        Cow::Owned(path.replace("//", "/"))
    } else {
        Cow::Borrowed(path)
    }
}

// Arc for sharing
let shared_name = std::sync::Arc::new("expensive_name".to_string());
let clone1 = std::sync::Arc::clone(&shared_name);  // Cheap clone
let clone2 = std::sync::Arc::clone(&shared_name);
```

## Buffer Reuse & Pooling

- **Vec::clear() + reuse**: Clear and reuse vector to avoid reallocations
- **Object pools**: Pre-allocate buffers; store in struct fields for reuse across calls
- **Split buffers**: Use `Vec::split_at_mut()` to avoid multiple allocations
- **SmallVec**: Stack-allocated for small data; heap fallback for larger

### Buffer Reuse Example

```rust
struct DataProcessor {
    input_buffer: Vec<u8>,
    output_buffer: Vec<u8>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            input_buffer: Vec::with_capacity(8192),
            output_buffer: Vec::with_capacity(8192),
        }
    }

    fn process(&mut self, data: &[u8]) -> &[u8] {
        self.output_buffer.clear();
        self.input_buffer.clear();
        self.input_buffer.extend_from_slice(data);

        // Process in-place
        for byte in &mut self.input_buffer {
            *byte = transform(*byte);
        }

        self.input_buffer.as_slice()
    }
}
```

## Lifetime Optimization

- **'static lifetimes**: For globals, thread-local data, long-lived borrowed references
- **Implicit elision**: Single input lifetime automatically applied to output (no annotation needed)
- **Struct lifetimes**: Minimize lifetimes in structs; simpler structs are easier to use
- **Function lifetimes**: Use `'_` (wildcard) for unused lifetimes in bounds

### Lifetime Example

```rust
// Avoid unnecessary lifetimes
struct BadParser<'a> {
    config: &'a str,  // Tied to parser lifetime
}

// Better: owned data or shorter borrowing
struct BetterParser {
    config: String,  // Owned, no lifetime
}

// Lifetime elision (single input → output)
fn parse(input: &str) -> &str {  // Output borrows from input
    &input[0..10]
}
```

## Tools for Safety Verification

- **Valgrind** (Linux): Detect memory leaks, use-after-free

  ```bash
  valgrind --leak-check=full --show-leak-kinds=all ./myapp
  valgrind --tool=massif ./myapp  # Heap profiler
  ```

- **AddressSanitizer (ASan)**: Compile-time instrumentation for memory errors

  ```bash
  RUSTFLAGS="-Z sanitizer=address" cargo +nightly build --target x86_64-unknown-linux-gnu
  ```

- **cargo-careful**: Run tests under Miri for undefined behavior detection

  ```bash
  cargo +nightly careful test
  MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test
  ```

- **Clippy**: Lint for lifetime issues

  ```bash
  cargo clippy --all-targets -- -D warnings
  ```

## Anti-Patterns

- **Clone instead of reference**: `fn process(data: Vec<u8>)` should be `&[u8]`; cloning is expensive and unnecessary
- **Unnecessary String allocations**: `format!()` when `.to_string()` or borrowing suffices
- **Holding guards across await points**: Mutex locks can deadlock; minimize borrow duration
- **Lifetime parameters without bounds**: Generic lifetimes should constrain actual relationships
- **Box<T> for simple owned data**: Unnecessary indirection; use direct ownership unless polymorphism needed
- **Ignoring Valgrind warnings**: Memory leaks snowball; fix immediately
- **No buffer pre-allocation**: Growing vectors repeatedly triggers reallocations; pre-allocate with capacity
- **'static everywhere**: Overly restrictive; use specific lifetimes for flexibility

## Best Practices Summary

1. **Default to references**: Function params should borrow when possible
1. **Use Cow for conditional ownership**: Avoid unnecessary clones
1. **Pre-allocate in hot loops**: Vec::with_capacity, String::with_capacity
1. **Profile before optimizing**: Use Valgrind/Instruments to find actual leaks
1. **Test with sanitizers**: AddressSanitizer + Miri catch hidden bugs
1. **Document lifetimes**: Explain non-obvious lifetime constraints in comments

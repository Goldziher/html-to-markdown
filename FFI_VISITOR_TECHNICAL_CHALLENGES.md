# FFI Visitor Adapter - Technical Challenges & Solutions

## Overview

This document details the specific technical challenges encountered when bridging C function pointers to Rust's HtmlVisitor trait, with concrete solutions for each.

## Challenge 1: BTreeMap<String, String> to C Arrays

### The Problem

Rust's `BTreeMap<String, String>` attributes cannot be directly passed to C. C expects:
- NULL-terminated array of key-value pairs
- Each string null-terminated
- Predictable memory layout for FFI

### Why It's Hard

1. **Multiple Allocation Levels**: BTreeMap contains Vec<Box<Node>>, each Node contains two Strings
2. **String Ownership**: Rust owns the data; must ensure C can't double-free or corrupt
3. **Lifetime Issues**: Temporary allocations must outlive the C callback but be freed after
4. **Layout**: BTreeMap's internal layout is unpredictable; cannot safely cast

### Solution Architecture

```
Rust BTreeMap              Intermediate Vec           C Array
┌──────────────┐          ┌──────────────┐          ┌─────────┐
│ "class"      │ clone    │ "class"      │ leak     │ ptr[0]  │ ─→ C string
│ "active"     │ ────────→│ "active"     │ ─────→  │ ptr[1]  │ ─→ C string
├──────────────┤          ├──────────────┤          ├─────────┤
│ "id"         │          │ "id"         │         │ ptr[2]  │ ─→ C string
│ "header"     │          │ "header"     │         │ ptr[3]  │ ─→ C string
└──────────────┘          └──────────────┘          └─────────┘
      iter()                forget Vec           heap allocated
                                                   (owns ptrs)
```

### Implementation Details

```rust
fn btreemap_to_c_array(
    attrs: &BTreeMap<String, String>,
) -> Result<(*mut CAttributePair, usize), String> {
    if attrs.is_empty() {
        return Ok((ptr::null_mut(), 0));  // NULL = no attributes
    }

    let mut c_attrs: Vec<CAttributePair> = Vec::with_capacity(attrs.len());

    for (key, value) in attrs.iter() {
        // Step 1: Convert each String to CString (validates no interior nulls)
        let key_c = CString::new(key.as_str())?;      // Allocates + validates
        let value_c = CString::new(value.as_str())?;   // Allocates + validates

        // Step 2: Convert CString to raw pointer (loses Rust ownership)
        // from_raw takes ownership of the data
        c_attrs.push(CAttributePair {
            key: key_c.into_raw() as *const c_char,
            value: value_c.into_raw() as *const c_char,
        });
    }

    // Step 3: Convert Vec to raw pointer (forget prevents Vec drop)
    let ptr = c_attrs.as_mut_ptr();
    let len = c_attrs.len();
    std::mem::forget(c_attrs);  // CRITICAL: prevents Vec drop + deallocation

    Ok((ptr, len))
}
```

### Critical Steps

1. **into_raw()**: Converts `CString` → `*mut c_char`, transferring ownership to Rust's FFI layer
2. **forget()**: Prevents Vec drop handler from deallocating the vector itself (not the Strings inside!)
3. **Cleanup**: `free_c_attributes_array` reconstructs the Vec to properly deallocate

### Pitfalls

| Mistake | Impact |
|---------|--------|
| Not calling `into_raw()` | CString dropped at end of loop; dangling pointers |
| Forgetting to `forget()` | Vec drop deallocates array; C receives dangling pointers |
| Using `Vec::leak()` | Cleaner but same effect as forget() |
| Not validating UTF-8 | Invalid UTF-8 in attributes causes silent corruption |

### Memory Lifecycle

```
Creation:
  BTreeMap (unchanged)
      ↓
  iter() + clone strings into Vec
      ↓
  into_raw() on each CString (ownership transfer)
      ↓
  Vec::forget() (array stays allocated)
      ↓
  Return (*mut CAttributePair, usize)

Usage:
  C callback receives pointer
      ↓
  Callback accesses fields via pointer arithmetic
      ↓
  Callback returns

Cleanup:
  free_c_attributes_array reconstructs Vec from raw parts
      ↓
  For each CAttributePair: CString::from_raw() (restore ownership)
      ↓
  CString drop (deallocates each string)
      ↓
  Vec drop (deallocates array itself)
```

## Challenge 2: NodeContext → CNodeContext Conversion

### The Problem

`NodeContext` contains:
- Rust String (heap-allocated, Rust-owned)
- BTreeMap<String, String> (complex structure)
- Enums (NodeType)
- Primitives (usize, bool)

C needs:
- Flat struct with pointers to C strings
- Predictable field layout
- No deep copies

### Why It's Hard

1. **Multiple String Fields**: tag_name, parent_tag must be independently allocated + freed
2. **Nested Structures**: Attributes require allocation of both array + strings
3. **Optional Fields**: parent_tag is Option<String> → NULL pointer in C
4. **Enum Conversion**: NodeType (61 variants) → u8 requires discriminant mapping
5. **Copy vs Move Semantics**: Attributes must be cloned (can't take ownership of &NodeContext)

### Solution

```rust
// Step 1: Convert each String individually
let tag_name_c = CString::new(ctx.tag_name.as_str())?;
let tag_name_ptr = tag_name_c.into_raw();  // Rust-owned C string

// Step 2: Handle Option<String>
let parent_tag_ptr = match &ctx.parent_tag {
    Some(name) => {
        let parent_c = CString::new(name.as_str())?;
        parent_c.into_raw() as *const c_char
    }
    None => ptr::null(),  // NULL = no parent
};

// Step 3: Convert complex field (attributes)
let (attrs_ptr, attrs_len) = btreemap_to_c_array(&ctx.attributes)?;

// Step 4: Construct CNodeContext with pointers
let c_ctx = CNodeContext {
    node_type: ctx.node_type as u8,
    tag_name: tag_name_ptr as *const c_char,
    attributes: attrs_ptr,
    attributes_len: attrs_len,
    depth: ctx.depth,
    index_in_parent: ctx.index_in_parent,
    parent_tag: parent_tag_ptr,
    is_inline: ctx.is_inline,
};

Ok(c_ctx)
```

### Cleanup

```rust
pub unsafe fn free_cnode_context(ctx: &CNodeContext) {
    // Free each string in reverse order of allocation

    // Free attributes array (this frees both keys and values)
    if !ctx.attributes.is_null() {
        free_c_attributes_array(ctx.attributes, ctx.attributes_len);
    }

    // Free tag_name
    if !ctx.tag_name.is_null() {
        drop(CString::from_raw(ctx.tag_name as *mut c_char));
    }

    // Free parent_tag if present
    if !ctx.parent_tag.is_null() {
        drop(CString::from_raw(ctx.parent_tag as *mut c_char));
    }
}
```

### Key Insight: LIFO Deallocation

Deallocate in **reverse order of allocation**:

```
Allocation Order           Deallocation Order
1. btreemap_to_c_array()   1. free_c_attributes_array()
2. tag_name_c              2. drop(tag_name_c)
3. parent_tag_c            3. drop(parent_tag_c)
```

This prevents use-after-free if an allocation fails mid-conversion.

## Challenge 3: Calling C Function Pointers from Rust

### The Problem

```rust
// What we have:
let callback: Option<extern "C" fn(*mut c_void, *const CNodeContext) -> c_int>;

// What can go wrong:
// 1. Callback panics
// 2. Callback dereferences invalid pointer
// 3. Callback writes to invalid memory
// 4. Callback never returns (infinite loop)
```

### Why It's Hard

1. **No Type Safety Across FFI**: C doesn't enforce pointer validity
2. **Panic Propagation**: Panics unwinding through C code cause undefined behavior
3. **Memory Corruption**: Invalid C code can corrupt heap/stack
4. **No Async Support**: C callbacks block the current thread
5. **Error Handling Mismatch**: C uses return codes; Rust uses Result/Option

### Solution 1: Panic Guard

```rust
fn call_c_function<F>(&mut self, callback_name: &str, f: F) -> Result<(), String>
where
    F: FnOnce() -> c_int + std::panic::UnwindSafe,
{
    // If previous callback panicked, all subsequent calls are no-ops
    if self.panic_occurred {
        return Err("visitor disabled due to previous callback panic".to_string());
    }

    // Catch panics without unwinding past FFI boundary
    match catch_unwind(f) {
        Ok(return_code) => {
            if return_code != 0 {
                Ok(())  // Return code 1 = success
            } else {
                Err(format!("callback {} returned error code 0", callback_name))
            }
        }
        Err(_) => {
            self.panic_occurred = true;  // Disable future callbacks
            Err(format!("callback {} panicked", callback_name))
        }
    }
}
```

### Solution 2: Error Code Convention

C callbacks use:
- Return code **1**: Success (continue with default behavior)
- Return code **0**: Error (visitor returns VisitResult::Error)

```c
int my_callback(void* user_data, const CNodeContext* ctx) {
    if (some_error_condition) {
        return 0;  // Error
    }
    return 1;     // Success
}
```

Rust side:

```rust
match self.call_c_function("visit_text", || {
    callback(self.user_data, &c_ctx, text_c.as_ptr())
}) {
    Ok(()) => VisitResult::Continue,    // Callback succeeded
    Err(msg) => VisitResult::Error(msg), // Callback failed or panicked
}
```

### Solution 3: panic_occurred Flag

Once any callback panics:
1. Set `panic_occurred = true`
2. All subsequent visitor methods become no-ops
3. Prevents cascading corruption from multiple panicking callbacks

```rust
// In visitor method:
if self.panic_occurred {
    return VisitResult::Continue;  // Skip callback
}

// ... call callback ...

// In call_c_function:
if self.panic_occurred {
    return Err("previous callback panicked");
}
```

## Challenge 4: NULL Pointer Safety

### The Problem

C pointers are inherently unsafe:
- May be NULL (uninitialized)
- May point to freed memory
- May point to misaligned memory
- May point to wrong data type

### Why It's Hard

1. **No Type Checking**: C compiler doesn't validate pointer types at runtime
2. **Silent Failures**: Dereferencing invalid pointer causes undefined behavior (not a panic)
3. **Security Risk**: Invalid pointers can be exploited
4. **Lifetime Bugs**: Pointer may be valid at callback time but invalid when callback returns

### Solution

```rust
// Step 1: Validate all input pointers before use
pub unsafe fn new(
    callbacks: *const CVisitorCallbacks,
    user_data: *mut c_void,
) -> Result<Self, String> {
    if callbacks.is_null() {
        return Err("callbacks pointer was null".to_string());
    }
    // user_data can be NULL (for no user context)

    // Only dereference after validation
    let callbacks = &*callbacks;
    // ...
}

// Step 2: Handle NULL string pointers in attributes
for (key, value) in attrs.iter() {
    let key_c = CString::new(key.as_str())?;
    let value_c = CString::new(value.as_str())?;

    c_attrs.push(CAttributePair {
        key: key_c.into_raw(),
        value: value_c.into_raw(),
        // These pointers are guaranteed non-NULL (CString ensures this)
    });
}

// Step 3: Accept NULL for optional fields
let title_ptr = title_c
    .as_ref()
    .map(|s| s.as_ptr())
    .unwrap_or(ptr::null());
// C callback receives NULL for missing optional field

// Step 4: Validate C-returned strings
unsafe fn c_visit_result_to_rust(c_result: CVisitResult) -> Result<VisitResult, String> {
    match c_result.kind {
        1 => {  // VISIT_CUSTOM
            if c_result.value.is_null() {
                Err("Custom result had null value pointer".to_string())
            } else {
                let rust_string = CStr::from_ptr(c_result.value)
                    .to_str()
                    .map(|s| s.to_string())?;  // Validates UTF-8
                Ok(VisitResult::Custom(rust_string))
            }
        }
        // ...
    }
}
```

### NULL Pointer Semantics

| Field | NULL Meaning |
|-------|--------------|
| callbacks | Invalid input (error) |
| user_data | No user context provided (valid, user_data not used) |
| tag_name in CNodeContext | Invalid (always present for real elements) |
| parent_tag in CNodeContext | Root element (no parent) |
| title in visit_link | No title attribute |
| id in visit_heading | No id attribute |
| value in CVisitResult | Invalid (error) |

## Challenge 5: Error Handling Across FFI

### The Problem

Rust uses:
- Result<T, E>
- Option<T>
- Exceptions (panics)

C uses:
- Return codes (0 = error, non-zero = success)
- errno global variable
- NULL pointers for errors

### Why It's Hard

1. **Impedance Mismatch**: Two error systems must interoperate
2. **No Exception Propagation**: Can't throw exceptions across FFI
3. **Context Loss**: C callback error reasons unclear to Rust
4. **Thread-Local State**: Multiple threads need independent error storage

### Solution 1: Return Code Convention

```rust
fn call_c_function<F>(&mut self, callback_name: &str, f: F) -> Result<(), String>
where
    F: FnOnce() -> c_int + std::panic::UnwindSafe,
{
    match catch_unwind(f) {
        Ok(return_code) => {
            // C convention: return_code != 0 means success
            if return_code != 0 {
                Ok(())
            } else {
                Err(format!("callback {} returned error code 0", callback_name))
            }
        }
        Err(_panic) => {
            self.panic_occurred = true;
            Err(format!("callback {} panicked", callback_name))
        }
    }
}
```

### Solution 2: Thread-Local Error Storage

```rust
thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(message: Option<String>) {
    LAST_ERROR.with(|cell| {
        let mut slot = cell.borrow_mut();
        *slot = message.and_then(|msg| CString::new(msg).ok());
    });
}

pub unsafe extern "C" fn html_to_markdown_last_error() -> *const c_char {
    LAST_ERROR.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|cstr| cstr.as_ptr())
            .unwrap_or(ptr::null())
    })
}
```

**Why thread-local?**
- Each thread has independent error state
- No synchronization overhead
- C callers check error immediately after function returns

### Solution 3: Result Variant Encoding

VisitResult variants encoded as discriminant + optional string:

```c
typedef enum {
    VISIT_CONTINUE = 0,        // Kind, no value
    VISIT_CUSTOM = 1,          // Kind=1, value=markdown
    VISIT_SKIP = 2,            // Kind, no value
    VISIT_PRESERVE_HTML = 3,   // Kind, no value
    VISIT_ERROR = 4,           // Kind=4, value=error_msg
} VisitResultKind;

typedef struct {
    uint8_t kind;
    char* value;  // Only used for VISIT_CUSTOM and VISIT_ERROR
} CVisitResult;
```

Conversion:

```rust
unsafe fn c_visit_result_to_rust(c_result: CVisitResult) -> Result<VisitResult, String> {
    match c_result.kind {
        0 => Ok(VisitResult::Continue),
        1 => {
            if c_result.value.is_null() {
                return Err("Custom result has null value".to_string());
            }
            let markdown = CStr::from_ptr(c_result.value)
                .to_str()?
                .to_string();
            Ok(VisitResult::Custom(markdown))
        }
        2 => Ok(VisitResult::Skip),
        3 => Ok(VisitResult::PreserveHtml),
        4 => {
            if c_result.value.is_null() {
                return Err("Error result has null message".to_string());
            }
            let msg = CStr::from_ptr(c_result.value)
                .to_str()?
                .to_string();
            Ok(VisitResult::Error(msg))
        }
        _ => Err(format!("unknown result kind: {}", c_result.kind)),
    }
}
```

## Challenge 6: Lifetime Management of Temporary Allocations

### The Problem

```rust
fn visit_link(&mut self, ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) {
    // Need to keep these alive for the callback duration:
    let c_ctx = node_context_to_c(ctx)?;      // Allocates strings + array
    let href_c = CString::new(href)?;
    let text_c = CString::new(text)?;
    let title_c = title.and_then(|t| CString::new(t).ok());

    // Call C callback (passes pointers)
    callback(...);

    // These must be freed AFTER callback returns
    // But BEFORE function exits (or pointers become invalid)
}
```

### Why It's Hard

1. **LIFO Constraint**: Resources must be freed in reverse order
2. **Exception Safety**: Panic between allocation and deallocation leaks memory
3. **Multiple Return Paths**: Early returns must still cleanup
4. **Nested Allocations**: Attributes array depends on Vec<CString>

### Solution: RAII (Resource Acquisition Is Initialization)

Rust automatically drops values when they go out of scope:

```rust
fn visit_link(
    &mut self,
    ctx: &NodeContext,
    href: &str,
    text: &str,
    title: Option<&str>,
) -> VisitResult {
    if let Some(callback) = self.visit_link_fn {
        // All allocations in this scope
        let c_ctx = match node_context_to_c(ctx) {
            Ok(c) => c,
            Err(e) => return VisitResult::Error(e),  // Early return
        };

        let href_c = match CString::new(href) {
            Ok(s) => s,
            Err(e) => {
                unsafe { free_cnode_context(&c_ctx); }  // Manual cleanup
                return VisitResult::Error(format!("href: {}", e));
            }
        };

        let text_c = match CString::new(text) {
            Ok(s) => s,
            Err(e) => {
                unsafe { free_cnode_context(&c_ctx); }  // Manual cleanup
                return VisitResult::Error(format!("text: {}", e));
            }
        };

        let title_c = title.and_then(|t| CString::new(t).ok());
        let title_ptr = title_c.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());

        // Call C callback with panic guard
        let result = unsafe {
            self.call_c_function("visit_link", || {
                callback(
                    self.user_data,
                    &c_ctx,
                    href_c.as_ptr(),
                    text_c.as_ptr(),
                    title_ptr,
                )
            })
        };

        // Cleanup (LIFO order)
        unsafe { free_cnode_context(&c_ctx); }
        drop(title_c);   // Deallocates if Some
        drop(text_c);    // Deallocates
        drop(href_c);    // Deallocates

        match result {
            Ok(()) => VisitResult::Continue,
            Err(e) => VisitResult::Error(e),
        }
    } else {
        VisitResult::Continue
    }
}
```

**Key Points**:
1. Allocations and cleanup are in same scope
2. Panic within callback is caught before cleanup
3. Cleanup happens in reverse order (drop order)
4. No manual try-catch needed (Rust panic safety)

## Challenge 7: Memory Ownership Across FFI Boundary

### The Problem

```
Rust owns:
- Input to C callback (pointers to Rust allocations)
- C callback cannot free this memory

C owns:
- Output from C callback (Custom/Error result strings)
- C allocated this memory
- Rust must not assume responsibility unless explicitly transferred

Mixed ownership:
- NodeContext.attributes: array allocated by Rust
- Each attribute key/value: allocated by Rust
- But C callback receives pointers and may read them
- C must NOT modify or free these allocations
```

### Why It's Hard

1. **Dual Responsibility**: Both Rust and C think they own parts of the same structure
2. **Use-After-Free**: If C frees Rust-owned memory, subsequent callbacks fail
3. **Double-Free**: If both free the same memory, heap corruption
4. **Dangling Pointers**: C keeps pointers after callback; they become invalid after cleanup

### Solution: Clear Ownership Model

```
┌─ Input Flow (Rust → C) ─────────────────────────────────┐
│                                                           │
│ Rust allocates:                                           │
│  ├─ CNodeContext (stack, Rust owns lifetime)             │
│  ├─ tag_name C string (heap, Rust owns)                  │
│  ├─ parent_tag C string (heap, Rust owns)                │
│  ├─ attributes array (heap, Rust owns)                   │
│  └─ Each key/value C string (heap, Rust owns)            │
│                                                           │
│ C callback receives pointers (READ-ONLY)                 │
│  ├─ Must not modify any pointed-to data                  │
│  ├─ Must not free any pointed-to data                    │
│  └─ Pointers only valid during callback execution        │
│                                                           │
│ Rust deallocates (after callback returns)                │
│                                                           │
└─────────────────────────────────────────────────────────┘

┌─ Output Flow (C → Rust) ────────────────────────────────┐
│                                                           │
│ C allocates (via malloc or similar):                      │
│  ├─ CVisitResult.value string (for Custom/Error)         │
│  └─ Must be valid UTF-8 C string                         │
│                                                           │
│ Rust receives pointer (takes ownership)                  │
│  ├─ Converts CString → Rust String                       │
│  └─ Will free the C allocation                           │
│                                                           │
│ Rust deallocates (via CString::from_raw + drop)          │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

### Documentation Requirements

Every public function must document ownership:

```rust
/// Visit a link element.
///
/// # Arguments
///
/// * `ctx`: NodeContext. Pointers within are Rust-owned and must not be
///          modified or freed by the callback. Valid only during callback.
/// * `href`: Link URL. Rust-owned. Callback must not free.
/// * `text`: Link text. Rust-owned. Callback must not free.
/// * `title`: Optional title. Rust-owned if non-NULL. Callback must not free.
///
/// # Return Value
///
/// C callback must allocate Custom/Error result strings via malloc.
/// Rust takes ownership and will free via free().
pub fn visit_link(
    &mut self,
    ctx: &NodeContext,
    href: &str,
    text: &str,
    title: Option<&str>,
) -> VisitResult
```

## Challenge 8: Thread Safety & Panic Propagation

### The Problem

```
Scenario 1: Callback called from different threads
  Thread A: conversion with visitor → callback invoked
  Thread B: different conversion with same visitor → undefined behavior!

Scenario 2: Callback blocks waiting for lock held by caller
  Thread A: html_to_markdown_convert_with_visitor(...)
            └─→ callback() [blocks waiting for lock]
            caller holds lock → DEADLOCK

Scenario 3: Callback panics
  C callback panics
    ↓
  panic unwinds toward FFI boundary
    ↓
  undefined behavior (FFI layer corrupted)
```

### Why It's Hard

1. **FfiVisitorAdapter Not Send**: Contains *mut c_void (not guaranteed thread-safe)
2. **No Mutex/RwLock**: Adding synchronization defeats performance
3. **C Library Limitations**: C callbacks may not be reentrant
4. **Deadlock Risk**: C callbacks may acquire locks that caller holds

### Solution 1: Single-Threaded Guarantee

Make FfiVisitorAdapter explicitly !Send + !Sync:

```rust
pub struct FfiVisitorAdapter {
    // ... fields ...

    // Prevent accidental Send/Sync (by containing PhantomData)
    _not_send_sync: std::marker::PhantomData<*mut ()>,
}

impl !Send for FfiVisitorAdapter {}
impl !Sync for FfiVisitorAdapter {}
```

Document:

```rust
/// FfiVisitorAdapter is NOT thread-safe.
///
/// # Safety
///
/// - Create one visitor per thread
/// - Do not share visitor between threads
/// - C callbacks are called from the thread invoking convert_with_visitor
///
/// # Example
///
/// ```c
/// // Thread A
/// visitor_a = html_to_markdown_visitor_new(&callbacks, context_a);
/// result_a = html_to_markdown_convert_with_visitor(html, visitor_a);
///
/// // Thread B (different visitor)
/// visitor_b = html_to_markdown_visitor_new(&callbacks, context_b);
/// result_b = html_to_markdown_convert_with_visitor(html, visitor_b);
/// ```
pub struct FfiVisitorAdapter { ... }
```

### Solution 2: Panic Guard

Use catch_unwind to prevent panic propagation:

```rust
fn call_c_function<F>(&mut self, callback_name: &str, f: F) -> Result<(), String>
where
    F: FnOnce() -> c_int + std::panic::UnwindSafe,
{
    if self.panic_occurred {
        return Err("previous callback panicked".to_string());
    }

    match catch_unwind(f) {
        Ok(return_code) => {
            if return_code != 0 { Ok(()) } else { Err(...) }
        }
        Err(_panic) => {
            self.panic_occurred = true;  // Disable future callbacks
            Err(format!("callback panicked"))
        }
    }
}
```

### Solution 3: Caller Responsibility

C caller must:
1. Create visitor for single thread only
2. Not hold locks when calling convert_with_visitor
3. Not call visitor from callback (no reentrancy)

Document prominently in C header:

```c
/// WARNING: Not thread-safe!
///
/// - Create one visitor per thread
/// - Do not call convert_with_visitor from callback
/// - Do not call convert_with_visitor while holding locks that callback acquires
html_to_markdown_visitor_t html_to_markdown_visitor_new(
    const CVisitorCallbacks* callbacks,
    void* user_data
);
```

## Summary Table

| Challenge | Root Cause | Solution |
|-----------|-----------|----------|
| BTreeMap → C arrays | Type mismatch | iter() + clone + into_raw() + forget() |
| NodeContext → CNodeContext | Nested structures | Per-field conversion + malloc tracking |
| C function pointers | No type safety | catch_unwind + return code convention |
| NULL pointers | FFI unvalidated | Check all inputs; NULL = optional field |
| Error handling | Exception vs return codes | Thread-local LAST_ERROR + result encoding |
| Temporary allocations | Lifetime issues | RAII + manual cleanup in reverse order |
| Ownership | Dual responsibility | Clear documentation of ownership model |
| Thread safety | Shared state | !Send + !Sync + per-thread visitor |

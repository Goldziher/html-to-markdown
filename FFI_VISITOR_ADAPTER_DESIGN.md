# FFI Visitor Adapter Architecture Design

## Executive Summary

This document describes the architecture for `crates/html-to-markdown-ffi/src/visitor/ffi.rs`, which bridges C function pointers to Rust's `HtmlVisitor` trait. The adapter enables language bindings (Go, Java, C#, etc.) to implement custom visitor callbacks during HTML-to-Markdown conversion.

## 1. Problem Statement

### Current State
- `crates/html-to-markdown/src/visitor.rs` defines the `HtmlVisitor` trait with 30+ virtual methods
- The FFI crate provides basic C FFI bindings for conversion
- No mechanism exists for C callers to provide custom visitor implementations

### Key Challenges
1. **Function Pointer Marshalling**: C uses raw function pointers; Rust uses trait objects
2. **Type Conversion**: `NodeContext` (Rust) ↔ `CNodeContext` (C); `BTreeMap<String, String>` attributes ↔ NULL-terminated C arrays
3. **Memory Ownership**: String lifetimes, attribute allocation/deallocation across FFI boundary
4. **Error Propagation**: Rust `Result<T, E>` ↔ C return codes with thread-local errors
5. **Thread Safety**: No built-in synchronization for C callbacks from Rust

## 2. Architecture Overview

### Module Structure

```
crates/html-to-markdown-ffi/src/
├── lib.rs                  # Main FFI entry points (existing)
├── conversion.rs           # Basic conversion (existing)
├── error.rs               # Error handling (existing)
├── strings.rs             # String utilities (existing)
├── profiling.rs           # Profiling (existing)
└── visitor/               # NEW MODULE
    ├── mod.rs             # Module exports + type definitions
    ├── ffi.rs             # FfiVisitorAdapter implementation
    ├── marshalling.rs     # C ↔ Rust type conversions
    └── callbacks.rs       # C callback function signatures
```

### Design Principles

1. **Minimum Overhead**: Zero-cost abstraction when no visitor is registered
2. **Memory Safety**: Rust manages all allocations; C caller owns user_data pointer
3. **Error Safety**: Failed callbacks propagate as `VisitResult::Error`
4. **Panic Safety**: C callbacks wrapped with panic guards
5. **Thread-Local Storage**: Each thread maintains independent visitor state

## 3. C Type Definitions

### C Callback Signatures

```c
// Forward declarations
typedef struct CNodeContext CNodeContext;
typedef struct CVisitResult CVisitResult;
typedef void* user_data_t;

// Attribute pair: key=value
typedef struct {
    const char* key;
    const char* value;
} CAttributePair;

// Node context passed to all callbacks
typedef struct CNodeContext {
    uint8_t node_type;           // NodeType enum value (0-60+)
    const char* tag_name;         // e.g., "div", "h1"
    CAttributePair* attributes;   // Array of key-value pairs
    size_t attributes_len;        // Number of attributes
    size_t depth;                 // DOM tree depth
    size_t index_in_parent;       // Zero-based position
    const char* parent_tag;       // NULL if root
    bool is_inline;               // Inline vs block
} CNodeContext;

// Visit result: encoded as discriminant + optional value
typedef struct {
    uint8_t kind;  // 0=Continue, 1=Custom, 2=Skip, 3=PreserveHtml, 4=Error
    char* value;   // Allocated by Rust for Custom/Error variants
} CVisitResult;

// Callback function types (return 0 on panic/error, 1 on success)
typedef int (*ffi_visit_element_start_t)(user_data_t, const CNodeContext*);
typedef int (*ffi_visit_element_end_t)(user_data_t, const CNodeContext*, const char*);
typedef int (*ffi_visit_text_t)(user_data_t, const CNodeContext*, const char*);
typedef int (*ffi_visit_link_t)(user_data_t, const CNodeContext*, const char* href,
                                const char* text, const char* title);
typedef int (*ffi_visit_image_t)(user_data_t, const CNodeContext*, const char* src,
                                 const char* alt, const char* title);
// ... (28+ more callback types, one per HtmlVisitor method)
```

### CVisitResult Encoding

To minimize memory overhead, results are encoded as:
- `kind=0`: `Continue` (no value)
- `kind=1`: `Custom` (value = markdown string, Rust-allocated)
- `kind=2`: `Skip` (no value)
- `kind=3`: `PreserveHtml` (no value)
- `kind=4`: `Error` (value = error message, Rust-allocated)

## 4. Rust Implementation

### 4.1 FfiVisitorAdapter Struct

```rust
/// Bridges C function pointers to the HtmlVisitor trait.
///
/// This struct stores C-compatible callback functions and user data,
/// implementing HtmlVisitor by dispatching to C callbacks.
///
/// # Safety Invariants
///
/// - All function pointers must point to valid, thread-safe C functions
/// - `user_data` is opaque; C caller is responsible for its validity
/// - All C strings passed to callbacks must be valid UTF-8 and null-terminated
/// - Attributes array must remain valid for the callback duration
/// - CVisitResult values with allocated strings must be freed by Rust
pub struct FfiVisitorAdapter {
    // Generic element hooks
    visit_element_start_fn: Option<extern "C" fn(*mut c_void, *const CNodeContext) -> c_int>,
    visit_element_end_fn: Option<extern "C" fn(*mut c_void, *const CNodeContext, *const c_char) -> c_int>,

    // Text nodes
    visit_text_fn: Option<extern "C" fn(*mut c_void, *const CNodeContext, *const c_char) -> c_int>,

    // Links and images
    visit_link_fn: Option<extern "C" fn(*mut c_void, *const CNodeContext, *const c_char, *const c_char, *const c_char) -> c_int>,
    visit_image_fn: Option<extern "C" fn(*mut c_void, *const CNodeContext, *const c_char, *const c_char, *const c_char) -> c_int>,

    // Headings, code, etc. (26+ more fields)
    // ...

    // User-provided opaque data passed to every callback
    user_data: *mut c_void,

    // Panic guard: if a callback panics, subsequent calls are no-ops
    panic_occurred: bool,
}

impl FfiVisitorAdapter {
    /// Create a new FFI visitor adapter from C callbacks.
    ///
    /// # Safety
    ///
    /// - All function pointers must point to valid C functions
    /// - `user_data` must remain valid for the duration of conversion
    /// - C callbacks must not access Rust data except through parameters
    pub unsafe fn new(
        callbacks: *const CVisitorCallbacks,
        user_data: *mut c_void,
    ) -> Result<Self, String> {
        if callbacks.is_null() {
            return Err("callbacks pointer was null".to_string());
        }

        Ok(FfiVisitorAdapter {
            visit_element_start_fn: (*callbacks).visit_element_start,
            visit_element_end_fn: (*callbacks).visit_element_end,
            visit_text_fn: (*callbacks).visit_text,
            visit_link_fn: (*callbacks).visit_link,
            visit_image_fn: (*callbacks).visit_image,
            // ... (copy all remaining fields)

            user_data,
            panic_occurred: false,
        })
    }

    /// Call a C callback safely, guarding against panics.
    ///
    /// This wrapper:
    /// 1. Detects panics and sets `panic_occurred` flag
    /// 2. Converts return codes (0=error/panic, 1=success)
    /// 3. Returns `Result<(), String>` for Rust error handling
    fn call_c_function<F>(&mut self, callback_name: &str, f: F) -> Result<(), String>
    where
        F: FnOnce() -> c_int + std::panic::UnwindSafe,
    {
        if self.panic_occurred {
            return Err("previous callback panicked; visitor is disabled".to_string());
        }

        match std::panic::catch_unwind(f) {
            Ok(return_code) => {
                if return_code == 0 {
                    Err(format!("callback {} failed", callback_name))
                } else {
                    Ok(())
                }
            }
            Err(_) => {
                self.panic_occurred = true;
                Err(format!("callback {} panicked", callback_name))
            }
        }
    }
}

impl Debug for FfiVisitorAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FfiVisitorAdapter")
            .field("user_data", &(self.user_data as usize))
            .field("panic_occurred", &self.panic_occurred)
            .finish()
    }
}
```

### 4.2 Type Conversion Functions

#### NodeContext → CNodeContext

```rust
/// Convert a Rust NodeContext to C representation.
///
/// # Memory Allocation Strategy
///
/// - tag_name: Rust string converted to C string (owned by allocation)
/// - parent_tag: Same as above
/// - attributes: Array allocated on heap; owned by conversion result
///
/// # Caller Responsibility
///
/// The returned CNodeContext must be freed via `free_cnode_context` after
/// the C callback completes.
fn node_context_to_c(ctx: &NodeContext) -> Result<CNodeContext, String> {
    // Convert tag_name
    let tag_name_c = string_to_c_string(ctx.tag_name.clone(), "tag_name")?;
    let tag_name_ptr = tag_name_c.into_raw();

    // Convert parent_tag
    let parent_tag_ptr = match &ctx.parent_tag {
        Some(name) => {
            let parent_c = string_to_c_string(name.clone(), "parent_tag")?;
            parent_c.into_raw()
        }
        None => ptr::null_mut(),
    };

    // Convert attributes BTreeMap to C array
    let (attrs_ptr, attrs_len) = btreemap_to_c_array(&ctx.attributes)?;

    Ok(CNodeContext {
        node_type: ctx.node_type as u8,
        tag_name: tag_name_ptr as *const c_char,
        attributes: attrs_ptr,
        attributes_len: attrs_len,
        depth: ctx.depth,
        index_in_parent: ctx.index_in_parent,
        parent_tag: parent_tag_ptr as *const c_char,
        is_inline: ctx.is_inline,
    })
}

/// Free a CNodeContext and its owned allocations.
///
/// # Safety
///
/// - `ctx` must be a valid pointer to CNodeContext created by node_context_to_c
/// - This must only be called once per context
/// - After this call, all pointers in the context are invalid
unsafe fn free_cnode_context(ctx: *mut CNodeContext) {
    if ctx.is_null() {
        return;
    }

    let ctx = &mut *ctx;

    // Free tag_name
    if !ctx.tag_name.is_null() {
        drop(CString::from_raw(ctx.tag_name as *mut c_char));
    }

    // Free parent_tag
    if !ctx.parent_tag.is_null() {
        drop(CString::from_raw(ctx.parent_tag as *mut c_char));
    }

    // Free attributes array
    free_c_attributes_array(ctx.attributes, ctx.attributes_len);
}

/// Convert BTreeMap<String, String> to NULL-terminated C array.
///
/// Structure:
/// ```
/// [
///   { key: "class", value: "active" },
///   { key: "id", value: "header" },
///   ...
/// ]
/// ```
///
/// # Returns
///
/// - Tuple of (pointer to array, length)
/// - Array is heap-allocated and must be freed via free_c_attributes_array
fn btreemap_to_c_array(
    attrs: &BTreeMap<String, String>,
) -> Result<(*mut CAttributePair, usize), String> {
    if attrs.is_empty() {
        return Ok((ptr::null_mut(), 0));
    }

    let mut c_attrs: Vec<CAttributePair> = Vec::with_capacity(attrs.len());

    for (key, value) in attrs.iter() {
        let key_c = string_to_c_string(key.clone(), "attribute key")?;
        let value_c = string_to_c_string(value.clone(), "attribute value")?;

        c_attrs.push(CAttributePair {
            key: key_c.into_raw() as *const c_char,
            value: value_c.into_raw() as *const c_char,
        });
    }

    let ptr = c_attrs.as_mut_ptr();
    let len = c_attrs.len();
    std::mem::forget(c_attrs); // Leak to caller; freed via free_c_attributes_array

    Ok((ptr, len))
}

/// Free a C attributes array created by btreemap_to_c_array.
unsafe fn free_c_attributes_array(attrs: *mut CAttributePair, len: usize) {
    if attrs.is_null() {
        return;
    }

    let slice = std::slice::from_raw_parts_mut(attrs, len);

    for attr in slice {
        if !attr.key.is_null() {
            drop(CString::from_raw(attr.key as *mut c_char));
        }
        if !attr.value.is_null() {
            drop(CString::from_raw(attr.value as *mut c_char));
        }
    }

    // Deallocate the array itself
    let layout = std::alloc::Layout::array::<CAttributePair>(len)
        .expect("attribute array layout");
    std::alloc::dealloc(attrs as *mut u8, layout);
}
```

#### CVisitResult → VisitResult

```rust
/// Convert a C visit result back to Rust.
///
/// # Memory Ownership
///
/// For Custom and Error variants, the C string `value` is owned by Rust
/// and must be freed after conversion.
unsafe fn c_visit_result_to_rust(c_result: CVisitResult) -> Result<VisitResult, String> {
    match c_result.kind {
        0 => Ok(VisitResult::Continue),
        1 => {
            // Custom variant: convert C string to owned String
            if c_result.value.is_null() {
                Err("Custom result had null value pointer".to_string())
            } else {
                let rust_string = CStr::from_ptr(c_result.value)
                    .to_str()
                    .map(|s| s.to_string())
                    .map_err(|_| "Custom value was invalid UTF-8".to_string())?;
                // Note: C caller is responsible for freeing c_result.value
                Ok(VisitResult::Custom(rust_string))
            }
        }
        2 => Ok(VisitResult::Skip),
        3 => Ok(VisitResult::PreserveHtml),
        4 => {
            // Error variant: convert C string to error message
            if c_result.value.is_null() {
                Err("Error result had null value pointer".to_string())
            } else {
                let error_msg = CStr::from_ptr(c_result.value)
                    .to_str()
                    .map(|s| s.to_string())
                    .map_err(|_| "Error value was invalid UTF-8".to_string())?;
                Ok(VisitResult::Error(error_msg))
            }
        }
        _ => Err(format!("unknown visit result kind: {}", c_result.kind)),
    }
}
```

### 4.3 HtmlVisitor Implementation Examples

#### visit_element_start

```rust
impl HtmlVisitor for FfiVisitorAdapter {
    fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Some(callback) = self.visit_element_start_fn {
            // Step 1: Convert Rust types to C types
            let c_ctx = match node_context_to_c(ctx) {
                Ok(c) => c,
                Err(e) => return VisitResult::Error(format!("context conversion failed: {}", e)),
            };

            // Step 2: Call C function with panic guard
            let result = unsafe {
                self.call_c_function("visit_element_start", || {
                    callback(self.user_data, &c_ctx)
                })
            };

            // Step 3: Free C allocations
            unsafe {
                free_cnode_context(&mut c_ctx as *mut CNodeContext);
            }

            // Step 4: Handle result
            match result {
                Ok(()) => VisitResult::Continue,
                Err(e) => VisitResult::Error(e),
            }
        } else {
            VisitResult::Continue
        }
    }
}
```

**Analysis**:
- Converts `&NodeContext` to `CNodeContext` (allocates strings + attributes)
- Calls C function pointer with error handling
- Frees all C allocations immediately
- Returns `VisitResult::Continue` if no callback registered

**Key Considerations**:
1. Panic guard in `call_c_function` prevents corruption
2. All C allocations freed before returning (exception-safe via RAII)
3. Conversion errors mapped to `VisitResult::Error`

#### visit_link

```rust
fn visit_link(&mut self, ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) -> VisitResult {
    if let Some(callback) = self.visit_link_fn {
        // Step 1: Convert context
        let c_ctx = match node_context_to_c(ctx) {
            Ok(c) => c,
            Err(e) => return VisitResult::Error(format!("context conversion failed: {}", e)),
        };

        // Step 2: Convert strings to C strings
        let href_c = match CString::new(href) {
            Ok(s) => s,
            Err(e) => {
                unsafe { free_cnode_context(&mut c_ctx as *mut CNodeContext); }
                return VisitResult::Error(format!("href contained null byte: {}", e));
            }
        };

        let text_c = match CString::new(text) {
            Ok(s) => s,
            Err(e) => {
                unsafe { free_cnode_context(&mut c_ctx as *mut CNodeContext); }
                return VisitResult::Error(format!("text contained null byte: {}", e));
            }
        };

        let title_c = title.and_then(|t| CString::new(t).ok());
        let title_ptr = title_c.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());

        // Step 3: Call C callback
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

        // Step 4: Free C allocations
        unsafe {
            free_cnode_context(&mut c_ctx as *mut CNodeContext);
        }
        drop(href_c);
        drop(text_c);
        drop(title_c);

        // Step 5: Handle result
        match result {
            Ok(()) => VisitResult::Continue,
            Err(e) => VisitResult::Error(e),
        }
    } else {
        VisitResult::Continue
    }
}
```

**Key Differences from visit_element_start**:
- Multiple string parameters converted to C strings
- Optional parameters (title) become NULL pointers
- `CString` values held until callback completes (RAII cleanup)
- All C strings freed in proper order

#### visit_heading

```rust
fn visit_heading(&mut self, ctx: &NodeContext, level: u32, text: &str, id: Option<&str>) -> VisitResult {
    if let Some(callback) = self.visit_heading_fn {
        let c_ctx = match node_context_to_c(ctx) {
            Ok(c) => c,
            Err(e) => return VisitResult::Error(format!("context conversion failed: {}", e)),
        };

        let text_c = match CString::new(text) {
            Ok(s) => s,
            Err(e) => {
                unsafe { free_cnode_context(&mut c_ctx as *mut CNodeContext); }
                return VisitResult::Error(format!("text contained null byte: {}", e));
            }
        };

        let id_c = id.and_then(|i| CString::new(i).ok());
        let id_ptr = id_c.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());

        let result = unsafe {
            self.call_c_function("visit_heading", || {
                callback(
                    self.user_data,
                    &c_ctx,
                    level as u32,
                    text_c.as_ptr(),
                    id_ptr,
                )
            })
        };

        unsafe {
            free_cnode_context(&mut c_ctx as *mut CNodeContext);
        }
        drop(text_c);
        drop(id_c);

        match result {
            Ok(()) => VisitResult::Continue,
            Err(e) => VisitResult::Error(e),
        }
    } else {
        VisitResult::Continue
    }
}
```

**Key Features**:
- Numeric parameters (level) passed directly
- String + optional parameters pattern
- Consistent error handling flow

#### visit_table_row (Complex: slice parameter)

```rust
fn visit_table_row(&mut self, ctx: &NodeContext, cells: &[String], is_header: bool) -> VisitResult {
    if let Some(callback) = self.visit_table_row_fn {
        let c_ctx = match node_context_to_c(ctx) {
            Ok(c) => c,
            Err(e) => return VisitResult::Error(format!("context conversion failed: {}", e)),
        };

        // Convert cells slice to C string array
        let (cells_array, cells_count) = {
            let mut c_cells: Vec<*const c_char> = Vec::with_capacity(cells.len());
            let mut cell_cstrings = Vec::with_capacity(cells.len());

            for cell in cells {
                match CString::new(cell.as_str()) {
                    Ok(c) => {
                        let ptr = c.as_ptr();
                        c_cells.push(ptr);
                        cell_cstrings.push(c);
                    }
                    Err(e) => {
                        unsafe { free_cnode_context(&mut c_ctx as *mut CNodeContext); }
                        return VisitResult::Error(format!("cell contained null byte: {}", e));
                    }
                }
            }

            (c_cells.as_ptr(), c_cells.len())
        };

        let result = unsafe {
            self.call_c_function("visit_table_row", || {
                callback(
                    self.user_data,
                    &c_ctx,
                    cells_array,
                    cells_count,
                    is_header as u8,
                )
            })
        };

        unsafe {
            free_cnode_context(&mut c_ctx as *mut CNodeContext);
        }

        match result {
            Ok(()) => VisitResult::Continue,
            Err(e) => VisitResult::Error(e),
        }
    } else {
        VisitResult::Continue
    }
}
```

**Key Challenges**:
- Convert slice to C array of pointers
- Keep CString objects alive for callback duration
- Proper cleanup of vector and owned strings

## 5. Memory Management Strategy

### Ownership Model

```
Rust → C Boundary Crossing:

NodeContext (Rust):
  ├─ tag_name ─[clone]→ CNodeContext.tag_name [C string, Rust-owned]
  ├─ parent_tag ─[clone]→ CNodeContext.parent_tag [C string, Rust-owned]
  ├─ attributes ─[convert]→ CNodeContext.attributes [C array, Rust-owned]
  │   └─ Each key/value pair ─[clone]→ C string [Rust-owned]
  └─ other fields ─[copy]→ CNodeContext fields [trivial types]

Callback Input Ownership:
- C receives pointers to Rust-owned allocations
- C must NOT free these allocations
- C must NOT access after callback returns

Callback Output Ownership:
- For Custom/Error results: C allocates the string
- Rust receives pointer and takes ownership
- Rust must free via html_to_markdown_free_string (in C code)
  OR free internally before returning to Rust caller
```

### Allocation Points

| Type | Allocator | Owner | Lifetime |
|------|-----------|-------|----------|
| tag_name | Rust (CString) | Rust | Until callback returns |
| parent_tag | Rust (CString) | Rust | Until callback returns |
| attributes array | Rust (Vec) | Rust | Until callback returns |
| attribute key | Rust (CString) | Rust | Until callback returns |
| attribute value | Rust (CString) | Rust | Until callback returns |
| Custom result | C (malloc) | Rust | Until converted to VisitResult |
| Error message | C (malloc) | Rust | Until converted to VisitResult |

### Deallocation Order (RAII)

```rust
// In visit_link example:
{
    let href_c = CString::new(href)?;     // Allocates
    let text_c = CString::new(text)?;     // Allocates
    let title_c = title.and_then(...)?;   // Allocates

    unsafe { callback(...) }               // Calls C

    drop(title_c);  // Deallocates (LIFO order preferred)
    drop(text_c);
    drop(href_c);
    unsafe { free_cnode_context(...) };   // Deallocates context
}
// All strings freed even if callback fails
```

RAII ensures deallocation happens even if callback returns an error.

## 6. Error Handling Strategy

### Error Propagation Paths

```
C Callback Error:
  callback returns 0 → call_c_function returns Err(msg)
  → visit_* returns VisitResult::Error(msg)
  → Conversion loop checks result and halts
  → Error message returned to C via last_error thread-local

Panic in Callback:
  panic caught by catch_unwind
  → panic_occurred flag set to true
  → call_c_function returns Err(panic message)
  → visit_* returns VisitResult::Error(panic message)
  → All subsequent callbacks are no-ops (fast path)

Null Pointer:
  callback function pointer is None
  → visit_* returns VisitResult::Continue immediately
  → No overhead for unimplemented callbacks

Type Conversion Error:
  CString::new fails (interior null byte)
  → callback not invoked
  → VisitResult::Error returned with context
  → Conversion halts gracefully
```

### Error Recovery

- **Panic Isolation**: Once a callback panics, visitor is disabled to prevent cascading failures
- **Partial Conversion**: If visitor returns error after partial success, content converted so far is discarded
- **C Error Inspection**: C caller checks `html_to_markdown_last_error()` after conversion fails

## 7. Thread Safety Considerations

### Thread-Local State

```rust
thread_local! {
    static VISITOR_STATE: RefCell<Option<Box<dyn HtmlVisitor>>> = const { RefCell::new(None) };
}
```

- Each thread has independent visitor instance
- No synchronization needed between threads
- C callers must NOT share visitor pointers across threads

### C-to-Rust Callbacks

```
C Thread A ────[call]───→ Rust (Tokio runtime on thread B)
                         └─→ visitor.visit_element_start(...)
                             └─→ call C callback from Rust
                                 └─→ potential deadlock if callback blocks on A
```

**Recommendation**: Document that C callbacks must not acquire locks held by C caller to prevent deadlock.

## 8. Module Definitions

### `visitor/mod.rs`

```rust
//! FFI visitor adapter for custom HTML→Markdown callbacks.
//!
//! This module bridges C function pointers to the Rust HtmlVisitor trait,
//! enabling language bindings (Go, Java, C#) to implement custom visitors.

pub mod ffi;
pub mod marshalling;
pub mod callbacks;

pub use ffi::FfiVisitorAdapter;
pub use callbacks::{CVisitorCallbacks, CNodeContext, CVisitResult, CAttributePair};
```

### `visitor/callbacks.rs`

```rust
//! C callback type definitions and structures for visitor pattern.

use std::os::raw::{c_char, c_int, c_void};

/// Visitor callback struct containing all function pointers.
#[repr(C)]
pub struct CVisitorCallbacks {
    pub visit_element_start: Option<extern "C" fn(*mut c_void, *const CNodeContext) -> c_int>,
    pub visit_element_end: Option<extern "C" fn(*mut c_void, *const CNodeContext, *const c_char) -> c_int>,
    pub visit_text: Option<extern "C" fn(*mut c_void, *const CNodeContext, *const c_char) -> c_int>,
    pub visit_link: Option<extern "C" fn(*mut c_void, *const CNodeContext, *const c_char, *const c_char, *const c_char) -> c_int>,
    pub visit_image: Option<extern "C" fn(*mut c_void, *const CNodeContext, *const c_char, *const c_char, *const c_char) -> c_int>,
    pub visit_heading: Option<extern "C" fn(*mut c_void, *const CNodeContext, u32, *const c_char, *const c_char) -> c_int>,
    // ... 24+ more fields
}

/// Node context passed to all callbacks.
#[repr(C)]
pub struct CNodeContext {
    pub node_type: u8,
    pub tag_name: *const c_char,
    pub attributes: *mut CAttributePair,
    pub attributes_len: usize,
    pub depth: usize,
    pub index_in_parent: usize,
    pub parent_tag: *const c_char,
    pub is_inline: bool,
}

/// Attribute key-value pair.
#[repr(C)]
pub struct CAttributePair {
    pub key: *const c_char,
    pub value: *const c_char,
}

/// Visit result with discriminant + optional value.
#[repr(C)]
pub struct CVisitResult {
    pub kind: u8,
    pub value: *mut c_char,
}
```

### `visitor/marshalling.rs`

```rust
//! Type conversion functions between Rust and C representations.

pub fn node_context_to_c(...) -> Result<CNodeContext, String> { ... }
pub unsafe fn free_cnode_context(...) { ... }
pub unsafe fn c_visit_result_to_rust(...) -> Result<VisitResult, String> { ... }
pub fn btreemap_to_c_array(...) -> Result<(*mut CAttributePair, usize), String> { ... }
pub unsafe fn free_c_attributes_array(...) { ... }
```

## 9. Public C FFI Functions

### Converting Visitor to FFI

```c
// Create a visitor from C callbacks
// Returns opaque visitor handle
html_to_markdown_visitor_t html_to_markdown_visitor_new(
    const CVisitorCallbacks* callbacks,
    void* user_data
);

// Free visitor handle
void html_to_markdown_visitor_free(html_to_markdown_visitor_t visitor);

// Convert HTML with visitor
// visitor can be NULL (use default behavior)
char* html_to_markdown_convert_with_visitor(
    const char* html,
    html_to_markdown_visitor_t visitor
);
```

## 10. Testing Strategy

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_context_conversion_round_trip() {
        // Create NodeContext → convert to C → verify fields
    }

    #[test]
    fn test_attributes_conversion_empty() {
        // Empty BTreeMap → NULL ptr
    }

    #[test]
    fn test_attributes_conversion_multiple() {
        // BTreeMap with 3 entries → verify array structure
    }

    #[test]
    fn test_callback_null_pointer() {
        // visit_element_start_fn = None → no callback invoked
    }

    #[test]
    fn test_panic_in_callback() {
        // Mock C callback that panics → panic_occurred set
    }

    #[test]
    fn test_callback_string_with_null_byte() {
        // CString::new fails → VisitResult::Error returned
    }

    #[test]
    fn test_attributes_interior_null_byte() {
        // Key or value contains null → conversion fails
    }
}
```

### Integration Tests (C FFI)

```c
// test_visitor_integration.c
void test_visitor_callback_called() {
    bool callback_invoked = false;
    CVisitorCallbacks callbacks = {
        .visit_element_start = mock_element_start
    };

    html_to_markdown_visitor_t visitor = html_to_markdown_visitor_new(&callbacks, &callback_invoked);
    char* markdown = html_to_markdown_convert_with_visitor("<h1>test</h1>", visitor);

    assert(callback_invoked == true);
    // ... verify markdown
}
```

## 11. Safety Checklist

| Concern | Mitigation |
|---------|-----------|
| Null pointer dereference | Check all input pointers; return errors |
| Use-after-free | RAII via CString/Vec; document lifetime boundaries |
| Memory leak | Track all allocations; free in reverse order |
| Panic in callback | catch_unwind wrapper; panic_occurred flag |
| Invalid UTF-8 | CStr::from_ptr validates; return Err on invalid |
| Interior null bytes | CString::new rejects; explicit error handling |
| Stack overflow (deep recursion) | Document max depth; log warnings at high depth |
| Concurrent access | Thread-local state; document threading constraints |
| Memory exhaustion | Track allocation sizes; return OOM errors |

## 12. Performance Considerations

### Call Overhead

Per callback invocation:
1. Rust → C conversion: O(n) where n = number of attributes (typically < 10)
2. C function call: O(1)
3. C → Rust conversion: O(1) for most variants, O(m) for Custom/Error strings
4. Allocation/deallocation: O(n) with RAII

Typical overhead: 1-5 µs per callback (benchmark to verify)

### Optimization Opportunities

1. **Batch Attributes**: Instead of individual CAttributePair allocations, use NULL-terminated key-value array
2. **String Interning**: Cache frequently-used strings (tag names)
3. **Zero-Copy Attributes**: Use slice of references instead of copying strings (requires C struct changes)
4. **Fast Path**: If no callbacks registered, skip conversion entirely

## 13. Example: Complete Flow

```
Input: <a href="https://example.com">Click here</a>
Visitor: Custom C callback for links

Flow:
1. Rust parser encounters <a> element
2. Conversion logic creates NodeContext:
   { node_type: Link, tag_name: "a", attributes: {"href": "...", ...}, ... }
3. Calls visitor.visit_link(&ctx, "https://example.com", "Click here", None)
4. FfiVisitorAdapter.visit_link:
   a. Converts NodeContext → CNodeContext (allocates strings + attributes array)
   b. Converts href, text to C strings
   c. Calls C callback: ffi_visit_link(user_data, &c_ctx, "https://example.com", "Click here", NULL)
   d. Frees all C allocations (except output string)
   e. Returns VisitResult to conversion loop
5. Conversion loop processes result:
   - If Continue: generate default markdown [Click here](https://example.com)
   - If Custom(output): use output as-is
   - If Skip: omit element
   - If Error(msg): halt conversion
6. Final markdown returned to caller
```

## 14. Future Enhancements

1. **Batch Callbacks**: Collect multiple elements before invoking callback for efficiency
2. **Visitor Chains**: Support multiple visitors (primary + fallback)
3. **Async Callbacks**: Allow C callbacks to perform async operations (requires event loop integration)
4. **Custom Allocator**: Allow C caller to provide malloc/free implementations
5. **Callback Profiling**: Track callback execution time for performance analysis

## 15. References

- Rust FFI Book: https://doc.rust-lang.org/nomicon/ffi.html
- C interop patterns: https://github.com/rust-lang/unsafe-code-guidelines
- HtmlVisitor trait: `/Users/naamanhirschfeld/workspace/html-to-markdown/crates/html-to-markdown/src/visitor.rs`

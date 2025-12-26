# Visitor FFI Implementation Guide

## Overview

This document provides a roadmap for implementing the visitor FFI bridge, based on the comprehensive architecture analysis in `FFI_ARCHITECTURE_REPORT.md` and code patterns in `FFI_QUICK_REFERENCE.md`.

---

## Phase 1: Design & Data Structures

### 1.1 Type Definitions for C Interop

Create new file: `crates/html-to-markdown-ffi/src/visitor_types.rs`

```rust
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

// Callback function types for visitor methods

/// Called when entering an element
/// Returns: 0 = Continue, 1 = Custom, 2 = Skip, 3 = PreserveHtml, 4 = Error
pub type VisitElementStartFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    tag: *const c_char,
    attr_keys: *const *const c_char,
    attr_values: *const *const c_char,
    attr_count: usize,
    node_type: c_int,
    depth: usize,
    index_in_parent: usize,
    parent_tag: *const c_char,
    is_inline: bool,
) -> c_int;

/// Called when exiting an element
pub type VisitElementEndFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    tag: *const c_char,
    output: *const c_char,
) -> c_int;

/// Called for text nodes
pub type VisitTextFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    text: *const c_char,
) -> c_int;

/// Called for links
pub type VisitLinkFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    href: *const c_char,
    text: *const c_char,
    title: *const c_char,  // NULL if no title
) -> c_int;

/// Called for images
pub type VisitImageFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    src: *const c_char,
    alt: *const c_char,
    title: *const c_char,  // NULL if no title
) -> c_int;

/// Called for headings
pub type VisitHeadingFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    level: c_int,  // 1-6
    text: *const c_char,
    id: *const c_char,  // NULL if no id
) -> c_int;

// ... more callback types for other visitor methods ...

// Callback struct: holds function pointers
#[repr(C)]
pub struct VisitorCallbacks {
    pub on_element_start: Option<VisitElementStartFn>,
    pub on_element_end: Option<VisitElementEndFn>,
    pub on_text: Option<VisitTextFn>,
    pub on_link: Option<VisitLinkFn>,
    pub on_image: Option<VisitImageFn>,
    pub on_heading: Option<VisitHeadingFn>,
    // ... more callbacks ...
}

// Opaque handle: visible to C but not inspectable
#[repr(C)]
pub struct VisitorHandle {
    // Completely private - C code cannot inspect
}

// Internal: not exposed to C
pub(crate) struct VisitorHandleInternal {
    callbacks: VisitorCallbacks,
    user_data: *mut c_void,
}
```

### 1.2 Result Enum for C

```rust
// Enum matching VisitResult from core library
#[repr(C)]
pub enum VisitResultC {
    Continue = 0,
    Custom = 1,
    Skip = 2,
    PreserveHtml = 3,
    Error = 4,
}

impl From<VisitResultC> for i32 {
    fn from(val: VisitResultC) -> Self {
        val as i32
    }
}

impl TryFrom<i32> for VisitResultC {
    type Error = String;

    fn try_from(val: i32) -> Result<Self, String> {
        match val {
            0 => Ok(VisitResultC::Continue),
            1 => Ok(VisitResultC::Custom),
            2 => Ok(VisitResultC::Skip),
            3 => Ok(VisitResultC::PreserveHtml),
            4 => Ok(VisitResultC::Error),
            n => Err(format!("invalid VisitResult discriminant: {n}")),
        }
    }
}
```

---

## Phase 2: Visitor Wrapper Implementation

Create new file: `crates/html-to-markdown-ffi/src/visitor.rs`

### 2.1 C-to-Rust Visitor Adapter

```rust
use std::cell::RefCell;
use std::rc::Rc;
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};

/// Wrapper that implements HtmlVisitor trait but calls C callbacks
struct CVisitorAdapter {
    callbacks: VisitorCallbacks,
    user_data: *mut c_void,
}

impl CVisitorAdapter {
    fn new(callbacks: VisitorCallbacks, user_data: *mut c_void) -> Self {
        CVisitorAdapter { callbacks, user_data }
    }

    /// Call a callback and marshal the result
    unsafe fn call_callback<F>(&self, f: F) -> VisitResult
    where
        F: FnOnce() -> c_int,
    {
        let result_code = f();
        match VisitResultC::try_from(result_code) {
            Ok(VisitResultC::Continue) => VisitResult::Continue,
            Ok(VisitResultC::Skip) => VisitResult::Skip,
            Ok(VisitResultC::PreserveHtml) => VisitResult::PreserveHtml,
            Ok(VisitResultC::Custom) => {
                // For Custom, would need output string via separate mechanism
                // Simplest: require callback to set error for now
                VisitResult::Error("Custom output not yet supported".to_string())
            }
            Ok(VisitResultC::Error) => {
                VisitResult::Error("Visitor callback returned error".to_string())
            }
            Err(e) => VisitResult::Error(format!("Invalid callback result: {e}")),
        }
    }
}

// Implement HtmlVisitor by delegating to C callbacks
impl HtmlVisitor for CVisitorAdapter {
    fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Some(callback) = self.callbacks.on_element_start {
            // Marshal NodeContext to C types
            let tag_c = match CString::new(ctx.tag_name.clone()) {
                Ok(s) => s,
                Err(_) => return VisitResult::Error("tag name contains null byte".to_string()),
            };

            // Marshal attributes
            let attr_keys: Vec<CString> = ctx.attributes.keys()
                .filter_map(|k| CString::new(k.clone()).ok())
                .collect();
            let attr_values: Vec<CString> = ctx.attributes.values()
                .filter_map(|v| CString::new(v.clone()).ok())
                .collect();

            let attr_key_ptrs: Vec<*const c_char> = attr_keys.iter()
                .map(|s| s.as_ptr())
                .collect();
            let attr_value_ptrs: Vec<*const c_char> = attr_values.iter()
                .map(|s| s.as_ptr())
                .collect();

            let parent_tag_c = ctx.parent_tag.as_ref()
                .and_then(|tag| CString::new(tag.clone()).ok());

            unsafe {
                self.call_callback(|| {
                    callback(
                        self.user_data,
                        tag_c.as_ptr(),
                        attr_key_ptrs.as_ptr(),
                        attr_value_ptrs.as_ptr(),
                        attr_keys.len(),
                        ctx.node_type as i32,
                        ctx.depth,
                        ctx.index_in_parent,
                        parent_tag_c.as_ref().map(|c| c.as_ptr()).unwrap_or(std::ptr::null()),
                        ctx.is_inline,
                    )
                })
            }
        } else {
            VisitResult::Continue
        }
    }

    fn visit_element_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.on_element_end {
            let tag_c = match CString::new(ctx.tag_name.clone()) {
                Ok(s) => s,
                Err(_) => return VisitResult::Error("tag name contains null byte".to_string()),
            };
            let output_c = match CString::new(output.to_string()) {
                Ok(s) => s,
                Err(_) => return VisitResult::Error("output contains null byte".to_string()),
            };

            unsafe {
                self.call_callback(|| {
                    callback(
                        self.user_data,
                        tag_c.as_ptr(),
                        output_c.as_ptr(),
                    )
                })
            }
        } else {
            VisitResult::Continue
        }
    }

    fn visit_text(&mut self, _ctx: &NodeContext, text: &str) -> VisitResult {
        if let Some(callback) = self.callbacks.on_text {
            let text_c = match CString::new(text.to_string()) {
                Ok(s) => s,
                Err(_) => return VisitResult::Error("text contains null byte".to_string()),
            };

            unsafe {
                self.call_callback(|| {
                    callback(self.user_data, text_c.as_ptr())
                })
            }
        } else {
            VisitResult::Continue
        }
    }

    fn visit_link(&mut self, _ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) -> VisitResult {
        if let Some(callback) = self.callbacks.on_link {
            let href_c = match CString::new(href.to_string()) {
                Ok(s) => s,
                Err(_) => return VisitResult::Error("href contains null byte".to_string()),
            };
            let text_c = match CString::new(text.to_string()) {
                Ok(s) => s,
                Err(_) => return VisitResult::Error("link text contains null byte".to_string()),
            };
            let title_c = title.and_then(|t| CString::new(t.to_string()).ok());

            unsafe {
                self.call_callback(|| {
                    callback(
                        self.user_data,
                        href_c.as_ptr(),
                        text_c.as_ptr(),
                        title_c.as_ref().map(|c| c.as_ptr()).unwrap_or(std::ptr::null()),
                    )
                })
            }
        } else {
            VisitResult::Continue
        }
    }

    // ... implement remaining visitor methods similarly ...
}
```

### 2.2 Visitor Creation & Cleanup

```rust
/// Create a visitor handle from callback pointers
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_visitor_create(
    callbacks: *const VisitorCallbacks,
    user_data: *mut c_void,
) -> *mut VisitorHandle {
    if callbacks.is_null() {
        set_last_error(Some("callbacks pointer was null".to_string()));
        return ptr::null_mut();
    }

    let cb = unsafe { *callbacks };

    // At least one callback should be provided
    if cb.on_element_start.is_none()
        && cb.on_element_end.is_none()
        && cb.on_text.is_none()
    {
        set_last_error(Some("at least one callback must be provided".to_string()));
        return ptr::null_mut();
    }

    let adapter = CVisitorAdapter::new(cb, user_data);
    let visitor: Rc<RefCell<dyn HtmlVisitor>> = Rc::new(RefCell::new(adapter));

    set_last_error(None);
    Box::into_raw(Box::new(VisitorHandle {
        visitor: Some(visitor),
    }))
}

/// Free a visitor handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_visitor_free(handle: *mut VisitorHandle) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle);
    }
}
```

---

## Phase 3: Integration with Conversion

### 3.1 Conversion Function with Visitor

Add to `crates/html-to-markdown-ffi/src/visitor.rs`:

```rust
/// Convert HTML to Markdown using a visitor
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_with_visitor(
    html: *const c_char,
    visitor: *mut VisitorHandle,
) -> *mut c_char {
    // Validation
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }

    if visitor.is_null() {
        set_last_error(Some("visitor pointer was null".to_string()));
        return ptr::null_mut();
    }

    // Convert C string to Rust string
    let html_str = match unsafe { CStr::from_ptr(html) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("html must be valid UTF-8".to_string()));
            return ptr::null_mut();
        }
    };

    // Extract visitor from handle
    let visitor_ref = unsafe { &mut *visitor };
    let visitor_clone = visitor_ref.visitor.clone();
    let mut visitor_mut = visitor_clone
        .borrow_mut();

    // Call core conversion with visitor
    match guard_panic(|| {
        profiling::maybe_profile(|| {
            // Get a mutable reference to the visitor
            // This is tricky due to trait object mutability
            // Use RefCell interior mutability
            html_to_markdown_rs::convert_with_visitor(html_str, None, Some(&mut visitor_mut))
        })
    }) {
        Ok(markdown) => {
            set_last_error(None);
            match string_to_c_string(markdown, "markdown result") {
                Ok(c_string) => c_string.into_raw(),
                Err(err) => {
                    set_last_error(Some(format!("failed to convert result: {err}")));
                    ptr::null_mut()
                }
            }
        }
        Err(err) => {
            capture_error(err);
            ptr::null_mut()
        }
    }
}
```

---

## Phase 4: Module Integration

### 4.1 Update lib.rs

```rust
// Add near the top
mod visitor_types;
pub use visitor_types::{
    VisitorCallbacks, VisitorHandle,
    VisitElementStartFn, VisitElementEndFn, VisitTextFn,
    VisitLinkFn, VisitImageFn, VisitHeadingFn,
};

mod visitor;
pub use visitor::{
    html_to_markdown_visitor_create,
    html_to_markdown_visitor_free,
    html_to_markdown_convert_with_visitor,
};

// Update Cargo.toml dependencies
[dependencies]
html-to-markdown-rs = {
    workspace = true,
    features = ["inline-images", "metadata", "visitor"]  # Add visitor
}
```

### 4.2 Update Cargo.toml Features

```toml
[features]
default = ["metadata", "visitor"]
metadata = ["html-to-markdown-rs/metadata"]
visitor = ["html-to-markdown-rs/visitor"]
profiling = ["dep:pprof"]
```

### 4.3 Update cbindgen.toml

```toml
[export]
include = [
    "html_to_markdown_convert",
    "html_to_markdown_free_string",
    "html_to_markdown_version",
    "html_to_markdown_last_error",
    # Visitor exports:
    "html_to_markdown_visitor_create",
    "html_to_markdown_visitor_free",
    "html_to_markdown_convert_with_visitor",
    # Types:
    "VisitorCallbacks",
    "VisitorHandle",
    "VisitElementStartFn",
    "VisitElementEndFn",
    "VisitTextFn",
    "VisitLinkFn",
    "VisitImageFn",
    "VisitHeadingFn",
]
```

---

## Phase 5: Testing

### 5.1 Unit Tests (in visitor.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Shared state for callbacks to record calls
    struct CallbackTracker {
        elements_visited: Vec<String>,
        text_visited: Vec<String>,
    }

    #[test]
    fn test_visitor_create_and_free() {
        unsafe {
            let mut callbacks = VisitorCallbacks {
                on_element_start: None,
                on_element_end: None,
                on_text: None,
                // ... rest of callbacks ...
            };

            let handle = html_to_markdown_visitor_create(&callbacks, std::ptr::null_mut());
            assert!(!handle.is_null());

            html_to_markdown_visitor_free(handle);
            // No crash on double-free:
            html_to_markdown_visitor_free(handle);
        }
    }

    #[test]
    fn test_visitor_null_callbacks() {
        unsafe {
            let handle = html_to_markdown_visitor_create(std::ptr::null(), std::ptr::null_mut());
            assert!(handle.is_null());
            let err = html_to_markdown_last_error();
            assert!(!err.is_null());
        }
    }

    #[test]
    fn test_convert_with_visitor_basic() {
        unsafe {
            let html = CString::new("<p>Hello</p>").unwrap();

            let tracker = Arc::new(Mutex::new(CallbackTracker {
                elements_visited: Vec::new(),
                text_visited: Vec::new(),
            }));

            let tracker_ptr = Box::into_raw(Box::new(tracker.clone()));

            let mut callbacks = VisitorCallbacks {
                on_element_start: Some(|user_data, tag, _, _, _, _, _, _, _, _| {
                    let tracker = user_data as *mut Arc<Mutex<CallbackTracker>>;
                    if !tracker.is_null() {
                        if let Ok(mut t) = (*tracker).lock() {
                            let tag_str = CStr::from_ptr(tag).to_str().unwrap_or("");
                            t.elements_visited.push(tag_str.to_string());
                        }
                    }
                    0  // Continue
                }),
                // ... other callbacks ...
                on_text: Some(|user_data, text| {
                    let tracker = user_data as *mut Arc<Mutex<CallbackTracker>>;
                    if !tracker.is_null() {
                        if let Ok(mut t) = (*tracker).lock() {
                            let text_str = CStr::from_ptr(text).to_str().unwrap_or("");
                            t.text_visited.push(text_str.to_string());
                        }
                    }
                    0  // Continue
                }),
                // ... rest of callbacks as None ...
            };

            let visitor = html_to_markdown_visitor_create(
                &callbacks,
                tracker_ptr as *mut c_void,
            );
            assert!(!visitor.is_null());

            let result = html_to_markdown_convert_with_visitor(html.as_ptr(), visitor);
            assert!(!result.is_null());

            html_to_markdown_free_string(result);
            html_to_markdown_visitor_free(visitor);

            let tracker_back = Box::from_raw(tracker_ptr);
            if let Ok(t) = tracker_back.lock() {
                assert!(t.elements_visited.contains(&"p".to_string()));
                assert!(t.text_visited.contains(&"Hello".to_string()));
            }
        }
    }
}
```

### 5.2 Integration Tests (new file: tests/visitor_ffi_test.rs)

```rust
use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};

#[test]
fn test_visitor_skip_element() {
    // Test VisitResult::Skip
}

#[test]
fn test_visitor_error_handling() {
    // Test VisitResult::Error
}

#[test]
fn test_visitor_preservehtml() {
    // Test VisitResult::PreserveHtml
}

#[test]
fn test_visitor_with_multiple_callbacks() {
    // Test multiple callbacks firing in correct order
}

#[test]
fn test_visitor_thread_safety() {
    // Test LAST_ERROR with visitors in multiple threads
}
```

---

## Phase 6: Documentation

### 6.1 C Header Comment Example

```c
/**
 * Create an HTML visitor with callback functions.
 *
 * This function creates an opaque visitor handle that can be used with
 * html_to_markdown_convert_with_visitor(). The callbacks are invoked
 * during HTML→Markdown conversion at appropriate points in the tree walk.
 *
 * # Arguments
 *
 * - `callbacks`: Pointer to VisitorCallbacks structure. At least one callback
 *   must be non-NULL. If NULL, returns NULL and sets an error.
 * - `user_data`: Opaque pointer passed to all callbacks. Can be NULL.
 *
 * # Returns
 *
 * Opaque visitor handle on success, NULL on failure. Check error with
 * html_to_markdown_last_error().
 *
 * # Safety
 *
 * - `callbacks` must be a valid pointer if not NULL
 * - Returned handle must be freed with html_to_markdown_visitor_free()
 * - Callbacks must not outlive the handle
 *
 * # Example
 *
 * ```c
 * VisitorCallbacks callbacks = {
 *     .on_text = my_text_handler,
 *     .on_link = my_link_handler,
 *     // ... other callbacks ...
 * };
 *
 * VisitorHandle* visitor = html_to_markdown_visitor_create(&callbacks, my_context);
 * if (visitor == NULL) {
 *     fprintf(stderr, "Error: %s\n", html_to_markdown_last_error());
 *     return;
 * }
 *
 * char* markdown = html_to_markdown_convert_with_visitor(html, visitor);
 * // ... use markdown ...
 * html_to_markdown_free_string(markdown);
 * html_to_markdown_visitor_free(visitor);
 * ```
 */
VisitorHandle *html_to_markdown_visitor_create(
    const VisitorCallbacks *callbacks,
    void *user_data
);
```

### 6.2 Examples Directory

Create `examples/visitor_callback.c`:

```c
#include <stdio.h>
#include "html_to_markdown.h"

// Global counter to track calls
static int text_count = 0;

int my_text_handler(void *user_data, const char *text) {
    text_count++;
    printf("Text %d: %s\n", text_count, text);
    return 0;  // Continue
}

int main() {
    const char* html = "<p>Hello <strong>world</strong></p>";

    VisitorCallbacks callbacks = {
        .on_text = my_text_handler,
        // Other callbacks NULL
    };

    VisitorHandle* visitor = html_to_markdown_visitor_create(&callbacks, NULL);
    if (visitor == NULL) {
        fprintf(stderr, "Failed to create visitor: %s\n", html_to_markdown_last_error());
        return 1;
    }

    char* result = html_to_markdown_convert_with_visitor(html, visitor);
    if (result == NULL) {
        fprintf(stderr, "Conversion failed: %s\n", html_to_markdown_last_error());
        html_to_markdown_visitor_free(visitor);
        return 1;
    }

    printf("Result:\n%s\n", result);
    printf("Processed %d text nodes\n", text_count);

    html_to_markdown_free_string(result);
    html_to_markdown_visitor_free(visitor);

    return 0;
}
```

---

## Checklist for Implementation

- [ ] Create `visitor_types.rs` with callback types and handles
- [ ] Create `visitor.rs` with CVisitorAdapter implementation
- [ ] Implement all 60+ visitor methods in CVisitorAdapter
- [ ] Add visitor module to lib.rs and module exports
- [ ] Update Cargo.toml with visitor feature
- [ ] Update Cargo.toml dependencies (add visitor feature to core)
- [ ] Update cbindgen.toml with visitor exports
- [ ] Add unit tests in visitor.rs
- [ ] Add integration tests in tests/
- [ ] Update documentation with C header examples
- [ ] Create example programs (C code showing visitor usage)
- [ ] Build and test: `cargo build -p html-to-markdown-ffi --features visitor`
- [ ] Generate and verify C header
- [ ] Cross-compile test (if targeting multiple platforms)
- [ ] Performance testing with profiling
- [ ] Documentation for language bindings (Go, Java, C#)

---

## Next Steps

1. **Review Architecture**: Ensure design aligns with project standards (see CLAUDE.md)
2. **Start with Phase 1**: Type definitions are foundation for all other phases
3. **Incremental Testing**: Implement and test one callback type at a time
4. **Document Thoroughly**: Update CLAUDE.md with visitor FFI guidelines
5. **Language Bindings**: After FFI layer complete, implement bindings for Go/Java/C#

---

## References

- `FFI_ARCHITECTURE_REPORT.md`: Complete architecture analysis
- `FFI_QUICK_REFERENCE.md`: Code patterns and examples
- `crates/html-to-markdown/src/visitor.rs`: Core visitor trait definition
- `CLAUDE.md`: Project guidelines and standards

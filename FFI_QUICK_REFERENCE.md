# FFI Quick Reference: Code Patterns & Examples

## Error Handling Template

```rust
// Template for FFI functions
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_some_function(
    input: *const c_char,
    output: *mut *mut c_char,
) -> *mut c_char {
    // 1. VALIDATE INPUTS
    if input.is_null() {
        set_last_error(Some("input pointer was null".to_string()));
        return ptr::null_mut();
    }
    if output.is_null() {
        set_last_error(Some("output pointer was null".to_string()));
        return ptr::null_mut();
    }

    // 2. CONVERT C STRING TO RUST
    let input_str = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("input must be valid UTF-8".to_string()));
            return ptr::null_mut();
        }
    };

    // 3. CALL CORE WITH PANIC GUARD & PROFILING
    match guard_panic(|| profiling::maybe_profile(|| core_operation(input_str))) {
        Ok(result) => {
            set_last_error(None);  // IMPORTANT: Clear errors on success

            // 4. CONVERT RUST STRING TO C STRING
            match string_to_c_string(result, "result") {
                Ok(c_string) => {
                    unsafe {
                        *output = c_string.into_raw();  // Transfer ownership
                    }
                    c_string.into_raw()  // Return result
                }
                Err(err) => {
                    set_last_error(Some(format!("failed to convert result: {err}")));
                    ptr::null_mut()
                }
            }
        }
        Err(err) => {
            capture_error(err);  // Capture panic or conversion error
            ptr::null_mut()
        }
    }
}
```

## String Marshalling Patterns

### Pattern 1: Simple C-String Output
```rust
// Caller provides HTML as C string
// Returns markdown as C string (caller must free)
pub unsafe extern "C" fn html_to_markdown_convert(html: *const c_char) -> *mut c_char {
    // ... validation and processing ...
    match string_to_c_string(markdown, "markdown result") {
        Ok(c_string) => c_string.into_raw(),  // Ownership -> caller
        Err(err) => {
            set_last_error(Some(format!("...{err}")));
            ptr::null_mut()
        }
    }
}
```

### Pattern 2: Output Parameter + Return Value
```rust
// Caller provides output pointer
// Function writes result via pointer, returns status
pub unsafe extern "C" fn html_to_markdown_function(
    input: *const c_char,
    output: *mut *mut c_char,
) -> bool {
    // ... processing ...
    match string_to_c_string(result, "result") {
        Ok(c_string) => {
            unsafe {
                *output = c_string.into_raw();  // Write via pointer
            }
            set_last_error(None);
            true  // Success
        }
        Err(err) => {
            set_last_error(Some(format!("...{err}")));
            false  // Failure
        }
    }
}
```

### Pattern 3: Length Reporting
```rust
// Return string + write length to output parameter
pub unsafe extern "C" fn html_to_markdown_convert_with_len(
    html: *const c_char,
    len_out: *mut usize,
) -> *mut c_char {
    if len_out.is_null() {
        set_last_error(Some("len_out pointer was null".to_string()));
        return ptr::null_mut();
    }

    // ... process ...
    match string_to_c_string(markdown, "markdown result") {
        Ok(c_string) => {
            unsafe {
                *len_out = c_string.as_bytes().len();  // Write length
            }
            c_string.into_raw()
        }
        Err(err) => {
            set_last_error(Some(format!("...{err}")));
            ptr::null_mut()
        }
    }
}
```

### Pattern 4: Dual Output (Markdown + Metadata JSON)
```rust
pub unsafe extern "C" fn html_to_markdown_convert_with_metadata(
    html: *const c_char,
    metadata_json_out: *mut *mut c_char,
) -> *mut c_char {
    if metadata_json_out.is_null() {
        set_last_error(Some("metadata_json_out pointer was null".to_string()));
        return ptr::null_mut();
    }

    // ... process and get (markdown, metadata) tuple ...

    // Convert metadata to JSON
    let metadata_json = match serde_json::to_vec(&metadata) {
        Ok(json) => json,
        Err(e) => {
            set_last_error(Some(format!("failed to serialize metadata: {}", e)));
            return ptr::null_mut();
        }
    };

    let metadata_c_string = match bytes_to_c_string(metadata_json, "metadata JSON") {
        Ok(s) => s,
        Err(err) => {
            set_last_error(Some(format!("failed to build CString: {err}")));
            return ptr::null_mut();
        }
    };

    unsafe {
        *metadata_json_out = metadata_c_string.into_raw();  // Write via pointer
    }

    // Convert markdown
    match string_to_c_string(markdown, "markdown result") {
        Ok(c_string) => {
            set_last_error(None);
            c_string.into_raw()  // Return markdown
        }
        Err(err) => {
            set_last_error(Some(format!("failed to convert markdown: {err}")));
            // CLEANUP: Free metadata on markdown error
            unsafe {
                if !metadata_json_out.is_null() && !(*metadata_json_out).is_null() {
                    drop(CString::from_raw(*metadata_json_out));
                    *metadata_json_out = ptr::null_mut();
                }
            }
            ptr::null_mut()
        }
    }
}
```

## Opaque Handle Pattern (for Visitor)

```rust
// Opaque handle for C FFI
#[repr(C)]
pub struct VisitorHandle {
    // Private fields - never exposed to C
}

// Create visitor from callbacks
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_visitor_create(
    on_element_start: VisitElementStartFn,
    on_text: VisitTextFn,
    user_data: *mut c_void,
) -> *mut VisitorHandle {
    if on_element_start.is_none() {
        set_last_error(Some("on_element_start callback is required".to_string()));
        return ptr::null_mut();
    }

    // Create visitor wrapper from callbacks
    let visitor = VisitorHandle {
        // ... construct internal visitor ...
    };

    set_last_error(None);
    Box::into_raw(Box::new(visitor))  // Return ownership to caller
}

// Use visitor in conversion
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_convert_with_visitor(
    html: *const c_char,
    visitor: *mut VisitorHandle,
) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }
    if visitor.is_null() {
        set_last_error(Some("visitor pointer was null".to_string()));
        return ptr::null_mut();
    }

    // ... extract visitor from handle ...
    let visitor_ref = unsafe { &mut *visitor };

    match guard_panic(|| {
        profiling::maybe_profile(|| {
            convert_with_visitor(html_str, visitor_ref.to_rust_visitor())
        })
    }) {
        Ok(markdown) => {
            set_last_error(None);
            match string_to_c_string(markdown, "markdown result") {
                Ok(c_string) => c_string.into_raw(),
                Err(err) => {
                    set_last_error(Some(format!("...{err}")));
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

// Free visitor handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn html_to_markdown_visitor_free(handle: *mut VisitorHandle) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle);  // Drop and deallocate
    }
}
```

## Byte Buffer API Pattern

```rust
pub unsafe extern "C" fn html_to_markdown_convert_bytes_with_len(
    html: *const u8,
    len: usize,
    len_out: *mut usize,
) -> *mut c_char {
    if html.is_null() {
        set_last_error(Some("html pointer was null".to_string()));
        return ptr::null_mut();
    }
    if len_out.is_null() {
        set_last_error(Some("len_out pointer was null".to_string()));
        return ptr::null_mut();
    }

    // Convert raw bytes to &str
    let html_bytes = unsafe { slice::from_raw_parts(html, len) };
    let html_str = match std::str::from_utf8(html_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(Some("html must be valid UTF-8".to_string()));
            return ptr::null_mut();
        }
    };

    // ... process as normal ...
}
```

## Error Checking in C

```c
// Typical C consumer code
const char* html = "<h1>Hello</h1>";
char* result = html_to_markdown_convert(html);

if (result == NULL) {
    // Error occurred - retrieve message
    const char* error = html_to_markdown_last_error();
    fprintf(stderr, "Conversion failed: %s\n", error ? error : "unknown error");
    // error pointer valid only until next FFI call, so copy if needed
    return;
}

// Use result
printf("%s\n", result);

// MUST free result
html_to_markdown_free_string(result);
```

## Null Pointer Checks

```rust
// Always check at function entry
if html.is_null() {
    set_last_error(Some("html pointer was null".to_string()));
    return ptr::null_mut();
}

// For output parameters
if output_ptr.is_null() {
    set_last_error(Some("output pointer was null".to_string()));
    return ptr::null_mut();
}

// For length parameters (can be 0)
// No null check needed - usize is never null
```

## UTF-8 Validation

```rust
// Pattern 1: C string to Rust str
let rust_str = match unsafe { CStr::from_ptr(c_str) }.to_str() {
    Ok(s) => s,  // Valid UTF-8
    Err(_) => {
        set_last_error(Some("input must be valid UTF-8".to_string()));
        return ptr::null_mut();
    }
};

// Pattern 2: Raw bytes to Rust str
let rust_str = match std::str::from_utf8(byte_slice) {
    Ok(s) => s,
    Err(_) => {
        set_last_error(Some("input must be valid UTF-8".to_string()));
        return ptr::null_mut();
    }
};

// Pattern 3: Rust string to C string (already UTF-8)
match string_to_c_string(rust_string, "context") {
    Ok(c_string) => c_string.into_raw(),
    Err(err) => {
        set_last_error(Some(format!("failed to convert: {err}")));
        ptr::null_mut()
    }
}
```

## Memory Leak Prevention

```rust
// When returning new allocations
let c_string = string_to_c_string(result, "context")?;
c_string.into_raw()  // Ownership transfer to caller

// Caller must call:
// html_to_markdown_free_string(result);

// For multiple outputs, cleanup on partial failure
match string_to_c_string(markdown, "markdown") {
    Ok(md) => {
        match string_to_c_string(metadata, "metadata") {
            Ok(meta) => {
                unsafe {
                    *markdown_out = md.into_raw();
                    *metadata_out = meta.into_raw();
                }
            }
            Err(err) => {
                // Free markdown before returning error
                drop(md);
                set_last_error(Some(format!("failed to convert metadata: {err}")));
                return ptr::null_mut();
            }
        }
    }
    Err(err) => {
        set_last_error(Some(format!("failed to convert markdown: {err}")));
        return ptr::null_mut();
    }
}
```

## Feature Gating

```rust
// In src/lib.rs
#[cfg(feature = "metadata")]
mod metadata;

#[cfg(feature = "metadata")]
pub use metadata::{
    html_to_markdown_convert_with_metadata,
    html_to_markdown_convert_with_metadata_with_len,
};

// In Cargo.toml
[features]
default = ["metadata"]
metadata = ["html-to-markdown-rs/metadata"]
profiling = ["dep:pprof"]
```

## cbindgen Configuration Updates

```toml
# cbindgen.toml - add visitor exports

[export]
include = [
    "html_to_markdown_convert",
    "html_to_markdown_free_string",
    "html_to_markdown_version",
    "html_to_markdown_last_error",
    # NEW:
    "html_to_markdown_visitor_create",
    "html_to_markdown_visitor_free",
    "html_to_markdown_convert_with_visitor",
    "VisitorHandle",
    "NodeContext",
    "VisitResult",
]
```

## Common Mistakes to Avoid

1. **Forgetting to clear error on success**
   ```rust
   // WRONG:
   set_last_error(Some(err_msg));
   Ok(result)  // Error still set!

   // CORRECT:
   set_last_error(None);
   Ok(result)
   ```

2. **Not checking output pointers**
   ```rust
   // WRONG:
   *output = value;  // Might be null, crash!

   // CORRECT:
   if output.is_null() {
       set_last_error(Some("output pointer was null".to_string()));
       return ptr::null_mut();
   }
   *output = value;
   ```

3. **Calling drop instead of free from C**
   ```rust
   // WRONG: C code calls drop
   // Rust-owned memory not deallocated

   // CORRECT: C code calls html_to_markdown_free_string
   ```

4. **Not wrapping core library calls**
   ```rust
   // WRONG:
   let result = convert(html_str, None)?;  // Can panic!

   // CORRECT:
   match guard_panic(|| convert(html_str, None)) {
       Ok(result) => { /* ... */ }
       Err(err) => { /* handle panic */ }
   }
   ```

5. **Forgetting UTF-8 validation**
   ```rust
   // WRONG:
   let str = unsafe { CStr::from_ptr(c_str) }.to_str().unwrap();

   // CORRECT:
   let str = match unsafe { CStr::from_ptr(c_str) }.to_str() {
       Ok(s) => s,
       Err(_) => {
           set_last_error(Some("must be valid UTF-8".to_string()));
           return ptr::null_mut();
       }
   };
   ```

6. **Not handling dual-output cleanup**
   ```rust
   // WRONG: Only cleanup if both succeed
   // Risk: metadata allocated, markdown fails, metadata leaked

   // CORRECT: Cleanup non-null outputs on any error
   if let Err(e) = process_markdown() {
       if !metadata_json_out.is_null() && !(*metadata_json_out).is_null() {
           drop(CString::from_raw(*metadata_json_out));
       }
       return error;
   }
   ```

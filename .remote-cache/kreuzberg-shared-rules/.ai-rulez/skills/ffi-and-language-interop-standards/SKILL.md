______________________________________________________________________

## priority: critical

# FFI and Language Interop Standards

## C API Design Principles

FFI code is a **contract** between languages. Breaking this contract causes crashes. Take extra care.

### 1. Explicit Ownership & Lifetime

Every pointer must have clear ownership semantics. Use SAFETY comments.

```rust
/// Allocate string owned by caller
/// # Safety
/// Caller must call `htm2md_free_string()` to release memory
#[no_mangle]
pub unsafe extern "C" fn htm2md_new_string(text: *const c_char) -> *mut c_char {
    if text.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(text).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match CString::new(c_str) {
        Ok(cs) => cs.into_raw(),  // Transfer ownership to caller
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free string allocated by Rust
/// # Safety
/// - `ptr` must be allocated by htm2md_new_string()
/// - `ptr` must not be used after this call
/// - `ptr` must not be freed by caller after this call
#[no_mangle]
pub unsafe extern "C" fn htm2md_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);  // Reclaim ownership and drop
    }
}
```

### 2. Handle Pattern for Complex Types

Never expose raw Rust types across FFI. Use opaque handles.

```rust
use std::ffi::CStr;
use std::os::raw::c_char;

// Opaque handle = void* to callers, real type internally
#[repr(transparent)]
pub struct HtmlConverterHandle(*mut html_to_markdown::HtmlConverter);

impl HtmlConverterHandle {
    fn from_ptr(ptr: *mut html_to_markdown::HtmlConverter) -> Self {
        HtmlConverterHandle(ptr)
    }

    fn into_ptr(self) -> *mut html_to_markdown::HtmlConverter {
        self.0
    }

    fn as_ref(&self) -> Option<&html_to_markdown::HtmlConverter> {
        unsafe { self.0.as_ref() }
    }
}

/// Create converter
/// # Safety
/// Caller must call `htm2md_converter_free()` to release
#[no_mangle]
pub unsafe extern "C" fn htm2md_converter_new() -> HtmlConverterHandle {
    let converter = Box::new(html_to_markdown::HtmlConverter::new());
    HtmlConverterHandle::from_ptr(Box::into_raw(converter))
}

/// Get version string
/// # Safety
/// - `handle` must be valid converter from `htm2md_converter_new()`
/// - Returned string is valid until `htm2md_converter_free(handle)` called
#[no_mangle]
pub unsafe extern "C" fn htm2md_converter_version(
    handle: HtmlConverterHandle,
) -> *const c_char {
    match handle.as_ref() {
        Some(conv) => {
            let version = conv.version();
            // Danger: String on stack! Must use thread-local or static
            // BAD: Will be freed when function returns
            version.as_ptr() as *const c_char
        }
        None => std::ptr::null(),
    }
}
```

### 3. Null Pointer Safety

Always check for null pointers:

```rust
/// Convert HTML to Markdown
/// # Safety
/// - `handle` must be valid converter
/// - `html` must be null-terminated UTF-8 string, or NULL
/// - Returns NULL on error
#[no_mangle]
pub unsafe extern "C" fn htm2md_convert(
    handle: HtmlConverterHandle,
    html: *const c_char,
) -> *mut c_char {
    // Validate inputs
    let converter = match handle.as_ref() {
        Some(c) => c,
        None => return std::ptr::null_mut(),  // NULL handle error
    };

    if html.is_null() {
        return std::ptr::null_mut();  // NULL input error
    }

    let html_str = match CStr::from_ptr(html).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),  // Invalid UTF-8 error
    };

    // Perform operation
    match converter.convert(html_str) {
        Ok(result) => {
            CString::new(result)
                .ok()
                .map(|s| s.into_raw())
                .unwrap_or_else(|| std::ptr::null_mut())
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free converter
#[no_mangle]
pub unsafe extern "C" fn htm2md_converter_free(handle: HtmlConverterHandle) {
    if !handle.0.is_null() {
        let _ = Box::from_raw(handle.0);
    }
}
```

## cbindgen Usage

**cbindgen** automatically generates C headers from Rust FFI code.

**cbindgen.toml**:

```toml
language = "C"
header = "/* Auto-generated FFI header */"
include_guard = "HTML_TO_MARKDOWN_FFI_H"
namespace = "htm2md"
autogen_warning = "// This is auto-generated from Rust source"

[defines]
"target_pointer_width = 32" = "HTM2MD_32BIT"
"target_pointer_width = 64" = "HTM2MD_64BIT"

[export]
exclude = ["internal_*"]
mangle = { "prefix" = "" }
```

**Generate header**:

```bash
# Install cbindgen
cargo install cbindgen

# Generate C header
cbindgen --crate html-to-markdown-ffi -o include/htm2md.h

# Verify in CI
cargo-binstall cbindgen
cbindgen --verify --crate html-to-markdown-ffi
```

**Example generated header**:

```c
#ifndef HTML_TO_MARKDOWN_FFI_H
#define HTML_TO_MARKDOWN_FFI_H

#include <stdint.h>
#include <stddef.h>

typedef struct HtmlConverterHandle HtmlConverterHandle;

HtmlConverterHandle* htm2md_converter_new(void);

const char* htm2md_convert(HtmlConverterHandle* handle, const char* html);

void htm2md_converter_free(HtmlConverterHandle* handle);

void htm2md_free_string(char* ptr);

#endif
```

## Pointer Safety Contracts

### Rule 1: Single Ownership

Each pointer has exactly one owner responsible for freeing:

```rust
// BAD: Caller allocates, Rust frees, C uses? Disaster!
// Transfer of ownership unclear

// GOOD: Clear ownership
pub unsafe extern "C" fn html_to_markdown(
    html: *const c_char,  // Caller owns, Rust borrows (reads)
) -> *mut c_char {        // Rust owns returned string, caller must free
    // ...
}
```

### Rule 2: No Use-After-Free

Document when pointers become invalid:

```rust
/// Get error message from last operation
/// # Safety
/// - `handle` must be valid
/// - Pointer is valid only until next `htm2md_*()` call on same handle
/// - Caller must NOT free returned string
#[no_mangle]
pub unsafe extern "C" fn htm2md_last_error(
    handle: HtmlConverterHandle,
) -> *const c_char {
    // Return borrowed reference to internal error string
    // SAFE: Lives in Handle until next operation
}
```

### Rule 3: No Double-Free

```rust
// BAD: Returned string is static, don't free!
#[no_mangle]
pub unsafe extern "C" fn htm2md_get_version() -> *const c_char {
    b"1.0.0\0".as_ptr() as *const c_char  // DO NOT FREE
}

// GOOD: Clear documentation
/// Get static version string
/// # Safety
/// Returned pointer is static and must NOT be freed
#[no_mangle]
pub unsafe extern "C" fn htm2md_get_version() -> *const c_char {
    c"1.0.0".as_ptr()
}
```

## SAFETY Comments Best Practices

Every `unsafe` block needs a SAFETY comment explaining why it's safe:

```rust
// BAD: No explanation
unsafe {
    Box::from_raw(ptr)
}

// GOOD: Explains safety invariant
// SAFETY: `ptr` is guaranteed to be:
// - non-NULL (checked above)
// - previously allocated by htm2md_converter_new()
// - not yet freed (lifetime contract enforced by handle)
unsafe {
    Box::from_raw(ptr)
}

// GOOD: Complex case with multiple conditions
// SAFETY: Safe because:
// 1. `handle.as_ref()` checks pointer is valid and aligned
// 2. &self borrow prevents concurrent mutation
// 3. Returned slice references only data inside converter
// 4. Pointer lifetime is limited to handle's existence
unsafe {
    handle.as_ref().unwrap().get_data()
}
```

## Null Handling Conventions

Consistent null checking across all functions:

```rust
/// Convert HTML to Markdown (NULL-safe)
/// Returns NULL on error (including NULL input)
#[no_mangle]
pub unsafe extern "C" fn htm2md_convert(
    handle: HtmlConverterHandle,
    html: *const c_char,
) -> *mut c_char {
    // All NULL cases return NULL
    let converter = handle.as_ref()
        .map(|h| h)
        .unwrap_or_else(|| return std::ptr::null_mut())?;

    if html.is_null() {
        return std::ptr::null_mut();
    }

    // Process...
}
```

## Language-Specific FFI Integration

### Go FFI Example

```go
package htm2md

/*
#include "htm2md.h"
*/
import "C"
import "unsafe"

type Converter struct {
    handle *C.struct_HtmlConverterHandle
}

func NewConverter() *Converter {
    handle := C.htm2md_converter_new()
    return &Converter{handle: handle}
}

func (c *Converter) Convert(html string) (string, error) {
    // Convert Go string to C string
    cHtml := C.CString(html)
    defer C.free(unsafe.Pointer(cHtml))

    // Call C function
    cResult := C.htm2md_convert(c.handle, cHtml)
    if cResult == nil {
        return "", errors.New("conversion failed")
    }

    // Convert C string back to Go
    defer C.htm2md_free_string(cResult)
    result := C.GoString(cResult)

    return result, nil
}

func (c *Converter) Close() {
    if c.handle != nil {
        C.htm2md_converter_free(c.handle)
        c.handle = nil
    }
}
```

### Java Panama FFM Example (Java 21+)

```java
import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;

public class HtmlToMarkdown {
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup SYMBOLS = LINKER.defaultLookup();

    static {
        System.loadLibrary("html_to_markdown");
    }

    private static final MethodHandle HTM2MD_CONVERTER_NEW;
    private static final MethodHandle HTM2MD_CONVERT;
    private static final MethodHandle HTM2MD_CONVERTER_FREE;

    static {
        Arena arena = Arena.ofAuto();

        HTM2MD_CONVERTER_NEW = LINKER.downcallHandle(
            SYMBOLS.find("htm2md_converter_new").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS)
        );

        HTM2MD_CONVERT = LINKER.downcallHandle(
            SYMBOLS.find("htm2md_convert").orElseThrow(),
            FunctionDescriptor.of(
                ValueLayout.ADDRESS,  // return: char*
                ValueLayout.ADDRESS,  // handle
                ValueLayout.ADDRESS   // html
            )
        );

        HTM2MD_CONVERTER_FREE = LINKER.downcallHandle(
            SYMBOLS.find("htm2md_converter_free").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        );
    }

    public static MemorySegment newConverter() throws Throwable {
        return (MemorySegment) HTM2MD_CONVERTER_NEW.invoke();
    }

    public static String convert(MemorySegment handle, String html) throws Throwable {
        Arena arena = Arena.ofAuto();
        MemorySegment htmlSegment = arena.allocateUtf8String(html);

        MemorySegment result = (MemorySegment) HTM2MD_CONVERT.invoke(handle, htmlSegment);
        if (result.address() == 0) {
            throw new RuntimeException("Conversion failed");
        }

        String javaString = result.getUtf8String(0);
        return javaString;
    }

    public static void freeConverter(MemorySegment handle) throws Throwable {
        HTM2MD_CONVERTER_FREE.invoke(handle);
    }
}
```

## Testing FFI Boundaries

```rust
#[cfg(test)]
mod ffi_tests {
    use super::*;

    #[test]
    fn test_null_handle() {
        unsafe {
            let result = htm2md_convert(
                HtmlConverterHandle(std::ptr::null_mut()),
                b"test\0".as_ptr() as *const c_char,
            );
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_null_input() {
        unsafe {
            let handle = htm2md_converter_new();
            let result = htm2md_convert(handle, std::ptr::null());
            assert!(result.is_null());
            htm2md_converter_free(handle);
        }
    }

    #[test]
    fn test_memory_leak() {
        unsafe {
            let handle = htm2md_converter_new();
            let html = CString::new("<h1>Test</h1>").unwrap();
            let result = htm2md_convert(handle, html.as_ptr());
            htm2md_free_string(result);  // Must free result
            htm2md_converter_free(handle);  // Must free handle
            // Valgrind should show 0 leaks
        }
    }
}
```

## Anti-Patterns to Avoid

1. **Exposing Rust types directly**:

   ```rust
   // BAD: Callers can't safely use Rust internals
   pub struct HtmlConverter { ... }  // Opaque to C
   pub fn get_config(&self) -> &ConversionConfig { ... }

   // GOOD: Opaque handles only
   pub fn get_config(handle: HtmlConverterHandle) -> ConfigHandle { ... }
   ```

1. **Forgetting null checks**:

   ```rust
   // BAD: Assumes valid pointer
   pub unsafe extern "C" fn convert(html: *const c_char) -> *mut c_char {
       let s = CStr::from_ptr(html).to_str().unwrap();
   }

   // GOOD: Check all inputs
   pub unsafe extern "C" fn convert(html: *const c_char) -> *mut c_char {
       if html.is_null() { return std::ptr::null_mut(); }
       let s = match CStr::from_ptr(html).to_str() {
           Ok(s) => s,
           Err(_) => return std::ptr::null_mut(),
       };
   }
   ```

1. **Returning stack data**:

   ```rust
   // BAD: Buffer is freed after function returns
   pub unsafe extern "C" fn get_status() -> *const c_char {
       let status = String::from("OK");
       status.as_ptr() as *const c_char  // DANGLING POINTER
   }

   // GOOD: Return static or heap-allocated data
   pub unsafe extern "C" fn get_status() -> *const c_char {
       b"OK\0".as_ptr() as *const c_char
   }
   ```

## Cross-references to Related Skills

- **binding-crate-architecture-patterns**: Wrapper layer above FFI
- **rust-async-await-module-patterns**: Async code across FFI
- **error-handling-strategy**: Error propagation through FFI
- **memory-safety-optimization-patterns**: Zero-copy patterns in FFI

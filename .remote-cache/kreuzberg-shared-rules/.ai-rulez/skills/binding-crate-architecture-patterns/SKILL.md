______________________________________________________________________

## priority: high

# Binding Crate Architecture Patterns

## Overview

Binding crates expose Rust libraries to host languages. Each binding framework has distinct patterns, but shared principles apply across all:

1. **Minimal wrapper layer**: Call Rust → host language is glue only
1. **Type translation**: Convert host language ↔ Rust with clear mapping
1. **Error conversion**: Rust errors → native exceptions/error types
1. **Memory safety**: Respect language-specific ownership models
1. **Testing**: Language-native test suite, not just Rust tests

## Crate Naming Conventions

**Pattern**: `{lib}-{language}` or `{lib}-{shortcode}`

```
html-to-markdown-py        # PyO3 → Python
html-to-markdown-node      # NAPI-RS → Node.js/TypeScript
html-to-markdown-rb        # Magnus → Ruby
html-to-markdown-php       # ext-php-rs → PHP
html-to-markdown-wasm      # wasm-bindgen → WebAssembly
html-to-markdown-ffi       # C FFI → Go, Java, C#
```

**In Cargo.toml**:

```toml
[package]
name = "html-to-markdown-py"
version = "0.5.0"
description = "Python bindings for HTML to Markdown conversion"
```

## PyO3 Pattern (Python)

**Directory structure**:

```
crates/html-to-markdown-py/
├── src/
│   ├── lib.rs              # PyO3 module definition
│   ├── converter.rs        # Binding for HtmlConverter
│   ├── config.rs           # Binding for ConversionConfig
│   ├── error.rs            # Rust errors → Python exceptions
│   └── utils.rs            # Helper functions
├── Cargo.toml              # Must define pyo3 feature
└── python/
    └── html_to_markdown.pyi # Type stub for IDE support
```

**lib.rs pattern**:

```rust
use pyo3::prelude::*;
use html_to_markdown::HtmlConverter as RustConverter;

#[pyclass]
pub struct HtmlConverter {
    inner: RustConverter,
}

#[pymethods]
impl HtmlConverter {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustConverter::new(),
        }
    }

    fn convert(&self, html: &str) -> PyResult<String> {
        self.inner.convert(html)
            .map_err(|e| PyException::new_err(e.to_string()))
    }
}

#[pymodule]
fn html_to_markdown(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<HtmlConverter>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
```

**Error conversion**:

```rust
use pyo3::exceptions::{PyException, PyValueError};

fn to_py_error(err: html_to_markdown::Error) -> PyErr {
    match err {
        html_to_markdown::Error::Parse(e) => {
            PyValueError::new_err(format!("Parse error: {}", e))
        }
        html_to_markdown::Error::Io(e) => {
            PyException::new_err(format!("IO error: {}", e))
        }
    }
}
```

## NAPI-RS Pattern (Node.js/TypeScript)

**Directory structure**:

```
crates/html-to-markdown-node/
├── src/
│   ├── lib.rs              # NAPI module
│   ├── converter.rs        # Binding for HtmlConverter
│   ├── error.rs            # Error conversion
│   └── async_bridge.rs     # Tokio runtime integration
├── Cargo.toml              # napi feature
├── package.json            # npm package metadata
└── index.d.ts              # TypeScript type definitions
```

**lib.rs pattern**:

```rust
use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ThreadSafeFunction, ThreadsafeFunctionCallMode},
};

#[napi]
pub struct HtmlConverter {
    inner: html_to_markdown::HtmlConverter,
}

#[napi]
impl HtmlConverter {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: html_to_markdown::HtmlConverter::new(),
        }
    }

    #[napi]
    pub fn convert(&self, html: String) -> napi::Result<String> {
        self.inner.convert(&html)
            .map_err(|e| napi::Error::new(
                napi::Status::GenericFailure,
                e.to_string()
            ))
    }

    #[napi]
    pub fn convert_async(
        &self,
        html: String,
        callback: ThreadSafeFunction<String>,
    ) -> napi::Result<()> {
        // Spawn async task
        let converter = self.inner.clone();
        std::thread::spawn(move || {
            match converter.convert(&html) {
                Ok(result) => {
                    let _ = callback.call(Ok(result), ThreadsafeFunctionCallMode::NonBlocking);
                }
                Err(e) => {
                    let _ = callback.call(
                        Err(napi::Error::new(napi::Status::GenericFailure, e.to_string())),
                        ThreadsafeFunctionCallMode::NonBlocking,
                    );
                }
            }
        });
        Ok(())
    }
}
```

**Type definitions** (index.d.ts):

```typescript
export class HtmlConverter {
  constructor();
  convert(html: string): string;
  convertAsync(html: string, callback: (err: Error | null, result?: string) => void): void;
}
```

## Magnus Pattern (Ruby)

**Directory structure**:

```
crates/html-to-markdown-rb/
├── src/
│   ├── lib.rs
│   ├── converter.rs
│   ├── error.rs
│   └── config.rs
└── Cargo.toml              # magnus feature
```

**lib.rs pattern**:

```rust
use magnus::{define_class, method, prelude::*};

#[magnus::wrap(class = "HtmlToMarkdown::Converter")]
pub struct HtmlConverter(html_to_markdown::HtmlConverter);

#[magnus::wrap(class = "HtmlToMarkdown::ConversionError")]
pub struct ConversionError(String);

impl From<html_to_markdown::Error> for magnus::RError {
    fn from(e: html_to_markdown::Error) -> Self {
        magnus::RError::new(
            magnus::exception::standard_error(),
            e.to_string()
        )
    }
}

pub fn init() -> magnus::Result<()> {
    let class = define_class("HtmlToMarkdown", Default::default())?;
    let converter_class = define_class("Converter", Default::default())?;

    converter_class.define_method("new", method!(|| {
        HtmlConverter(html_to_markdown::HtmlConverter::new())
    }))?;

    converter_class.define_method("convert", method!(|conv: HtmlConverter, html: String| {
        conv.0.convert(&html).map_err(|e| e.into())
    }))?;

    Ok(())
}
```

## ext-php-rs Pattern (PHP)

**Directory structure**:

```
crates/html-to-markdown-php/
├── src/
│   ├── lib.rs
│   ├── converter.rs
│   ├── error.rs
│   └── zend.rs             # Zend API integration
└── Cargo.toml              # ext-php-rs feature
```

**lib.rs pattern**:

```rust
use ext_php_rs::prelude::*;

#[php_class]
pub struct HtmlConverter {
    inner: html_to_markdown::HtmlConverter,
}

#[php_impl]
impl HtmlConverter {
    pub fn new() -> Self {
        Self {
            inner: html_to_markdown::HtmlConverter::new(),
        }
    }

    pub fn convert(&self, html: &str) -> String {
        self.inner.convert(html)
            .unwrap_or_else(|e| format!("Error: {}", e))
    }
}

#[php_module]
pub mod html_to_markdown {
    use super::*;

    pub fn hello() -> String {
        "Hello from Rust".to_string()
    }
}
```

## FFI Pattern (C-compatible for Go, Java, C#)

**Use cbindgen to generate C headers from Rust**:

```toml
[package]
name = "html-to-markdown-ffi"
```

**cbindgen.toml**:

```toml
language = "C"
header = "/* html-to-markdown FFI */"
include_guard = "HTML_TO_MARKDOWN_FFI_H"
namespace = "htm2md"
```

**lib.rs pattern**:

```rust
// SAFETY: All FFI functions must be explicitly safe
// Document ownership and lifetime expectations

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct HtmlConverterHandle {
    ptr: *mut html_to_markdown::HtmlConverter,
}

/// Create a new converter instance
/// # Safety: Caller must eventually call htm2md_converter_free()
#[no_mangle]
pub unsafe extern "C" fn htm2md_converter_new() -> HtmlConverterHandle {
    let converter = Box::new(html_to_markdown::HtmlConverter::new());
    HtmlConverterHandle {
        ptr: Box::into_raw(converter),
    }
}

/// Convert HTML to Markdown
/// # Safety:
/// - `handle` must be a valid converter created by htm2md_converter_new()
/// - `html` must be null-terminated valid UTF-8
/// - Caller owns returned string; must call htm2md_free_string()
#[no_mangle]
pub unsafe extern "C" fn htm2md_convert(
    handle: &HtmlConverterHandle,
    html: *const c_char,
) -> *const c_char {
    if html.is_null() {
        return std::ptr::null();
    }

    let html_str = match CStr::from_ptr(html).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };

    let converter = &*(handle.ptr);
    match converter.convert(html_str) {
        Ok(result) => {
            let c_string = CString::new(result).unwrap_or_default();
            c_string.into_raw() // Caller must free
        }
        Err(_) => std::ptr::null(),
    }
}

/// Free resources
#[no_mangle]
pub unsafe extern "C" fn htm2md_converter_free(handle: HtmlConverterHandle) {
    if !handle.ptr.is_null() {
        let _ = Box::from_raw(handle.ptr);
    }
}

/// Free string returned by htm2md_convert()
#[no_mangle]
pub unsafe extern "C" fn htm2md_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}
```

## wasm-bindgen Pattern (WebAssembly)

**lib.rs pattern**:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct HtmlConverter {
    inner: html_to_markdown::HtmlConverter,
}

#[wasm_bindgen]
impl HtmlConverter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> HtmlConverter {
        HtmlConverter {
            inner: html_to_markdown::HtmlConverter::new(),
        }
    }

    #[wasm_bindgen]
    pub fn convert(&self, html: &str) -> Result<String, JsValue> {
        self.inner.convert(html)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

## Type Mapping Conventions

Common type mappings across frameworks:

| Rust | Python | Node.js | Ruby | PHP | C/FFI |
|------|--------|---------|------|-----|-------|
| `String` | `str` | `string` | `String` | `string` | `char*` |
| `&str` | `str` | `string` | `String` | `string` | `const char*` |
| `u64` | `int` | `BigInt` | `Integer` | `int` | `uint64_t` |
| `bool` | `bool` | `boolean` | `true/false` | `bool` | `bool` |
| `Result<T>` | Exception | Error thrown | raises | Exception | null/error code |
| `Option<T>` | `None/T` | `null/T` | `nil/T` | `null/T` | `NULL/T` |

## Error Conversion Best Practices

```rust
// Define conversion for your error type
impl From<html_to_markdown::Error> for PyErr {
    fn from(err: html_to_markdown::Error) -> Self {
        match err {
            html_to_markdown::Error::Parse(msg) => {
                PyValueError::new_err(format!("Parse failed: {}", msg))
            }
            html_to_markdown::Error::Io(e) => {
                PyIOError::new_err(e.to_string())
            }
            html_to_markdown::Error::InvalidConfig => {
                PyValueError::new_err("Invalid configuration")
            }
        }
    }
}
```

## Anti-Patterns to Avoid

1. **Exposing Rust internals**:

   ```rust
   // BAD: Users see Rust implementation details
   pub struct Converter(html_to_markdown::HtmlConverter);

   // GOOD: Opaque wrapper with clean API
   #[pyclass]
   pub struct Converter { inner: ... }
   ```

1. **Synchronous blocking in async contexts**:

   ```rust
   // BAD: Blocks Tokio runtime
   async fn convert_async(&self, html: String) -> Result<String> {
       self.inner.convert(&html)  // Blocking call!
   }

   // GOOD: Use blocking thread pool
   async fn convert_async(&self, html: String) -> Result<String> {
       tokio::task::spawn_blocking(move || {
           self.inner.convert(&html)
       }).await?
   }
   ```

1. **Memory ownership confusion**:

   ```rust
   // BAD: Dangling pointers in FFI
   pub unsafe extern "C" fn get_string() -> *const c_char {
       let s = String::from("hello");
       s.as_ptr() as *const c_char  // s is dropped!
   }

   // GOOD: Transfer ownership to caller
   pub unsafe extern "C" fn get_string() -> *mut c_char {
       CString::new("hello").unwrap().into_raw()
   }
   ```

1. **Missing error handling**:

   ```rust
   // BAD: Panics in FFI code
   pub fn convert(html: &str) -> String {
       self.converter.convert(html).unwrap()  // Can panic!
   }

   // GOOD: Return error codes/exceptions
   pub fn convert(html: &str) -> Result<String> {
       self.converter.convert(html)
   }
   ```

## Cross-references to Related Skills

- **ffi-and-language-interop-standards**: Memory safety, pointer handling
- **workspace-dependency-management**: Coordinating binding crates with core
- **polyglot-bindings**: Framework for all language bindings
- **error-handling-strategy**: Error propagation across FFI boundaries

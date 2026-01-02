# Language Binding Patterns

## Overview

This document defines the patterns, conventions, and best practices for implementing the 10 language bindings. Each binding is a thin wrapper around the Rust core, following consistent patterns to ensure API parity and maintainability.

## PyO3 Patterns (Python)

### Basic Structure

```rust
// bindings/python/src/lib.rs
use pyo3::prelude::*;

#[pymodule]
fn mylib(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<Config>()?;
    m.add_class::<Error>()?;
    Ok(())
}

// Binding for Config struct
#[pyclass]
#[derive(Clone)]
pub struct Config {
    inner: crate::Config,
}

#[pymethods]
impl Config {
    #[new]
    fn new(timeout_ms: u64, max_retries: u32) -> Self {
        Config {
            inner: crate::Config {
                timeout_ms,
                max_retries,
            }
        }
    }

    #[getter]
    fn timeout_ms(&self) -> u64 {
        self.inner.timeout_ms
    }

    #[setter]
    fn set_timeout_ms(&mut self, value: u64) {
        self.inner.timeout_ms = value;
    }
}
```

### Async Method Handling

```rust
// PyO3 async methods require special handling
#[pymethods]
impl Client {
    fn execute_async<'p>(
        &self,
        request: Request,
        py: Python<'p>,
    ) -> PyResult<&'p PyAny> {
        let client = self.clone();  // Requires Clone
        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.inner.execute(request.inner)
                .await
                .map(|r| Response { inner: r })
                .map_err(PyErr::from)
        })
    }

    #[pyo3(text_signature = "(request)")]
    fn execute(&self, request: Request) -> PyResult<Response> {
        // Blocking wrapper for sync-only contexts
        let result = pyo3_asyncio::tokio::block_on(
            self.inner.execute(request.inner)
        );
        result
            .map(|r| Response { inner: r })
            .map_err(PyErr::from)
    }
}
```

### Error Conversion

```rust
impl From<crate::Error> for PyErr {
    fn from(err: crate::Error) -> PyErr {
        match err {
            crate::Error::Timeout(msg) => {
                pyo3::exceptions::PyTimeoutError::new_err(msg)
            }
            crate::Error::InvalidConfig(msg) => {
                pyo3::exceptions::PyValueError::new_err(msg)
            }
            crate::Error::NetworkError(msg) => {
                pyo3::exceptions::PyRuntimeError::new_err(msg)
            }
            crate::Error::ParseError(msg) => {
                pyo3::exceptions::PyValueError::new_err(msg)
            }
            crate::Error::Internal(msg) => {
                pyo3::exceptions::PyRuntimeError::new_err(msg)
            }
        }
    }
}
```

## NAPI-RS Patterns (TypeScript/Node)

### Basic Structure

```rust
// bindings/typescript/src/lib.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct Config {
    pub timeout_ms: u64,
    pub max_retries: u32,
}

#[napi]
impl Config {
    #[napi(constructor)]
    pub fn new(timeout_ms: u64, max_retries: u32) -> Self {
        Config { timeout_ms, max_retries }
    }

    #[napi(getter)]
    pub fn get_timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    #[napi(setter)]
    pub fn set_timeout_ms(&mut self, value: u64) {
        self.timeout_ms = value;
    }
}
```

### Async Method Handling

```rust
#[napi]
impl Client {
    #[napi]
    pub async fn execute(&self, request: Request) -> Result<Response> {
        self.inner.execute(request.inner)
            .await
            .map(|r| Response { inner: r })
            .map_err(|e| Error::new(
                napi::Status::GenericFailure,
                e.to_string(),
            ))
    }

    // Callback-based async for older Node versions
    #[napi]
    pub fn execute_with_callback(
        &self,
        request: Request,
        callback: Function<(Option<String>, Option<Response>)>,
    ) -> Result<()> {
        let client = self.clone();
        std::thread::spawn(move || {
            let result = napi_asyncio::tokio::block_on(
                client.inner.execute(request.inner)
            );

            match result {
                Ok(response) => {
                    let _ = callback.call((None, Some(Response { inner: response })));
                }
                Err(e) => {
                    let _ = callback.call((Some(e.to_string()), None));
                }
            }
        });
        Ok(())
    }
}
```

## Magnus Patterns (Ruby)

### Basic Structure

```rust
// bindings/ruby/src/lib.rs
use magnus::{prelude::*, define_module, define_class, Error};

#[magnus::init]
fn init() -> Result<(), Error> {
    let module = define_module("Mylib")?;

    let config_class = define_class("Config", Default::new)?;
    config_class.define_singleton_method("new", magnus::function!(
        Config::new, 2
    ))?;
    module.define_class("Config", config_class)?;

    Ok(())
}

#[derive(Clone)]
pub struct Config {
    inner: crate::Config,
}

impl Config {
    fn new(timeout_ms: u64, max_retries: u32) -> Self {
        Config {
            inner: crate::Config { timeout_ms, max_retries },
        }
    }

    fn timeout_ms(&self) -> u64 {
        self.inner.timeout_ms
    }
}
```

### Async Method Handling

```rust
#[magnus::init]
fn init() -> Result<(), Error> {
    let client_class = define_class("Client", Default::new)?;

    // Async method returning a Promise
    client_class.define_method("execute_async", magnus::function!(
        Client::execute_async, 1
    ))?;

    Ok(())
}

impl Client {
    fn execute_async(&self, request: Request) -> Result<magnus::Value, Error> {
        let client = self.clone();
        let promise = magnus::eval(
            r#"Fiber.new { puts "executing" }"#
        )?;
        // Actual async handling via Fiber or Concurrent-ruby
        Ok(promise)
    }
}
```

## ext-php-rs Patterns (PHP)

### Basic Structure

```rust
// bindings/php/src/lib.rs
use ext_php_rs::prelude::*;

#[php_module]
pub mod mylib {
    use ext_php_rs::prelude::*;

    #[php_class(name = "Config")]
    pub struct Config {
        inner: crate::Config,
    }

    #[php_impl]
    impl Config {
        #[constructor]
        pub fn new(timeout_ms: i64, max_retries: i64) -> PhpResult<Self> {
            Ok(Config {
                inner: crate::Config {
                    timeout_ms: timeout_ms as u64,
                    max_retries: max_retries as u32,
                },
            })
        }

        pub fn get_timeout_ms(&self) -> i64 {
            self.inner.timeout_ms as i64
        }

        pub fn set_timeout_ms(&mut self, ms: i64) {
            self.inner.timeout_ms = ms as u64;
        }
    }
}
```

### Async Method Handling

```rust
#[php_impl]
impl Client {
    pub fn execute_async(&self, request: Request) -> PhpResult<()> {
        // PHP async typically requires returning a Promise
        // or using ext-async for true async support
        let result = std::thread::block_in_place(
            tokio::task::block_in_place,
            || {
                tokio::runtime::Handle::current()
                    .block_on(self.inner.execute(request.inner))
            },
        );

        result.map(|_| ()).map_err(|e| {
            PhpException::new(e.to_string(), 0)
        })
    }
}
```

## FFM API Patterns (Java 21+)

### Basic Structure

```rust
// bindings/java/src/lib.rs
use jni::prelude::*;
use jni::objects::{JClass, JString};
use jni::sys::{jstring, jlong};

#[no_mangle]
pub extern "system" fn Java_com_mylib_Config_new(
    env: JNIEnv,
    _class: JClass,
    timeout_ms: jlong,
    max_retries: jint,
) -> jlong {
    let config = crate::Config {
        timeout_ms: timeout_ms as u64,
        max_retries: max_retries as u32,
    };

    Box::into_raw(Box::new(config)) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_mylib_Config_getTimeoutMs(
    _env: JNIEnv,
    _class: JClass,
    config_ptr: jlong,
) -> jlong {
    unsafe {
        let config = &*(config_ptr as *const crate::Config);
        config.timeout_ms as jlong
    }
}
```

### Memory Management

```rust
// Proper cleanup with JNI
#[no_mangle]
pub extern "system" fn Java_com_mylib_Config_delete(
    _env: JNIEnv,
    _class: JClass,
    config_ptr: jlong,
) {
    unsafe {
        let _ = Box::from_raw(config_ptr as *mut crate::Config);
    }
}

// Safe wrapper with SAFETY comments
#[no_mangle]
pub extern "system" fn Java_com_mylib_Client_execute(
    env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
    request_ptr: jlong,
) -> jlong {
    // SAFETY: Pointer validity must be guaranteed by Java wrapper class
    unsafe {
        let client = &*(client_ptr as *const crate::Client);
        let request = *(request_ptr as *const crate::Request);

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            pyo3_asyncio::tokio::block_on(client.execute(request))
        })) {
            Ok(Ok(response)) => Box::into_raw(Box::new(response)) as jlong,
            Ok(Err(e)) => {
                let error_msg = env.new_string(e.to_string())
                    .expect("Failed to create error message");
                env.throw(error_msg).expect("Failed to throw exception");
                0
            }
            Err(_) => {
                env.throw_new("java/lang/RuntimeException", "Rust panic")
                    .expect("Failed to throw exception");
                0
            }
        }
    }
}
```

## cgo Patterns (Go)

### Basic Structure

```rust
// bindings/go/lib.rs
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn create_config(
    timeout_ms: u64,
    max_retries: u32,
) -> *mut crate::Config {
    Box::into_raw(Box::new(crate::Config {
        timeout_ms,
        max_retries,
    }))
}

#[no_mangle]
pub extern "C" fn free_config(config: *mut crate::Config) {
    // SAFETY: Pointer must be valid and created by create_config
    unsafe {
        let _ = Box::from_raw(config);
    }
}

#[no_mangle]
pub extern "C" fn config_get_timeout_ms(
    config: *const crate::Config,
) -> u64 {
    // SAFETY: Pointer must be valid and point to initialized Config
    unsafe { (*config).timeout_ms }
}
```

### Go Wrapper Pattern

```go
// bindings/go/lib/config.go
package mylib

// #include "lib.h"
// typedef unsigned long long uint64_t;
// unsigned long long create_config(uint64_t timeout_ms, unsigned int max_retries);
// void free_config(void* config);
// uint64_t config_get_timeout_ms(void* config);
import "C"

type Config struct {
    ptr unsafe.Pointer
}

func NewConfig(timeoutMs uint64, maxRetries uint32) *Config {
    ptr := C.create_config(C.uint64_t(timeoutMs), C.uint(maxRetries))
    return &Config{ptr: ptr}
}

func (c *Config) GetTimeoutMs() uint64 {
    return uint64(C.config_get_timeout_ms(c.ptr))
}

func (c *Config) Close() error {
    if c.ptr == nil {
        return nil
    }
    C.free_config(c.ptr)
    c.ptr = nil
    return nil
}
```

## P/Invoke Patterns (C#)

### Basic Structure

```rust
// bindings/csharp/lib.rs
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn create_client(
    timeout_ms: u64,
    max_retries: u32,
) -> *mut crate::Client {
    match crate::Client::new(crate::Config {
        timeout_ms,
        max_retries,
    }) {
        Ok(client) => Box::into_raw(Box::new(client)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn free_client(client: *mut crate::Client) {
    // SAFETY: Pointer must be valid
    unsafe {
        let _ = Box::from_raw(client);
    }
}
```

### C# Wrapper Pattern

```csharp
// bindings/csharp/Client.cs
using System;
using System.Runtime.InteropServices;

public class Client : IDisposable
{
    private IntPtr _handle;

    [DllImport("mylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr create_client(UInt64 timeout_ms, UInt32 max_retries);

    [DllImport("mylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern void free_client(IntPtr client);

    public Client(UInt64 timeoutMs, UInt32 maxRetries)
    {
        _handle = create_client(timeoutMs, maxRetries);
        if (_handle == IntPtr.Zero)
            throw new Exception("Failed to create client");
    }

    public void Dispose()
    {
        if (_handle != IntPtr.Zero)
        {
            free_client(_handle);
            _handle = IntPtr.Zero;
        }
    }
}
```

## Rustler Patterns (Elixir)

### Basic Structure

```rust
// bindings/elixir/native/src/lib.rs
use rustler::{Env, ResourceArc};

#[rustler::nif]
pub fn create_config(timeout_ms: u64, max_retries: u32) -> ResourceArc<ConfigResource> {
    let config = crate::Config { timeout_ms, max_retries };
    ResourceArc::new(ConfigResource(config))
}

pub struct ConfigResource(crate::Config);

rustler::atoms! {
    ok,
    error,
}

#[rustler::nif]
pub fn execute_async(client_ref: ResourceArc<ClientResource>) {
    // Rustler provides async support via erlang processes
    let client = &client_ref.0;
    // Implementation
}

#[rustler::nif]
pub fn get_timeout_ms(config: ResourceArc<ConfigResource>) -> u64 {
    config.0.timeout_ms
}
```

## wasm-bindgen Patterns (WebAssembly)

### Basic Structure

```rust
// bindings/wasm/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Config {
    inner: crate::Config,
}

#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new(timeout_ms: u64, max_retries: u32) -> Config {
        Config {
            inner: crate::Config {
                timeout_ms,
                max_retries,
            }
        }
    }

    #[wasm_bindgen(getter)]
    pub fn timeout_ms(&self) -> u64 {
        self.inner.timeout_ms
    }

    #[wasm_bindgen(setter)]
    pub fn set_timeout_ms(&mut self, value: u64) {
        self.inner.timeout_ms = value;
    }
}
```

### Async WASM Handling

```rust
#[wasm_bindgen]
impl Client {
    #[wasm_bindgen]
    pub async fn execute(&self, request: &Request) -> Result<Response, JsValue> {
        self.inner.execute(request.inner.clone())
            .await
            .map(|r| Response { inner: r })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

## Type Mapping Strategies

### Standard Type Mappings

```yaml
Rust Type          → Python        → TypeScript  → Java          → Go
u64                → int           → number      → long          → uint64
u32                → int           → number      → int           → uint32
String             → str           → string      → String        → string
Vec<T>             → List[T]       → T[]         → List<T>       → []T
Option<T>          → T|None        → T|null     → Optional<T>    → *T
Result<T, E>       → Exception     → throw Error → Exception     → (T, error)
Struct             → dataclass     → class       → class         → struct
Enum               → Enum          → union type  → Enum          → int const
```

### Memory Layout Guarantees

```rust
#[repr(C)]
pub struct Request {
    pub url: *const u8,
    pub url_len: usize,
    pub method: *const u8,
    pub method_len: usize,
}

// Rust-side helper
impl Request {
    pub fn from_strings(url: &str, method: &str) -> Self {
        Request {
            url: url.as_ptr(),
            url_len: url.len(),
            method: method.as_ptr(),
            method_len: method.len(),
        }
    }
}
```

## Error Conversion Patterns

### Standard Error to Language Mapping

```rust
pub fn convert_error<L: LanguageBinding>(err: crate::Error) -> L::Error {
    match err {
        crate::Error::Timeout(msg) => L::timeout_error(msg),
        crate::Error::InvalidConfig(msg) => L::config_error(msg),
        crate::Error::NetworkError(msg) => L::network_error(msg),
        crate::Error::ParseError(msg) => L::parse_error(msg),
        crate::Error::Internal(msg) => L::internal_error(msg),
    }
}
```

## Cross-References

- **Polyglot Design**: See [01-polyglot-design.md](01-polyglot-design.md)
- **Rust Core Design**: See [02-rust-core-design.md](02-rust-core-design.md)
- **Build System**: See [04-build-system.md](04-build-system.md)
- **Testing Strategy**: See [05-testing-strategy.md](05-testing-strategy.md)
- **Security Model**: See [07-security-model.md](07-security-model.md)

## Implementation Checklist

- [ ] All 10 bindings follow thin wrapper pattern
- [ ] Async methods properly handled for each language
- [ ] Error conversion implements Error protocol
- [ ] Memory management safe (no leaks, no use-after-free)
- [ ] Type mappings tested (see 05-testing-strategy.md)
- [ ] Documentation generated in each language's format
- [ ] FFI boundaries marked with SAFETY comments

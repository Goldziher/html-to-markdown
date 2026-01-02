# Rust Core Design Architecture

## Overview

The Rust core library is the authoritative implementation of business logic, data structures, and algorithms. All bindings delegate to this core, ensuring consistency and maintainability across the polyglot ecosystem. This document defines the internal architecture of the core library.

## Core Library Architecture and Module Organization

### High-Level Module Structure

```
src/
├── lib.rs                 # Public API and re-exports
├── error.rs              # Error types and conversions
├── config.rs             # Configuration management
├── client.rs             # Primary client interface
├── models/               # Data structures
│   ├── mod.rs
│   ├── request.rs
│   ├── response.rs
│   └── types.rs
├── transport/            # Network abstraction
│   ├── mod.rs
│   ├── http.rs
│   ├── retry.rs
│   └── timeout.rs
├── async_runtime/        # Async/concurrency utilities
│   ├── mod.rs
│   └── executor.rs
├── serialization/        # Codec and encoding
│   ├── mod.rs
│   ├── json.rs
│   └── binary.rs
├── cache/                # Caching strategies
│   ├── mod.rs
│   └── lru.rs
├── middleware/           # Request/response processing
│   ├── mod.rs
│   ├── logging.rs
│   └── metrics.rs
└── util/                 # Internal utilities
    ├── mod.rs
    └── macros.rs
```

### Module Responsibilities

Each module has a clear boundary and responsibility:

1. **error.rs** - Error types, Display impl, conversions
1. **config.rs** - Configuration parsing, validation, defaults
1. **client.rs** - Primary public API, orchestration
1. \**models/* - Data structures, serialization support
1. \**transport/* - HTTP/protocol layer, retries
1. \**async_runtime/* - Tokio integration, executor wrapping
1. \**serialization/* - Encoding/decoding, codec selection
1. \**cache/* - Optional caching, eviction policies
1. \**middleware/* - Logging, tracing, metrics hooks
1. \**util/* - Macros, helper functions (non-public)

### Visibility Rules

```rust
// Top-level public items
pub struct Client { /* */ }
pub enum Error { /* */ }
pub trait Transport { /* */ }

// Module-public items (visible to sibling modules)
pub(crate) struct InternalCache { /* */ }

// Private items (implementation details)
struct PrivateHelper { /* */ }

// Re-exports in lib.rs
pub use models::{Request, Response};
pub use error::Error;
pub use client::Client;
```

## Public API Design Principles

### The Public API Contract

The public API represents a contract with users across all 10 languages. Changes must follow semantic versioning:

```rust
// src/lib.rs - The public contract
pub struct Client { /* ... */ }

pub enum Error {
    Timeout(String),
    InvalidConfig(String),
    NetworkError(String),
    ParseError(String),
}

pub struct Config {
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub enable_logging: bool,
}

pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

pub struct Response {
    pub status_code: u32,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Client {
    /// Creates a new client with the given configuration.
    pub fn new(config: Config) -> Result<Self, Error> { /* */ }

    /// Executes an async request and returns the response.
    pub async fn execute(&self, request: Request) -> Result<Response, Error> { /* */ }

    /// Configures the client at runtime.
    pub fn set_timeout(&mut self, ms: u64) { /* */ }
}
```

### Stability Guarantees

1. **Public API Stability**

   - No breaking changes in minor versions
   - Deprecation warnings for 2+ minor versions before removal
   - Clear CHANGELOG.md entries

1. **Behavioral Stability**

   - Error messages are part of the contract
   - Return types and order are stable
   - Default values documented and unchanging

1. **Performance Stability**

   - No regressions in time complexity
   - Memory usage within 10% of previous versions
   - Async operations maintain latency bounds

### Documentation Standard

Every public item has comprehensive documentation:

````rust
/// Executes a request with automatic retry and timeout handling.
///
/// # Arguments
///
/// * `request` - The HTTP request to execute
///
/// # Returns
///
/// Returns the HTTP response or an error if the request fails.
///
/// # Errors
///
/// Returns `Error::Timeout` if the request exceeds configured timeout.
/// Returns `Error::NetworkError` if the network is unreachable.
///
/// # Examples
///
/// ```
/// # use mylib::{Client, Config, Request};
/// # async fn example() -> Result<(), mylib::Error> {
/// let config = Config::default();
/// let client = Client::new(config)?;
/// let request = Request {
///     url: "http://example.com".to_string(),
///     ..Default::default()
/// };
/// let response = client.execute(request).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute(&self, request: Request) -> Result<Response, Error> {
    // implementation
}
````

## Feature Gate Strategy

Feature gates control compilation and expose optional functionality:

```toml
# Cargo.toml
[features]
default = ["std", "logging", "metrics"]

# Core features
std = []
alloc = []

# Optional capabilities
logging = ["tracing", "tracing-subscriber"]
metrics = ["prometheus"]
compression = ["flate2"]
caching = ["lru"]
tls = ["rustls"]

# Language-specific features
python = []
nodejs = []
wasm = []

# Development features
bench = []
mocking = []

# All features
all = ["logging", "metrics", "compression", "caching", "tls"]
```

### Feature Implementation Pattern

```rust
// Feature-gated module
#[cfg(feature = "caching")]
pub mod cache {
    pub struct Cache { /* */ }
    impl Cache {
        pub fn new(capacity: usize) -> Self { /* */ }
    }
}

// Feature-gated implementation
#[cfg(feature = "metrics")]
impl Client {
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}

// Default implementation when feature disabled
#[cfg(not(feature = "metrics"))]
impl Client {
    pub fn metrics(&self) -> &Metrics {
        &NOOP_METRICS
    }
}

// Re-export based on feature
pub use cache::Cache;

#[cfg(feature = "logging")]
pub use logging::Logger;

#[cfg(not(feature = "logging"))]
pub struct Logger;
```

### Binding Feature Coordination

```yaml
# bindings/python/Cargo.toml
features = ["default", "python"]  # Enable Python-specific features

# bindings/typescript/Cargo.toml
features = ["default", "nodejs"]  # Enable Node.js features

# bindings/wasm/Cargo.toml
features = ["alloc", "wasm"]      # Minimal WASM features
```

## Error Handling Patterns

### Result Type Convention

```rust
// Standard Result type for the library
pub type Result<T> = std::result::Result<T, Error>;

// All public functions return this Result type
pub async fn execute(&self, req: Request) -> Result<Response> { /* */ }
```

### Error Types and Conversions

```rust
// src/error.rs - Comprehensive error enumeration
#[derive(Debug, Clone)]
pub enum Error {
    /// Timeout occurred
    Timeout(String),

    /// Invalid configuration
    InvalidConfig(String),

    /// Network connectivity error
    NetworkError(String),

    /// Request parsing failed
    ParseError(String),

    /// Unknown internal error
    Internal(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Error::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

// Conversions from external crates
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::ParseError(err.to_string())
    }
}

// No panics in public API
pub fn parse_config(s: &str) -> Result<Config> {
    // Returns Error, never panics
    serde_json::from_str(s)
        .map_err(Error::from)
}
```

### Error Propagation Patterns

```rust
// Pattern 1: Question mark operator
pub async fn execute(&self, request: Request) -> Result<Response> {
    validate_request(&request)?;        // Early return on error
    let transport = self.get_transport()?;
    let response = transport.send(request).await?;
    Ok(response)
}

// Pattern 2: map_err for context
pub fn load_config(path: &str) -> Result<Config> {
    std::fs::read_to_string(path)
        .map_err(|e| Error::InvalidConfig(format!("Cannot read config: {}", e)))?
        .parse()
        .map_err(Error::from)
}

// Pattern 3: Explicit error handling
pub async fn execute_with_retry(&self, req: Request) -> Result<Response> {
    let mut last_error = Error::Internal("No attempts made".to_string());

    for attempt in 0..self.config.max_retries {
        match self.execute_attempt(&req).await {
            Ok(response) => return Ok(response),
            Err(e) if is_retryable(&e) => {
                last_error = e;
                tokio::time::sleep(Duration::from_millis(
                    100 * 2_u64.pow(attempt as u32)
                )).await;
            }
            Err(e) => return Err(e),  // Non-retryable error
        }
    }

    Err(last_error)
}
```

## Async/Await Patterns with Tokio

### Tokio Runtime Integration

```rust
// src/async_runtime/mod.rs
use tokio::runtime::Runtime;

pub struct AsyncContext {
    runtime: Runtime,
}

impl AsyncContext {
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::Internal(e.to_string()))?;
        Ok(AsyncContext { runtime })
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.runtime.block_on(future)
    }
}
```

### Async Public API

```rust
impl Client {
    /// Executes an async request.
    ///
    /// # Panics
    ///
    /// Never panics; returns errors instead.
    pub async fn execute(&self, request: Request) -> Result<Response> {
        // Uses Tokio internally
        tokio::time::timeout(
            Duration::from_millis(self.config.timeout_ms),
            self.execute_internal(request),
        )
        .await
        .map_err(|_| Error::Timeout("Request timeout".to_string()))?
    }

    async fn execute_internal(&self, request: Request) -> Result<Response> {
        // Implementation using await
        self.transport.send(request).await
    }
}
```

### Concurrent Operations

```rust
// Safe concurrent operations with proper error handling
pub async fn execute_batch(&self, requests: Vec<Request>) -> Result<Vec<Response>> {
    let handles: Vec<_> = requests
        .into_iter()
        .map(|req| {
            let transport = Arc::clone(&self.transport);
            tokio::spawn(async move {
                transport.send(req).await
            })
        })
        .collect();

    // Wait for all tasks and collect results
    let mut responses = Vec::new();
    for handle in handles {
        let response = handle
            .await
            .map_err(|e| Error::Internal(e.to_string()))??;
        responses.push(response);
    }

    Ok(responses)
}
```

## Cross-References

- **Polyglot Design**: See [01-polyglot-design.md](01-polyglot-design.md)
- **Binding Patterns**: See [03-binding-patterns.md](03-binding-patterns.md)
- **Build System**: See [04-build-system.md](04-build-system.md)
- **Testing Strategy**: See [05-testing-strategy.md](05-testing-strategy.md)
- **Security Model**: See [07-security-model.md](07-security-model.md)

## Implementation Checklist

- [ ] Module organization matches structure above
- [ ] All public items have doc comments
- [ ] Feature gates tested independently
- [ ] Error handling has no panics in public API
- [ ] Async functions use tokio::time::timeout for timeouts
- [ ] Result<T> type alias used everywhere
- [ ] Visibility rules enforced (pub vs pub(crate))
- [ ] Tests cover all error paths (see 05-testing-strategy.md)

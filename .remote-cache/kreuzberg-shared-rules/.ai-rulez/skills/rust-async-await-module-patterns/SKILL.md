______________________________________________________________________

## priority: medium

# Rust Async/Await & Module Patterns

## Tokio Runtime Patterns

**Single runtime instance** should be created once and shared across the application.

### Pattern 1: Global Runtime (for library bindings)

Use `tokio::runtime::Runtime` to manage async code in synchronous contexts (required for FFI bindings).

```rust
// src/async_runtime.rs
use tokio::runtime::Runtime;
use std::sync::OnceLock;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("htm2md-worker")
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

pub fn block_on<F>(f: F) -> F::Output
where
    F: std::future::Future,
{
    get_runtime().block_on(f)
}
```

**Usage in sync binding**:

```rust
// For PyO3, NAPI-RS, etc.
#[pyclass]
pub struct HtmlConverter {
    inner: Arc<html_to_markdown::AsyncConverter>,
}

#[pymethods]
impl HtmlConverter {
    pub fn convert(&self, html: String) -> PyResult<String> {
        let result = crate::async_runtime::block_on(
            self.inner.convert(html)
        )?;
        Ok(result)
    }
}
```

### Pattern 2: Dedicated Thread Pool (for async bindings)

For Node.js (NAPI-RS), JavaScript runs on main thread. Use thread pool for Rust async tasks:

```rust
// src/worker_pool.rs
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct AsyncWorkerPool {
    runtime: Arc<Runtime>,
}

impl AsyncWorkerPool {
    pub fn new(worker_count: usize) -> Self {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(worker_count)
                .build()
                .expect("Failed to create runtime")
        );

        Self { runtime }
    }

    pub fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(task)
    }
}
```

**NAPI-RS integration**:

```rust
thread_local! {
    static POOL: AsyncWorkerPool = AsyncWorkerPool::new(4);
}

#[napi]
pub async fn convert_async(html: String) -> napi::Result<String> {
    POOL.with(|pool| {
        let task = async move {
            html_to_markdown::convert(&html).await
        };
        pool.spawn(task)
    })
}
```

### Pattern 3: Feature-Gated Async Runtime

Conditionally enable async code path:

```toml
[features]
default = ["sync"]
async-runtime = ["tokio"]

[[example]]
name = "async_example"
required-features = ["async-runtime"]
```

```rust
#[cfg(feature = "async-runtime")]
pub async fn convert_async(html: &str) -> Result<String> {
    // Async implementation
}

#[cfg(not(feature = "async-runtime"))]
pub fn convert_sync(html: &str) -> Result<String> {
    // Sync fallback
}
```

## Send + Sync Requirements in FFI

**Critical for thread safety across FFI boundaries**.

### Invariant: All types exposed to FFI must be Send + Sync

```rust
// BAD: Uses Rc (not Send + Sync)
pub struct Converter {
    state: Rc<RefCell<State>>,
}

// GOOD: Uses Arc + Mutex (Send + Sync)
pub struct Converter {
    state: Arc<Mutex<State>>,
}

// Verify at compile time
fn _assert_send_sync() {
    fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<Converter>();
}
```

### Common Send + Sync Mistakes

| Type | Send | Sync | Fix |
|------|------|------|-----|
| `Rc<T>` | ❌ | ❌ | Use `Arc<T>` |
| `RefCell<T>` | ❌ (if T not Sync) | ❌ | Use `Mutex<T>` |
| `Cell<T>` | ❌ | ❌ | Use `AtomicU32` or similar |
| `*const T` | ❌ | ❌ | Use `Box<T>` or unsafe wrapper |

### Proper Async Type Pattern

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AsyncState {
    data: Arc<Mutex<String>>,
}

// SAFETY: Arc<Mutex<T>> is Send + Sync if T is Send
// Verified at compile time below
#[cfg(test)]
const _: fn() = || {
    const fn is_send<T: Send>() {}
    const fn is_sync<T: Sync>() {}

    const fn check<T: Send + Sync>() {
        is_send::<T>();
        is_sync::<T>();
    }

    check::<AsyncState>();
};
```

## Blocking Bridge Patterns for Sync Languages

**Problem**: Host language (Python, Ruby, PHP) is synchronous but Rust code is async.

### Pattern: spawn_blocking + block_on

```rust
// src/bridge.rs
use tokio::task;

pub async fn async_operation() -> Result<String> {
    // CPU-intensive work
}

pub fn sync_wrapper() -> Result<String> {
    // Get runtime
    let rt = crate::runtime::get_runtime();

    // Offload to blocking thread pool
    rt.block_on(async {
        task::spawn_blocking(|| {
            // Actually blocking CPU work
            expensive_computation()
        })
        .await
        .map_err(|e| Error::JoinError(e))?
    })
}
```

**Why spawn_blocking?**

- Prevents blocking the Tokio runtime thread
- Tokio tracks when threads are blocked
- Spawns new threads if all workers are blocked

### Pattern: AsyncBridge for Long-Running Operations

```rust
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct AsyncBridge {
    tx: mpsc::UnboundedSender<AsyncTask>,
}

pub enum AsyncTask {
    Convert(String, Box<dyn Fn(Result<String>) + Send>),
}

impl AsyncBridge {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn background task
        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                match task {
                    AsyncTask::Convert(html, callback) => {
                        let result = html_to_markdown::convert(&html).await;
                        callback(result);
                    }
                }
            }
        });

        Self { tx }
    }

    pub fn convert<F>(&self, html: String, callback: F) -> Result<()>
    where
        F: Fn(Result<String>) + Send + 'static,
    {
        self.tx.send(AsyncTask::Convert(html, Box::new(callback)))
            .map_err(|_| Error::BridgeClosed)
    }
}
```

## Async Module Organization

Structure async code separately from sync:

```
src/
├── lib.rs              # Main API (sync)
├── async/              # Async implementations
│   ├── mod.rs          # Re-exports
│   ├── converter.rs    # AsyncConverter
│   ├── stream.rs       # Streaming processing
│   └── runtime.rs      # Runtime management
├── sync/               # Sync implementations
│   ├── mod.rs
│   └── converter.rs    # Converter
└── bridge.rs           # sync/async bridge layer
```

**Module re-export pattern**:

```rust
// src/lib.rs
pub mod sync;
pub mod async_;

#[cfg(feature = "async-runtime")]
pub use async_::AsyncConverter;

pub use sync::Converter;
```

## Testing Async Code

### Unit Tests with #[tokio::test]

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_convert_async() {
        let converter = AsyncConverter::new();
        let result = converter.convert("<h1>Test</h1>").await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_concurrent_converts() {
        let converter = Arc::new(AsyncConverter::new());
        let mut tasks = vec![];

        for i in 0..10 {
            let conv = converter.clone();
            tasks.push(tokio::spawn(async move {
                conv.convert(&format!("<h1>{}</h1>", i)).await
            }));
        }

        for task in tasks {
            assert!(task.await.unwrap().is_ok());
        }
    }
}
```

### Integration Tests with Real Runtime

```rust
#[test]
fn test_sync_wrapper() {
    let result = crate::bridge::sync_wrapper();
    assert!(result.is_ok());
}
```

## Anti-Patterns to Avoid

1. **Multiple runtimes**:

   ```rust
   // BAD: Creates new runtime every call
   pub fn convert(html: &str) -> Result<String> {
       tokio::runtime::Runtime::new()?
           .block_on(async { ... })
   }

   // GOOD: Reuse single runtime
   pub fn convert(html: &str) -> Result<String> {
       crate::runtime::block_on(async { ... })
   }
   ```

1. **Blocking in async without spawn_blocking**:

   ```rust
   // BAD: Blocks Tokio executor
   async fn convert(html: &str) -> Result<String> {
       std::thread::sleep(Duration::from_secs(1));  // Blocks!
       Ok(...)
   }

   // GOOD: Offload to blocking pool
   async fn convert(html: &str) -> Result<String> {
       tokio::task::spawn_blocking(|| {
           std::thread::sleep(Duration::from_secs(1));
       }).await?;
       Ok(...)
   }
   ```

1. **Mixing Send + non-Send in FFI**:

   ```rust
   // BAD: RefCell not Send
   #[pyclass]
   pub struct Converter {
       state: RefCell<State>,  // Can't cross FFI safely!
   }

   // GOOD: Mutex is Send + Sync
   #[pyclass]
   pub struct Converter {
       state: Mutex<State>,
   }
   ```

1. **Panics in async contexts**:

   ```rust
   // BAD: Panic kills entire runtime
   async fn convert(html: &str) -> Result<String> {
       let parts: Vec<_> = html.split("broke").collect();
       Ok(parts[99].to_string())  // Panics if < 100 parts!
   }

   // GOOD: Return error
   async fn convert(html: &str) -> Result<String> {
       let parts: Vec<_> = html.split("broke").collect();
       let part = parts.get(99)
           .ok_or(Error::NotFound)?;
       Ok(part.to_string())
   }
   ```

## Cross-references to Related Skills

- **binding-crate-architecture-patterns**: Integrating async runtimes in FFI bindings
- **ffi-and-language-interop-standards**: Memory safety in async FFI code
- **testing-philosophy-coverage**: Testing async code paths
- **error-handling-strategy**: Error propagation in async/await

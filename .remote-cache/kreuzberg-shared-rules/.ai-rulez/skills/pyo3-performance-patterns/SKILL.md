______________________________________________________________________

## priority: high

# PyO3 Performance Patterns

Use `pyo3_async_runtimes` for async Python callbacks (~28x faster than spawn_blocking for fast ops).

Pattern: Check `__await__` attribute, use `pyo3_async_runtimes::tokio::into_future()` for async, fallback to spawn_blocking for sync. Release GIL before awaiting. Use Python::attach() not with_gil().

spawn_blocking for long ops (OCR), block_in_place for quick ops (PostProcessor/Validator). **CRITICAL: spawn_blocking on PostProcessor/Validator causes GIL deadlocks.**

Reference: crates/kreuzberg-py/README.md

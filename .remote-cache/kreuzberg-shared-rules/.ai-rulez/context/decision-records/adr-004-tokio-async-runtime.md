# ADR 004: Tokio Async Runtime

**Date:** 2024-09-05

**Status:** Accepted

## Context

The ai-rulez project supports asynchronous operations for rule processing, allowing users to process multiple rule sets concurrently. The Rust core uses async/await syntax extensively for performance and scalability.

When exposing async operations through language bindings, we need a consistent async execution model that works across diverse languages with different concurrency paradigms:

- **Python:** asyncio, threading, callbacks
- **JavaScript:** Promises, async/await, callbacks
- **Go:** Goroutines, channels
- **Java:** Threads, CompletableFuture, reactive libraries
- **C#:** async/await, Tasks
- **Ruby:** Fibers, Enumerator, callbacks

Without a standardized approach, async operations would need to be implemented differently in each binding, leading to inconsistent performance characteristics and debugging complexity.

We selected Tokio as the Rust async runtime during the core implementation. The question is: should this runtime be embedded in the core, or should bindings manage their own async coordination?

## Decision

Tokio will be the standard async runtime for all asynchronous operations in the Rust core. This runtime is embedded in the core and cannot be replaced. Language bindings will:

1. **Use Tokio internally:** The Rust core manages a Tokio runtime that handles async operations
1. **Provide blocking bridges:** Bindings for synchronous languages (Python, PHP, Ruby) provide blocking adapter functions that use `tokio::task::block_on()` or similar mechanisms
1. **Expose native async patterns:** Bindings for async-first languages (JavaScript, Go, C#) map Tokio futures to language-native async abstractions
1. **Standard initialization:** Bindings handle runtime initialization transparently; users don't need to manually initialize Tokio

For example:

- **Python async binding:** Returns `asyncio.Future` that bridges Tokio tasks
- **JavaScript binding:** Returns native Promise
- **Go binding:** Uses channel-based adapters
- **Synchronous Python/PHP:** Uses blocking calls with thread pool management

## Consequences

### Positive

- **Consistent async semantics:** All async operations behave identically across bindings
- **Proven runtime:** Tokio is battle-tested, production-grade, widely used
- **High performance:** Tokio provides excellent performance for concurrent operations
- **Built-in tools:** Tokio's ecosystem includes excellent debugging and profiling tools
- **Future compatibility:** Tokio is actively maintained with strong community support
- **Predictable resource usage:** Unified runtime prevents resource contention between multiple runtimes

### Negative

- **Initialization overhead:** Tokio runtime incurs startup costs, even for synchronous operations
- **Memory overhead:** Tokio maintains thread pool and scheduler state
- **Blocking bridge complexity:** Synchronous language bindings must handle blocking semantics correctly
- **Potential deadlocks:** Improper use of blocking bridges can lead to deadlocks if not carefully designed
- **Thread pool sizing:** Determining optimal thread pool size requires configuration
- **Single runtime limitation:** Cannot have multiple independent Tokio runtimes for partitioning

### Configuration

Tokio runtime configuration will be exposed through binding APIs:

- Worker thread count (default: number of CPU cores)
- Thread stack size (default: platform default)
- Enable/disable busy wait optimization
- Custom panic hook for task panics

### Blocking Operations Guidelines

For synchronous languages:

1. Use `tokio::task::block_on()` to block on async operations
1. Ensure blocking operations don't hold locks across boundaries
1. Document which operations are blocking to users
1. Provide timeout parameters for all blocking operations
1. Handle Tokio runtime panics gracefully

### Future Considerations

- We may provide stream-based APIs for high-throughput scenarios
- Consider worker pool integration for compute-intensive operations
- Evaluate custom Tokio runtime variants for specialized use cases

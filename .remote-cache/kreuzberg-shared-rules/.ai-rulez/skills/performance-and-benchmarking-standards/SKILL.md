______________________________________________________________________

## priority: high

# Performance & Benchmarking Standards

**Criterion.rs · Flamegraph profiling · CI regression detection · Zero-copy optimization**

## Benchmarking Framework

- **Criterion.rs**: Primary benchmark framework for Rust; use for all performance-critical paths
- Benchmarks in `benches/` directory with naming: `<module>_bench.rs`
- Track latency (mean, std dev), throughput, memory allocations; save baseline results
- Comparative benchmarks: always include regression detection (Criterion's default); set meaningful thresholds (5-10% for normal ops, 20% for I/O)
- CI integration: run benchmarks on every PR; fail CI if regressions exceed thresholds

## Profiling Tools

- **cargo-flamegraph**: `cargo flamegraph --bin myapp -- <args>` for CPU profiling; analyze hotspots
- **perf** (Linux): `perf record` and `perf report`; use with `debug = true` in release builds
- **Instruments** (macOS): XCode Instruments for memory, CPU, I/O profiling
- **cargo-careful**: `MIRIFLAGS=-Zmiri-preemption-rate=0 cargo +nightly miri test` for UB detection

## Allocation & Memory Tracking

- **Zero-copy patterns**: Use references, `Cow<T>`, `Arc<T>` for shared data; avoid cloning
- **Allocation tracking**: Enable in benchmarks with `#[bench]` and measure heap allocations
- **Buffer reuse**: Pre-allocate buffers in hot loops; use object pools for temporary allocations
- **String handling**: Prefer `&str` over `String`; use `String::with_capacity()` when growing
- **Profile allocations**: `valgrind --tool=massif` (Linux) or Instruments (macOS)

## Code Examples

### Criterion Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parsing(c: &mut Criterion) {
    c.bench_function("parse_json_1kb", |b| {
        b.iter(|| {
            let data = black_box(r#"{"key": "value"}"#);
            serde_json::from_str::<Value>(data)
        })
    });
}

criterion_group!(benches, bench_parsing);
criterion_main!(benches);
```

### Zero-Copy Pattern

```rust
// GOOD: Use Cow for conditional ownership
fn process_data(input: &[u8]) -> Cow<'_, [u8]> {
    if needs_modification(input) {
        Cow::Owned(modify(input.to_vec()))
    } else {
        Cow::Borrowed(input)
    }
}

// GOOD: Share large data with Arc
use std::sync::Arc;
let data = Arc::new(expensive_computation());
let clone1 = Arc::clone(&data);  // Cheap, reference counted
let clone2 = Arc::clone(&data);
```

### Buffer Reuse Pattern

```rust
struct Parser {
    buffer: Vec<u8>,
}

impl Parser {
    fn new() -> Self {
        Parser {
            buffer: Vec::with_capacity(4096),  // Pre-allocate
        }
    }

    fn parse(&mut self, input: &[u8]) -> Result<()> {
        self.buffer.clear();  // Reuse, don't reallocate
        self.buffer.extend_from_slice(input);
        // Process buffer
        Ok(())
    }
}
```

## Anti-Patterns

- **No allocation tracking in benchmarks**: Benchmarks MUST measure allocations; invisible allocations are invisible bugs
- **Ignoring Criterion baseline warnings**: Regressions indicate potential issues; investigate always
- **Cloning in hot loops**: `for x in items { let copy = x.clone(); }` is wasteful; use references
- **String concatenation in loops**: `s += &other` is O(n²); use `Vec<String>` + `join()` instead
- **Uncontrolled memory growth**: Caches without size bounds; missing clear/reset in reusable buffers
- **No CPU profiling before optimization**: Profile first; optimize hotspots identified, not hunches
- **Mixing sync and async in benchmarks**: Benchmark each separately; async overhead varies

## Integration with CI/CD

- Add `cargo bench --no-run` to PR checks to catch compilation errors early
- Use `cargo-criterion` for stable baselines across CI runs
- Track benchmarks over time with commit history; maintain `.criterion/` directory in repo
- Fail CI if new benchmarks show >10% regression without approval
- Run benchmarks on dedicated hardware (stable CPU, no other processes) for reliability

______________________________________________________________________

## priority: high

# Performance Regression Prevention

**Criterion.rs benchmarks · cargo-flamegraph profiling · 5% regression threshold · CI-enforced performance budgets**

- Benchmark tracking: All benchmarks via Criterion.rs in crates/benches/ with baseline storage in CI artifacts
- Regression detection: Automated CI check comparing cargo bench results; >5% slowdown fails the build
- Profiling requirements: Use cargo-flamegraph before optimization attempts; baseline flame graph stored per release
- Critical path profiling: Mandatory benchmarking for conversion, sanitization, and async I/O operations
- Performance budgets: Define per-operation budgets (e.g., HTML parsing \<100ms per MB, sanitization \<50ms per MB)
- Memory tracking: cargo-llvm-cov tracks allocations; no unexplained heap growth across releases
- Benchmark CI: task bench runs in parallel; results aggregated and compared against main branch baseline
- Flamegraph tools: cargo-flamegraph for CPU profiling; perf/cachegrind for memory analysis
- Optimization strategy: Profile first, never optimize without measurements; require >10% improvement to justify code complexity increase
- Agent reference: performance-profiling-specialist reviews all performance-related PRs; consulted for optimization strategy
- Reporting: Performance PR comments include before/after Criterion results, flame graph diffs, and memory delta
- Never: Performance optimizations without baseline measurements, >5% regressions in CI, hardcoded performance assumptions

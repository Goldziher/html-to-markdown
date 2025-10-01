# Baseline Benchmarks - v1 (BeautifulSoup)

**Date**: October 2, 2025
**Branch**: v2-dev
**Commit**: Stage 1 - Functional API with dataclass options
**Backend**: Python with BeautifulSoup + nh3

## Summary

This baseline establishes performance metrics for the v1 implementation before migrating to Rust (lol-html + ammonia). These numbers will be compared against v2 to measure improvements.

## Key Metrics

### Wikipedia Documents (Real-world HTML)

| Document           | Size  | Mean Time | Throughput (ops/sec) | StdDev   |
| ------------------ | ----- | --------- | -------------------- | -------- |
| Lists (Timeline)   | 130KB | 26.10ms   | 38.31                | ±4.63ms  |
| Tables (Countries) | 361KB | 101.25ms  | 9.88                 | ±9.46ms  |
| Python Article     | 656KB | 188.44ms  | 5.31                 | ±14.13ms |

### Scalability (Synthetic Documents)

| Size  | Mean Time | Throughput (ops/sec) |
| ----- | --------- | -------------------- |
| 5KB   | 6.88ms    | 145.18               |
| 10KB  | 13.24ms   | 75.45                |
| 25KB  | 32.09ms   | 31.16                |
| 50KB  | 62.98ms   | 15.88                |
| 100KB | 125.56ms  | 7.96                 |

### Streaming Performance

| Document Type | Mean Time | Throughput (ops/sec) |
| ------------- | --------- | -------------------- |
| Small (130KB) | 26.34ms   | 37.97                |
| Large (656KB) | 188.13ms  | 5.32                 |

**Observation**: Streaming performance nearly identical to non-streaming in v1, as expected (BeautifulSoup builds full DOM regardless).

### API Comparison (v1 kwargs vs v2 dataclasses)

| API          | Mean Time | Throughput (ops/sec) |
| ------------ | --------- | -------------------- |
| v1 (kwargs)  | 152.17ms  | 6.57                 |
| v2 (options) | 152.28ms  | 6.57                 |

**Observation**: V2 API has zero overhead (wrapper around v1 for now).

## Expected v2 Improvements

Based on technology stack:

### Conservative Estimates

- **Throughput**: 10-15x improvement
- **Memory**: 50-60% reduction
- **Streaming**: 80-90% memory reduction for large docs

### Technology Benefits

- **ammonia direct** (vs nh3 bindings): 2-3x improvement
- **lol-html** (vs BeautifulSoup): 5-10x improvement
- **Combined**: 10-30x total improvement expected

## Target Goals for v2

### Minimum Acceptable Performance (10x improvement)

| Document       | v1 Baseline | v2 Target (10x)           |
| -------------- | ----------- | ------------------------- |
| Small (130KB)  | 26.10ms     | \<2.61ms (>383 ops/sec)   |
| Medium (361KB) | 101.25ms    | \<10.13ms (>98.8 ops/sec) |
| Large (656KB)  | 188.44ms    | \<18.84ms (>53.1 ops/sec) |

### Stretch Goals (20x improvement)

| Document       | v1 Baseline | v2 Stretch (20x)          |
| -------------- | ----------- | ------------------------- |
| Small (130KB)  | 26.10ms     | \<1.31ms (>766 ops/sec)   |
| Medium (361KB) | 101.25ms    | \<5.06ms (>197.6 ops/sec) |
| Large (656KB)  | 188.44ms    | \<9.42ms (>106.2 ops/sec) |

## Benchmark Reproduction

To reproduce these benchmarks:

```bash
# Run all benchmarks
uv run pytest tests/benchmark*.py --benchmark-only

# Save as baseline
uv run pytest tests/benchmark*.py --benchmark-only \
    --benchmark-save=baseline_v1_beautifulsoup --benchmark-save-data

# Compare future runs against this baseline
uv run pytest tests/benchmark*.py --benchmark-only \
    --benchmark-compare=baseline_v1_beautifulsoup
```

## Test Environment

- **Python**: 3.12.10
- **Platform**: macOS (Darwin 24.6.0)
- **CPU**: Apple Silicon
- **Dependencies**:
  - BeautifulSoup4: 4.12.x
  - lxml: 5.3.x
  - nh3: 0.x.x (Python bindings to ammonia)

## Notes

- All benchmarks run with default options unless specified
- Wikipedia documents downloaded October 2, 2025
- Benchmark results saved in `.benchmarks/Darwin-CPython-3.12-64bit/`
- Full benchmark data available in `.benchmarks/` directory

## Next Steps

1. ✅ Baseline established
1. ⏳ Implement Rust backend (lol-html + ammonia)
1. ⏳ Re-run benchmarks with v2
1. ⏳ Compare and validate improvements
1. ⏳ Document final performance characteristics

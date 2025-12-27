# Visitor Callback Profiling - Integration Guide

## Overview

The visitor callback profiling scripts (`profile_visitor.{py,js,rb}`) are specialized tools designed to measure the performance overhead introduced by visitor callbacks in language bindings.

These scripts complement the existing benchmark-harness by focusing specifically on:
- **Binding overhead**: Cost of crossing FFI/language boundaries
- **Callback marshalling**: Time to convert types between languages
- **GC impact**: Memory pressure from callback handling

## Architecture

```
html-to-markdown/
├── tools/
│   ├── benchmark-harness/          # General performance benchmarking
│   │   ├── src/                    # Rust harness implementation
│   │   ├── fixtures/               # Test document fixtures
│   │   └── results/                # Benchmark results
│   │
│   └── profile_visitor.{py,js,rb}  # Visitor callback profiling
│       ├── Python profiler
│       ├── Node.js profiler
│       └── Ruby profiler
```

## Differences from benchmark-harness

| Aspect | benchmark-harness | profile_visitor |
|--------|-------------------|-----------------|
| **Purpose** | Full conversion performance | Callback overhead only |
| **Scope** | All bindings + CLI | Single language binding |
| **Metrics** | Ops/sec, throughput, GC | Callback overhead, marshalling |
| **Output** | HTML reports, flamegraphs | JSON metrics, cProfile/stats |
| **Scenarios** | Document types | Visitor workload types |
| **Language** | Rust (runs all bindings) | Native (Python, Node, Ruby) |

## When to Use Each Tool

### Use benchmark-harness when:
- Measuring overall conversion performance
- Comparing performance across languages
- Creating performance reports/dashboards
- Profiling full conversion pipeline
- Generating flamegraphs for full execution

### Use profile_visitor when:
- Investigating visitor callback overhead
- Optimizing binding FFI layer
- Analyzing type marshalling cost
- Measuring GC impact of callbacks
- Profiling specific language binding performance
- Detailed timing breakdowns per callback

## Integration Workflow

### Step 1: Baseline with benchmark-harness
```bash
cd tools/benchmark-harness
cargo run --release -- --fixtures /path/to/fixtures
# Generates results/, flamegraphs/, HTML reports
```

### Step 2: Profile visitor overhead per language
```bash
# Python
python tools/profile_visitor.py --scenario all --iterations 30

# Node.js
node tools/profile_visitor.js --scenario all --iterations 30

# Ruby
ruby tools/profile_visitor.rb --scenario all --iterations 30
```

### Step 3: Analyze results
```bash
# Compare overhead across languages
python -c "
import json
for lang in ['py', 'js', 'rb']:
    with open(f'visitor_profile_results_{lang}/results.json') as f:
        data = json.load(f)
        for r in data['results']:
            print(f'{lang:3} {r[\"scenario\"]:15} overhead: {r[\"overhead_percent\"]:5.1f}%')
"
```

## Output Integration

### Combining Results

The visitor profiling results can be combined with benchmark-harness results:

```json
{
  "benchmark_harness": {
    "html": "medium_python.html",
    "baseline_conversion": "2.45ms",
    "ops_per_sec": 408
  },
  "visitor_profiling": {
    "scenario": "simple",
    "callback_overhead": "27.3%",
    "avg_callback_time": "1.28µs",
    "callbacks": 523
  }
}
```

This shows that visitor callbacks add ~27% overhead, which translates to:
- ~0.67ms additional time per conversion
- ~0.27ms/sec throughput reduction (at 408 ops/sec)

### Result Storage

```
tools/
├── benchmark-harness/
│   └── results-<identifier>/
│       ├── results.json
│       ├── flamegraphs/
│       └── report.html
│
└── visitor_profile_results/
    ├── results.json              # Main metrics
    ├── profile_no-op.txt         # cProfile (Python)
    ├── profile_simple.txt
    ├── profile_custom-output.txt
    └── profile_complex.txt
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Visitor Profiling

on:
  pull_request:
    paths:
      - 'crates/**'
      - 'packages/**'

jobs:
  profile-visitor:
    runs-on: ubuntu-latest

    strategy:
      matrix:
        language: [python, node, ruby]

    steps:
      - uses: actions/checkout@v4

      - name: Build bindings
        run: |
          if [ "${{ matrix.language }}" = "python" ]; then
            cd packages/python && pip install -e .
          elif [ "${{ matrix.language }}" = "node" ]; then
            cd packages/typescript && pnpm install && pnpm build
          else
            cd packages/ruby && bundle install && rake compile
          fi

      - name: Profile visitor overhead
        run: |
          if [ "${{ matrix.language }}" = "python" ]; then
            python tools/profile_visitor.py --iterations 20
          elif [ "${{ matrix.language }}" = "node" ]; then
            node tools/profile_visitor.js --iterations 20
          else
            ruby tools/profile_visitor.rb --iterations 20
          fi

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: visitor-profile-${{ matrix.language }}
          path: visitor_profile_results/results.json
```

## Performance Benchmarking Standards

### Measurement Protocol

1. **Warm-up**: 2-3 iterations to warm up JIT/GC
2. **Baseline**: Run without visitor N times
3. **Profiled**: Run with visitor N times
4. **Calculate**: Overhead = (profiled - baseline) / baseline * 100

### Statistical Significance

- **Minimum iterations**: 10 (for quick checks)
- **Standard**: 20-30 (for reliable results)
- **Detailed**: 50+ (for statistical analysis)
- **Sample size**: Each result is average of N iterations

### Acceptable Variance

- **Good**: ±5% variance between runs
- **Acceptable**: ±10% variance
- **Poor**: >15% variance (increase iterations)

## Troubleshooting Integration

### Results show unexpected overhead

1. Check visitor implementation complexity
2. Verify marshalling costs in cProfile (Python)
3. Profile with --expose-gc (Node.js)
4. Use Benchmark module carefully (Ruby)

### Comparing with other benchmarks

- Visitor profiling measures **only** callback overhead
- Full conversion time includes parsing, processing, output
- Overhead percentage varies by document complexity
- GC impact depends on allocation patterns

### Regression detection

```python
# Compare against baseline
baseline_overhead = 27.3  # %
current_overhead = 29.1   # %

if current_overhead > baseline_overhead * 1.1:
    print("REGRESSION: Overhead increased >10%")
```

## Documentation Files

The visitor profiling suite includes:

1. **PROFILING_README.md**: Comprehensive documentation
   - Detailed usage for each language
   - Output format specification
   - Workflow examples
   - Troubleshooting guide

2. **VISITOR_PROFILING_GUIDE.md**: Quick reference
   - Common commands
   - Key metrics explained
   - Analysis examples
   - Quick troubleshooting

3. **VISITOR_PROFILING_INTEGRATION.md**: This file
   - Integration with existing tools
   - CI/CD setup
   - Performance standards
   - Advanced workflows

## Next Steps

### For Initial Setup
1. Run baseline profiles for each language
2. Store baseline results for regression detection
3. Document expected overhead ranges
4. Set up CI/CD integration

### For Optimization
1. Identify high-overhead scenarios
2. Use cProfile/detailed stats for bottlenecks
3. Implement optimizations
4. Re-profile to measure improvements
5. Update baseline thresholds

### For Monitoring
1. Run profiling on each PR
2. Alert on regressions >10%
3. Track overhead trends over time
4. Use results to guide architecture decisions

## Related Tools

### Rust Core Profiling
```bash
# Profile Rust core with benchmarks
cd crates/html-to-markdown
cargo bench
```

### Full Conversion Benchmarking
```bash
# Profile complete conversion pipeline
cd tools/benchmark-harness
cargo run --release
```

### Language-Specific Profiling
```bash
# Python: detailed cProfile
python tools/profile_visitor.py --cprofile

# Node.js: V8 CPU profile
node --prof tools/profile_visitor.js

# Ruby: detailed benchmarking
ruby tools/profile_visitor.rb --iterations 50
```

## Performance Goals

### Target Overhead Ranges

| Language | No-op | Simple | Complex |
|----------|-------|--------|---------|
| Python | <15% | <25% | <45% |
| Node.js | <20% | <35% | <55% |
| Ruby | <15% | <25% | <40% |

These are guidelines; actual values depend on:
- Document complexity (more callbacks = better amortization)
- System load
- JIT warmup status
- GC pressure

## Contributing

When submitting changes that affect visitor callbacks:

1. Run baseline profiles before changes
2. Run profiles after changes
3. Include overhead metrics in PR description
4. Flag if overhead increases >5%
5. Explain any regression with optimization trade-offs

Example PR description:
```
## Visitor Callback Changes

- Updated visitor callback marshalling
- Reduces object allocations per callback

### Performance Impact
- Python: 27.3% → 24.1% overhead (-3.2%)
- Node.js: 32.5% → 31.0% overhead (-1.5%)
- Ruby: 26.8% → 25.2% overhead (-1.6%)
```

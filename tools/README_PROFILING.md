# Visitor Callback Profiling Tools

This directory contains profiling scripts to measure visitor callback overhead in html-to-markdown language bindings.

## Files Created

### Executable Scripts

1. **profile_visitor.py** (10 KB)
   - Python profiler using cProfile
   - Measures PyO3 binding overhead
   - Generates cProfile statistics
   - Use: `python tools/profile_visitor.py --scenario simple`

2. **profile_visitor.js** (10 KB)
   - Node.js profiler using perf_hooks
   - Measures NAPI-RS ThreadsafeFunction overhead
   - Tracks V8 GC impact
   - Use: `node tools/profile_visitor.js --scenario simple`

3. **profile_visitor.rb** (8.4 KB)
   - Ruby profiler using Benchmark module
   - Measures Magnus FFI binding overhead
   - Supports custom visitor classes
   - Use: `ruby tools/profile_visitor.rb --scenario simple`

### Documentation

1. **PROFILING_README.md** (9 KB)
   - Comprehensive documentation for all scripts
   - Detailed usage examples
   - Output format specification
   - Troubleshooting guide
   - Performance expectations

2. **VISITOR_PROFILING_GUIDE.md** (7.5 KB)
   - Quick reference guide
   - Common commands and shortcuts
   - Metric explanation
   - Analysis examples
   - Interpretation tips

3. **VISITOR_PROFILING_INTEGRATION.md** (8 KB)
   - Integration with benchmark-harness
   - CI/CD setup examples
   - Performance standards
   - Result combination strategies

## Quick Start

### Profile All Scenarios
```bash
# Python
python tools/profile_visitor.py

# Node.js
node tools/profile_visitor.js

# Ruby
ruby tools/profile_visitor.rb
```

### Profile Single Scenario
```bash
# Python - no-op visitor (pure binding overhead)
python tools/profile_visitor.py --scenario no-op

# Node.js - simple visitor (text extraction)
node tools/profile_visitor.js --scenario simple

# Ruby - complex visitor (aggregation)
ruby tools/profile_visitor.rb --scenario complex
```

### Different HTML Sizes
```bash
python tools/profile_visitor.py --html small
python tools/profile_visitor.py --html medium
python tools/profile_visitor.py --html large
```

## Test Scenarios

Each script tests 4 scenarios:

1. **no-op**: Empty visitor - measures pure binding overhead
2. **simple**: Text extraction - minimal work per callback
3. **custom-output**: Building output structures
4. **complex**: Multiple operations (stats tracking, aggregation)

## Key Features

### Python (profile_visitor.py)
- ✓ cProfile integration for detailed timing
- ✓ Flamegraph-compatible output
- ✓ Systematic warm-up measurement
- ✓ Custom output directories
- ✓ Per-callback timing breakdown

### Node.js (profile_visitor.js)
- ✓ perf_hooks for microsecond precision
- ✓ V8 GC measurement capability
- ✓ Heap growth tracking
- ✓ ThreadsafeFunction overhead measurement
- ✓ Support for --expose-gc flag

### Ruby (profile_visitor.rb)
- ✓ Ruby Benchmark module precision
- ✓ Multiple visitor class support
- ✓ Format-compatible output
- ✓ Clean console formatting
- ✓ Custom iteration control

## Output

All scripts produce `results.json` with:

```json
{
  "html_size": 45000,
  "element_count": 523,
  "timestamp": 1735312000,
  "results": [
    {
      "scenario": "no-op",
      "baseline_ms": 2.45,
      "visitor_ms": 3.12,
      "overhead_ms": 0.67,
      "overhead_percent": 27.3,
      "avg_callback_time_us": 1.28,
      "callback_invocations": 523,
      "iterations": 10
    }
  ]
}
```

## Metrics

| Metric | Meaning |
|--------|---------|
| baseline_ms | Conversion time without visitor |
| visitor_ms | Conversion time with visitor |
| overhead_ms | Absolute time cost (visitor - baseline) |
| overhead_percent | Relative overhead % |
| avg_callback_time_us | Average microseconds per callback |
| callback_invocations | Total visitor callbacks invoked |

## Use Cases

### Identify Binding Bottlenecks
```bash
python tools/profile_visitor.py --cprofile --scenario complex
# Examine profile_complex.txt for hot functions
```

### Compare Languages
```bash
# Run same scenario across all languages
python tools/profile_visitor.py --scenario simple --html large
node tools/profile_visitor.js --scenario simple --html large
ruby tools/profile_visitor.rb --scenario simple --html large
# Compare overhead_percent values
```

### Measure Scaling
```bash
# Test how overhead scales with document size
python tools/profile_visitor.py --html small
python tools/profile_visitor.py --html medium
python tools/profile_visitor.py --html large
# Overhead should remain relatively constant or improve
```

### Detect Regressions
```bash
# Compare against baseline
baseline=27.3  # Previous overhead %
current=29.1   # New overhead %

# Alert if >10% regression
if (( $(echo "$current > $baseline * 1.1" | bc -l) )); then
  echo "REGRESSION: Overhead increased"
fi
```

## Integration with CI/CD

All scripts exit with status 0 on success, making them CI-friendly:

```yaml
- name: Profile visitor overhead
  run: python tools/profile_visitor.py --iterations 20

- name: Check for regressions
  run: python -c "
    import json
    with open('visitor_profile_results/results.json') as f:
      data = json.load(f)
      for r in data['results']:
        if r['overhead_percent'] > 50:
          raise Exception(f'High overhead: {r[\"scenario\"]} {r[\"overhead_percent\"]}%')
  "
```

## Performance Expectations

### No-op Overhead
- Expected: 5-30% (pure binding overhead)
- Higher indicates expensive FFI crossings

### Simple Visitor
- Expected: 10-40% overhead
- Includes marshalling cost + light work

### Complex Visitor
- Expected: 20-60% overhead
- Marshalling + heavy computation

## Files Used

- Test HTML: `/test_documents/html/wikipedia/`
  - small_html.html (~5 KB)
  - medium_python.html (~45 KB)
  - large_rust.html (~500 KB)

- Output: `visitor_profile_results/` (default)
  - results.json (all scripts)
  - profile_*.txt (Python only, with --cprofile)

## Documentation Structure

```
tools/
├── README_PROFILING.md (this file)
├── PROFILING_README.md (comprehensive guide)
├── VISITOR_PROFILING_GUIDE.md (quick reference)
├── VISITOR_PROFILING_INTEGRATION.md (integration guide)
├── profile_visitor.py
├── profile_visitor.js
└── profile_visitor.rb
```

Start with **VISITOR_PROFILING_GUIDE.md** for quick reference.
See **PROFILING_README.md** for comprehensive documentation.
Check **VISITOR_PROFILING_INTEGRATION.md** for CI/CD setup.

## Troubleshooting

### Module not found errors
- Python: `pip install -e packages/python`
- Node.js: `cd packages/typescript && pnpm install && pnpm build`
- Ruby: `cd packages/ruby && bundle install && rake compile`

### Results show 0ms overhead
- Visitor API may not be implemented in binding
- Check if `convert_with_visitor` is available
- Regular `convert` is used as fallback

### High variance in results
- Increase iterations: `--iterations 30+`
- Close other applications
- Run multiple times
- System should be idle

### GC warnings in Node.js
- Normal: "GC measurements unavailable"
- To enable: `node --expose-gc tools/profile_visitor.js`
- Provides detailed gcHeapBefore/After metrics

## Next Steps

1. Run baseline profiles for current state
2. Store baseline results for regression detection
3. Set up CI/CD integration (see INTEGRATION guide)
4. Monitor for regressions on PRs
5. Use results to guide optimization efforts

## Related Tools

- **benchmark-harness**: General conversion performance
- **cProfile** (Python): Detailed function call profiling
- **V8 inspector** (Node.js): Full JavaScript profiling
- **ruby-prof** (Ruby): Advanced Ruby profiling

## License

These profiling tools are part of html-to-markdown and follow the same license.

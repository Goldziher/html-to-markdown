# Quick Reference: Visitor Callback Profiling

## Quick Start

### Python
```bash
cd /workspace/html-to-markdown
python tools/profile_visitor.py --scenario simple --html medium
```

### Node.js
```bash
cd /workspace/html-to-markdown
node tools/profile_visitor.js --scenario simple --html medium
```

### Ruby
```bash
cd /workspace/html-to-markdown
ruby tools/profile_visitor.rb --scenario simple --html medium
```

## What Gets Profiled

### Test Scenarios

Each language runs 4 identical test scenarios:

```
┌─────────────────┬──────────────────────────┐
│ Scenario        │ Description              │
├─────────────────┼──────────────────────────┤
│ no-op           │ Empty callbacks          │
│ simple          │ Text extraction          │
│ custom-output   │ Building output          │
│ complex         │ Multiple operations      │
└─────────────────┴──────────────────────────┘
```

### HTML Sizes

```
┌─────────┬──────────────┐
│ Size    │ Approx.      │
├─────────┼──────────────┤
│ small   │ 5 KB         │
│ medium  │ 45 KB        │
│ large   │ 500 KB       │
└─────────┴──────────────┘
```

## Output Format

All scripts produce a single `results.json` file with structure:

```json
{
  "html_size": 45000,
  "html_file": "medium_python.html",
  "element_count": 523,
  "timestamp": 1735312000,
  "results": [
    {
      "scenario": "no-op",
      "html_size_bytes": 45000,
      "element_count": 523,
      "baseline_ms": 2.45,
      "visitor_ms": 3.12,
      "overhead_ms": 0.67,
      "overhead_percent": 27.3,
      "callback_invocations": 523,
      "avg_callback_time_us": 1.28,
      "iterations": 10
    }
  ]
}
```

## Key Metrics Explained

| Metric | Meaning |
|--------|---------|
| `baseline_ms` | Time to convert without visitor (per iteration) |
| `visitor_ms` | Time to convert with visitor (per iteration) |
| `overhead_ms` | Absolute overhead: `visitor_ms - baseline_ms` |
| `overhead_percent` | Relative overhead: `(overhead / baseline) * 100` |
| `avg_callback_time_us` | Average microseconds per callback invocation |
| `callback_invocations` | Total number of visitor callbacks invoked |

## Common Commands

### Profile single scenario
```bash
python tools/profile_visitor.py --scenario no-op
node tools/profile_visitor.js --scenario simple
ruby tools/profile_visitor.rb --scenario complex
```

### Profile all scenarios with custom iterations
```bash
python tools/profile_visitor.py --scenario all --iterations 20
node tools/profile_visitor.js --scenario all --iterations 20
ruby tools/profile_visitor.rb --scenario all --iterations 20
```

### Custom output location
```bash
python tools/profile_visitor.py --output ./benchmark_results
node tools/profile_visitor.js --output ./benchmark_results
ruby tools/profile_visitor.rb --output ./benchmark_results
```

### Python: Generate detailed cProfile
```bash
python tools/profile_visitor.py --cprofile --scenario simple
# Examine profile_simple.txt for hot functions
```

### Node.js: Profile with GC tracking
```bash
node --expose-gc tools/profile_visitor.js --gc-profile
# Check gcHeapBefore/After in results.json
```

## Analysis Examples

### Compare languages
```bash
# Run on same HTML size across all languages
python tools/profile_visitor.py --html large --scenario simple --output py_results
node tools/profile_visitor.js --html large --scenario simple --output js_results
ruby tools/profile_visitor.rb --html large --scenario simple --output rb_results

# Extract overhead_percent from each results.json and compare
```

### Find bottleneck scenario
```bash
# Run all scenarios
python tools/profile_visitor.py --scenario all --iterations 30

# Review results.json:
# - No-op: baseline binding overhead
# - Simple: with minimal work
# - Custom-output: with output building
# - Complex: with aggregation/computation
# Higher values = more expensive visitor pattern
```

### Measure scaling
```bash
# Small HTML
python tools/profile_visitor.py --html small

# Medium HTML
python tools/profile_visitor.py --html medium

# Large HTML
python tools/profile_visitor.py --html large

# Compare how overhead scales with document size
```

## Interpreting Results

### Overhead Ranges

- **< 10%**: Negligible overhead, visitor is well-optimized
- **10-20%**: Acceptable overhead for most use cases
- **20-50%**: Moderate overhead, consider optimization opportunities
- **> 50%**: High overhead, significant marshalling/GC cost

### Scenario Analysis

**no-op overhead** shows pure binding overhead:
- Python PyO3: Expected 5-15%
- Node.js NAPI-RS: Expected 5-20% (may be higher due to ThreadsafeFunction)
- Ruby Magnus: Expected 5-15%

**increasing from no-op → simple → complex** shows:
- Marshalling cost (object creation per callback)
- GC pressure (object allocation overhead)
- Type conversion overhead

### Scaling Analysis

Overhead should remain **roughly constant** (as percentage) or **improve** with larger documents because:
- Fixed overhead gets amortized over more callbacks
- Callback cost per byte should decrease
- GC pressure spreads over more work

If overhead **increases** with size, indicates:
- O(n) overhead per callback (inefficient)
- GC thrashing from too many allocations
- Memory pressure from marshalling

## Script Features by Language

### Python (profile_visitor.py)
- cProfile integration for detailed timing
- Flamegraph-compatible output
- Systematic warm-up and baseline measurement
- Support for custom output directory

### Node.js (profile_visitor.js)
- V8 GC heap measurement
- Performance measurement via perf_hooks
- Supports --expose-gc for detailed GC tracking
- Memory usage tracking (gcHeapBefore/After)

### Ruby (profile_visitor.rb)
- Ruby Benchmark module for precision
- Support for custom visitor classes
- Format-compatible output with other scripts
- Clean console output with formatting

## Troubleshooting

### High variability in results
- Increase iterations: `--iterations 30` or more
- Run on idle system (close other apps)
- Run multiple times and average
- Consider warm-up time

### Results show 0ms
- Visitor API may not be implemented in your binding
- Check if `convert_with_visitor` is available
- Binding may be falling back to regular convert

### Script execution errors
- Ensure test HTML exists: `ls test_documents/html/wikipedia/`
- Check language runtime: `python --version`, `node --version`, `ruby --version`
- Build bindings if needed: `pnpm build` (Node), `bundle exec rake compile` (Ruby)

## Next Steps

After profiling:

1. **Identify bottleneck**: Which scenario shows highest overhead?
2. **Analyze**: Review cProfile (Python) or detailed logs for hot functions
3. **Optimize**: Focus on highest-cost operations
4. **Re-measure**: Run profiling again to verify improvements
5. **Track**: Store baseline results for regression detection

## Files Created

- `/tools/profile_visitor.py` - Python profiler
- `/tools/profile_visitor.js` - Node.js profiler
- `/tools/profile_visitor.rb` - Ruby profiler
- `/tools/PROFILING_README.md` - Detailed documentation
- `/tools/VISITOR_PROFILING_GUIDE.md` - This quick reference

All scripts are executable and can be run standalone.

# Visitor Callback Profiling Scripts

Profiling tools to measure visitor callback overhead in html-to-markdown bindings across Python, Node.js, and Ruby.

## Overview

These scripts profile the performance impact of visitor callbacks during HTML to Markdown conversion, measuring:

- **Callback invocation overhead**: Time spent calling visitor methods
- **Context marshalling cost**: Time to convert between language and Rust types
- **Result conversion overhead**: Time to handle return values
- **GC impact**: Memory allocation and garbage collection pressure
- **Per-callback timing**: Average time per visitor invocation

## Scripts

### 1. Python Profiling (`profile_visitor.py`)

Uses `cProfile` to profile visitor callback invocation in the PyO3 binding.

**Features:**
- Multiple visitor scenarios (no-op, simple, custom-output, complex)
- cProfile integration for detailed timing breakdowns
- Flamegraph-compatible output
- Measures callback marshalling and GC impact

**Usage:**
```bash
# Profile default scenario with medium HTML
python tools/profile_visitor.py

# Profile specific scenario
python tools/profile_visitor.py --scenario simple

# Use different HTML sizes
python tools/profile_visitor.py --html small
python tools/profile_visitor.py --html medium
python tools/profile_visitor.py --html large

# Run with custom iterations
python tools/profile_visitor.py --iterations 20

# Generate cProfile data for detailed analysis
python tools/profile_visitor.py --cprofile

# Specify output directory
python tools/profile_visitor.py --output ./my_results
```

**Available scenarios:**
- `no-op`: Empty visitor, measures baseline overhead
- `simple`: Extracts text content from nodes
- `custom-output`: Builds custom output structure
- `complex`: Performs multiple operations per callback

**Output:**
- `results.json`: Timing metrics in structured format
- `profile_*.txt`: cProfile statistics (if --cprofile used)

**JSON Results Format:**
```json
{
  "html_size": 45000,
  "html_file": "medium_python.html",
  "element_count": 523,
  "timestamp": 1735312000.123,
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

### 2. Node.js Profiling (`profile_visitor.js`)

Profiles visitor callback overhead in NAPI-RS bindings with V8 GC monitoring.

**Features:**
- ThreadsafeFunction callback profiling
- V8 garbage collection impact measurement
- Heap memory tracking
- Performance measurement via Node.js perf_hooks

**Usage:**
```bash
# Profile default scenario
node tools/profile_visitor.js

# Profile specific scenario
node tools/profile_visitor.js --scenario simple

# Use different HTML sizes
node tools/profile_visitor.js --html small
node tools/profile_visitor.js --html medium
node tools/profile_visitor.js --html large

# Enable GC profiling (requires Node.js --expose-gc flag)
node --expose-gc tools/profile_visitor.js --gc-profile

# Custom iterations
node tools/profile_visitor.js --iterations 15

# Specify output directory
node tools/profile_visitor.js --output ./my_results
```

**Available scenarios:**
- `no-op`: Empty visitor callback
- `simple`: Text extraction
- `custom-output`: Builds output structure
- `complex`: Multiple operations per callback

**Output:**
- `results.json`: Timing metrics and GC stats

**JSON Results Format:**
```json
{
  "htmlSize": 45000,
  "htmlFile": "medium_python.html",
  "elementCount": 523,
  "timestamp": 1735312000123,
  "results": [
    {
      "scenario": "no-op",
      "htmlSizeBytes": 45000,
      "elementCount": 523,
      "baselineMs": 2.45,
      "visitorMs": 3.12,
      "overheadMs": 0.67,
      "overheadPercent": 27.3,
      "callbackInvocations": 523,
      "avgCallbackTimeUs": 1.28,
      "iterations": 10,
      "gcHeapBefore": 8388608,
      "gcHeapAfter": 8912896
    }
  ]
}
```

### 3. Ruby Profiling (`profile_visitor.rb`)

Profiles visitor callback overhead in Magnus FFI bindings.

**Features:**
- Ruby Benchmark module for precise timing
- GC state monitoring
- Measures callback overhead and marshalling cost
- Support for custom visitor classes

**Usage:**
```bash
# Profile default scenario
ruby tools/profile_visitor.rb

# Profile specific scenario
ruby tools/profile_visitor.rb --scenario simple

# Use different HTML sizes
ruby tools/profile_visitor.rb --html small
ruby tools/profile_visitor.rb --html medium
ruby tools/profile_visitor.rb --html large

# Custom iterations
ruby tools/profile_visitor.rb --iterations 20

# Specify output directory
ruby tools/profile_visitor.rb --output ./my_results
```

**Available scenarios:**
- `no-op`: Empty visitor
- `simple`: Text extraction
- `custom-output`: Builds output structure
- `complex`: Multiple operations per callback

**Output:**
- `results.json`: Timing metrics

**JSON Results Format:**
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

## Test Documents

All scripts use the same test HTML documents from `test_documents/html/wikipedia/`:

- `small_html.html` (~5KB): Small Wikipedia article
- `medium_python.html` (~45KB): Medium Python documentation page
- `large_rust.html` (~500KB): Large Rust documentation

## Interpreting Results

### Overhead Metrics

- **baseline_ms**: Average conversion time without visitor
- **visitor_ms**: Average conversion time with visitor
- **overhead_ms**: Absolute time cost of using a visitor
- **overhead_percent**: Overhead as a percentage of baseline
- **avg_callback_time_us**: Average microseconds per callback invocation

### What's Being Measured

1. **No-op Visitor**: Pure callback invocation overhead with no user code
2. **Simple Visitor**: Minimal work per callback (object property access)
3. **Custom Output Visitor**: Building output structures in callbacks
4. **Complex Visitor**: Multiple operations and data aggregation per callback

### Performance Expectations

- **No-op overhead**: 5-30% (pure binding overhead)
- **Simple visitor**: 10-40% (marshalling + light work)
- **Complex visitor**: 20-60% (marshalling + computation)

Higher overhead indicates:
- Heavy object marshalling between languages
- GC pressure from callback objects
- Thread synchronization costs (Node.js ThreadsafeFunction)

## Workflow

1. **Baseline measurement**: Establish baseline without visitors
2. **Scenario profiling**: Test different visitor workloads
3. **Bottleneck identification**: Compare scenarios to find overhead sources
4. **Optimization**: Use detailed timing to guide optimizations

## Examples

### Python: Find bottleneck scenario
```bash
python tools/profile_visitor.py --html small --iterations 20 --cprofile
# Then examine profile_<scenario>.txt for hot functions
```

### Node.js: Profile GC impact
```bash
node --expose-gc tools/profile_visitor.js --html large --gc-profile
# Check gcHeapBefore/After to see memory pressure
```

### Ruby: Compare all scenarios
```bash
ruby tools/profile_visitor.rb --html medium --iterations 15
# Compare results across all scenarios
```

### Cross-language comparison
```bash
# Run all three on same HTML size for language comparison
python tools/profile_visitor.py --html medium --scenario simple
node tools/profile_visitor.js --html medium --scenario simple
ruby tools/profile_visitor.rb --html medium --scenario simple
# Compare overhead_percent across languages
```

## Integration with CI

These scripts can be integrated into CI/CD pipelines to:
- Track performance regression with each PR
- Monitor visitor callback overhead over time
- Compare binding implementations across languages
- Generate performance reports

Example GitHub Actions workflow could use these scripts to generate
baseline metrics and alert on regressions.

## Troubleshooting

### Python: Module not found
```bash
# Make sure to build the extension
cd packages/python && pip install -e .
```

### Node.js: Module not found
```bash
# Build TypeScript bindings
cd packages/typescript && pnpm install && pnpm build
```

### Ruby: Gem not found
```bash
# Build and install gem
cd packages/ruby && bundle install && rake compile
```

### Results show 0ms overhead
- The visitor feature may not be fully implemented in your binding
- Check if `convert_with_visitor` is available
- Fall back to regular `convert` for comparison

### High variance in results
- Increase iterations with `--iterations`
- Close other applications
- Disable other processes
- Run multiple times and average results

## File Locations

- Scripts: `/tools/profile_visitor.{py,js,rb}`
- Test HTML: `/test_documents/html/wikipedia/*.html`
- Results: `visitor_profile_results/results.json` (default)

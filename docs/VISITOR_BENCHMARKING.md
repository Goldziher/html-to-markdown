# Visitor Pattern Benchmarking Guide

This document describes the visitor pattern benchmarking infrastructure integrated into the CI/CD pipeline.

## Overview

The visitor pattern in html-to-markdown allows users to intervene in the HTML→Markdown conversion process with custom callbacks. This benchmarking suite measures the performance overhead introduced by the visitor infrastructure across different usage patterns.

## Benchmark Categories

The benchmarking system measures overhead across four visitor configuration levels:

### 1. **Baseline (No-op Visitor)**
- **Threshold**: <10% overhead
- **Description**: Measures the infrastructure overhead of instantiating a no-op visitor that returns `Continue` for all callbacks
- **Use Case**: Detecting overhead from visitor dispatch and callback invocation machinery
- **Fixtures**: `visitor_baseline.toml`

### 2. **Simple Callbacks**
- **Threshold**: <30% overhead
- **Description**: Measures overhead with simple visitor callbacks (e.g., text, link, image callbacks)
- **Use Case**: Typical user-facing visitor patterns with basic transformations
- **Fixtures**: `visitor_callbacks.toml`

### 3. **Custom Transformations**
- **Threshold**: <40% overhead
- **Description**: Measures overhead with custom element transformations that return modified markdown
- **Use Case**: Advanced visitor patterns with element replacement
- **Fixtures**: `visitor_custom.toml`

### 4. **Complex Visitors**
- **Threshold**: <60% overhead
- **Description**: Measures overhead with complex visitors that maintain state, traverse multiple elements, or perform expensive computations
- **Use Case**: Worst-case visitor scenarios with multiple callback types and stateful tracking
- **Fixtures**: `visitor_complex.toml`

## Fixture Files

Fixture files are TOML configurations defining benchmark scenarios. Located in `tools/benchmark-harness/fixtures/`:

```toml
[[fixtures]]
id = "visitor_baseline_small"
name = "Visitor Baseline (Small - No-op)"
path = "test_documents/html/wikipedia/small_html.html"
category = "visitor-baseline"
iterations = 200
options = { visitor_mode = "noop" }
```

**Fields:**
- `id`: Unique identifier for the fixture
- `name`: Human-readable name
- `path`: Path to HTML test document
- `category`: Benchmark category
- `iterations`: Number of conversion iterations per run
- `options`: Framework-specific options (e.g., `visitor_mode`)

## GitHub Actions Integration

### Visitor Benchmarks Job

Runs on every push/PR to main when benchmark-related paths change:

```yaml
visitor-benchmarks:
  runs-on: ubuntu-latest
  strategy:
    matrix:
      framework: [python, ruby, typescript]
```

**Steps:**
1. Checkout and setup environment for each framework
2. Build extensions with `visitor` feature enabled
3. Run 4 fixture sets (baseline, callbacks, custom, complex)
4. Analyze results for performance regressions
5. Upload results as artifacts (90-day retention)

### Visitor Benchmarks Schedule Job

Runs weekly on **Monday at 2 AM UTC** (`0 2 * * 1`) for comprehensive multi-framework testing:

```yaml
visitor-benchmarks-schedule:
  runs-on: ubuntu-latest
  if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
  strategy:
    matrix:
      framework: [rust, python, ruby, typescript, node, php, go, java, csharp]
```

This job:
- Tests all 9 frameworks
- Runs 5 iterations + 2 warmup (more comprehensive than PR checks)
- Generates detailed results
- Uploads comprehensive results (180-day retention)

## Performance Regression Detection

### How It Works

The `scripts/ci/analyze-visitor-benchmarks.py` script:

1. **Loads results** from all 4 benchmark categories
2. **Extracts metrics** for the tested framework
3. **Compares** each category against baseline metrics
4. **Calculates overhead** as: `(comparison - baseline) / baseline * 100%`
5. **Validates** overhead against thresholds
6. **Reports** any regressions and exits with appropriate status code

### Overhead Calculation

For fixture `visitor_baseline_small`:

```
Baseline:   2.50ms
Callbacks:  2.80ms
Overhead:   (2.80 - 2.50) / 2.50 * 100% = 12%
Status:     REGRESSION (threshold: 10%)
```

### Thresholds by Category

Default thresholds (configurable):

| Category | Threshold | Rationale |
|----------|-----------|-----------|
| No-op visitor | 10% | Infrastructure overhead should be minimal |
| Simple callbacks | 30% | Typical user patterns with acceptable cost |
| Custom transforms | 40% | More complex operations permitted |
| Complex visitors | 60% | Worst-case scenarios with stateful logic |

## Running Locally

### Manual Benchmark Execution

```bash
# Run visitor baseline benchmarks for Python
cargo run --release --features visitor \
  --manifest-path tools/benchmark-harness/Cargo.toml -- \
  run --fixtures "tools/benchmark-harness/fixtures/visitor_baseline.toml" \
  --frameworks python \
  --iterations 3 --warmup 1 \
  --format both --output tools/benchmark-harness/results/visitor-baseline
```

### Analysis Script

```bash
# Analyze results
python3 scripts/ci/analyze-visitor-benchmarks.py \
  --baseline tools/benchmark-harness/results/visitor-baseline/results.json \
  --callbacks tools/benchmark-harness/results/visitor-callbacks/results.json \
  --custom tools/benchmark-harness/results/visitor-custom/results.json \
  --complex tools/benchmark-harness/results/visitor-complex/results.json \
  --framework python \
  --thresholds '{"baseline": 10, "callbacks": 30, "complex": 60}'
```

## Interpreting Results

### Artifact Structure

Results are organized as:

```
visitor-benchmark-results-{framework}/
├── visitor-baseline/
│   ├── results.json       # Raw benchmark data
│   ├── summary.json       # Statistical summary
│   └── report.html        # HTML report (if available)
├── visitor-callbacks/
├── visitor-custom/
└── visitor-complex/
```

### Sample Result Output

```
======================================================================
Visitor Benchmark Analysis: SIMPLE CALLBACKS
Threshold: 30% overhead
======================================================================
✓ visitor_callbacks_small:
    Baseline:   2.50ms
    Comparison: 2.95ms
    Overhead:   +18.0% (OK)

✗ visitor_callbacks_medium:
    Baseline:   8.20ms
    Comparison: 10.50ms
    Overhead:   +28.0% (OK)

✗ visitor_callbacks_large:
    Baseline:   45.30ms
    Comparison: 62.40ms
    Overhead:   +37.8% (REGRESSION)
```

## Adding New Visitor Benchmarks

### Creating a New Fixture

1. Create HTML test document in `test_documents/html/visitor/` or reference existing Wikipedia fixtures
2. Add fixture entry to appropriate TOML file:

```toml
[[fixtures]]
id = "visitor_custom_edge_case"
name = "Custom Edge Case Handler"
path = "test_documents/html/visitor/edge_cases.html"
category = "visitor-custom"
iterations = 100
options = { visitor_mode = "custom_transforms" }
```

3. Update fixture files in workflow if needed

### Updating Thresholds

Thresholds are configured in `profiling.yaml` in the "Analyze performance regressions" step:

```yaml
--thresholds '{"baseline": 10, "callbacks": 30, "custom": 40, "complex": 60}'
```

Update based on empirical measurements and performance goals.

## Troubleshooting

### High Baseline Overhead (>10%)

- Indicates infrastructure overhead is significant
- May need to optimize `VisitorHandle` dispatch
- Check if visitor feature has compile-time impact

### Callback Overhead Exceeds Threshold (>30%)

- Typical causes:
  - Allocation overhead in callback dispatch
  - String cloning in context building
  - Unnecessary box/enum allocations
- Solutions:
  - Profile with flamegraph in full profiling job
  - Consider zero-copy patterns
  - Evaluate callback dispatch optimization

### Inconsistent Results Across Frameworks

- Framework-specific FFI overhead can vary
- Python/Ruby/PHP may have higher overhead due to FFI marshaling
- Consider framework-specific thresholds if needed

### Missing Results

- Verify fixture TOML path is correct
- Check HTML test documents exist
- Ensure framework is built with `--features visitor`
- Check benchmark harness output for errors

## Integration with GitHub Pages

Future integration could display visitor benchmark trends:

1. Generate historical data after each run
2. Upload to GitHub Pages branch
3. Create interactive dashboard showing:
   - Per-framework overhead trends
   - Regression detection history
   - Threshold compliance timeline

## Performance Best Practices

### For Library Maintainers

- Keep baseline overhead under 10% where possible
- Use inline visitor dispatch when applicable
- Avoid allocations in hot paths of visitor infrastructure
- Consider context pre-allocation for reuse

### For Library Users

- Use visitors only when needed (zero-cost when feature is disabled)
- Keep visitor callbacks lightweight
- Batch operations when possible
- Profile with framework-specific tools for detailed analysis

## References

- [Visitor Pattern Documentation](../docs/VISITOR.md)
- [Benchmarking Harness](../tools/benchmark-harness/README.md)
- [Performance Guidelines](../docs/PERFORMANCE.md)

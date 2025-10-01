# html-to-markdown v2 Implementation Plan

## Overview

This document outlines the staged implementation approach for v2, which replaces BeautifulSoup with lol-html (Rust streaming parser) and nh3 (Python ammonia bindings) with direct Rust ammonia.

## Migration Notes

### nh3 → ammonia (Direct Rust)

- **Current (v1)**: Uses nh3 (Python bindings to ammonia crate) for HTML sanitization
- **New (v2)**: Direct use of ammonia in Rust layer
- **Why**: Eliminates Python→Rust boundary overhead, better integration with lol-html
- **Performance Impact**: Expected additional 2-3x improvement from eliminating bindings layer

### BeautifulSoup → lol-html

- **Current (v1)**: BeautifulSoup (DOM-based, Python)
- **New (v2)**: lol-html (streaming, Rust)
- **Why**: True streaming support, 2x faster than html5ever, memory efficient
- **Performance Impact**: 10-20x improvement expected

## Stage 1: Python API Refactor + Baseline Benchmarks

**Goal**: Update to new functional API, establish performance baseline with current BeautifulSoup backend.

### 1.1: Update Python API

- [ ] Create option dataclasses (`ConversionOptions`, `PreprocessingOptions`, etc.)
- [ ] Implement new functional API (`convert()`, `convert_stream()`)
- [ ] Keep backward compatibility layer (`convert_to_markdown()` with deprecation warning)
- [ ] Update type hints and docstrings
- [ ] Maintain all existing features (hOCR, metadata extraction, streaming, etc.)

**Files to modify**:

- `html_to_markdown/__init__.py` - New exports
- `html_to_markdown/options.py` - New file for dataclasses
- `html_to_markdown/processing.py` - Refactor to use new API
- `html_to_markdown/converters.py` - Update function signatures if needed

### 1.2: Update Tests

- [ ] Update all tests to use new API
- [ ] Ensure 100% of existing tests pass
- [ ] Verify coverage remains at 100%
- [ ] Test backward compatibility layer

**Files to modify**:

- `tests/*.py` - Update test fixtures and assertions

### 1.3: Add Comprehensive Benchmarks

Create benchmark suite using `pytest-benchmark` to measure:

#### Throughput Benchmarks

- **Small documents** (< 10KB): Simple HTML, single page
- **Medium documents** (10-100KB): Wikipedia articles, blog posts
- **Large documents** (> 100KB): Long-form content, documentation
- **hOCR documents**: OCR output (special handling)

Metrics:

- Operations per second
- Time per document size
- Scaling characteristics

#### Memory Benchmarks

- **Peak memory usage**: Maximum RSS during conversion
- **Memory scaling**: Memory usage vs document size
- **Streaming efficiency**: Memory usage difference between `convert()` and `convert_stream()`

Metrics (using `memory_profiler` or `tracemalloc`):

- Peak memory (MB)
- Memory growth rate
- Allocations count

#### Feature Benchmarks

- **Table conversion**: Complex tables with rowspan/colspan
- **List conversion**: Deeply nested lists
- **Metadata extraction**: Documents with extensive metadata
- **Custom converters**: Impact of custom element handlers
- **Preprocessing**: Different preset levels (minimal/standard/aggressive)

#### Test Documents

**Current test documents** (verify we have these):

- Simple HTML samples
- Complex tables
- hOCR documents
- Nested structures

**New Wikipedia documents to download**:

- [ ] Short article (~5KB): <https://en.wikipedia.org/wiki/HTML>
- [ ] Medium article (~50KB): <https://en.wikipedia.org/wiki/Python_(programming_language)>
- [ ] Long article (~200KB): <https://en.wikipedia.org/wiki/Rust_(programming_language)>
- [ ] Table-heavy article: <https://en.wikipedia.org/wiki/List_of_countries_by_population_(United_Nations)>
- [ ] List-heavy article: <https://en.wikipedia.org/wiki/Timeline_of_computing>

**Storage**:

```
tests/benchmark_documents/
├── wikipedia/
│   ├── small_html.html
│   ├── medium_python.html
│   ├── large_rust.html
│   ├── tables_countries.html
│   └── lists_timeline.html
├── hocr/
│   └── sample_ocr.html
└── synthetic/
    ├── deep_nesting.html
    ├── complex_table.html
    └── large_metadata.html
```

#### Benchmark Implementation

**File structure**:

```
tests/benchmarks/
├── __init__.py
├── conftest.py              # Shared fixtures
├── test_throughput.py       # Ops/sec benchmarks
├── test_memory.py           # Memory profiling
├── test_features.py         # Feature-specific benchmarks
└── test_scaling.py          # Scaling characteristics
```

**Example benchmark**:

```python
# tests/benchmarks/test_throughput.py
import pytest
from html_to_markdown import convert, ConversionOptions

@pytest.fixture
def small_doc():
    with open("tests/benchmark_documents/wikipedia/small_html.html") as f:
        return f.read()

@pytest.fixture
def options():
    return ConversionOptions(
        heading_style="atx",
        list_indent_width=2,
    )

def test_convert_small_document(benchmark, small_doc, options):
    """Benchmark conversion of small Wikipedia article."""
    result = benchmark(convert, small_doc, options)
    assert len(result) > 0

@pytest.mark.parametrize("doc_size", ["small", "medium", "large"])
def test_convert_scaling(benchmark, doc_size, options):
    """Benchmark conversion scaling across document sizes."""
    with open(f"tests/benchmark_documents/wikipedia/{doc_size}.html") as f:
        html = f.read()

    result = benchmark(convert, html, options)
    assert len(result) > 0
```

### 1.4: Run Baseline Benchmarks

- [ ] Run full benchmark suite with pytest-benchmark
- [ ] Save results with `--benchmark-save=baseline_v1`
- [ ] Generate comparison report
- [ ] Document results in `BENCHMARKS.md`

**Commands**:

```bash
# Run benchmarks
uv run pytest tests/benchmarks/ --benchmark-only --benchmark-save=baseline_v1

# Generate report
uv run pytest-benchmark compare baseline_v1 --csv=baseline_v1.csv

# With memory profiling
uv run pytest tests/benchmarks/test_memory.py --memray
```

### 1.5: Commit Stage 1

- [ ] Verify all tests pass (100% coverage)
- [ ] Verify benchmarks run successfully
- [ ] Commit changes with benchmark results
- [ ] Tag as `v2.0.0-alpha.1`

**Commit message**:

```
feat: refactor to functional API with baseline benchmarks

- Replace class-based API with functional design
- Introduce ConversionOptions dataclasses
- Add comprehensive pytest-benchmark suite
- Add Wikipedia test documents for benchmarking
- Establish baseline: [X] ops/sec, [Y] MB peak memory
- Maintain 100% test coverage and backward compatibility

BREAKING CHANGE: New API uses convert() with options instead of
convert_to_markdown() with kwargs. Old API deprecated but still works.

Benchmark results:
- Small docs (5KB): X ops/sec
- Medium docs (50KB): Y ops/sec
- Large docs (200KB): Z ops/sec
- Peak memory: A MB
```

## Stage 2: Rust Backend Implementation

**Goal**: Replace BeautifulSoup + nh3 with lol-html + ammonia (direct Rust).

### 2.1: Core Rust Implementation

- [ ] Add lol-html and ammonia to `Cargo.toml`
- [ ] Implement HTML sanitization with ammonia
- [ ] Implement streaming parser with lol-html
- [ ] Port element converters from Python to Rust
- [ ] Implement ElementContext abstraction
- [ ] Support custom element handlers (via callbacks from Rust→Python)

**Files to create/modify**:

```
crates/html-to-markdown/src/
├── lib.rs                    # Public API
├── parser.rs                 # lol-html integration
├── sanitizer.rs              # ammonia integration
├── converters/
│   ├── mod.rs
│   ├── headings.rs
│   ├── links.rs
│   ├── lists.rs
│   ├── tables.rs
│   └── inline.rs
├── options.rs                # Option structs (mirror Python dataclasses)
└── context.rs                # ElementContext type
```

### 2.2: Python Bindings

- [ ] Create PyO3 bindings in `crates/html-to-markdown-py`
- [ ] Expose `convert()` and `convert_stream()` to Python
- [ ] Convert Python dataclasses → Rust structs
- [ ] Handle custom converters (Python callbacks)
- [ ] Implement streaming with Python generators

**Files**:

```
crates/html-to-markdown-py/src/
├── lib.rs                    # PyO3 module
├── options.rs                # Python→Rust option conversion
└── handlers.rs               # Custom handler bridge
```

### 2.3: Port Special Features

- [ ] Port hOCR detection and processing
- [ ] Port metadata extraction
- [ ] Port table handling (rowspan/colspan)
- [ ] Port whitespace normalization
- [ ] Ensure feature parity with v1

### 2.4: Testing

- [ ] All v1 tests pass with Rust backend
- [ ] Add Rust unit tests
- [ ] Test custom converters work
- [ ] Test streaming functionality
- [ ] Verify 100% feature parity

### 2.5: Benchmarking

- [ ] Run same benchmark suite from Stage 1
- [ ] Save results with `--benchmark-save=v2_rust`
- [ ] Compare against baseline: `pytest-benchmark compare baseline_v1 v2_rust`
- [ ] Document improvements in `BENCHMARKS.md`

**Expected improvements**:

- **Throughput**: 10-20x faster
- **Memory**: 50-70% reduction in peak memory
- **Streaming**: 80-90% memory reduction for large documents

### 2.6: Commit Stage 2

- [ ] All tests pass
- [ ] Benchmarks show significant improvement
- [ ] Update CHANGELOG.md with migration notes
- [ ] Tag as `v2.0.0-beta.1`

## Stage 3: Documentation & Release

### 3.1: Update Documentation

- [ ] Update README.md with new API examples
- [ ] Document migration guide (v1 → v2)
- [ ] Update API reference
- [ ] Add performance comparison section
- [ ] Document nh3 → ammonia migration

### 3.2: CHANGELOG.md

Add section:

```markdown
## [2.0.0] - YYYY-MM-DD

### Changed

- **BREAKING**: New functional API with dataclass options
- **BREAKING**: Minimum Python version 3.10
- Replaced BeautifulSoup with lol-html (Rust streaming parser)
- Replaced nh3 (Python ammonia bindings) with direct Rust ammonia integration

### Performance

- 10-20x faster conversion throughput
- 50-70% reduction in memory usage
- True streaming support with minimal memory overhead

### Added

- Comprehensive benchmark suite with pytest-benchmark
- ConversionOptions, PreprocessingOptions, StreamingOptions dataclasses
- ElementContext abstraction for custom handlers

### Deprecated

- convert_to_markdown() with kwargs (use convert() with options instead)
- BeautifulSoup instance as input (will be removed in v3.0)

### Migration Notes

- **nh3 users**: HTML sanitization now happens in Rust layer via ammonia
  - No API changes needed
  - 2-3x additional performance improvement from eliminating Python bindings
- **Custom converters**: Update to use ElementContext instead of BeautifulSoup Tag
- See MIGRATION.md for detailed guide
```

### 3.3: Release

- [ ] Final testing on all platforms (Linux, macOS, Windows)
- [ ] Build wheels with maturin
- [ ] Test installation from wheels
- [ ] Tag `v2.0.0`
- [ ] Publish to PyPI
- [ ] Publish crates to crates.io
- [ ] Create GitHub release with benchmark comparison

## Success Criteria

### Stage 1 Complete When

- ✅ All tests pass with new API
- ✅ 100% code coverage maintained
- ✅ Baseline benchmarks saved
- ✅ Wikipedia test documents downloaded

### Stage 2 Complete When

- ✅ All tests pass with Rust backend
- ✅ 10x+ throughput improvement shown in benchmarks
- ✅ 50%+ memory reduction shown in benchmarks
- ✅ Feature parity with v1 verified

### Release Ready When

- ✅ Documentation updated
- ✅ Migration guide written
- ✅ CHANGELOG.md complete
- ✅ Wheels build on all platforms
- ✅ Performance improvements validated

## Timeline Estimate

- **Stage 1** (API + Benchmarks): 1-2 days
- **Stage 2** (Rust Implementation): 3-5 days
- **Stage 3** (Docs + Release): 1-2 days

**Total**: 5-9 days for complete v2.0.0 release

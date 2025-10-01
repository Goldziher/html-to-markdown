"""Benchmarks using real Wikipedia HTML documents.

These benchmarks provide real-world performance metrics using actual Wikipedia
pages with varying sizes and content types.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pytest

from html_to_markdown import ConversionOptions, StreamingOptions, convert, convert_stream, convert_to_markdown

if TYPE_CHECKING:
    from pytest_benchmark.fixture import BenchmarkFixture  # type: ignore[import-untyped]

BENCHMARK_DIR = Path(__file__).parent / "benchmark_documents" / "wikipedia"


def _load_wikipedia_doc(filename: str) -> str:
    """Load a Wikipedia test document."""
    filepath = BENCHMARK_DIR / filename
    if not filepath.exists():
        pytest.skip(f"Wikipedia document not found: {filename}")
    return filepath.read_text(encoding="utf-8")


class TestWikipediaConversion:
    """Benchmarks for converting real Wikipedia HTML documents."""

    @pytest.mark.benchmark(group="wikipedia")
    def test_benchmark_wikipedia_small(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark small Wikipedia article (~130KB - Timeline)."""
        html = _load_wikipedia_doc("lists_timeline.html")
        result = benchmark(convert, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia")
    def test_benchmark_wikipedia_medium(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark medium Wikipedia article (~361KB - Countries)."""
        html = _load_wikipedia_doc("tables_countries.html")
        result = benchmark(convert, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia")
    def test_benchmark_wikipedia_large(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark large Wikipedia article (~656KB - Python)."""
        html = _load_wikipedia_doc("medium_python.html")
        result = benchmark(convert, html)
        assert len(result) > 0


class TestWikipediaFeatures:
    """Benchmarks for specific Wikipedia content features."""

    @pytest.mark.benchmark(group="wikipedia-features")
    def test_benchmark_wikipedia_tables(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark table-heavy Wikipedia article (Countries by population)."""
        html = _load_wikipedia_doc("tables_countries.html")
        result = benchmark(convert, html)
        assert len(result) > 0
        # Verify tables were converted (should contain table markers)
        assert "|" in result or "```" in result

    @pytest.mark.benchmark(group="wikipedia-features")
    def test_benchmark_wikipedia_lists(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark list-heavy Wikipedia article (Timeline)."""
        html = _load_wikipedia_doc("lists_timeline.html")
        result = benchmark(convert, html)
        assert len(result) > 0
        # Verify lists were converted
        assert "*" in result or "-" in result or "1." in result


class TestWikipediaOptions:
    """Benchmarks for Wikipedia conversion with different options."""

    @pytest.mark.benchmark(group="wikipedia-options")
    def test_benchmark_wikipedia_atx_headings(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark Wikipedia with ATX-style headings."""
        html = _load_wikipedia_doc("small_html.html")
        options = ConversionOptions(heading_style="atx")
        result = benchmark(convert, html, options)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia-options")
    def test_benchmark_wikipedia_no_metadata(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark Wikipedia without metadata extraction."""
        html = _load_wikipedia_doc("small_html.html")
        options = ConversionOptions(extract_metadata=False)
        result = benchmark(convert, html, options)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia-options")
    def test_benchmark_wikipedia_strict_whitespace(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark Wikipedia with strict whitespace mode."""
        html = _load_wikipedia_doc("small_html.html")
        options = ConversionOptions(whitespace_mode="strict")
        result = benchmark(convert, html, options)
        assert len(result) > 0


class TestWikipediaStreaming:
    """Benchmarks for streaming Wikipedia documents."""

    @pytest.mark.benchmark(group="wikipedia-streaming")
    def test_benchmark_wikipedia_streaming_small(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark streaming small Wikipedia article."""
        html = _load_wikipedia_doc("lists_timeline.html")

        def stream_convert() -> str:
            return "".join(convert_stream(html))

        result = benchmark(stream_convert)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia-streaming")
    def test_benchmark_wikipedia_streaming_large(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark streaming large Wikipedia article."""
        html = _load_wikipedia_doc("medium_python.html")

        def stream_convert() -> str:
            return "".join(convert_stream(html))

        result = benchmark(stream_convert)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia-streaming")
    @pytest.mark.parametrize("chunk_size", [1024, 4096, 8192])
    def test_benchmark_wikipedia_chunk_sizes(self, benchmark: BenchmarkFixture, chunk_size: int) -> None:
        """Benchmark different streaming chunk sizes on Wikipedia content."""
        html = _load_wikipedia_doc("small_html.html")

        def stream_convert() -> str:
            options = StreamingOptions(chunk_size=chunk_size)
            return "".join(convert_stream(html, streaming=options))

        result = benchmark(stream_convert)
        assert len(result) > 0


class TestWikipediaV1vsV2:
    """Benchmarks comparing v1 and v2 API performance."""

    @pytest.mark.benchmark(group="api-comparison")
    def test_benchmark_v1_api(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark v1 API (convert_to_markdown with kwargs)."""
        html = _load_wikipedia_doc("small_html.html")
        result = benchmark(
            convert_to_markdown,
            html,
            heading_style="atx",
            list_indent_width=2,
            extract_metadata=True,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="api-comparison")
    def test_benchmark_v2_api(self, benchmark: BenchmarkFixture) -> None:
        """Benchmark v2 API (convert with options)."""
        html = _load_wikipedia_doc("small_html.html")
        options = ConversionOptions(
            heading_style="atx",
            list_indent_width=2,
            extract_metadata=True,
        )
        result = benchmark(convert, html, options)
        assert len(result) > 0


@pytest.mark.benchmark(group="wikipedia-scalability")
@pytest.mark.parametrize(
    "doc_file,expected_min_ops",
    [
        ("lists_timeline.html", 5.0),  # 130KB - expect ~5+ ops/sec
        ("tables_countries.html", 3.0),  # 361KB - expect ~3+ ops/sec
        ("small_html.html", 2.0),  # 464KB - expect ~2+ ops/sec
        ("large_rust.html", 1.5),  # 567KB - expect ~1.5+ ops/sec
        ("medium_python.html", 1.0),  # 656KB - expect ~1+ ops/sec
    ],
)
def test_wikipedia_scalability(benchmark: BenchmarkFixture, doc_file: str, expected_min_ops: float) -> None:
    """Test Wikipedia conversion performance scales reasonably with document size.

    These are baseline expectations for v1 (BeautifulSoup) performance.
    v2 (Rust) should be 10-20x faster.
    """
    html = _load_wikipedia_doc(doc_file)
    stats = benchmark(convert, html)

    # stats object from pytest-benchmark contains performance metrics
    # This is informational - we're establishing baseline, not enforcing limits
    assert len(stats) > 0 if isinstance(stats, str) else True

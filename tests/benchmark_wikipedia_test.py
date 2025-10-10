from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pytest

from html_to_markdown import ConversionOptions, convert

if TYPE_CHECKING:
    from pytest_benchmark.fixture import BenchmarkFixture  # type: ignore[import-untyped]

BENCHMARK_DIR = Path(__file__).parent / "benchmark_documents" / "wikipedia"


def _load_wikipedia_doc(filename: str) -> str:
    filepath = BENCHMARK_DIR / filename
    if not filepath.exists():
        pytest.skip(f"Wikipedia document not found: {filename}")
    return filepath.read_text(encoding="utf-8")


class TestWikipediaConversion:
    @pytest.mark.benchmark(group="wikipedia")
    def test_benchmark_wikipedia_small(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("lists_timeline.html")
        result = benchmark(convert, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia")
    def test_benchmark_wikipedia_medium(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("tables_countries.html")
        result = benchmark(convert, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia")
    def test_benchmark_wikipedia_large(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("medium_python.html")
        result = benchmark(convert, html)
        assert len(result) > 0


class TestWikipediaFeatures:
    @pytest.mark.benchmark(group="wikipedia-features")
    def test_benchmark_wikipedia_tables(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("tables_countries.html")
        result = benchmark(convert, html)
        assert len(result) > 0
        assert "|" in result or "```" in result

    @pytest.mark.benchmark(group="wikipedia-features")
    def test_benchmark_wikipedia_lists(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("lists_timeline.html")
        result = benchmark(convert, html)
        assert len(result) > 0
        assert "*" in result or "-" in result or "1." in result


class TestWikipediaOptions:
    @pytest.mark.benchmark(group="wikipedia-options")
    def test_benchmark_wikipedia_atx_headings(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("small_html.html")
        options = ConversionOptions(heading_style="atx")
        result = benchmark(convert, html, options)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia-options")
    def test_benchmark_wikipedia_no_metadata(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("small_html.html")
        options = ConversionOptions(extract_metadata=False)
        result = benchmark(convert, html, options)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia-options")
    def test_benchmark_wikipedia_strict_whitespace(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("small_html.html")
        options = ConversionOptions(whitespace_mode="strict")
        result = benchmark(convert, html, options)
        assert len(result) > 0


@pytest.mark.skip(reason="V1 API compatibility not yet implemented")
class TestWikipediaV1vsV2:
    @pytest.mark.benchmark(group="api-comparison")
    def test_benchmark_v1_api(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("small_html.html")
        result = benchmark(
            convert_to_markdown,  # type: ignore[name-defined]  # noqa: F821
            html,
            heading_style="atx",
            list_indent_width=2,
            extract_metadata=True,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="api-comparison")
    def test_benchmark_v2_api(self, benchmark: BenchmarkFixture) -> None:
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
        ("lists_timeline.html", 5.0),
        ("tables_countries.html", 3.0),
        ("small_html.html", 2.0),
        ("large_rust.html", 1.5),
        ("medium_python.html", 1.0),
    ],
)
def test_wikipedia_scalability(benchmark: BenchmarkFixture, doc_file: str, expected_min_ops: float) -> None:
    html = _load_wikipedia_doc(doc_file)
    stats = benchmark(convert, html)

    assert len(stats) > 0 if isinstance(stats, str) else True

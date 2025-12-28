"""Benchmark comparison between v1 and v2 APIs.

This module benchmarks the v1 compatibility layer (convert_to_markdown with kwargs)
versus the modern v2 API (convert with ConversionOptions dataclass).

Both APIs use the same Rust backend, so this measures API overhead only.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from html_to_markdown import ConversionOptions, PreprocessingOptions, convert
from html_to_markdown.v1_compat import convert_to_markdown

from .conftest import TEST_DOCUMENTS_DIR

if TYPE_CHECKING:
    from pytest_benchmark.fixture import BenchmarkFixture  # type: ignore[import-not-found]

pytestmark = pytest.mark.filterwarnings("ignore::DeprecationWarning")

try:
    from .performance_test import generate_complex_html
except ImportError:
    from tests.performance_test import generate_complex_html

BENCHMARK_DIR = TEST_DOCUMENTS_DIR / "html" / "wikipedia"


def _load_wikipedia_doc(filename: str) -> str:
    filepath = BENCHMARK_DIR / filename
    if not filepath.exists():
        pytest.skip(f"Wikipedia document not found: {filename}")
    return filepath.read_text(encoding="utf-8")


class TestAPIOverhead:
    @pytest.mark.benchmark(group="api_overhead_small")
    def test_v1_api_small_doc(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=10)

        result = benchmark(
            convert_to_markdown,
            html,
            heading_style="atx",
            list_indent_width=2,
            escape_asterisks=False,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="api_overhead_small")
    def test_v2_api_small_doc(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=10)
        options = ConversionOptions(
            heading_style="atx",
            list_indent_width=2,
            escape_asterisks=False,
        )

        result = benchmark(convert, html, options)
        assert len(result) > 0

    @pytest.mark.benchmark(group="api_overhead_medium")
    def test_v1_api_medium_doc(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=50)

        result = benchmark(
            convert_to_markdown,
            html,
            heading_style="atx",
            list_indent_width=2,
            strong_em_symbol="*",
            escape_asterisks=False,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="api_overhead_medium")
    def test_v2_api_medium_doc(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=50)
        options = ConversionOptions(
            heading_style="atx",
            list_indent_width=2,
            strong_em_symbol="*",
            escape_asterisks=False,
            code_block_style="backticks",
        )

        result = benchmark(convert, html, options)
        assert len(result) > 0

    @pytest.mark.benchmark(group="api_overhead_large")
    def test_v1_api_large_doc(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=100)

        result = benchmark(
            convert_to_markdown,
            html,
            heading_style="atx",
            list_indent_width=2,
            bullets="*+-",
            strong_em_symbol="*",
            escape_asterisks=False,
            escape_underscores=False,
            escape_misc=False,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="api_overhead_large")
    def test_v2_api_large_doc(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=100)
        options = ConversionOptions(
            heading_style="atx",
            list_indent_width=2,
            bullets="*+-",
            strong_em_symbol="*",
            escape_asterisks=False,
            escape_underscores=False,
            escape_misc=False,
        )

        result = benchmark(convert, html, options)
        assert len(result) > 0


class TestWikipediaAPIComparison:
    @pytest.mark.benchmark(group="wikipedia_api")
    def test_v1_wikipedia_timeline(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("lists_timeline.html")

        result = benchmark(
            convert_to_markdown,
            html,
            heading_style="atx",
            list_indent_width=2,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia_api")
    def test_v2_wikipedia_timeline(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("lists_timeline.html")
        options = ConversionOptions(
            heading_style="atx",
            list_indent_width=2,
        )

        result = benchmark(convert, html, options)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia_api")
    def test_v1_wikipedia_countries(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("tables_countries.html")

        result = benchmark(
            convert_to_markdown,
            html,
            heading_style="atx",
            list_indent_width=2,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia_api")
    def test_v2_wikipedia_countries(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("tables_countries.html")
        options = ConversionOptions(
            heading_style="atx",
            list_indent_width=2,
        )

        result = benchmark(convert, html, options)
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia_api")
    def test_v1_wikipedia_python(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("medium_python.html")

        result = benchmark(
            convert_to_markdown,
            html,
            heading_style="atx",
            list_indent_width=2,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="wikipedia_api")
    def test_v2_wikipedia_python(self, benchmark: BenchmarkFixture) -> None:
        html = _load_wikipedia_doc("medium_python.html")
        options = ConversionOptions(
            heading_style="atx",
            list_indent_width=2,
        )

        result = benchmark(convert, html, options)
        assert len(result) > 0


class TestPreprocessingOverhead:
    @pytest.mark.benchmark(group="preprocessing")
    def test_v1_with_preprocessing(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=50)

        result = benchmark(
            convert_to_markdown,
            html,
            preprocess=True,
            preprocessing_preset="aggressive",
            remove_navigation=True,
            remove_forms=True,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="preprocessing")
    def test_v2_with_preprocessing(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=50)
        preprocessing = PreprocessingOptions(
            enabled=True,
            preset="aggressive",
            remove_navigation=True,
            remove_forms=True,
        )

        result = benchmark(convert, html, preprocessing=preprocessing)
        assert len(result) > 0

    @pytest.mark.benchmark(group="preprocessing")
    def test_v1_no_preprocessing(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=50)

        result = benchmark(
            convert_to_markdown,
            html,
            preprocess=False,
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="preprocessing")
    def test_v2_no_preprocessing(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=50)

        result = benchmark(convert, html)
        assert len(result) > 0


class TestConfigurationComplexity:
    @pytest.mark.benchmark(group="config_complexity")
    def test_v1_minimal_config(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=30)

        result = benchmark(convert_to_markdown, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="config_complexity")
    def test_v2_minimal_config(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=30)

        result = benchmark(convert, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="config_complexity")
    def test_v1_full_config(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=30)

        result = benchmark(
            convert_to_markdown,
            html,
            heading_style="atx",
            list_indent_width=2,
            bullets="*+-",
            strong_em_symbol="*",
            escape_asterisks=True,
            escape_underscores=True,
            escape_misc=True,
            code_language="python",
            autolinks=True,
            default_title=False,
            br_in_tables=False,
            highlight_style="double-equal",
            extract_metadata=True,
            whitespace_mode="normalized",
            newline_style="backslash",
        )
        assert len(result) > 0

    @pytest.mark.benchmark(group="config_complexity")
    def test_v2_full_config(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=30)
        options = ConversionOptions(
            heading_style="atx",
            list_indent_width=2,
            bullets="*+-",
            strong_em_symbol="*",
            escape_asterisks=True,
            escape_underscores=True,
            escape_misc=True,
            code_language="python",
            autolinks=True,
            default_title=False,
            br_in_tables=False,
            highlight_style="double-equal",
            extract_metadata=True,
            whitespace_mode="normalized",
            newline_style="backslash",
        )

        result = benchmark(convert, html, options)
        assert len(result) > 0

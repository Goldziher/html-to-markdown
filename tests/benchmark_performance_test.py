"""Modern pytest-benchmark integration for html-to-markdown performance testing.

This module provides pytest-benchmark integration for continuous performance monitoring
in CI/CD pipelines, complementing the existing comprehensive performance_test.py.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from html_to_markdown import convert_to_markdown, convert_to_markdown_stream

if TYPE_CHECKING:
    from pytest_benchmark.fixture import BenchmarkFixture

try:
    from .performance_test import generate_complex_html
except ImportError:
    from tests.performance_test import generate_complex_html


class TestBenchmarkCore:
    @pytest.mark.benchmark(group="conversion")
    def test_benchmark_small_document(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=5)
        result = benchmark(convert_to_markdown, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="conversion")
    def test_benchmark_medium_document(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=25)
        result = benchmark(convert_to_markdown, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="conversion")
    def test_benchmark_large_document(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=100)
        result = benchmark(convert_to_markdown, html)
        assert len(result) > 0

    @pytest.mark.benchmark(group="streaming")
    def test_benchmark_streaming_small(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=5)

        def stream_convert() -> str:
            return "".join(convert_to_markdown_stream(html, chunk_size=1024))

        result = benchmark(stream_convert)
        assert len(result) > 0

    @pytest.mark.benchmark(group="streaming")
    def test_benchmark_streaming_large(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=100)

        def stream_convert() -> str:
            return "".join(convert_to_markdown_stream(html, chunk_size=4096))

        result = benchmark(stream_convert)
        assert len(result) > 0


class TestBenchmarkFeatures:
    @pytest.mark.benchmark(group="features")
    def test_benchmark_tables(self, benchmark: BenchmarkFixture) -> None:
        html = (
            """
        <html><body>
        """
            + "\n".join(
                [
                    f"""<table>
                <tr><th>Col1</th><th>Col2</th><th>Col3</th><th>Col4</th></tr>
                {"".join(f"<tr><td>Data{i}-{j}</td><td>Value{i}-{j}</td><td>Info{i}-{j}</td><td>Result{i}-{j}</td></tr>" for j in range(10))}
            </table>"""
                    for i in range(20)
                ]
            )
            + """
        </body></html>
        """
        )

        result = benchmark(convert_to_markdown, html)
        assert "| Col1 |" in result

    @pytest.mark.benchmark(group="features")
    def test_benchmark_lists(self, benchmark: BenchmarkFixture) -> None:
        html = (
            "<html><body>"
            + "\n".join(
                [
                    f"""<ul>
                {"".join(f'<li>List item {i}-{j} with <strong>formatting</strong> and <a href="#">links</a></li>' for j in range(50))}
            </ul>"""
                    for i in range(10)
                ]
            )
            + "</body></html>"
        )

        result = benchmark(convert_to_markdown, html)
        assert "* List item" in result

    @pytest.mark.benchmark(group="features")
    def test_benchmark_mixed_formatting(self, benchmark: BenchmarkFixture) -> None:
        html = (
            "<html><body>"
            + "\n".join(
                [
                    f"<p>Paragraph {i} with <strong>bold</strong>, <em>italic</em>, <code>code</code>, "
                    f"<a href='#link{i}'>links</a>, <mark>highlights</mark>, and <del>strikethrough</del>.</p>"
                    for i in range(500)
                ]
            )
            + "</body></html>"
        )

        result = benchmark(convert_to_markdown, html)
        assert "**bold**" in result


class TestBenchmarkConfiguration:
    @pytest.mark.benchmark(group="config")
    def test_benchmark_different_parsers(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=20)

        result = benchmark(convert_to_markdown, html, parser="html.parser")
        assert len(result) > 0

    @pytest.mark.benchmark(group="config")
    def test_benchmark_whitespace_modes(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=20)

        result = benchmark(convert_to_markdown, html, whitespace_mode="normalized")
        assert len(result) > 0

    @pytest.mark.benchmark(group="config")
    def test_benchmark_preprocessing_levels(self, benchmark: BenchmarkFixture) -> None:
        html = generate_complex_html(size_factor=20)

        result = benchmark(convert_to_markdown, html, preprocessing_preset="aggressive")
        assert len(result) > 0


@pytest.mark.benchmark(group="scalability")
@pytest.mark.parametrize("size_factor", [5, 10, 25, 50, 100])
def test_benchmark_scalability(benchmark: BenchmarkFixture, size_factor: int) -> None:
    html = generate_complex_html(size_factor=size_factor)
    result = benchmark(convert_to_markdown, html)
    assert len(result) > 0

    input_size_mb = len(html) / (1024 * 1024)
    benchmark.extra_info["input_size_mb"] = round(input_size_mb, 3)
    benchmark.extra_info["size_factor"] = size_factor


@pytest.mark.benchmark(group="chunk_optimization")
@pytest.mark.parametrize("chunk_size", [256, 512, 1024, 2048, 4096, 8192])
def test_benchmark_chunk_sizes(benchmark: BenchmarkFixture, chunk_size: int) -> None:
    html = generate_complex_html(size_factor=50)

    def stream_with_chunk() -> str:
        return "".join(convert_to_markdown_stream(html, chunk_size=chunk_size))

    result = benchmark(stream_with_chunk)
    assert len(result) > 0

    benchmark.extra_info["chunk_size"] = chunk_size
    benchmark.extra_info["chunks_produced"] = len(list(convert_to_markdown_stream(html, chunk_size=chunk_size)))

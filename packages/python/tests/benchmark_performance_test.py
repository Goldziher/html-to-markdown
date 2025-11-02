from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from html_to_markdown import convert_to_markdown

if TYPE_CHECKING:
    from pytest_benchmark.fixture import BenchmarkFixture

# Suppress deprecation warnings for v1 compatibility benchmarks
pytestmark = pytest.mark.filterwarnings("ignore::DeprecationWarning")

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

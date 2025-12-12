"""Performance tests and benchmarks for HTML to Markdown conversion."""

import gc
import os
import statistics
import time
from collections.abc import Callable, Generator
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Any

import pytest

from html_to_markdown import convert_to_markdown

pytestmark = pytest.mark.filterwarnings("ignore::DeprecationWarning")

try:
    import psutil

    MEMORY_AVAILABLE = True
except ImportError:
    MEMORY_AVAILABLE = False


try:
    import cProfile
    import pstats
    from io import StringIO

    PROFILING_AVAILABLE = True
except ImportError:
    PROFILING_AVAILABLE = False


@dataclass
class PerformanceMetrics:
    name: str
    execution_time: float
    memory_before: float
    memory_after: float
    memory_peak: float
    output_size: int
    chunks_count: int = 1
    throughput_mb_s: float = 0.0

    @property
    def memory_delta(self) -> float:
        return self.memory_after - self.memory_before

    @property
    def memory_efficiency(self) -> float:
        if self.memory_delta <= 0:
            return float("inf")
        return self.output_size / (self.memory_delta * 1024 * 1024)


@contextmanager
def memory_monitor() -> Generator[dict[str, float], None, None]:
    if not MEMORY_AVAILABLE:
        yield {"before": 0.0, "after": 0.0, "peak": 0.0}
        return

    process = psutil.Process(os.getpid())

    gc.collect()

    memory_before = process.memory_info().rss / 1024 / 1024
    peak_memory = memory_before

    def update_peak() -> None:
        nonlocal peak_memory
        current_memory = process.memory_info().rss / 1024 / 1024
        peak_memory = max(peak_memory, current_memory)

    _ = process.memory_info

    metrics = {"before": memory_before, "after": 0.0, "peak": 0.0}

    try:
        yield metrics
    finally:
        update_peak()
        memory_after = process.memory_info().rss / 1024 / 1024

        metrics.update({"after": memory_after, "peak": peak_memory})


def generate_complex_html(size_factor: int = 100) -> str:
    html_parts = [
        "<!DOCTYPE html>",
        "<html>",
        "<head>",
        "  <title>Performance Test Document</title>",
        "  <meta name='description' content='A complex HTML document for performance testing'>",
        "  <meta name='keywords' content='html, markdown, performance, test'>",
        "</head>",
        "<body>",
    ]

    for i in range(size_factor):
        section_html = [
            f"<article id='section-{i}'>",
            f"  <header><h1>Section {i}: Complex Content</h1></header>",
            f"  <p>This is paragraph {i} with <strong>bold</strong>, <em>italic</em>, ",
            "  <code>inline code</code>, and <mark>highlighted</mark> text.</p>",
            "  <section>",
            "    <h2>Subsection with Lists</h2>",
            "    <ul>",
            "      <li>First item with <a href='https://example.com'>external link</a></li>",
            "      <li>Second item with <kbd>Ctrl+C</kbd> keyboard shortcut</li>",
            "      <li>Third item with <time datetime='2023-01-01'>timestamp</time></li>",
            "      <li><input type='checkbox' checked> Completed task</li>",
            "      <li><input type='checkbox'> Pending task</li>",
            "    </ul>",
            "    <ol>",
            "      <li>Numbered item with <abbr title='HyperText Markup Language'>HTML</abbr></li>",
            "      <li>Another item with <sub>subscript</sub> and <sup>superscript</sup></li>",
            "    </ol>",
            "  </section>",
            "  <blockquote cite='https://example.com/quote'>",
            f"    <p>This is a quote in section {i} with <cite>proper citation</cite>.</p>",
            "  </blockquote>",
            "  <figure>",
            "    <table>",
            "      <caption>Data Table for Section " + str(i) + "</caption>",
            "      <thead>",
            "        <tr><th>Column 1</th><th>Column 2</th><th>Column 3</th></tr>",
            "      </thead>",
            "      <tbody>",
            f"        <tr><td>Data {i}-1</td><td>Value {i}-A</td><td><progress value='75' max='100'>75%</progress></td></tr>",
            f"        <tr><td>Data {i}-2</td><td>Value {i}-B</td><td><meter value='0.8' min='0' max='1'>80%</meter></td></tr>",
            "      </tbody>",
            "    </table>",
            "    <figcaption>Performance data visualization</figcaption>",
            "  </figure>",
            "  <details>",
            "    <summary>Collapsible Content</summary>",
            "    <p>This content is initially hidden and contains <del>deleted</del> and <ins>inserted</ins> text.</p>",
            "    <pre><code class='language-python'>",
            "def example_function():",
            "    return 'Hello, World!'",
            "</code></pre>",
            "  </details>",
            "  <aside>",
            "    <p><small>Note: This is supplementary information for section " + str(i) + ".</small></p>",
            "  </aside>",
            "</article>",
        ]
        html_parts.extend(section_html)

    html_parts.extend(["</body>", "</html>"])
    return "\n".join(html_parts)


def benchmark_function(
    func: Callable[..., Any], *args: Any, iterations: int = 5, warmup: int = 2, **kwargs: Any
) -> PerformanceMetrics:
    all_times = []
    memory_deltas = []
    output_size = 0
    chunks_count = 1

    for _ in range(warmup):
        func(*args, **kwargs)

    for _ in range(iterations):
        with memory_monitor() as memory_metrics:
            start_time = time.perf_counter_ns()
            result = func(*args, **kwargs)
            end_time = time.perf_counter_ns()

            if isinstance(result, str):
                output_size = len(result)
                chunks_count = 1
            elif hasattr(result, "__iter__") and not isinstance(result, str):
                chunks = list(result)
                chunks_count = len(chunks)
                output_size = sum(len(chunk) for chunk in chunks)
            else:
                output_size = len(str(result))

        execution_time = (end_time - start_time) / 1_000_000_000
        all_times.append(execution_time)
        memory_deltas.append(memory_metrics["after"] - memory_metrics["before"])

    median_time = statistics.median(all_times)
    median_memory_delta = statistics.median(memory_deltas) if memory_deltas else 0.0

    input_size_mb = len(args[0]) / (1024 * 1024) if args else 0.0
    throughput = input_size_mb / median_time if median_time > 0 else 0.0

    return PerformanceMetrics(
        name=func.__name__,
        execution_time=median_time,
        memory_before=0.0,
        memory_after=median_memory_delta,
        memory_peak=median_memory_delta,
        output_size=output_size,
        chunks_count=chunks_count,
        throughput_mb_s=throughput,
    )


def profile_function(func: Callable[..., Any], *args: Any, **kwargs: Any) -> str:
    if not PROFILING_AVAILABLE:
        return "Profiling not available (cProfile not installed)"

    profiler = cProfile.Profile()
    profiler.enable()

    try:
        _ = func(*args, **kwargs)
    finally:
        profiler.disable()

    stats_stream = StringIO()
    stats = pstats.Stats(profiler, stream=stats_stream)
    stats.sort_stats("cumulative")
    stats.print_stats(10)

    return stats_stream.getvalue()


def test_streaming_performance() -> None:
    html = generate_complex_html(10)
    result = convert_to_markdown(html)
    assert len(result) > 0, "Should produce output"


def run_comprehensive_benchmark() -> None:
    print("🚀 HTML to Markdown Performance Benchmark")  # noqa: T201
    print("=" * 50)  # noqa: T201

    sizes = [10, 50, 100, 200]

    for size in sizes:
        print(f"\n📊 Testing with document size factor: {size}")  # noqa: T201
        html = generate_complex_html(size)
        input_size_mb = len(html) / (1024 * 1024)
        print(f"   Input size: {input_size_mb:.2f} MB")  # noqa: T201

        metrics = benchmark_function(convert_to_markdown, html)
        print(  # noqa: T201
            f"   Processing: {metrics.execution_time:.3f}s, {metrics.throughput_mb_s:.2f} MB/s"
        )


def profile_bottlenecks() -> None:
    if not PROFILING_AVAILABLE:
        print("⚠️  Profiling not available - install cProfile for detailed analysis")  # noqa: T201
        return

    print("\n🔍 Profiling Performance Bottlenecks")  # noqa: T201
    print("=" * 40)  # noqa: T201

    html = generate_complex_html(50)

    print("\n📈 Processing Profile:")  # noqa: T201
    profile_result = profile_function(convert_to_markdown, html)
    print(profile_result)  # noqa: T201


if __name__ == "__main__":
    test_streaming_performance()

    run_comprehensive_benchmark()

    profile_bottlenecks()

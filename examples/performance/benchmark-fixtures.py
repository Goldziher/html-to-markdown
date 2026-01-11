#!/usr/bin/env python3
"""
Benchmark html-to-markdown conversion on different fixture sizes.

Demonstrates performance characteristics across small, medium, and large documents.
Measures latency (ms), throughput (docs/sec), and bandwidth (MB/s).

Usage:
    python benchmark-fixtures.py --size small
    python benchmark-fixtures.py --size medium
    python benchmark-fixtures.py --size large
    python benchmark-fixtures.py --all
"""

from __future__ import annotations

import argparse
import sys

try:
    from html_to_markdown import ConversionOptions, convert_with_handle, create_options_handle
except ImportError:
    sys.exit(1)

from utils import FIXTURES, TimingMeasurement, format_number, print_result_row, print_section_header


def run_benchmark(
    html: str,
    fixture_name: str,
    fixture_size: str,
    iterations: int = 50,
) -> dict[str, float]:
    """
    Run benchmark on a single fixture.

    Args:
        html: HTML content to convert
        fixture_name: Display name of fixture
        fixture_size: Size category (small/medium/large)
        iterations: Number of iterations to run

    Returns:
        Dictionary with benchmark results
    """
    html_bytes = len(html.encode("utf-8"))

    # Warmup run (not counted)
    convert_with_handle(html, None)

    # Measure conversions
    with TimingMeasurement() as timer:
        for _ in range(iterations):
            convert_with_handle(html, None)

    # Calculate metrics
    avg_time_ms = (timer.elapsed / iterations) * 1000
    throughput_docs_sec = iterations / timer.elapsed
    bytes_processed = html_bytes * iterations
    bandwidth_mb_sec = (bytes_processed / (1024 * 1024)) / timer.elapsed

    return {
        "fixture": fixture_name,
        "size_category": fixture_size,
        "html_size_bytes": html_bytes,
        "iterations": iterations,
        "total_time_sec": timer.elapsed,
        "avg_time_ms": avg_time_ms,
        "throughput_docs_sec": throughput_docs_sec,
        "bandwidth_mb_sec": bandwidth_mb_sec,
    }


def run_with_options_benchmark(
    html: str,
    fixture_name: str,
    fixture_size: str,
    iterations: int = 50,
) -> dict[str, float]:
    """
    Benchmark with ConversionOptions (shows overhead of option handling).

    Args:
        html: HTML content to convert
        fixture_name: Display name of fixture
        fixture_size: Size category (small/medium/large)
        iterations: Number of iterations to run

    Returns:
        Dictionary with benchmark results
    """
    html_bytes = len(html.encode("utf-8"))

    # Create options handle (recommended approach for repeated conversions)
    options = ConversionOptions(sanitize=True)
    handle = create_options_handle(options)

    # Warmup run
    convert_with_handle(html, handle)

    # Measure conversions
    with TimingMeasurement() as timer:
        for _ in range(iterations):
            convert_with_handle(html, handle)

    # Calculate metrics
    avg_time_ms = (timer.elapsed / iterations) * 1000
    throughput_docs_sec = iterations / timer.elapsed
    bytes_processed = html_bytes * iterations
    bandwidth_mb_sec = (bytes_processed / (1024 * 1024)) / timer.elapsed

    return {
        "fixture": fixture_name,
        "size_category": fixture_size,
        "scenario": "with_options",
        "html_size_bytes": html_bytes,
        "iterations": iterations,
        "total_time_sec": timer.elapsed,
        "avg_time_ms": avg_time_ms,
        "throughput_docs_sec": throughput_docs_sec,
        "bandwidth_mb_sec": bandwidth_mb_sec,
    }


def print_results(results: list[dict[str, float]]) -> None:
    """Print benchmark results in formatted tables."""
    # Group by scenario
    default_results = [r for r in results if "scenario" not in r]
    options_results = [r for r in results if r.get("scenario") == "with_options"]

    print_section_header("Benchmark Results - Default Conversion")
    if default_results:
        for result in default_results:
            print_result_row("Fixture", result["fixture"])
            print_result_row("Size (bytes)", format_number(result["html_size_bytes"], 0))
            print_result_row("Avg Time (ms)", f"{result['avg_time_ms']:.3f}")
            print_result_row("Throughput (docs/sec)", format_number(result["throughput_docs_sec"], 1))
            print_result_row("Bandwidth (MB/s)", f"{result['bandwidth_mb_sec']:.2f}")
            print()

    if options_results:
        print_section_header("Benchmark Results - With Options")
        for result in options_results:
            print_result_row("Fixture", result["fixture"])
            print_result_row("Size (bytes)", format_number(result["html_size_bytes"], 0))
            print_result_row("Avg Time (ms)", f"{result['avg_time_ms']:.3f}")
            print_result_row("Throughput (docs/sec)", format_number(result["throughput_docs_sec"], 1))
            print_result_row("Bandwidth (MB/s)", f"{result['bandwidth_mb_sec']:.2f}")
            print()

        print_section_header("Options Overhead Comparison")
        for default, options in zip(default_results, options_results, strict=False):
            overhead_pct = ((options["avg_time_ms"] - default["avg_time_ms"]) / default["avg_time_ms"]) * 100
            print_result_row(default["fixture"], f"{overhead_pct:+.1f}%")
            print()

    print_section_header("Summary by Size Category")
    for category in ["small", "medium", "large"]:
        cat_results = [r for r in default_results if r["size_category"] == category]
        if cat_results:
            result = cat_results[0]
            print_result_row(result["fixture"], f"{result['avg_time_ms']:.3f} ms")
            print()


def main() -> None:
    """Main benchmark runner."""
    parser = argparse.ArgumentParser(description="Benchmark html-to-markdown conversion performance")
    parser.add_argument(
        "--size",
        choices=["small", "medium", "large"],
        help="Fixture size to benchmark (default: all)",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Run all benchmarks (default behavior)",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=50,
        help="Number of iterations per fixture (default: 50)",
    )
    parser.add_argument(
        "--with-options",
        action="store_true",
        help="Also benchmark with ConversionOptions",
    )

    args = parser.parse_args()

    # Determine which fixtures to run
    sizes_to_run = [args.size] if args.size else list(FIXTURES.keys())

    # Run benchmarks
    results = []
    for size in sizes_to_run:
        fixture = FIXTURES[size]

        result = run_benchmark(
            fixture["html"],
            fixture["name"],
            size,
            iterations=args.iterations,
        )
        results.append(result)

        # Run with options if requested
        if args.with_options:
            result_opts = run_with_options_benchmark(
                fixture["html"],
                fixture["name"],
                size,
                iterations=args.iterations,
            )
            results.append(result_opts)

    # Print results
    print_results(results)


if __name__ == "__main__":
    main()

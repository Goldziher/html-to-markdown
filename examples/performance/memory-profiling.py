#!/usr/bin/env python3
"""
Memory profiling for html-to-markdown conversion.

Tracks peak memory usage during conversion and compares different scenarios.
Uses tracemalloc for accurate memory measurement.

Usage:
    python memory-profiling.py --size small
    python memory-profiling.py --size medium
    python memory-profiling.py --all
"""

from __future__ import annotations

import argparse
import gc
import sys
import tracemalloc

from typing_extensions import Self

try:
    from html_to_markdown import (
        ConversionOptions,
        MetadataConfig,
        convert,
        convert_with_handle,
        convert_with_metadata,
        create_options_handle,
    )
except ImportError:
    sys.exit(1)


# Test fixtures
FIXTURES = {
    "small": {
        "name": "Small Document (2 KB)",
        "html": """
        <html>
        <head><title>Small Example</title></head>
        <body>
        <h1>Introduction to Performance</h1>
        <p>This is a small example document for baseline benchmarking.</p>
        <p>It contains minimal HTML structure.</p>
        <ul>
            <li>Point 1</li>
            <li>Point 2</li>
            <li>Point 3</li>
        </ul>
        <p>Perfect for testing cold-start overhead and baseline performance.</p>
        </body>
        </html>
        """,
    },
    "medium": {
        "name": "Medium Document (~25 KB)",
        "html": """
        <html>
        <head><title>Blog Post: Web Performance</title></head>
        <body>
        <h1>Web Performance Techniques</h1>
        <p>This document represents a typical blog post with moderate complexity.</p>
        """
        + """
        <h2>Section with repeated content</h2>
        <p>Performance optimization requires a holistic approach covering caching,
        database optimization, frontend delivery, and continuous monitoring.</p>
        <ul>
            <li><strong>Caching</strong> - Store results to avoid recomputation</li>
            <li><strong>Indexing</strong> - Speed up database queries</li>
            <li><strong>Compression</strong> - Reduce file sizes</li>
            <li><strong>CDN</strong> - Distribute content globally</li>
        </ul>
        <table>
            <tr><th>Technique</th><th>Benefit</th></tr>
            <tr><td>Caching</td><td>Faster retrieval</td></tr>
            <tr><td>Indexing</td><td>Faster queries</td></tr>
        </table>
        """
        * 10,  # Repeat to reach ~25 KB
    },
    "large": {
        "name": "Large Document (~150 KB)",
        "html": """
        <html>
        <head><title>Large Wikipedia Article</title></head>
        <body>
        <h1>History of Computing</h1>
        """
        + """
        <h2>Repeated Section with complex content</h2>
        <p>The history of computing spans thousands of years. Early humans used mechanical
        aids for arithmetic, including the abacus. This technology evolved through multiple
        generations, from simple mechanical machines to modern electronic computers.</p>
        <ol>
            <li>Abacus (3000 BCE) - Ancient calculating tool</li>
            <li>Slide rule (1600s) - Mechanical calculation</li>
            <li>Jacquard loom (1804) - Programmable machine</li>
            <li>Babbage's Analytical Engine (1837) - First computer concept</li>
            <li>ENIAC (1946) - Electronic computer</li>
            <li>Transistor (1947) - Replaced vacuum tubes</li>
            <li>Integrated circuits (1958) - Miniaturization</li>
            <li>Microprocessors (1971) - Personal computers</li>
            <li>World Wide Web (1989) - Internet revolution</li>
        </ol>
        <table>
            <tr>
                <th>Era</th>
                <th>Technology</th>
                <th>Size</th>
                <th>Speed</th>
                <th>Cost</th>
            </tr>
            <tr>
                <td>1940s-1950s</td>
                <td>Vacuum tubes</td>
                <td>Room-sized</td>
                <td>kHz</td>
                <td>$100k+</td>
            </tr>
            <tr>
                <td>1960s-1970s</td>
                <td>Transistors</td>
                <td>Refrigerator-sized</td>
                <td>MHz</td>
                <td>$10k+</td>
            </tr>
            <tr>
                <td>1980s-1990s</td>
                <td>ICs</td>
                <td>Desktop</td>
                <td>GHz</td>
                <td>$1k+</td>
            </tr>
            <tr>
                <td>2000s-2010s</td>
                <td>Microprocessors</td>
                <td>Laptop/Mobile</td>
                <td>GHz+</td>
                <td>$100+</td>
            </tr>
            <tr>
                <td>2020s+</td>
                <td>Multi-core/GPU</td>
                <td>Pocket-sized</td>
                <td>GHz+</td>
                <td>$10+</td>
            </tr>
        </table>
        <p>Key milestones in computing history include the invention of the transistor
        at Bell Labs in 1947, which enabled the replacement of power-hungry, heat-generating
        vacuum tubes. This led to smaller, more reliable machines that could operate in
        regular office environments rather than specially cooled machine rooms.</p>
        """
        * 15,  # Repeat to reach ~150 KB
    },
}


class MemoryTracker:
    """Context manager for tracking memory usage during operations."""

    def __init__(self, operation_name: str = "Operation") -> None:
        self.operation_name = operation_name
        self.baseline_memory = 0
        self.peak_memory = 0
        self.allocations = 0
        self.deallocations = 0

    def __enter__(self) -> Self:
        gc.collect()
        tracemalloc.start()
        self.baseline_memory = tracemalloc.get_traced_memory()[0]
        return self

    def __exit__(self, *args: object) -> None:
        _current, peak = tracemalloc.get_traced_memory()
        self.peak_memory = peak
        tracemalloc.stop()

    @property
    def memory_used_kb(self) -> float:
        """Memory used during operation in KB."""
        return self.peak_memory / 1024

    @property
    def memory_used_mb(self) -> float:
        """Memory used during operation in MB."""
        return self.peak_memory / (1024 * 1024)


def measure_conversion(
    html: str,
    fixture_name: str,
    fixture_size: str,
    scenario: str = "default",
) -> dict[str, float | str]:
    """
    Measure memory usage for a single conversion.

    Args:
        html: HTML content to convert
        fixture_name: Display name of fixture
        fixture_size: Size category (small/medium/large)
        scenario: Conversion scenario (default/with_options/with_metadata)

    Returns:
        Dictionary with memory measurement results
    """
    html_bytes = len(html.encode("utf-8"))

    with MemoryTracker(fixture_name) as tracker:
        if scenario == "default":
            convert(html)
        elif scenario == "with_options":
            options = ConversionOptions(sanitize=True)
            handle = create_options_handle(options)
            convert_with_handle(html, handle)
        elif scenario == "with_metadata":
            config = MetadataConfig(
                extract_headers=True,
                extract_links=True,
                extract_images=True,
            )
            convert_with_metadata(html, metadata_config=config)

    return {
        "fixture": fixture_name,
        "size_category": fixture_size,
        "html_size_bytes": html_bytes,
        "scenario": scenario,
        "peak_memory_kb": tracker.memory_used_kb,
        "peak_memory_mb": tracker.memory_used_mb,
        "ratio_html_to_memory": tracker.peak_memory / html_bytes if html_bytes > 0 else 0,
    }


def measure_batch_processing(
    html: str,
    fixture_name: str,
    fixture_size: str,
    batch_size: int = 10,
) -> dict[str, float | str]:
    """
    Measure memory for batch processing scenario.

    Args:
        html: HTML content to convert
        fixture_name: Display name of fixture
        fixture_size: Size category (small/medium/large)
        batch_size: Number of documents to process

    Returns:
        Dictionary with memory measurement results
    """
    html_bytes = len(html.encode("utf-8"))
    total_bytes = html_bytes * batch_size

    with MemoryTracker("Batch processing") as tracker:
        for _ in range(batch_size):
            convert(html)

    return {
        "fixture": fixture_name,
        "size_category": fixture_size,
        "html_size_bytes": html_bytes,
        "batch_size": batch_size,
        "total_html_bytes": total_bytes,
        "scenario": "batch_processing",
        "peak_memory_kb": tracker.memory_used_kb,
        "peak_memory_mb": tracker.memory_used_mb,
        "memory_per_document_kb": tracker.memory_used_kb / batch_size,
        "ratio_total_to_memory": tracker.peak_memory / total_bytes if total_bytes > 0 else 0,
    }


def print_memory_results(results: list[dict[str, float | str]]) -> None:
    """Print memory profiling results in formatted tables."""
    # Group by scenario
    default_results = [r for r in results if r.get("scenario") == "default"]
    options_results = [r for r in results if r.get("scenario") == "with_options"]
    metadata_results = [r for r in results if r.get("scenario") == "with_metadata"]
    batch_results = [r for r in results if r.get("scenario") == "batch_processing"]

    if default_results:
        for result in default_results:
            result["html_size_bytes"]
            result["ratio_html_to_memory"]

    if options_results:
        for result in options_results:
            result["html_size_bytes"]
            result["ratio_html_to_memory"]

    if metadata_results:
        for result in metadata_results:
            result["html_size_bytes"]
            result["ratio_html_to_memory"]

    if batch_results:
        for result in batch_results:
            pass

    for size_cat in ["small", "medium", "large"]:
        default = next(
            (r for r in default_results if r.get("size_category") == size_cat),
            None,
        )
        options = next(
            (r for r in options_results if r.get("size_category") == size_cat),
            None,
        )
        metadata = next(
            (r for r in metadata_results if r.get("size_category") == size_cat),
            None,
        )

        if default:
            f"{default['peak_memory_mb']:.2f} MB"
            f"{options['peak_memory_mb']:.2f} MB" if options else "N/A"
            f"{metadata['peak_memory_mb']:.2f} MB" if metadata else "N/A"


def main() -> None:
    """Main memory profiling runner."""
    parser = argparse.ArgumentParser(description="Profile memory usage during html-to-markdown conversion")
    parser.add_argument(
        "--size",
        choices=["small", "medium", "large"],
        help="Fixture size to profile (default: all)",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Run all profiling scenarios (default behavior)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=10,
        help="Number of documents for batch processing test (default: 10)",
    )

    args = parser.parse_args()

    # Determine which fixtures to run
    sizes_to_run = []
    sizes_to_run = [args.size] if args.size else list(FIXTURES.keys())

    # Run profiling
    results = []

    for size in sizes_to_run:
        fixture = FIXTURES[size]
        html = fixture["html"]

        # Test default conversion
        result = measure_conversion(html, fixture["name"], size, scenario="default")
        results.append(result)

        # Test with options
        result = measure_conversion(html, fixture["name"], size, scenario="with_options")
        results.append(result)

        # Test with metadata
        result = measure_conversion(html, fixture["name"], size, scenario="with_metadata")
        results.append(result)

        # Test batch processing
        result = measure_batch_processing(html, fixture["name"], size, batch_size=args.batch_size)
        results.append(result)

    # Print results
    print_memory_results(results)


if __name__ == "__main__":
    main()

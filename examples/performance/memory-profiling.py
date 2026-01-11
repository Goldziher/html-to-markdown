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
import sys
from typing import Any

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

from utils import FIXTURES, MemoryTracker, print_result_row, print_section_header


def measure_conversion(
    html: str,
    fixture_name: str,
    fixture_size: str,
    scenario: str = "default",
) -> dict[str, Any]:
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
) -> dict[str, Any]:
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


def print_memory_results(results: list[dict[str, Any]]) -> None:
    """Print memory profiling results in formatted tables."""
    # Group by scenario
    default_results = [r for r in results if r.get("scenario") == "default"]
    options_results = [r for r in results if r.get("scenario") == "with_options"]
    metadata_results = [r for r in results if r.get("scenario") == "with_metadata"]
    batch_results = [r for r in results if r.get("scenario") == "batch_processing"]

    print_section_header("Memory Profile - Default Conversion")
    if default_results:
        for result in default_results:
            print_result_row("Fixture", result["fixture"])
            print_result_row("HTML size (bytes)", f"{result['html_size_bytes']:,}")
            print_result_row("Peak memory (MB)", f"{result['peak_memory_mb']:.2f}")
            print_result_row("Ratio (memory/HTML)", f"{result['ratio_html_to_memory']:.2f}x")
            print()

    print_section_header("Memory Profile - With Options")
    if options_results:
        for result in options_results:
            print_result_row("Fixture", result["fixture"])
            print_result_row("HTML size (bytes)", f"{result['html_size_bytes']:,}")
            print_result_row("Peak memory (MB)", f"{result['peak_memory_mb']:.2f}")
            print_result_row("Ratio (memory/HTML)", f"{result['ratio_html_to_memory']:.2f}x")
            print()

    print_section_header("Memory Profile - With Metadata")
    if metadata_results:
        for result in metadata_results:
            print_result_row("Fixture", result["fixture"])
            print_result_row("HTML size (bytes)", f"{result['html_size_bytes']:,}")
            print_result_row("Peak memory (MB)", f"{result['peak_memory_mb']:.2f}")
            print_result_row("Ratio (memory/HTML)", f"{result['ratio_html_to_memory']:.2f}x")
            print()

    print_section_header("Batch Processing Analysis")
    if batch_results:
        for result in batch_results:
            print_result_row("Fixture", result["fixture"])
            print_result_row("Batch size", result["batch_size"])
            print_result_row("Total HTML size (MB)", f"{result['total_html_bytes'] / (1024 * 1024):.2f}")
            print_result_row("Peak memory (MB)", f"{result['peak_memory_mb']:.2f}")
            print_result_row("Memory per document (KB)", f"{result['memory_per_document_kb']:.2f}")
            print()

    print_section_header("Scenario Comparison by Size")
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
            print_result_row(default["fixture"], f"Default: {default['peak_memory_mb']:.2f} MB")
            if options:
                print_result_row("", f"w/ Options: {options['peak_memory_mb']:.2f} MB")
            if metadata:
                print_result_row("", f"w/ Metadata: {metadata['peak_memory_mb']:.2f} MB")
            print()


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

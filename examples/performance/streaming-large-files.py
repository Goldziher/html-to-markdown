#!/usr/bin/env python3
"""
Streaming and chunked processing for large HTML files.

Demonstrates memory-efficient conversion strategies for large documents
that exceed available memory or need to be processed incrementally.

Usage:
    python streaming-large-files.py
    python streaming-large-files.py --size large
"""

from __future__ import annotations

import argparse
import sys
import time

try:
    from html_to_markdown import convert
except ImportError:
    sys.exit(1)


def create_large_html_file(size_mb: int = 5) -> str:
    """
    Create a synthetic large HTML file for testing.

    Args:
        size_mb: Target size in megabytes

    Returns:
        HTML content
    """
    # Create a repeating section that can be scaled
    section_template = """
    <div class="article-section">
        <h2>Section {num}</h2>
        <p>This is a section of the large document. It contains multiple paragraphs
        to reach a reasonable size for streaming and chunking demonstrations.</p>

        <h3>Subsection 1</h3>
        <p>The first subsection discusses various topics related to the main theme.
        Performance optimization is crucial for handling large documents efficiently.</p>

        <h3>Subsection 2</h3>
        <p>The second subsection provides additional details and examples. Streaming
        approaches allow processing documents larger than available RAM.</p>

        <ul>
            <li>Point 1 - Basic consideration</li>
            <li>Point 2 - Advanced technique</li>
            <li>Point 3 - Optimization strategy</li>
            <li>Point 4 - Best practice</li>
        </ul>

        <h3>Subsection 3</h3>
        <p>The final subsection wraps up the discussion with conclusions and
        recommendations for future work.</p>

        <table>
            <tr>
                <th>Column 1</th>
                <th>Column 2</th>
                <th>Column 3</th>
            </tr>
            <tr>
                <td>Data 1-1</td>
                <td>Data 1-2</td>
                <td>Data 1-3</td>
            </tr>
            <tr>
                <td>Data 2-1</td>
                <td>Data 2-2</td>
                <td>Data 2-3</td>
            </tr>
        </table>
    </div>
    """

    # Build HTML header
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Large Document for Streaming Performance Test</title>
    </head>
    <body>
    <h1>Large Document Streaming Test</h1>
    <p>This document is designed to test streaming and chunking approaches
    for handling large HTML files efficiently.</p>
    """

    # Calculate number of sections needed
    section_size = len(section_template.encode("utf-8"))
    target_size = size_mb * 1024 * 1024
    num_sections = max(1, (target_size // section_size) + 1)

    # Add sections
    for i in range(num_sections):
        html += section_template.format(num=i + 1)

    # Close HTML
    html += """
    </body>
    </html>
    """

    return html


def measure_full_conversion(html: str) -> dict[str, float]:
    """
    Measure time and memory for full document conversion.

    Args:
        html: HTML content

    Returns:
        Dictionary with performance metrics
    """
    html_size = len(html.encode("utf-8"))

    start = time.perf_counter()
    markdown = convert(html)
    elapsed = time.perf_counter() - start

    markdown_size = len(markdown.encode("utf-8"))

    return {
        "strategy": "full_document",
        "html_size_mb": html_size / (1024 * 1024),
        "markdown_size_mb": markdown_size / (1024 * 1024),
        "conversion_time_sec": elapsed,
        "throughput_mb_sec": (html_size / (1024 * 1024)) / elapsed,
    }


def split_html_into_chunks(
    html: str,
    chunk_size_kb: int = 50,
) -> list[str]:
    """
    Split HTML into reasonable chunks while preserving structure.

    This is a simple approach that splits by size. A production implementation
    would preserve semantic boundaries (sections, paragraphs, etc.).

    Args:
        html: HTML content
        chunk_size_kb: Target size per chunk in KB

    Returns:
        List of HTML chunks
    """
    chunk_size = chunk_size_kb * 1024
    chunks = []

    # For this demo, we'll extract the body content and split it
    # Production code would handle this more carefully
    body_start = html.find("<body>")
    body_end = html.find("</body>")

    if body_start == -1 or body_end == -1:
        return [html]

    header = html[: body_start + 6]  # Include opening <body> tag
    footer = html[body_end:]
    body_content = html[body_start + 6 : body_end]

    # Split body by major sections
    sections = body_content.split('<div class="article-section">')

    current_chunk = header
    for i, section in enumerate(sections):
        if i > 0:
            section = '<div class="article-section">' + section

        # Check if adding this section would exceed chunk size
        potential_chunk = current_chunk + section
        if len(potential_chunk.encode("utf-8")) > chunk_size and current_chunk != header:
            # Save current chunk and start a new one
            chunks.append(current_chunk + footer)
            current_chunk = header + section
        else:
            current_chunk += section

    # Add final chunk
    if current_chunk != header:
        chunks.append(current_chunk + footer)

    return chunks


def measure_chunked_conversion(
    html: str,
    chunk_size_kb: int = 50,
) -> dict[str, float]:
    """
    Measure chunked conversion approach.

    Args:
        html: HTML content
        chunk_size_kb: Target chunk size in KB

    Returns:
        Dictionary with performance metrics
    """
    html_size = len(html.encode("utf-8"))
    chunks = split_html_into_chunks(html, chunk_size_kb=chunk_size_kb)

    start = time.perf_counter()
    markdowns = [convert(chunk) for chunk in chunks]
    elapsed = time.perf_counter() - start

    combined_markdown = "\n\n".join(markdowns)
    markdown_size = len(combined_markdown.encode("utf-8"))

    return {
        "strategy": f"chunked_{chunk_size_kb}kb",
        "num_chunks": len(chunks),
        "html_size_mb": html_size / (1024 * 1024),
        "markdown_size_mb": markdown_size / (1024 * 1024),
        "conversion_time_sec": elapsed,
        "throughput_mb_sec": (html_size / (1024 * 1024)) / elapsed,
    }


def measure_incremental_conversion(
    html: str,
    chunk_size_kb: int = 50,
) -> dict[str, float]:
    """
    Measure incremental conversion (streaming-like approach).

    Simulates processing chunks one at a time without holding all in memory.

    Args:
        html: HTML content
        chunk_size_kb: Target chunk size in KB

    Returns:
        Dictionary with performance metrics
    """
    html_size = len(html.encode("utf-8"))
    chunks = split_html_into_chunks(html, chunk_size_kb=chunk_size_kb)

    start = time.perf_counter()
    total_markdown_size = 0

    # Process chunks one at a time (simulating streaming)
    for chunk in chunks:
        markdown = convert(chunk)
        total_markdown_size += len(markdown.encode("utf-8"))

    elapsed = time.perf_counter() - start

    return {
        "strategy": f"incremental_{chunk_size_kb}kb",
        "num_chunks": len(chunks),
        "html_size_mb": html_size / (1024 * 1024),
        "markdown_size_mb": total_markdown_size / (1024 * 1024),
        "conversion_time_sec": elapsed,
        "throughput_mb_sec": (html_size / (1024 * 1024)) / elapsed,
    }


def print_streaming_results(results: list[dict[str, float]]) -> None:
    """Print streaming performance results."""
    for result in results:
        strategy = result["strategy"]
        result["html_size_mb"]
        result["conversion_time_sec"]
        result["throughput_mb_sec"]

        # Add strategy-specific notes
        if "chunked" in strategy or "incremental" in strategy:
            result.get("num_chunks", 0)
        else:
            pass

    full_result = next((r for r in results if r["strategy"] == "full_document"), None)
    if full_result:
        baseline_time = full_result["conversion_time_sec"]
        baseline_throughput = full_result["throughput_mb_sec"]

        for result in results:
            if result["strategy"] != "full_document":
                result["conversion_time_sec"] / baseline_time
                result["throughput_mb_sec"] / baseline_throughput


def main() -> None:
    """Main streaming benchmark runner."""
    parser = argparse.ArgumentParser(description="Benchmark streaming approaches for large file conversion")
    parser.add_argument(
        "--size",
        type=int,
        default=5,
        help="Document size in MB to test (default: 5)",
    )
    parser.add_argument(
        "--chunk-size",
        type=int,
        default=50,
        help="Chunk size in KB for chunked approaches (default: 50)",
    )

    args = parser.parse_args()

    # Create large test document
    html = create_large_html_file(size_mb=args.size)

    # Run benchmarks
    results = []

    result = measure_full_conversion(html)
    results.append(result)

    result = measure_chunked_conversion(html, chunk_size_kb=args.chunk_size)
    results.append(result)

    result = measure_incremental_conversion(html, chunk_size_kb=args.chunk_size)
    results.append(result)

    # Print results
    print_streaming_results(results)


if __name__ == "__main__":
    main()

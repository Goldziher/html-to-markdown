#!/usr/bin/env python3
"""Performance test for streaming HTML to Markdown conversion.

This script demonstrates the memory efficiency benefits of streaming processing.
"""

import sys
import time
from typing import List

from html_to_markdown import convert_to_markdown, convert_to_markdown_stream

# Try to import psutil for memory measurement, but make it optional
try:
    import psutil

    MEMORY_AVAILABLE = True
except ImportError:
    MEMORY_AVAILABLE = False


def generate_large_html(size_kb: int = 1000) -> str:
    """Generate a large HTML document for testing."""
    base_content = """
    <div>
        <h1>Section Header</h1>
        <p>This is a paragraph with <strong>bold</strong> and <em>italic</em> text.
        It contains <a href="https://example.com">links</a> and other markup.</p>
        <ul>
            <li>List item 1 with some content</li>
            <li>List item 2 with more content</li>
            <li>List item 3 with even more content</li>
        </ul>
        <table>
            <tr><th>Column 1</th><th>Column 2</th><th>Column 3</th></tr>
            <tr><td>Data 1</td><td>Data 2</td><td>Data 3</td></tr>
        </table>
    </div>
    """

    # Calculate how many repetitions we need to reach the desired size
    content_size = len(base_content.encode("utf-8"))
    repetitions = (size_kb * 1024) // content_size

    return base_content * repetitions


def measure_memory_usage() -> float:
    """Simple memory usage measurement (approximation)."""
    if not MEMORY_AVAILABLE:
        return 0.0
    import os

    memory_bytes = psutil.Process(os.getpid()).memory_info().rss
    return float(memory_bytes / 1024 / 1024)  # MB


def run_performance_test() -> None:
    """Run performance comparison between regular and streaming processing."""
    # Generate test data
    html = generate_large_html(500)  # 500KB document

    # Test regular processing
    if MEMORY_AVAILABLE:
        mem_before = measure_memory_usage()

    start_time = time.time()
    result_regular = convert_to_markdown(html)
    regular_time = time.time() - start_time

    if MEMORY_AVAILABLE:
        mem_after = measure_memory_usage()
        mem_used_regular = mem_after - mem_before
    else:
        mem_used_regular = 0.0

    # Test streaming processing
    if MEMORY_AVAILABLE:
        mem_before = measure_memory_usage()

    start_time = time.time()
    result_streaming = convert_to_markdown(html, stream_processing=True, chunk_size=1024)
    streaming_time = time.time() - start_time

    if MEMORY_AVAILABLE:
        mem_after = measure_memory_usage()
        mem_used_streaming = mem_after - mem_before
    else:
        mem_used_streaming = 0.0

    # Verify results are identical
    results_match = result_regular == result_streaming

    # Test pure streaming API
    start_time = time.time()
    chunks: List[str] = []
    chunk_count = 0

    for chunk in convert_to_markdown_stream(html, chunk_size=1024):
        chunks.append(chunk)
        chunk_count += 1

    result_pure_streaming = "".join(chunks)
    pure_streaming_time = time.time() - start_time

    # Validate all results
    assert results_match, "Regular and streaming results don't match"
    assert result_regular == result_pure_streaming, "Regular and pure streaming results don't match"
    assert chunk_count > 0, "No chunks were generated"
    assert regular_time > 0, "Regular processing took no time"
    assert streaming_time > 0, "Streaming processing took no time"
    assert pure_streaming_time > 0, "Pure streaming took no time"

    # Optional: Write results if run as main script
    if __name__ == "__main__":
        sys.stdout.write(f"Generated HTML document: {len(html.encode('utf-8')) / 1024:.1f} KB\n")
        sys.stdout.write(f"Regular processing time: {regular_time:.3f} seconds\n")
        if MEMORY_AVAILABLE:
            sys.stdout.write(f"Regular memory used: {mem_used_regular:.1f} MB\n")
        sys.stdout.write(f"Streaming processing time: {streaming_time:.3f} seconds\n")
        if MEMORY_AVAILABLE:
            sys.stdout.write(f"Streaming memory used: {mem_used_streaming:.1f} MB\n")
        sys.stdout.write(f"Pure streaming time: {pure_streaming_time:.3f} seconds\n")
        sys.stdout.write(f"Number of chunks: {chunk_count}\n")
        sys.stdout.write(f"Results identical: {results_match}\n")


if __name__ == "__main__":
    run_performance_test()

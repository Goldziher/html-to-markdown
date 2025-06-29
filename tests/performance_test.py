"""Performance tests for streaming functionality."""

import time
import sys
from html_to_markdown import convert_to_markdown, convert_to_markdown_stream

# Try to import psutil for memory measurement, but make it optional
try:
    import psutil
    memory_available = True
except ImportError:
    memory_available = False
    print("psutil not available - memory measurements disabled")


def generate_large_html(size_kb=1000):
    """Create a large HTML document for testing."""
    html_parts = []
    
    # Add various types of content
    for i in range(size_kb):
        html_parts.extend([
            f"<h2>Section {i}</h2>",
            f"<p>This is paragraph {i} with some <strong>bold text</strong> and <em>italic text</em>.</p>",
            "<ul>",
            "  <li>First item with <code>inline code</code></li>",
            "  <li>Second item</li>",
            "  <li>Third item with <a href='http://example.com'>link</a></li>",
            "</ul>",
            "<blockquote>",
            f"  <p>This is a quote in section {i}.</p>",
            "</blockquote>",
            "<table>",
            "  <tr><th>Header 1</th><th>Header 2</th></tr>",
            f"  <tr><td>Data {i}</td><td>More data {i}</td></tr>",
            "</table>",
        ])
    
    return "".join(html_parts)


def measure_memory_usage():
    if not memory_available:
        return 0
    import os
    return psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024  # MB


def test_streaming_performance():
    print("Generating large HTML document...")
    html = generate_large_html(500)  # 500KB document
    print(f"Generated HTML document: {len(html.encode('utf-8')) / 1024:.1f} KB")

    # Test regular processing
    print("\n--- Regular Processing ---")
    if memory_available:
        mem_before = measure_memory_usage()
    start_time = time.time()
    result_regular = convert_to_markdown(html)
    regular_time = time.time() - start_time
    if memory_available:
        mem_after = measure_memory_usage()
        mem_used_regular = mem_after - mem_before
    print(f"Regular processing time: {regular_time:.3f} seconds")
    if memory_available:
        print(f"Memory used: {mem_used_regular:.1f} MB")
    print(f"Output size: {len(result_regular.encode('utf-8')) / 1024:.1f} KB")

    # Test streaming processing
    print("\n--- Streaming Processing ---")
    if memory_available:
        mem_before = measure_memory_usage()
    start_time = time.time()
    result_streaming = convert_to_markdown(html, stream_processing=True, chunk_size=1024)
    streaming_time = time.time() - start_time
    if memory_available:
        mem_after = measure_memory_usage()
        mem_used_streaming = mem_after - mem_before
    print(f"Streaming processing time: {streaming_time:.3f} seconds")
    if memory_available:
        print(f"Memory used: {mem_used_streaming:.1f} MB")
    print(f"Output size: {len(result_streaming.encode('utf-8')) / 1024:.1f} KB")

    # Verify results are identical
    results_match = result_regular == result_streaming
    print(f"\nResults identical: {results_match}")
    if memory_available and mem_used_regular > 0:
        memory_improvement = ((mem_used_regular - mem_used_streaming) / mem_used_regular) * 100
        print(f"Memory improvement: {memory_improvement:.1f}%")

    # Test pure streaming API
    print("\n--- Pure Streaming API ---")
    start_time = time.time()
    chunks = []
    chunk_count = 0
    for chunk in convert_to_markdown_stream(html, chunk_size=1024):
        chunks.append(chunk)
        chunk_count += 1
    result_pure_streaming = ''.join(chunks)
    pure_streaming_time = time.time() - start_time
    print(f"Pure streaming time: {pure_streaming_time:.3f} seconds")
    print(f"Number of chunks: {chunk_count}")
    print(f"Average chunk size: {len(result_pure_streaming) / chunk_count:.1f} characters")
    print(f"Results identical to regular: {result_regular == result_pure_streaming}")


if __name__ == "__main__":
    test_streaming_performance()

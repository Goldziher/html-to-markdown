#!/usr/bin/env python3
"""Performance test for streaming HTML to Markdown conversion.
This script demonstrates the memory efficiency benefits of streaming processing.
"""

import time

from html_to_markdown import convert_to_markdown, convert_to_markdown_stream


def generate_large_html(size_kb=1000):
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


def measure_memory_usage():
    """Simple memory usage measurement (approximation)."""
    import os

    import psutil
    return psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024  # MB


def test_streaming_performance():
    """Test the performance difference between regular and streaming processing."""
    print("Generating large HTML document...")
    html = generate_large_html(500)  # 500KB document
    print(f"Generated HTML document: {len(html.encode('utf-8')) / 1024:.1f} KB")

    try:
        import psutil
        memory_available = True
    except ImportError:
        memory_available = False
        print("psutil not available - memory measurements disabled")

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

    result_pure_streaming = "".join(chunks)
    pure_streaming_time = time.time() - start_time

    print(f"Pure streaming time: {pure_streaming_time:.3f} seconds")
    print(f"Number of chunks: {chunk_count}")
    print(f"Average chunk size: {len(result_pure_streaming) / chunk_count:.1f} characters")
    print(f"Results identical to regular: {result_regular == result_pure_streaming}")


if __name__ == "__main__":
    test_streaming_performance()

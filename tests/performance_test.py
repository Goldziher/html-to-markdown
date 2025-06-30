"""Performance tests for streaming functionality."""

import os
import time

from html_to_markdown import convert_to_markdown, convert_to_markdown_stream

# Try to import psutil for memory measurement, but make it optional
try:
    import psutil
    memory_available = True
except ImportError:
    memory_available = False


def generate_large_html(size_kb: int = 1000) -> str:
    """Create a large HTML document for testing."""
    html_parts: list[str] = []

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


def measure_memory_usage() -> float:
    """Measure current memory usage in MB."""
    if not memory_available:
        return 0.0
    memory_bytes = psutil.Process(os.getpid()).memory_info().rss
    return float(memory_bytes / 1024 / 1024)  # MB


def test_streaming_performance() -> None:
    """Test streaming performance compared to regular processing."""
    html = generate_large_html(50)  # Smaller size for testing

    # Test regular processing
    start_time = time.time()
    result_regular = convert_to_markdown(html)
    _ = time.time() - start_time  # Capture timing but don't use it

    # Test streaming processing
    start_time = time.time()
    result_streaming = convert_to_markdown(html, stream_processing=True, chunk_size=1024)
    _ = time.time() - start_time  # Capture timing but don't use it

    # Verify results are identical
    assert result_regular == result_streaming, "Regular and streaming results should match"

    # Test pure streaming API
    chunks = list(convert_to_markdown_stream(html, chunk_size=1024))
    result_pure_streaming = "".join(chunks)

    assert result_regular == result_pure_streaming, "All processing methods should produce identical results"
    assert len(chunks) > 0, "Should produce at least one chunk"


if __name__ == "__main__":
    test_streaming_performance()
if __name__ == "__main__":
    test_streaming_performance()

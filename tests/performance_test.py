"""Performance tests for streaming functionality."""

import time
import psutil
import os
from html_to_markdown import convert_to_markdown, convert_to_markdown_stream


def get_memory_usage() -> float:
    """Get current memory usage in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024


def create_large_html(size_factor: int = 1000) -> str:
    """Create a large HTML document for testing."""
    html_parts = []
    
    # Add various types of content
    for i in range(size_factor):
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


def test_memory_usage_comparison():
    """Compare memory usage between regular and streaming processing."""
    print("\\n=== Memory Usage Comparison ===")
    
    # Create test document
    print("Creating large HTML document...")
    html = create_large_html(500)  # Moderate size for testing
    print(f"HTML document size: {len(html):,} characters")
    
    # Test regular processing
    print("\\nTesting regular processing...")
    memory_before_regular = get_memory_usage()
    start_time = time.time()
    
    result_regular = convert_to_markdown(html)
    
    regular_time = time.time() - start_time
    memory_after_regular = get_memory_usage()
    regular_memory_usage = memory_after_regular - memory_before_regular
    
    print(f"Regular processing time: {regular_time:.2f}s")
    print(f"Regular memory usage: {regular_memory_usage:.2f}MB")
    print(f"Output size: {len(result_regular):,} characters")
    
    # Test streaming processing
    print("\\nTesting streaming processing...")
    memory_before_streaming = get_memory_usage()
    start_time = time.time()
    
    chunks = []
    for chunk in convert_to_markdown_stream(html, chunk_size=1024):
        chunks.append(chunk)
    result_streaming = "".join(chunks)
    
    streaming_time = time.time() - start_time
    memory_after_streaming = get_memory_usage()
    streaming_memory_usage = memory_after_streaming - memory_before_streaming
    
    print(f"Streaming processing time: {streaming_time:.2f}s")
    print(f"Streaming memory usage: {streaming_memory_usage:.2f}MB")
    print(f"Number of chunks: {len(chunks)}")
    print(f"Output size: {len(result_streaming):,} characters")
    
    # Verify results are identical
    assert result_regular == result_streaming, "Results should be identical"
    print("\\n✓ Results are identical")
    
    # Show comparison
    if streaming_memory_usage < regular_memory_usage:
        memory_savings = ((regular_memory_usage - streaming_memory_usage) / regular_memory_usage) * 100
        print(f"\\n🎉 Streaming saved {memory_savings:.1f}% memory!")
    
    if streaming_time < regular_time:
        time_savings = ((regular_time - streaming_time) / regular_time) * 100
        print(f"🚀 Streaming was {time_savings:.1f}% faster!")


def test_progress_reporting():
    """Test progress reporting functionality."""
    print("\\n=== Progress Reporting Test ===")
    
    html = create_large_html(200)
    progress_updates = []
    
    def progress_callback(processed: int, total: int) -> None:
        progress_updates.append((processed, total))
        if len(progress_updates) % 5 == 0:  # Print every 5th update
            percent = (processed / total) * 100 if total > 0 else 0
            print(f"Progress: {percent:.1f}% ({processed:,}/{total:,} bytes)")
    
    print("Processing with progress reporting...")
    start_time = time.time()
    
    chunks = list(convert_to_markdown_stream(
        html, 
        chunk_size=512,
        progress_callback=progress_callback
    ))
    
    processing_time = time.time() - start_time
    
    print(f"\\nCompleted in {processing_time:.2f}s")
    print(f"Total progress updates: {len(progress_updates)}")
    print(f"Final progress: {progress_updates[-1] if progress_updates else 'None'}")
    print(f"Number of chunks: {len(chunks)}")


def test_large_document_processing():
    """Test processing of very large documents."""
    print("\\n=== Large Document Processing Test ===")
    
    # Create a very large document
    print("Creating very large HTML document...")
    html = create_large_html(2000)  # Very large document
    print(f"HTML document size: {len(html):,} characters (~{len(html)/1024/1024:.1f}MB)")
    
    # Test streaming processing
    print("\\nProcessing with streaming...")
    start_time = time.time()
    memory_before = get_memory_usage()
    
    chunk_count = 0
    total_output_size = 0
    
    for chunk in convert_to_markdown_stream(html, chunk_size=2048):
        chunk_count += 1
        total_output_size += len(chunk)
        
        # Print progress every 100 chunks
        if chunk_count % 100 == 0:
            current_memory = get_memory_usage()
            memory_used = current_memory - memory_before
            print(f"Processed {chunk_count} chunks, memory usage: {memory_used:.2f}MB")
    
    processing_time = time.time() - start_time
    final_memory = get_memory_usage()
    total_memory_used = final_memory - memory_before
    
    print(f"\\nProcessing completed!")
    print(f"Total time: {processing_time:.2f}s")
    print(f"Total chunks: {chunk_count:,}")
    print(f"Output size: {total_output_size:,} characters")
    print(f"Peak memory usage: {total_memory_used:.2f}MB")
    print(f"Processing rate: {len(html)/processing_time/1024/1024:.2f} MB/s")


def test_chunk_callback_performance():
    """Test performance with chunk callbacks."""
    print("\\n=== Chunk Callback Performance Test ===")
    
    html = create_large_html(300)
    chunks_received = []
    
    def chunk_callback(chunk: str) -> None:
        chunks_received.append(len(chunk))  # Just store the length
    
    print("Processing with chunk callback...")
    start_time = time.time()
    
    result = convert_to_markdown(
        html,
        stream_processing=True,
        chunk_size=1024,
        chunk_callback=chunk_callback
    )
    
    processing_time = time.time() - start_time
    
    print(f"Processing time: {processing_time:.2f}s")
    print(f"Chunks received: {len(chunks_received)}")
    print(f"Average chunk size: {sum(chunks_received)/len(chunks_received):.1f} chars")
    print(f"Total output size: {len(result):,} characters")


if __name__ == "__main__":
    try:
        test_memory_usage_comparison()
        test_progress_reporting()
        test_large_document_processing() 
        test_chunk_callback_performance()
        print("\\n🎉 All performance tests completed successfully!")
    except ImportError as e:
        if "psutil" in str(e):
            print("⚠️  psutil not available, skipping memory usage tests")
            print("Install psutil with: pip install psutil")
        else:
            raise
    except Exception as e:
        print(f"❌ Test failed: {e}")
        raise

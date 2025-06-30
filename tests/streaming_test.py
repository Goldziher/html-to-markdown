"""Tests for streaming functionality."""

import pytest

from html_to_markdown import convert_to_markdown, convert_to_markdown_stream


class TestStreamingProcessing:
    """Test streaming functionality."""

    def test_streaming_basic(self) -> None:
        """Test basic streaming functionality."""
        html = "<p>Hello <strong>world</strong>!</p>"

        # Collect chunks from streaming
        chunks = list(convert_to_markdown_stream(html, chunk_size=10))
        result_streaming = "".join(chunks)

        # Compare with regular processing
        result_regular = convert_to_markdown(html)

        assert result_streaming == result_regular

    def test_streaming_large_document(self) -> None:
        """Test streaming with a larger document."""
        # Create a large HTML document using list comprehension
        html_parts = [
            f"<p>This is paragraph {i} with some <strong>bold text</strong> and <em>italic text</em>.</p>"
            for i in range(100)
        ]

        html = "".join(html_parts)

        # Test streaming
        chunks = list(convert_to_markdown_stream(html, chunk_size=500))
        result_streaming = "".join(chunks)

        # Compare with regular processing
        result_regular = convert_to_markdown(html)

        assert result_streaming == result_regular
        assert len(chunks) > 1  # Should be multiple chunks

    def test_streaming_with_nested_tags(self) -> None:
        """Test streaming with deeply nested tags."""
        # Create nested structure
        html = "<div>" * 50 + "Deeply nested content" + "</div>" * 50

        chunks = list(convert_to_markdown_stream(html, chunk_size=20))
        result_streaming = "".join(chunks)

        result_regular = convert_to_markdown(html)

        assert result_streaming == result_regular

    def test_progress_callback(self) -> None:
        """Test progress callback functionality."""
        html = "<p>Test</p>" * 50
        progress_calls = []

        def progress_callback(processed: int, total: int) -> None:
            progress_calls.append((processed, total))

        list(convert_to_markdown_stream(html, progress_callback=progress_callback))

        # Should have received progress updates
        assert len(progress_calls) > 0

        # Check that progress is non-decreasing
        for i in range(1, len(progress_calls)):
            assert progress_calls[i][0] >= progress_calls[i - 1][0]

    def test_chunk_callback_in_convert_to_markdown(self) -> None:
        """Test chunk callback in main convert_to_markdown function."""
        html = "<p>Test</p>" * 20
        chunks_received = []

        def chunk_callback(chunk: str) -> None:
            chunks_received.append(chunk)

        result = convert_to_markdown(html, stream_processing=True, chunk_size=50, chunk_callback=chunk_callback)

        # Should have received chunks
        assert len(chunks_received) > 0
        assert "".join(chunks_received) == result

    def test_streaming_preserves_formatting(self) -> None:
        """Test that streaming preserves complex formatting."""
        html = """
        <div>
            <h1>Main Title</h1>
            <p>This is a paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
            <ul>
                <li>First item</li>
                <li>Second item with <code>code</code></li>
                <li>Third item</li>
            </ul>
            <blockquote>
                <p>This is a quote with <a href="http://example.com">a link</a>.</p>
            </blockquote>
            <table>
                <tr>
                    <th>Header 1</th>
                    <th>Header 2</th>
                </tr>
                <tr>
                    <td>Cell 1</td>
                    <td>Cell 2</td>
                </tr>
            </table>
        </div>
        """

        chunks = list(convert_to_markdown_stream(html, chunk_size=100))
        result_streaming = "".join(chunks)

        result_regular = convert_to_markdown(html)

        assert result_streaming == result_regular

    def test_empty_html_streaming(self) -> None:
        """Test streaming with empty HTML."""
        with pytest.raises(ValueError, match="The input HTML is empty"):
            list(convert_to_markdown_stream(""))

    def test_streaming_with_all_options(self) -> None:
        """Test streaming with various options."""
        html = "<p>Test with <strong>formatting</strong> and *asterisks*</p>"

        chunks = list(
            convert_to_markdown_stream(
                html, chunk_size=20, escape_asterisks=False, strong_em_symbol="_", heading_style="atx"
            )
        )
        result_streaming = "".join(chunks)

        result_regular = convert_to_markdown(html, escape_asterisks=False, strong_em_symbol="_", heading_style="atx")

        assert result_streaming == result_regular

    def test_memory_efficiency(self) -> None:
        """Test that streaming is more memory efficient (conceptual test)."""
        # Create a very large document
        html = "<p>Large content</p>" * 1000

        # This should work without issues using streaming
        chunks = list(convert_to_markdown_stream(html, chunk_size=1000))
        assert len(chunks) > 1

        # Result should be correct
        result_streaming = "".join(chunks)
        result_regular = convert_to_markdown(html)
        assert result_streaming == result_regular


class TestBackwardCompatibility:
    """Test that existing API remains unchanged."""

    def test_original_api_unchanged(self) -> None:
        """Test that the original API works as before."""
        html = "<p>Hello <strong>world</strong>!</p>"

        # Original call should work
        result = convert_to_markdown(html)
        assert "Hello **world**!" in result

    def test_new_parameters_default_to_false(self) -> None:
        """Test that new streaming parameters default to non-streaming behavior."""
        html = "<p>Test</p>"

        # These should be equivalent
        result1 = convert_to_markdown(html)
        result2 = convert_to_markdown(html, stream_processing=False)

        assert result1 == result2


class TestEdgeCases:
    """Test edge cases and error conditions."""

    def test_very_small_chunk_size(self) -> None:
        """Test with very small chunk size."""
        html = "<p>Short</p>"
        chunks = list(convert_to_markdown_stream(html, chunk_size=1))
        result = "".join(chunks)

        expected = convert_to_markdown(html)
        assert result == expected

    def test_chunk_size_larger_than_content(self) -> None:
        """Test with chunk size larger than content."""
        html = "<p>Short</p>"
        chunks = list(convert_to_markdown_stream(html, chunk_size=10000))

        # Should still work, just one chunk
        assert len(chunks) == 1
        result = "".join(chunks)
        expected = convert_to_markdown(html)
        assert result == expected

    def test_complex_nesting_streaming(self) -> None:
        """Test complex nesting doesn't break streaming."""
        html = """
        <div>
            <table>
                <tr>
                    <td>
                        <ul>
                            <li>
                                <p>Nested <strong>content</strong></p>
                                <blockquote>
                                    <p>Quote in list in table</p>
                                </blockquote>
                            </li>
                        </ul>
                    </td>
                </tr>
            </table>
        </div>
        """

        chunks = list(convert_to_markdown_stream(html, chunk_size=50))
        result_streaming = "".join(chunks)

        result_regular = convert_to_markdown(html)
        assert result_streaming == result_regular

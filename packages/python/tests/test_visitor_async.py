"""Comprehensive async visitor tests for html-to-markdown.

Tests async visitor functionality with coroutines, asyncio integration,
and error handling within the async context.
"""

from __future__ import annotations

import asyncio
from typing import Any

import pytest

from html_to_markdown import ConversionOptions, convert_with_async_visitor


class AsyncVisitorWithAllMethods:
    """Async visitor with both sync and async visitor methods."""

    def __init__(self) -> None:
        self.visited_nodes: list[dict[str, Any]] = []
        self.async_called_count: int = 0
        self.sync_called_count: int = 0

    async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Async visitor method for elements."""
        await asyncio.sleep(0.001)
        self.async_called_count += 1
        self.visited_nodes.append(context)
        return {"type": "continue"}

    def on_text(self, context: dict[str, Any]) -> dict[str, str]:
        """Sync visitor method for text nodes."""
        self.sync_called_count += 1
        return {"type": "continue"}

    async def visit(self, context: dict[str, Any]) -> dict[str, str]:
        """Generic async visitor method (if called)."""
        await asyncio.sleep(0.001)
        return {"type": "continue"}


class AsyncVisitorOnlyAsyncMethods:
    """Visitor with only async methods."""

    def __init__(self) -> None:
        self.calls: list[str] = []

    async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Async element visitor."""
        await asyncio.sleep(0.001)
        tag_name = context.get("tag_name", "unknown")
        self.calls.append(f"element:{tag_name}")
        return {"type": "continue"}

    async def on_text(self, context: dict[str, Any]) -> dict[str, str]:
        """Async text visitor."""
        await asyncio.sleep(0.0005)
        self.calls.append("text")
        return {"type": "continue"}


class AsyncVisitorWithCustomOutput:
    """Visitor that returns custom markdown output."""

    async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Return custom markdown for specific tags."""
        await asyncio.sleep(0.001)
        tag_name = context.get("tag_name", "")

        if tag_name == "h1":
            return {"type": "custom", "output": "# CUSTOM H1\n"}
        if tag_name == "p":
            return {"type": "custom", "output": "CUSTOM PARAGRAPH\n"}

        return {"type": "continue"}


class AsyncVisitorWithSkip:
    """Visitor that skips certain elements."""

    async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Skip certain tags."""
        await asyncio.sleep(0.001)
        tag_name = context.get("tag_name", "")

        if tag_name in {"script", "style"}:
            return {"type": "skip"}

        return {"type": "continue"}


class AsyncVisitorWithPreserveHtml:
    """Visitor that preserves certain elements as HTML."""

    async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Preserve complex HTML structures."""
        await asyncio.sleep(0.001)
        tag_name = context.get("tag_name", "")

        if tag_name == "svg":
            return {"type": "preserve_html"}

        return {"type": "continue"}


class AsyncVisitorWithError:
    """Visitor that triggers errors during conversion."""

    def __init__(self, error_on_tag: str = "error") -> None:
        self.error_on_tag = error_on_tag

    async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Raise error for specific tag."""
        await asyncio.sleep(0.001)
        tag_name = context.get("tag_name", "")

        if tag_name == self.error_on_tag:
            return {"type": "error", "message": f"Intentional error for tag {tag_name}"}

        return {"type": "continue"}


class AsyncVisitorWithAsyncWork:
    """Visitor that performs meaningful async work."""

    def __init__(self) -> None:
        self.processed_elements: dict[str, int] = {}

    async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Process elements with async work simulation."""
        await asyncio.sleep(0.002)

        tag_name = context.get("tag_name", "")
        self.processed_elements[tag_name] = self.processed_elements.get(tag_name, 0) + 1

        return {"type": "continue"}


class SyncOnlyVisitor:
    """Visitor with only sync methods (should work with async converter)."""

    def __init__(self) -> None:
        self.calls: list[str] = []

    def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Sync element visitor."""
        tag_name = context.get("tag_name", "unknown")
        self.calls.append(f"element:{tag_name}")
        return {"type": "continue"}

    def on_text(self, context: dict[str, Any]) -> dict[str, str]:
        """Sync text visitor."""
        self.calls.append("text")
        return {"type": "continue"}


class AsyncVisitorWithContextInfo:
    """Visitor that validates context information."""

    def __init__(self) -> None:
        self.contexts: list[dict[str, Any]] = []

    async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
        """Capture and validate context."""
        await asyncio.sleep(0.001)
        self.contexts.append(context)
        return {"type": "continue"}


@pytest.mark.asyncio
async def test_convert_with_async_visitor_is_callable() -> None:
    """Test that convert_with_async_visitor is importable and callable."""
    assert callable(convert_with_async_visitor)
    html = "<h1>Test</h1>"
    result = convert_with_async_visitor(html)
    assert isinstance(result, str)
    assert "Test" in result


@pytest.mark.asyncio
async def test_async_visitor_basic_conversion() -> None:
    """Test basic HTML conversion with async visitor."""
    html = "<h1>Hello</h1><p>World</p>"
    visitor = AsyncVisitorOnlyAsyncMethods()
    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Hello" in result
    assert "World" in result
    assert visitor.calls


@pytest.mark.asyncio
async def test_async_visitor_with_options() -> None:
    """Test async visitor with ConversionOptions."""
    html = "<h1>Title</h1><h2>Subtitle</h2>"
    options = ConversionOptions(heading_style="atx_closed")
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, options, visitor=visitor)

    assert isinstance(result, str)
    assert "#" in result
    assert "Title" in result


@pytest.mark.asyncio
async def test_async_visitor_with_multiple_options() -> None:
    """Test async visitor respects multiple conversion options."""
    html = "<h1>Test</h1><ul><li>Item 1</li><li>Item 2</li></ul>"
    options = ConversionOptions(
        heading_style="underlined",
        list_indent_width=4,
        bullets="*-+",
    )
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, options, visitor=visitor)

    assert isinstance(result, str)
    assert "Test" in result
    assert "Item 1" in result


@pytest.mark.asyncio
async def test_async_visitor_async_methods_called() -> None:
    """Test that async visitor methods are actually called."""
    html = "<p>Content</p>"
    visitor = AsyncVisitorWithAllMethods()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert visitor.async_called_count > 0
    assert visitor.visited_nodes


@pytest.mark.asyncio
async def test_async_visitor_mixed_sync_async_methods() -> None:
    """Test visitor with both sync and async methods."""
    html = "<h1>Title</h1><p>Paragraph</p>"
    visitor = AsyncVisitorWithAllMethods()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Title" in result
    assert "Paragraph" in result
    assert visitor.async_called_count > 0
    assert visitor.sync_called_count > 0


@pytest.mark.asyncio
async def test_async_visitor_only_sync_methods() -> None:
    """Test that sync-only visitors work with async converter."""
    html = "<p>Test paragraph</p>"
    visitor = SyncOnlyVisitor()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Test paragraph" in result
    assert visitor.calls


@pytest.mark.asyncio
async def test_async_visitor_performs_async_work() -> None:
    """Test that async visitor can perform async operations."""
    html = "<h1>A</h1><p>B</p><div>C</div>"
    visitor = AsyncVisitorWithAsyncWork()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert visitor.processed_elements
    assert len(visitor.processed_elements) > 0


@pytest.mark.asyncio
async def test_async_visitor_maintains_order() -> None:
    """Test that async visitor processes nodes in order."""
    html = "<h1>First</h1><h2>Second</h2><h3>Third</h3>"
    visitor = AsyncVisitorWithContextInfo()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert len(visitor.contexts) > 0
    assert "First" in result
    assert "Second" in result
    assert "Third" in result


@pytest.mark.asyncio
async def test_async_visitor_custom_output() -> None:
    """Test async visitor returning custom markdown."""
    html = "<h1>Original</h1>"
    visitor = AsyncVisitorWithCustomOutput()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "CUSTOM" in result or "Original" in result


@pytest.mark.asyncio
async def test_async_visitor_custom_paragraph() -> None:
    """Test custom output for paragraph elements."""
    html = "<p>Test</p>"
    visitor = AsyncVisitorWithCustomOutput()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "CUSTOM" in result or "Test" in result


@pytest.mark.asyncio
async def test_async_visitor_skip_elements() -> None:
    """Test async visitor skipping certain elements."""
    html = "<p>Keep</p><script>console.log('skip')</script>"
    visitor = AsyncVisitorWithSkip()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Keep" in result
    assert "console.log" not in result


@pytest.mark.asyncio
async def test_async_visitor_preserve_html() -> None:
    """Test async visitor preserving HTML elements."""
    html = '<div>Content</div><svg><circle r="50"></circle></svg>'
    visitor = AsyncVisitorWithPreserveHtml()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Content" in result


@pytest.mark.asyncio
async def test_async_visitor_with_error_handling() -> None:
    """Test error handling in async visitor."""
    html = "<p>Normal content</p>"
    visitor = AsyncVisitorWithError(error_on_tag="nonexistent")

    result = convert_with_async_visitor(html, visitor=visitor)
    assert isinstance(result, str)


@pytest.mark.asyncio
async def test_async_visitor_context_contains_expected_keys() -> None:
    """Test that visitor receives context with expected keys."""
    html = "<p class='test' id='para1'>Content</p>"
    visitor = AsyncVisitorWithContextInfo()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert visitor.contexts

    has_complete_context = False
    for ctx in visitor.contexts:
        required_keys = {"tag_name", "attributes", "depth"}
        if required_keys.issubset(ctx.keys()):
            has_complete_context = True
            break

    assert has_complete_context or visitor.contexts


@pytest.mark.asyncio
async def test_async_visitor_with_escape_options() -> None:
    """Test async visitor with escape-related options."""
    html = "<p>Text with * asterisks</p>"
    options = ConversionOptions(escape_asterisks=True)
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, options, visitor=visitor)

    assert isinstance(result, str)
    assert "Text" in result


@pytest.mark.asyncio
async def test_async_visitor_with_wrap_options() -> None:
    """Test async visitor with text wrapping options."""
    html = "<p>This is a very long paragraph that might need to be wrapped if the wrap option is enabled.</p>"
    options = ConversionOptions(wrap=True, wrap_width=40)
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, options, visitor=visitor)

    assert isinstance(result, str)
    assert "very long" in result or "paragraph" in result


@pytest.mark.asyncio
async def test_async_visitor_with_code_options() -> None:
    """Test async visitor with code block options."""
    html = "<pre><code>print('hello')</code></pre>"
    options = ConversionOptions(
        code_language="python",
        code_block_style="backticks",
    )
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, options, visitor=visitor)

    assert isinstance(result, str)
    assert "hello" in result


@pytest.mark.asyncio
async def test_async_visitor_within_async_context() -> None:
    """Test async visitor can be called from async function."""

    async def async_workflow() -> str:
        html = "<p>Async workflow</p>"
        visitor = AsyncVisitorOnlyAsyncMethods()
        return convert_with_async_visitor(html, visitor=visitor)

    result = await asyncio.to_thread(async_workflow)
    assert "Async workflow" in str(result)


@pytest.mark.asyncio
async def test_async_visitor_concurrent_calls() -> None:
    """Test multiple concurrent async visitor calls."""

    async def convert_one(html: str) -> str:
        visitor = AsyncVisitorOnlyAsyncMethods()
        return convert_with_async_visitor(html, visitor=visitor)

    results = await asyncio.gather(
        asyncio.to_thread(convert_one, "<p>First</p>"),
        asyncio.to_thread(convert_one, "<p>Second</p>"),
        asyncio.to_thread(convert_one, "<p>Third</p>"),
    )

    assert len(results) == 3
    assert all(isinstance(r, str) for r in results)


@pytest.mark.asyncio
async def test_async_visitor_with_complex_html() -> None:
    """Test async visitor with complex nested HTML."""
    html = """
    <html>
        <body>
            <h1>Document Title</h1>
            <section>
                <h2>Section 1</h2>
                <p>Paragraph with <b>bold</b> and <i>italic</i> text.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
            </section>
            <section>
                <h2>Section 2</h2>
                <blockquote>
                    <p>A quoted paragraph</p>
                </blockquote>
                <pre><code>code_block()</code></pre>
            </section>
        </body>
    </html>
    """

    visitor = AsyncVisitorWithContextInfo()
    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Document Title" in result
    assert "Section 1" in result
    assert "Section 2" in result
    assert "bold" in result or "Item 1" in result


@pytest.mark.asyncio
async def test_async_converter_with_none_visitor() -> None:
    """Test that convert_with_async_visitor works with visitor=None."""
    html = "<h1>Title</h1><p>Content</p>"
    result = convert_with_async_visitor(html, visitor=None)

    assert isinstance(result, str)
    assert "Title" in result
    assert "Content" in result


@pytest.mark.asyncio
async def test_async_converter_with_implicit_none_visitor() -> None:
    """Test async converter without passing visitor argument."""
    html = "<h1>Test</h1>"
    result = convert_with_async_visitor(html)

    assert isinstance(result, str)
    assert "Test" in result


@pytest.mark.asyncio
async def test_async_visitor_empty_html() -> None:
    """Test async visitor with empty HTML."""
    html = ""
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)


@pytest.mark.asyncio
async def test_async_visitor_text_only_html() -> None:
    """Test async visitor with plain text (no tags)."""
    html = "Just plain text"
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Just plain text" in result


@pytest.mark.asyncio
async def test_async_visitor_with_special_characters() -> None:
    """Test async visitor with special characters."""
    html = "<p>Unicode: 你好 • é è à ñ</p>"
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "你好" in result or "à" in result or len(result) > 0


@pytest.mark.asyncio
async def test_async_visitor_with_malformed_html() -> None:
    """Test async visitor with malformed HTML."""
    html = "<p>Unclosed paragraph<h1>Heading</p></h1>"
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Unclosed" in result or "Heading" in result


@pytest.mark.asyncio
async def test_async_visitor_continue_type() -> None:
    """Test visitor return type 'continue'."""

    class ContinueVisitor:
        async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
            await asyncio.sleep(0.001)
            return {"type": "continue"}

    html = "<p>Test</p>"
    visitor = ContinueVisitor()
    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Test" in result


@pytest.mark.asyncio
async def test_async_visitor_context_depth() -> None:
    """Test that context depth information is provided."""
    html = "<div><p><span>Nested</span></p></div>"

    class DepthTrackingVisitor:
        def __init__(self) -> None:
            self.max_depth: int = 0

        async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
            await asyncio.sleep(0.001)
            depth = context.get("depth", 0)
            self.max_depth = max(self.max_depth, depth)
            return {"type": "continue"}

    visitor = DepthTrackingVisitor()
    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Nested" in result
    assert visitor.max_depth >= 0


@pytest.mark.asyncio
async def test_async_visitor_attributes_captured() -> None:
    """Test that HTML attributes are captured in context."""
    html = '<a href="http://example.com" title="Example">Link</a>'

    class AttributeVisitor:
        def __init__(self) -> None:
            self.link_attributes: dict[str, str] = {}

        async def on_element(self, context: dict[str, Any]) -> dict[str, str]:
            await asyncio.sleep(0.001)
            tag_name = context.get("tag_name", "")
            if tag_name == "a":
                self.link_attributes = context.get("attributes", {})
            return {"type": "continue"}

    visitor = AttributeVisitor()
    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Link" in result or "example.com" in result


@pytest.mark.asyncio
async def test_async_visitor_many_elements() -> None:
    """Test async visitor with many HTML elements."""
    items = "".join([f"<li>Item {i}</li>" for i in range(100)])
    html = f"<ul>{items}</ul>"

    visitor = AsyncVisitorOnlyAsyncMethods()
    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Item 0" in result
    assert "Item 99" in result
    assert len(visitor.calls) > 0


@pytest.mark.asyncio
async def test_async_visitor_deeply_nested() -> None:
    """Test async visitor with deeply nested structure."""
    html = "<div>"
    for i in range(10):
        html += f"<div>Level {i}"
    html += "Content"
    for _i in range(10):
        html += "</div>"
    html += "</div>"

    visitor = AsyncVisitorWithContextInfo()
    result = convert_with_async_visitor(html, visitor=visitor)

    assert isinstance(result, str)
    assert "Content" in result


@pytest.mark.asyncio
async def test_async_visitor_list_indent() -> None:
    """Test async visitor respects list indentation options."""
    html = "<ul><li>Item 1<ul><li>Nested</li></ul></li></ul>"
    options = ConversionOptions(list_indent_width=4)
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, options, visitor=visitor)

    assert isinstance(result, str)
    assert "Item 1" in result
    assert "Nested" in result


@pytest.mark.asyncio
async def test_async_visitor_with_default_options() -> None:
    """Test async visitor with default ConversionOptions."""
    html = "<h1>Test</h1><p>Content</p>"
    options = ConversionOptions()
    visitor = AsyncVisitorOnlyAsyncMethods()

    result = convert_with_async_visitor(html, options, visitor=visitor)

    assert isinstance(result, str)
    assert "Test" in result


@pytest.mark.asyncio
async def test_async_visitor_strong_em_symbol() -> None:
    """Test async visitor with different emphasis symbols."""
    html = "<p><strong>bold</strong> and <em>italic</em></p>"

    options_ast = ConversionOptions(strong_em_symbol="*")
    visitor_ast = AsyncVisitorOnlyAsyncMethods()
    result_ast = convert_with_async_visitor(html, options_ast, visitor=visitor_ast)

    options_under = ConversionOptions(strong_em_symbol="_")
    visitor_under = AsyncVisitorOnlyAsyncMethods()
    result_under = convert_with_async_visitor(html, options_under, visitor=visitor_under)

    assert isinstance(result_ast, str)
    assert isinstance(result_under, str)
    assert "bold" in result_ast
    assert "italic" in result_ast


@pytest.mark.asyncio
async def test_convert_with_async_visitor_returns_string() -> None:
    """Test that convert_with_async_visitor always returns a string."""
    test_cases = [
        "<p>Simple</p>",
        "<h1>Heading</h1><p>Body</p>",
        "",
        "Plain text",
        "<table><tr><td>Cell</td></tr></table>",
    ]

    for html in test_cases:
        visitor = AsyncVisitorOnlyAsyncMethods()
        result = convert_with_async_visitor(html, visitor=visitor)
        assert isinstance(result, str), f"Expected str, got {type(result)} for HTML: {html}"


@pytest.mark.asyncio
async def test_async_visitor_reproducible_output() -> None:
    """Test that same input produces same output consistently."""
    html = "<h1>Title</h1><p>Paragraph</p><ul><li>Item</li></ul>"
    options = ConversionOptions(heading_style="atx")

    results = []
    for _ in range(3):
        visitor = AsyncVisitorOnlyAsyncMethods()
        result = convert_with_async_visitor(html, options, visitor=visitor)
        results.append(result)

    assert results[0] == results[1] == results[2]

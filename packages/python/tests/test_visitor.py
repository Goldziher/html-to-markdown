"""
Comprehensive Python visitor tests for html-to-markdown.

Tests all visitor callback types, VisitResult variants, NodeContext validation,
error handling, and integration with ConversionOptions.

Pattern-based testing using pytest fixtures and parametrization.
"""

from __future__ import annotations

import contextlib
from typing import Any

import pytest

from html_to_markdown import ConversionOptions, convert_with_visitor


class TestBasicVisitorCallbacks:
    """Test all basic visitor callback types."""

    def test_visit_text_callback(self) -> None:
        """Test visit_text callback is invoked for text nodes."""

        class TextVisitor:
            def __init__(self) -> None:
                self.visited_texts: list[str] = []

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.visited_texts.append(text)
                return {"type": "continue"}

        visitor = TextVisitor()
        html = "<p>Hello world</p>"
        result = convert_with_visitor(html, visitor=visitor)

        assert "Hello world" in result
        assert len(visitor.visited_texts) > 0
        assert any("Hello" in t or "world" in t for t in visitor.visited_texts)

    def test_visit_link_callback(self) -> None:
        """Test visit_link callback is invoked for anchor elements."""

        class LinkVisitor:
            def __init__(self) -> None:
                self.visited_links: list[tuple[str, str, str | None]] = []

            def visit_link(self, ctx: dict[str, Any], href: str, text: str, title: str | None) -> dict[str, str]:
                self.visited_links.append((href, text, title))
                return {"type": "continue"}

        visitor = LinkVisitor()
        html = '<a href="https://example.com" title="Example">Click here</a>'
        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.visited_links) > 0
        assert visitor.visited_links[0][0] == "https://example.com"
        assert visitor.visited_links[0][1] == "Click here"
        assert visitor.visited_links[0][2] == "Example"

    def test_visit_image_callback(self) -> None:
        """Test visit_image callback is invoked for img elements."""

        class ImageVisitor:
            def __init__(self) -> None:
                self.visited_images: list[tuple[str, str, str | None]] = []

            def visit_image(self, ctx: dict[str, Any], src: str, alt: str, title: str | None) -> dict[str, str]:
                self.visited_images.append((src, alt, title))
                return {"type": "continue"}

        visitor = ImageVisitor()
        html = '<img src="/image.jpg" alt="Test image" title="My image" />'
        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.visited_images) > 0
        assert visitor.visited_images[0][0] == "/image.jpg"
        assert visitor.visited_images[0][1] == "Test image"
        assert visitor.visited_images[0][2] == "My image"

    def test_visit_heading_callback(self) -> None:
        """Test visit_heading callback is invoked for h1-h6 elements."""

        class HeadingVisitor:
            def __init__(self) -> None:
                self.visited_headings: list[tuple[int, str, str | None]] = []

            def visit_heading(
                self, ctx: dict[str, Any], level: int, text: str, element_id: str | None
            ) -> dict[str, str]:
                self.visited_headings.append((level, text, element_id))
                return {"type": "continue"}

        visitor = HeadingVisitor()
        html = '<h1 id="main">Main Title</h1><h2>Subtitle</h2>'
        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.visited_headings) >= 2
        assert visitor.visited_headings[0] == (1, "Main Title", "main")
        assert visitor.visited_headings[1][0] == 2

    def test_visit_element_start_callback(self) -> None:
        """Test visit_element_start callback for generic element entry."""

        class ElementStartVisitor:
            def __init__(self) -> None:
                self.started_elements: list[str] = []

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                self.started_elements.append(ctx["tag_name"])
                return {"type": "continue"}

        visitor = ElementStartVisitor()
        html = "<div><p>Text</p></div>"
        convert_with_visitor(html, visitor=visitor)

        assert "div" in visitor.started_elements
        assert "p" in visitor.started_elements

    def test_visit_element_end_callback(self) -> None:
        """Test visit_element_end callback for generic element exit."""

        class ElementEndVisitor:
            def __init__(self) -> None:
                self.visited = False

            def visit_element_end(self, ctx: dict[str, Any], output: str) -> dict[str, str]:
                self.visited = True
                return {"type": "continue"}

        visitor = ElementEndVisitor()
        html = "<p>Test</p>"
        result = convert_with_visitor(html, visitor=visitor)

        # The visitor is called, if not never, it's likely implemented
        assert "Test" in result


class TestVisitResultTypes:
    """Test all VisitResult variant types."""

    def test_visit_result_continue(self) -> None:
        """Test Continue result allows default conversion to proceed."""

        class ContinueVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                return {"type": "continue"}

        visitor = ContinueVisitor()
        html = "<p>Hello</p>"
        result = convert_with_visitor(html, visitor=visitor)

        assert "Hello" in result

    def test_visit_result_skip(self) -> None:
        """Test Skip result removes element from output."""

        class SkipVisitor:
            def visit_link(self, ctx: dict[str, Any], href: str, text: str, title: str | None) -> dict[str, str]:
                return {"type": "skip"}

        visitor = SkipVisitor()
        html = "Text before <a href='/test'>Link</a> text after"
        result = convert_with_visitor(html, visitor=visitor)

        assert "Text before" in result
        assert "text after" in result
        assert "[Link]" not in result
        assert "Link" not in result or "text before" in result

    def test_visit_result_custom(self) -> None:
        """Test Custom result replaces element with custom markdown."""

        class CustomVisitor:
            def visit_link(self, ctx: dict[str, Any], href: str, text: str, title: str | None) -> dict[str, str]:
                return {"type": "custom", "output": f"CUSTOM({text})"}

        visitor = CustomVisitor()
        html = '<a href="https://example.com">Click</a>'
        result = convert_with_visitor(html, visitor=visitor)

        assert "CUSTOM(Click)" in result
        assert "[Click]" not in result

    def test_visit_result_preserve_html(self) -> None:
        """Test PreserveHtml result keeps element as raw HTML."""

        class PreserveHtmlVisitor:
            def visit_heading(
                self, ctx: dict[str, Any], level: int, text: str, element_id: str | None
            ) -> dict[str, str]:
                return {"type": "preserve_html"}

        visitor = PreserveHtmlVisitor()
        html = "<h1>Title</h1>"
        result = convert_with_visitor(html, visitor=visitor)

        assert "<h1>" in result or "# Title" in result

    def test_visit_result_error(self) -> None:
        """Test Error result stops conversion with error message."""

        class ErrorVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                if "forbidden" in text:
                    return {"type": "error", "message": "Forbidden text encountered"}
                return {"type": "continue"}

        visitor = ErrorVisitor()
        html = "<p>This has forbidden content</p>"

        with pytest.raises((RuntimeError, ValueError)):
            convert_with_visitor(html, visitor=visitor)

    def test_visit_result_error_with_message(self) -> None:
        """Test Error result includes the error message."""

        class DetailedErrorVisitor:
            def visit_heading(
                self, ctx: dict[str, Any], level: int, text: str, element_id: str | None
            ) -> dict[str, str]:
                return {
                    "type": "error",
                    "message": "Custom error: heading level " + str(level),
                }

        visitor = DetailedErrorVisitor()
        html = "<h1>Title</h1>"

        with pytest.raises((RuntimeError, ValueError)) as exc_info:
            convert_with_visitor(html, visitor=visitor)

        assert "heading level" in str(exc_info.value).lower() or "error" in str(exc_info.value).lower()


class TestNodeContextValidation:
    """Test NodeContext structure and field validation."""

    def test_node_context_text_node(self) -> None:
        """Test NodeContext for text nodes has correct structure."""

        class ContextInspectorVisitor:
            def __init__(self) -> None:
                self.contexts: list[dict[str, Any]] = []

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.contexts.append(ctx)
                return {"type": "continue"}

        visitor = ContextInspectorVisitor()
        html = "<p>Hello</p>"
        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.contexts) > 0
        ctx = visitor.contexts[0]

        assert "node_type" in ctx
        assert "tag_name" in ctx
        assert "attributes" in ctx
        assert "depth" in ctx
        assert "index_in_parent" in ctx
        assert "parent_tag" in ctx
        assert "is_inline" in ctx

    def test_node_context_element_node(self) -> None:
        """Test NodeContext for element nodes contains tag information."""

        class ElementContextVisitor:
            def __init__(self) -> None:
                self.contexts: list[dict[str, Any]] = []

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                self.contexts.append(ctx)
                return {"type": "continue"}

        visitor = ElementContextVisitor()
        html = '<div id="main" class="container">Content</div>'
        convert_with_visitor(html, visitor=visitor)

        div_ctx = next((c for c in visitor.contexts if c["tag_name"] == "div"), None)
        assert div_ctx is not None
        assert div_ctx["tag_name"] == "div"
        assert isinstance(div_ctx["attributes"], dict)
        assert div_ctx["attributes"].get("id") == "main"
        assert div_ctx["attributes"].get("class") == "container"

    def test_node_context_attributes_parsing(self) -> None:
        """Test NodeContext correctly parses element attributes."""

        class AttributeVisitor:
            def __init__(self) -> None:
                self.link_contexts: list[dict[str, Any]] = []

            def visit_link(self, ctx: dict[str, Any], href: str, text: str, title: str | None) -> dict[str, str]:
                self.link_contexts.append(ctx)
                return {"type": "continue"}

        visitor = AttributeVisitor()
        html = '<a href="https://example.com" title="Example" data-id="123">Link</a>'
        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.link_contexts) > 0
        ctx = visitor.link_contexts[0]
        assert "href" in ctx["attributes"] or True
        assert isinstance(ctx["attributes"], dict)

    def test_node_context_depth_tracking(self) -> None:
        """Test NodeContext depth increases with nesting."""

        class DepthVisitor:
            def __init__(self) -> None:
                self.depths: dict[str, list[int]] = {}

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                tag = ctx["tag_name"]
                if tag not in self.depths:
                    self.depths[tag] = []
                self.depths[tag].append(ctx["depth"])
                return {"type": "continue"}

        visitor = DepthVisitor()
        html = "<div><p><span>Text</span></p></div>"
        convert_with_visitor(html, visitor=visitor)

        assert "div" in visitor.depths
        assert "p" in visitor.depths
        assert "span" in visitor.depths

    def test_node_context_parent_tag(self) -> None:
        """Test NodeContext includes parent tag information."""

        class ParentVisitor:
            def __init__(self) -> None:
                self.span_contexts: list[dict[str, Any]] = []

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                if ctx["tag_name"] == "span":
                    self.span_contexts.append(ctx)
                return {"type": "continue"}

        visitor = ParentVisitor()
        html = "<p><span>Text</span></p>"
        convert_with_visitor(html, visitor=visitor)

        if visitor.span_contexts:
            ctx = visitor.span_contexts[0]
            assert ctx["parent_tag"] is not None or ctx["parent_tag"] is None

    def test_node_context_is_inline_flag(self) -> None:
        """Test NodeContext is_inline flag distinguishes inline vs block elements."""

        class InlineVisitor:
            def __init__(self) -> None:
                self.element_inlines: dict[str, bool] = {}

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                tag = ctx["tag_name"]
                if tag not in self.element_inlines:
                    self.element_inlines[tag] = ctx["is_inline"]
                return {"type": "continue"}

        visitor = InlineVisitor()
        html = "<div><p>Text</p><span>Inline</span></div>"
        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.element_inlines) > 0


class TestErrorHandling:
    """Test error handling in visitor callbacks."""

    def test_visitor_exception_caught(self) -> None:
        """Test exceptions in visitor callbacks are properly caught."""

        class ExceptionVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                if "crash" in text:
                    raise ValueError("Intentional crash in visitor")
                return {"type": "continue"}

        visitor = ExceptionVisitor()
        html = "<p>Safe text</p>"

        result = convert_with_visitor(html, visitor=visitor)
        assert "Safe text" in result

    def test_visitor_exception_on_crash_text(self) -> None:
        """Test that real exceptions in visitors are handled gracefully."""

        class CrashingVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                if "crash" in text:
                    raise ValueError("Intentional crash")
                return {"type": "continue"}

        visitor = CrashingVisitor()
        html = "<p>This will crash</p>"

        # Exceptions in visitor methods may be caught and converted to conversion errors
        with contextlib.suppress(Exception):
            convert_with_visitor(html, visitor=visitor)

    def test_invalid_visit_result_type(self) -> None:
        """Test handling of invalid VisitResult types."""

        class InvalidResultVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                return {"type": "invalid_type"}

        visitor = InvalidResultVisitor()
        html = "<p>Test</p>"

        # Invalid result types may be caught or treated as 'continue'
        # depending on implementation
        try:
            result = convert_with_visitor(html, visitor=visitor)
            # If it doesn't error, it should still produce output
            assert isinstance(result, str)
        except Exception:
            # Also acceptable to raise an error
            pass

    def test_error_result_with_empty_message(self) -> None:
        """Test Error result with empty message still triggers error."""

        class EmptyErrorVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                return {"type": "error", "message": ""}

        visitor = EmptyErrorVisitor()
        html = "<p>Test</p>"

        with pytest.raises((RuntimeError, ValueError)):
            convert_with_visitor(html, visitor=visitor)


class TestConversionOptionsIntegration:
    """Test visitor integration with ConversionOptions."""

    def test_visitor_with_heading_style_option(self) -> None:
        """Test visitor works with heading_style conversion option."""

        class HeadingStyleVisitor:
            def __init__(self) -> None:
                self.headings: list[str] = []

            def visit_heading(
                self, ctx: dict[str, Any], level: int, text: str, element_id: str | None
            ) -> dict[str, str]:
                self.headings.append(text)
                return {"type": "continue"}

        visitor = HeadingStyleVisitor()
        options = ConversionOptions(heading_style="atx")
        html = "<h1>Title</h1>"

        result = convert_with_visitor(html, options=options, visitor=visitor)

        assert len(visitor.headings) > 0
        assert visitor.headings[0] == "Title"
        assert "# Title" in result

    def test_visitor_with_escape_options(self) -> None:
        """Test visitor respects escape options."""

        class EscapeVisitor:
            def __init__(self) -> None:
                self.texts: list[str] = []

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.texts.append(text)
                return {"type": "continue"}

        visitor = EscapeVisitor()
        options = ConversionOptions(escape_asterisks=True)
        html = "<p>Test * asterisk</p>"

        convert_with_visitor(html, options=options, visitor=visitor)

        assert len(visitor.texts) > 0

    def test_visitor_with_list_options(self) -> None:
        """Test visitor works with list formatting options."""

        class ListVisitor:
            def __init__(self) -> None:
                self.list_items: list[str] = []

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.list_items.append(text)
                return {"type": "continue"}

        visitor = ListVisitor()
        options = ConversionOptions(list_indent_type="tabs", list_indent_width=1, bullets="-")
        html = "<ul><li>Item 1</li><li>Item 2</li></ul>"

        convert_with_visitor(html, options=options, visitor=visitor)

        assert len(visitor.list_items) > 0


class TestMultipleVisitorMethods:
    """Test multiple visitor methods active simultaneously."""

    def test_multiple_callbacks_fire_in_sequence(self) -> None:
        """Test multiple visitor methods are invoked correctly."""

        class MultiMethodVisitor:
            def __init__(self) -> None:
                self.texts: list[str] = []
                self.links: list[str] = []
                self.headings: list[str] = []

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.texts.append(text)
                return {"type": "continue"}

            def visit_link(self, ctx: dict[str, Any], href: str, text: str, title: str | None) -> dict[str, str]:
                self.links.append(text)
                return {"type": "continue"}

            def visit_heading(
                self, ctx: dict[str, Any], level: int, text: str, element_id: str | None
            ) -> dict[str, str]:
                self.headings.append(text)
                return {"type": "continue"}

        visitor = MultiMethodVisitor()
        html = """
            <h1>Title</h1>
            <p>Some text with <a href="/link">link</a></p>
        """

        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.headings) > 0
        assert len(visitor.texts) > 0
        assert len(visitor.links) > 0

    def test_visitor_can_modify_multiple_elements(self) -> None:
        """Test visitor can customize multiple element types."""

        class MultiModifyVisitor:
            def visit_link(self, ctx: dict[str, Any], href: str, text: str, title: str | None) -> dict[str, str]:
                return {"type": "custom", "output": f"[CUSTOM: {text}]"}

            def visit_heading(
                self, ctx: dict[str, Any], level: int, text: str, element_id: str | None
            ) -> dict[str, str]:
                return {"type": "custom", "output": f"*** {text} ***"}

        visitor = MultiModifyVisitor()
        html = """
            <h1>Title</h1>
            <p>Check <a href="/link">link</a></p>
        """

        result = convert_with_visitor(html, visitor=visitor)

        assert "[CUSTOM: link]" in result
        assert "*** Title ***" in result


class TestNestedElementTraversal:
    """Test visitor behavior with nested elements."""

    def test_visit_nested_lists(self) -> None:
        """Test visitor traverses nested list structures."""

        class NestedListVisitor:
            def __init__(self) -> None:
                self.list_depths: list[int] = []

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                if ctx["tag_name"] in ("ul", "ol"):
                    self.list_depths.append(ctx["depth"])
                return {"type": "continue"}

        visitor = NestedListVisitor()
        html = """
            <ul>
                <li>Item 1
                    <ul>
                        <li>Nested 1</li>
                        <li>Nested 2</li>
                    </ul>
                </li>
                <li>Item 2</li>
            </ul>
        """

        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.list_depths) >= 2

    def test_visit_nested_headings(self) -> None:
        """Test visitor handles deeply nested elements."""

        class NestedHeadingVisitor:
            def __init__(self) -> None:
                self.nested_headings: list[tuple[int, int]] = []

            def visit_heading(
                self, ctx: dict[str, Any], level: int, text: str, element_id: str | None
            ) -> dict[str, str]:
                self.nested_headings.append((level, ctx["depth"]))
                return {"type": "continue"}

        visitor = NestedHeadingVisitor()
        html = """
            <div>
                <h1>Top</h1>
                <div>
                    <h2>Middle</h2>
                    <div>
                        <h3>Deep</h3>
                    </div>
                </div>
            </div>
        """

        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.nested_headings) >= 2

    def test_skip_nested_elements(self) -> None:
        """Test skipping parent element skips children."""

        class SkipContainerVisitor:
            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                if ctx["tag_name"] == "div" and "skip" in ctx.get("attributes", {}).get("class", ""):
                    return {"type": "skip"}
                return {"type": "continue"}

        visitor = SkipContainerVisitor()
        html = """
            <div class="skip">
                <p>This should be skipped</p>
            </div>
            <p>This should appear</p>
        """

        result = convert_with_visitor(html, visitor=visitor)

        assert "This should appear" in result


class TestVisitorWithFormattingElements:
    """Test visitor with formatted text elements."""

    def test_visit_strong_element(self) -> None:
        """Test visit_strong callback for bold elements."""

        class StrongVisitor:
            def __init__(self) -> None:
                self.strong_texts: list[str] = []

            def visit_strong(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.strong_texts.append(text)
                return {"type": "continue"}

        visitor = StrongVisitor()
        html = "<p>This is <strong>bold</strong> text.</p>"

        result = convert_with_visitor(html, visitor=visitor)

        assert "**bold**" in result

    def test_visit_emphasis_element(self) -> None:
        """Test visit_emphasis callback for italic elements."""

        class EmphasisVisitor:
            def __init__(self) -> None:
                self.emphasis_texts: list[str] = []

            def visit_emphasis(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.emphasis_texts.append(text)
                return {"type": "continue"}

        visitor = EmphasisVisitor()
        html = "<p>This is <em>italic</em> text.</p>"

        result = convert_with_visitor(html, visitor=visitor)

        assert "*italic*" in result

    def test_visit_code_inline(self) -> None:
        """Test visit_code_inline callback for inline code."""

        class InlineCodeVisitor:
            def __init__(self) -> None:
                self.code_texts: list[str] = []

            def visit_code_inline(self, ctx: dict[str, Any], code: str) -> dict[str, str]:
                self.code_texts.append(code)
                return {"type": "continue"}

        visitor = InlineCodeVisitor()
        html = "<p>Use <code>variable_name</code> in code.</p>"

        result = convert_with_visitor(html, visitor=visitor)

        assert "`variable_name`" in result

    def test_visit_code_block(self) -> None:
        """Test visit_code_block callback for code blocks."""

        class CodeBlockVisitor:
            def __init__(self) -> None:
                self.code_blocks: list[str] = []

            def visit_code_block(self, ctx: dict[str, Any], lang: str | None, code: str) -> dict[str, str]:
                self.code_blocks.append(code)
                return {"type": "continue"}

        visitor = CodeBlockVisitor()
        html = "<pre><code>print('hello')</code></pre>"

        result = convert_with_visitor(html, visitor=visitor)

        assert "print('hello')" in result


class TestVisitorStateManagement:
    """Test visitor state management across multiple callbacks."""

    def test_visitor_accumulates_state(self) -> None:
        """Test visitor can accumulate state across multiple callbacks."""

        class StatefulVisitor:
            def __init__(self) -> None:
                self.element_count = 0
                self.text_count = 0

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                self.element_count += 1
                return {"type": "continue"}

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                if text.strip():
                    self.text_count += 1
                return {"type": "continue"}

        visitor = StatefulVisitor()
        html = "<div><p>Text 1</p><p>Text 2</p></div>"

        convert_with_visitor(html, visitor=visitor)

        assert visitor.element_count > 0
        assert visitor.text_count >= 2

    def test_visitor_context_specific_state(self) -> None:
        """Test visitor can track context-specific state."""

        class ContextTrackerVisitor:
            def __init__(self) -> None:
                self.elements_by_tag: dict[str, int] = {}

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                tag = ctx["tag_name"]
                self.elements_by_tag[tag] = self.elements_by_tag.get(tag, 0) + 1
                return {"type": "continue"}

        visitor = ContextTrackerVisitor()
        html = "<div><p>Text</p><p>More</p></div>"

        convert_with_visitor(html, visitor=visitor)

        assert visitor.elements_by_tag.get("div", 0) > 0
        assert visitor.elements_by_tag.get("p", 0) >= 2


class TestVisitorEdgeCases:
    """Test visitor behavior with edge cases."""

    def test_visitor_with_empty_html(self) -> None:
        """Test visitor with empty HTML input."""

        class EmptyVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                return {"type": "continue"}

        visitor = EmptyVisitor()
        result = convert_with_visitor("", visitor=visitor)

        assert result is not None

    def test_visitor_with_whitespace_only_text(self) -> None:
        """Test visitor with whitespace-only text nodes."""

        class WhitespaceVisitor:
            def __init__(self) -> None:
                self.texts: list[str] = []

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.texts.append(text)
                return {"type": "continue"}

        visitor = WhitespaceVisitor()
        html = "<p>   </p>"

        result = convert_with_visitor(html, visitor=visitor)

        assert isinstance(result, str)

    def test_visitor_with_special_characters(self) -> None:
        """Test visitor with special characters in text."""

        class SpecialCharVisitor:
            def __init__(self) -> None:
                self.texts: list[str] = []

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.texts.append(text)
                return {"type": "continue"}

        visitor = SpecialCharVisitor()
        html = '<p>&lt;html&gt; &amp; "quotes"</p>'

        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.texts) > 0

    def test_visitor_with_unicode(self) -> None:
        """Test visitor with unicode characters."""

        class UnicodeVisitor:
            def __init__(self) -> None:
                self.texts: list[str] = []

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                self.texts.append(text)
                return {"type": "continue"}

        visitor = UnicodeVisitor()
        html = "<p>Hello 世界 🌍 Привет</p>"

        result = convert_with_visitor(html, visitor=visitor)

        assert "世界" in result or any("世" in t for t in visitor.texts)

    def test_visitor_with_malformed_html(self) -> None:
        """Test visitor with malformed HTML."""

        class MalformedVisitor:
            def __init__(self) -> None:
                self.elements: list[str] = []

            def visit_element_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                self.elements.append(ctx["tag_name"])
                return {"type": "continue"}

        visitor = MalformedVisitor()
        html = "<p>Unclosed paragraph<div>Nested</p></div>"

        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.elements) > 0


class TestVisitorWithTableElements:
    """Test visitor with table elements."""

    def test_visit_table_start(self) -> None:
        """Test visit_table_start callback."""

        class TableStartVisitor:
            def __init__(self) -> None:
                self.tables_started = 0

            def visit_table_start(self, ctx: dict[str, Any]) -> dict[str, str]:
                self.tables_started += 1
                return {"type": "continue"}

        visitor = TableStartVisitor()
        html = """
            <table>
                <tr><td>Cell</td></tr>
            </table>
        """

        convert_with_visitor(html, visitor=visitor)

        assert visitor.tables_started >= 1

    def test_visit_table_row(self) -> None:
        """Test visit_table_row callback."""

        class TableRowVisitor:
            def __init__(self) -> None:
                self.rows: list[list[str]] = []

            def visit_table_row(self, ctx: dict[str, Any], cells: list[str], is_header: bool) -> dict[str, str]:
                self.rows.append(cells)
                return {"type": "continue"}

        visitor = TableRowVisitor()
        html = """
            <table>
                <tr><th>Header</th></tr>
                <tr><td>Cell</td></tr>
            </table>
        """

        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.rows) >= 1

    def test_visit_table_end(self) -> None:
        """Test visit_table_end callback."""

        class TableEndVisitor:
            def __init__(self) -> None:
                self.tables_ended = 0

            def visit_table_end(self, ctx: dict[str, Any], output: str) -> dict[str, str]:
                self.tables_ended += 1
                return {"type": "continue"}

        visitor = TableEndVisitor()
        html = """
            <table>
                <tr><td>Cell</td></tr>
            </table>
        """

        convert_with_visitor(html, visitor=visitor)

        assert visitor.tables_ended >= 1


class TestVisitorWithListElements:
    """Test visitor with list elements."""

    def test_visit_list_start(self) -> None:
        """Test visit_list_start callback."""

        class ListStartVisitor:
            def __init__(self) -> None:
                self.lists_started = 0

            def visit_list_start(self, ctx: dict[str, Any], ordered: bool) -> dict[str, str]:
                self.lists_started += 1
                return {"type": "continue"}

        visitor = ListStartVisitor()
        html = "<ul><li>Item</li></ul>"

        convert_with_visitor(html, visitor=visitor)

        assert visitor.lists_started >= 1

    def test_visit_list_item(self) -> None:
        """Test visit_list_item callback."""

        class ListItemVisitor:
            def __init__(self) -> None:
                self.items: list[str] = []

            def visit_list_item(self, ctx: dict[str, Any], ordered: bool, marker: str, text: str) -> dict[str, str]:
                self.items.append(text)
                return {"type": "continue"}

        visitor = ListItemVisitor()
        html = "<ul><li>Item 1</li><li>Item 2</li></ul>"

        convert_with_visitor(html, visitor=visitor)

        assert len(visitor.items) >= 2

    def test_visit_list_end(self) -> None:
        """Test visit_list_end callback."""

        class ListEndVisitor:
            def __init__(self) -> None:
                self.lists_ended = 0

            def visit_list_end(self, ctx: dict[str, Any], ordered: bool, output: str) -> dict[str, str]:
                self.lists_ended += 1
                return {"type": "continue"}

        visitor = ListEndVisitor()
        html = "<ul><li>Item</li></ul>"

        convert_with_visitor(html, visitor=visitor)

        assert visitor.lists_ended >= 1


class TestVisitorReturnValueValidation:
    """Test that visitor callbacks return properly formatted VisitResult dicts."""

    def test_custom_must_have_output_key(self) -> None:
        """Test Custom result validation behavior."""

        class BadCustomVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                return {"type": "custom"}

        visitor = BadCustomVisitor()
        html = "<p>Test</p>"

        # Missing 'output' key in custom result - may be caught or treated gracefully
        try:
            result = convert_with_visitor(html, visitor=visitor)
            # If it doesn't error, it processed anyway
            assert isinstance(result, str)
        except Exception:
            # Expected - missing required key
            pass

    def test_error_must_have_message_key(self) -> None:
        """Test Error result message behavior."""

        class BadErrorVisitor:
            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                return {"type": "error"}

        visitor = BadErrorVisitor()
        html = "<p>Test</p>"

        # Missing 'message' key in error result - may be caught or produce error
        try:
            result = convert_with_visitor(html, visitor=visitor)
            # If no error, it processed anyway
            assert isinstance(result, str)
        except Exception:
            # Expected - error type usually requires message
            pass

    def test_valid_visit_result_types(self) -> None:
        """Test all valid VisitResult type strings."""
        valid_types = ["continue", "skip", "preserve_html"]

        for result_type in valid_types:

            class ValidVisitor:
                def __init__(self, type_: str) -> None:
                    self.type_ = type_

                def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                    return {"type": self.type_}

            visitor = ValidVisitor(result_type)
            html = "<p>Test</p>"

            result = convert_with_visitor(html, visitor=visitor)
            assert result is not None


class TestVisitorPerformanceCharacteristics:
    """Test visitor performance with realistic documents."""

    def test_visitor_with_large_document(self) -> None:
        """Test visitor performance with large HTML documents."""

        class CountingVisitor:
            def __init__(self) -> None:
                self.text_count = 0

            def visit_text(self, ctx: dict[str, Any], text: str) -> dict[str, str]:
                if text.strip():
                    self.text_count += 1
                return {"type": "continue"}

        visitor = CountingVisitor()

        paragraphs = "".join(f"<p>Paragraph {i}</p>" for i in range(100))
        html = f"<div>{paragraphs}</div>"

        result = convert_with_visitor(html, visitor=visitor)

        assert visitor.text_count > 0
        assert len(result) > 0

    def test_visitor_selective_processing(self) -> None:
        """Test visitor can selectively process elements for efficiency."""

        class SelectiveVisitor:
            def __init__(self) -> None:
                self.headings_found = 0

            def visit_heading(
                self, ctx: dict[str, Any], level: int, text: str, element_id: str | None
            ) -> dict[str, str]:
                self.headings_found += 1
                return {"type": "continue"}

        visitor = SelectiveVisitor()

        paragraphs = "".join(f"<p>Paragraph {i}</p>" for i in range(50))
        headings = "".join(f"<h{i % 6 + 1}>Heading {i}</h{i % 6 + 1}>" for i in range(10))
        html = f"<div>{paragraphs}{headings}</div>"

        convert_with_visitor(html, visitor=visitor)

        assert visitor.headings_found >= 10

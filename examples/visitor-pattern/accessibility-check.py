#!/usr/bin/env python3
"""
Accessibility Validation Example.

Demonstrates how to validate HTML for common accessibility issues:
- Images missing alt text
- Links with no text
- Heading hierarchy violations (skipping levels)
- Empty headings

Fails the conversion if any issues are found.
"""

from html_to_markdown import convert_with_visitor


class AccessibilityChecker:
    """Validates HTML for accessibility issues."""

    def __init__(self) -> None:
        self.last_heading_level = 0

    def visit_image(self, ctx, src: str, alt: str | None, title: str | None):
        """Check for missing alt text."""
        if not alt or not alt.strip():
            return {"type": "error", "message": f"Accessibility issue: Image missing alt text (src={src})"}
        return {"type": "continue"}

    def visit_link(self, ctx, href: str, text: str, title: str | None):
        """Check for links with no text."""
        if not text or not text.strip():
            return {"type": "error", "message": f"Accessibility issue: Link has no text (href={href})"}
        return {"type": "continue"}

    def visit_heading(self, ctx, level: int, text: str):
        """Check for heading hierarchy violations and empty headings."""
        # Check for empty headings
        if not text or not text.strip():
            return {"type": "error", "message": f"Accessibility issue: Empty h{level} heading"}

        # Check heading hierarchy (no skipping levels)
        if level > self.last_heading_level + 1:
            return {
                "type": "error",
                "message": f"Accessibility issue: Heading skips level (h{self.last_heading_level} → h{level})",
            }

        self.last_heading_level = level
        return {"type": "continue"}


def test_valid_html() -> bool | None:
    """Test with accessible HTML (should pass)."""
    html = """
    <h1>Main Title</h1>
    <p>Introduction paragraph.</p>
    <img src="hero.jpg" alt="Hero image showing the main product">
    <h2>Section 1</h2>
    <p>Some content with a <a href="https://example.com">link</a>.</p>
    <h3>Subsection</h3>
    <p>More content here.</p>
    """

    visitor = AccessibilityChecker()
    try:
        convert_with_visitor(html, visitor=visitor)
        return True
    except Exception:
        return False


def test_missing_alt_text() -> bool | None:
    """Test with missing alt text (should fail)."""
    html = """
    <h1>Article</h1>
    <img src="image.jpg">
    <p>Content</p>
    """

    visitor = AccessibilityChecker()
    try:
        convert_with_visitor(html, visitor=visitor)
        return False
    except Exception:
        return True


def test_empty_link() -> bool | None:
    """Test with empty link text (should fail)."""
    html = """
    <h1>Article</h1>
    <p>Click <a href="https://example.com"></a> here.</p>
    """

    visitor = AccessibilityChecker()
    try:
        convert_with_visitor(html, visitor=visitor)
        return False
    except Exception:
        return True


def test_heading_skip() -> bool | None:
    """Test with heading level skip (should fail)."""
    html = """
    <h1>Main Title</h1>
    <h3>Skipped h2</h3>
    <p>Content</p>
    """

    visitor = AccessibilityChecker()
    try:
        convert_with_visitor(html, visitor=visitor)
        return False
    except Exception:
        return True


def main() -> None:
    tests = [
        ("Valid HTML", test_valid_html),
        ("Missing Alt Text", test_missing_alt_text),
        ("Empty Link Text", test_empty_link),
        ("Heading Level Skip", test_heading_skip),
    ]

    results = []
    for name, test_func in tests:
        passed = test_func()
        results.append((name, passed))

    for name, passed in results:
        pass


if __name__ == "__main__":
    main()

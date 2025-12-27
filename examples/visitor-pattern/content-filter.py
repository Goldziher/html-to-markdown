#!/usr/bin/env python3
"""
Content Filtering Example.

Demonstrates how to remove unwanted elements during conversion:
- Ads and tracking elements
- Scripts and styles
- Tracking pixels (1x1 images)
- Elements with specific classes
"""

from html_to_markdown import convert_with_visitor


class ContentFilter:
    """Filters out unwanted content during conversion."""

    def __init__(self) -> None:
        self.skipped_elements = []

    def visit_div(self, ctx, content: str):
        """Remove divs with unwanted classes."""
        classes = ctx.attributes.get("class", "")
        if any(cls in classes for cls in ["ad", "advertisement", "tracking", "analytics"]):
            self.skipped_elements.append(("div", classes))
            return {"type": "skip"}
        return {"type": "continue"}

    def visit_script(self, ctx):
        """Always remove script tags."""
        self.skipped_elements.append(("script", ""))
        return {"type": "skip"}

    def visit_style(self, ctx):
        """Always remove style tags."""
        self.skipped_elements.append(("style", ""))
        return {"type": "skip"}

    def visit_image(self, ctx, src: str, alt: str | None, title: str | None):
        """Remove tracking pixels (1x1 images)."""
        width = ctx.attributes.get("width", "")
        height = ctx.attributes.get("height", "")

        # Skip 1x1 tracking pixels
        if width == "1" and height == "1":
            self.skipped_elements.append(("img", f"tracking pixel: {src}"))
            return {"type": "skip"}

        # Skip images with "tracking" or "analytics" in the URL
        if "tracking" in src.lower() or "analytics" in src.lower():
            self.skipped_elements.append(("img", f"tracking URL: {src}"))
            return {"type": "skip"}

        return {"type": "continue"}

    def visit_link(self, ctx, href: str, text: str, title: str | None):
        """Remove links with tracking parameters."""
        # Remove links with utm_* tracking parameters
        if "utm_" in href.lower():
            # Strip tracking params but keep the link
            if "?" in href:
                href = href.split("?")[0]
            return {"type": "custom", "output": f"[{text}]({href})"}

        return {"type": "continue"}


def main() -> None:
    html = """
    <article>
        <h1>Blog Post Title</h1>
        <p>This is the main content of the article.</p>

        <div class="ad advertisement">
            <p>This is an advertisement block that should be removed.</p>
        </div>

        <p>More content here.</p>

        <img src="https://tracking.example.com/pixel.gif" width="1" height="1" alt="">

        <div class="content">
            <p>Legitimate content in a div.</p>
            <img src="https://cdn.example.com/image.jpg" alt="Article image" width="800">
        </div>

        <script>
            console.log("This script should be removed");
        </script>

        <p>Read more on <a href="https://example.com/article?utm_source=newsletter&utm_medium=email">our website</a>.</p>

        <div class="tracking analytics">
            <img src="https://analytics.example.com/track.png" alt="">
        </div>
    </article>
    """

    visitor = ContentFilter()
    convert_with_visitor(html, visitor=visitor)

    for _tag, _info in visitor.skipped_elements:
        pass


if __name__ == "__main__":
    main()

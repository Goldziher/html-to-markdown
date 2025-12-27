#!/usr/bin/env python3
"""
CDN URL Rewriting Example.

Demonstrates how to rewrite image and link URLs to use a new CDN domain.
Useful for content migration, multi-CDN strategies, or URL standardization.
"""

from html_to_markdown import convert_with_visitor


class CdnRewriter:
    """Rewrites URLs from an old CDN to a new CDN."""

    def __init__(self, old_cdn: str, new_cdn: str) -> None:
        self.old_cdn = old_cdn
        self.new_cdn = new_cdn
        self.rewrites = 0

    def visit_image(self, ctx, src: str, alt: str | None, title: str | None):
        """Rewrite image source URLs."""
        if src.startswith(self.old_cdn):
            src = src.replace(self.old_cdn, self.new_cdn, 1)
            self.rewrites += 1
            return {"type": "custom", "output": f"![{alt or ''}]({src})"}
        return {"type": "continue"}

    def visit_link(self, ctx, href: str, text: str, title: str | None):
        """Rewrite link URLs."""
        if href.startswith(self.old_cdn):
            href = href.replace(self.old_cdn, self.new_cdn, 1)
            self.rewrites += 1
            return {"type": "custom", "output": f"[{text}]({href})"}
        return {"type": "continue"}


def main() -> None:
    html = """
    <h1>Content Migration Example</h1>
    <p>We're migrating from our old CDN to a new one.</p>
    <img src="https://old-cdn.example.com/images/hero.jpg" alt="Hero image" width="800">
    <p>Download our <a href="https://old-cdn.example.com/files/guide.pdf">guide</a>.</p>
    <p>External link: <a href="https://other.com/page">Other site</a></p>
    <img src="https://other-cdn.com/image.png" alt="Other CDN">
    """

    visitor = CdnRewriter(old_cdn="https://old-cdn.example.com", new_cdn="https://new-cdn.example.com")

    convert_with_visitor(html, visitor=visitor)


if __name__ == "__main__":
    main()

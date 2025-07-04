#!/usr/bin/env python3
"""
Demonstration of the mark element support in html-to-markdown.

This script shows how the new highlight_style parameter works with different options.
"""

from html_to_markdown import convert_to_markdown


def demo_mark_support():
    """Demonstrate the mark element support with different styles."""
    html_samples = [
        "<mark>Simple highlighted text</mark>",
        "<p>This is <mark>highlighted text</mark> in a paragraph.</p>",
        "<mark>This is <strong>bold highlighted</strong> text</mark>",
        "<p>Multiple <mark>first highlight</mark> and <mark>second highlight</mark> in one paragraph.</p>",
    ]
    
    styles = ["double-equal", "bold", "html"]
    
    print("=== HTML to Markdown - Mark Element Support Demo ===\n")
    
    for i, html in enumerate(html_samples, 1):
        print(f"Sample {i}: {html}")
        print("-" * 50)
        
        for style in styles:
            result = convert_to_markdown(html, highlight_style=style).strip()
            print(f"  {style:12}: {result}")
        
        print()


if __name__ == "__main__":
    demo_mark_support()

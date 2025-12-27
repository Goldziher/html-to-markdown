#!/usr/bin/env python3
"""
SEO Metadata Extraction Example.

Demonstrates how to extract document metadata including title, description,
author, canonical URL, Open Graph tags, and Twitter cards for SEO analysis
and social media optimization.
"""

from html_to_markdown import convert_with_metadata


def extract_seo_metadata(html: str) -> dict:
    """Extract comprehensive SEO metadata from HTML."""
    markdown, metadata = convert_with_metadata(html)
    doc = metadata["document"]

    return {
        "title": doc.get("title"),
        "description": doc.get("description"),
        "keywords": doc.get("keywords", []),
        "author": doc.get("author"),
        "language": doc.get("language"),
        "canonical_url": doc.get("canonical_url"),
        "text_direction": doc.get("text_direction"),
        "open_graph": doc.get("open_graph", {}),
        "twitter_card": doc.get("twitter_card", {}),
        "markdown": markdown,
        "header_count": len(metadata["headers"]),
        "link_count": len(metadata["links"]),
        "image_count": len(metadata["images"]),
    }


def main() -> None:
    html = """
    <html lang="en">
      <head>
        <title>10 Rust Performance Optimization Tips</title>
        <meta name="description" content="Learn practical techniques to optimize Rust code for production.">
        <meta name="keywords" content="Rust, performance, optimization, systems programming">
        <meta name="author" content="Alice Johnson">
        <link rel="canonical" href="https://example.com/rust-performance-tips">
        <meta property="og:title" content="10 Rust Performance Optimization Tips">
        <meta property="og:description" content="Expert tips for making your Rust code faster.">
        <meta property="og:image" content="https://example.com/images/rust-performance.jpg">
        <meta property="og:url" content="https://example.com/rust-performance-tips">
        <meta property="og:type" content="article">
        <meta name="twitter:card" content="summary_large_image">
        <meta name="twitter:creator" content="@alicedeveloper">
        <meta name="twitter:title" content="10 Rust Performance Optimization Tips">
        <meta name="twitter:image" content="https://example.com/images/rust-performance.jpg">
      </head>
      <body>
        <h1>10 Rust Performance Optimization Tips</h1>
        <p>Written by Alice Johnson • Published 2025-01-15</p>

        <h2>Introduction</h2>
        <p>Rust is already fast, but there are techniques to make it even faster. In this guide, we'll explore 10 practical tips for optimizing Rust code in production environments.</p>

        <h2>1. Use Release Mode for Benchmarks</h2>
        <p>Always compile with <code>--release</code> when measuring performance. Debug builds are much slower due to lack of optimizations.</p>

        <h2>2. Profile Your Code</h2>
        <p>Use tools like <code>cargo-flamegraph</code> and <code>perf</code> to identify bottlenecks. Don't guess where time is spent.</p>

        <h2>3. Reduce Allocations</h2>
        <p>Heap allocations are expensive. Use stack-allocated types (<code>Vec::with_capacity</code>, <code>String::with_capacity</code>) when you know the size upfront.</p>

        <h2>External Resources</h2>
        <p>Learn more at <a href="https://docs.rust-embedded.org/book/">The Embedded Rust Book</a> and <a href="https://doc.rust-lang.org/book/">The Rust Book</a>.</p>

        <h2>Author Links</h2>
        <p>Find me on <a href="https://twitter.com/alicedeveloper">Twitter</a>, <a href="https://github.com/alicedeveloper">GitHub</a>, or <a href="mailto:alice@example.com">email me</a>.</p>

        <img src="https://example.com/images/rust-logo.png" alt="Rust programming language logo" width="200" height="200">
      </body>
    </html>
    """

    seo = extract_seo_metadata(html)

    # Document metadata

    # Open Graph metadata
    if seo["open_graph"]:
        for _key, _value in seo["open_graph"].items():
            pass
    else:
        pass

    # Twitter Card metadata
    if seo["twitter_card"]:
        for _key, _value in seo["twitter_card"].items():
            pass
    else:
        pass

    # Content analysis

    # Preview of converted markdown
    lines = seo["markdown"].split("\n")
    preview = "\n".join(lines[:15])
    if len(lines) > 15:
        preview += f"\n... ({len(lines) - 15} more lines)"


if __name__ == "__main__":
    main()

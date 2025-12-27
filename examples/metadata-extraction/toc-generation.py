#!/usr/bin/env python3
"""
Table of Contents Generation Example.

Demonstrates how to build a hierarchical table of contents from HTML headers.
Generates anchor-based navigation with proper heading hierarchy detection.
"""

from html_to_markdown import convert_with_metadata


def build_table_of_contents(html: str) -> str:
    """Generate a markdown table of contents from HTML headers."""
    _markdown, metadata = convert_with_metadata(html)
    headers = metadata["headers"]

    if not headers:
        return "# Table of Contents\n\nNo headers found in document."

    lines = ["# Table of Contents\n"]

    for header in headers:
        level = header["level"]

        # Create anchor from ID or from text
        anchor = header.get("id") or header["text"].lower().replace(" ", "-")

        # Indent based on heading level
        indent = "  " * (level - 1)

        # Format list item
        lines.append(f"{indent}- [{header['text']}](#{anchor})")

    return "\n".join(lines)


def analyze_heading_structure(html: str) -> dict:
    """Analyze heading structure for hierarchy issues."""
    _markdown, metadata = convert_with_metadata(html)
    headers = metadata["headers"]

    structure = {
        "total_headers": len(headers),
        "by_level": {},
        "hierarchy_issues": [],
        "headers_without_ids": [],
    }

    # Count headers by level
    for header in headers:
        level = header["level"]
        structure["by_level"][level] = structure["by_level"].get(level, 0) + 1

        # Track headers without explicit IDs
        if not header.get("id"):
            structure["headers_without_ids"].append(header["text"])

    # Check for hierarchy issues
    prev_level = None
    for header in headers:
        level = header["level"]

        # Skip if this is the first header
        if prev_level is None:
            prev_level = level
            continue

        # Warn about skipped levels
        if level > prev_level + 1:
            structure["hierarchy_issues"].append(f"Jumped from H{prev_level} to H{level}: '{header['text']}'")

        prev_level = level

    return structure


def main() -> None:
    html = """
    <html>
      <head>
        <title>Advanced Rust Programming Guide</title>
      </head>
      <body>
        <h1 id="intro">Introduction to Advanced Rust</h1>
        <p>This guide covers advanced Rust concepts and patterns.</p>

        <h2 id="memory">Memory Management</h2>
        <p>Understanding ownership and borrowing is fundamental to Rust.</p>

        <h3 id="ownership">Ownership Rules</h3>
        <p>Each value has a single owner.</p>

        <h3 id="borrowing">Borrowing and References</h3>
        <p>References allow temporary access without transfer.</p>

        <h2 id="concurrency">Concurrency Patterns</h2>
        <p>Rust provides powerful concurrency primitives.</p>

        <h3 id="threads">Thread Safety</h3>
        <p>The type system ensures thread safety at compile time.</p>

        <h3 id="async">Async/Await Programming</h3>
        <p>Non-blocking concurrent code with async/await.</p>

        <h4 id="async-futures">Futures and Tasks</h4>
        <p>Understanding the foundations of async Rust.</p>

        <h2 id="performance">Performance Optimization</h2>
        <p>Practical techniques for optimizing Rust code.</p>

        <h3 id="profiling">Profiling Tools</h3>
        <p>Use cargo-flamegraph and perf for analysis.</p>

        <h3 id="benchmarking">Benchmarking with Criterion</h3>
        <p>Accurate performance measurements using criterion.rs.</p>

        <h1 id="conclusion">Conclusion</h1>
        <p>Mastering these patterns unlocks Rust's full potential.</p>
      </body>
    </html>
    """

    # Generate TOC
    build_table_of_contents(html)

    # Analyze structure
    analysis = analyze_heading_structure(html)
    for level in sorted(analysis["by_level"].keys()):
        analysis["by_level"][level]

    if analysis["hierarchy_issues"]:
        for _issue in analysis["hierarchy_issues"]:
            pass

    if analysis["headers_without_ids"]:
        for text in analysis["headers_without_ids"]:
            text.lower().replace(" ", "-")


if __name__ == "__main__":
    main()

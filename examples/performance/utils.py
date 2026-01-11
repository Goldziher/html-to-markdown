#!/usr/bin/env python3
"""
Shared utilities for performance measurement examples.

This module provides common functions and classes for:
- Fixture data management
- Performance measurement (time, memory, throughput)
- Result formatting and display
"""

from __future__ import annotations

import gc
import sys
import time
import tracemalloc
from typing import Any

from typing_extensions import Self

try:
    from html_to_markdown import convert
except ImportError:
    sys.exit(1)


# Shared test fixtures used across all performance examples
FIXTURES = {
    "small": {
        "name": "Small Document (2 KB)",
        "html": """
        <html>
        <head><title>Small Example</title></head>
        <body>
        <h1>Introduction to Performance</h1>
        <p>This is a small example document for baseline benchmarking.</p>
        <p>It contains minimal HTML structure.</p>
        <ul>
            <li>Point 1</li>
            <li>Point 2</li>
            <li>Point 3</li>
        </ul>
        <p>Perfect for testing cold-start overhead and baseline performance.</p>
        </body>
        </html>
        """,
    },
    "medium": {
        "name": "Medium Document (25 KB equivalent)",
        "html": """
        <html>
        <head><title>Blog Post: Web Performance Techniques</title></head>
        <body>
        <h1>Web Performance Techniques</h1>
        <p>This document represents a typical blog post with moderate complexity.</p>

        <h2>Section 1: Caching Strategies</h2>
        <p>Effective caching is crucial for web performance. There are several strategies:</p>
        <ul>
            <li><strong>Browser Caching</strong> - Store static assets locally</li>
            <li><strong>Server-side Caching</strong> - Cache computed results</li>
            <li><strong>CDN Caching</strong> - Distribute content globally</li>
        </ul>

        <h2>Section 2: Database Optimization</h2>
        <p>Database queries often become the bottleneck. Key techniques include:</p>
        <ol>
            <li>Index frequently queried columns</li>
            <li>Use query optimization tools</li>
            <li>Implement connection pooling</li>
            <li>Cache query results</li>
        </ol>

        <h3>2.1 Indexes</h3>
        <p>Database indexes speed up <code>SELECT</code> queries and <code>WHERE</code> clauses.</p>

        <h3>2.2 Query Analysis</h3>
        <p>Use <code>EXPLAIN</code> to analyze query execution plans:</p>
        <pre><code class="language-sql">EXPLAIN SELECT * FROM users WHERE age > 25;</code></pre>

        <h2>Section 3: Frontend Optimization</h2>
        <p>The frontend contributes significantly to page load time:</p>
        <blockquote>
            <p>"In general, the backend does not dominate the page load time. Only 5-20% of
            the end user response time is spent on the backend server. The other 80-95% is
            spent on the frontend downloading the components of the page: images, stylesheets,
            JavaScripts, Flash, etc."</p>
        </blockquote>

        <h2>Section 4: Tools and Monitoring</h2>
        <p>Use these tools to measure and monitor performance:</p>
        <table>
            <tr>
                <th>Tool</th>
                <th>Purpose</th>
                <th>Type</th>
            </tr>
            <tr>
                <td>Chrome DevTools</td>
                <td>Browser profiling</td>
                <td>Frontend</td>
            </tr>
            <tr>
                <td>Lighthouse</td>
                <td>Web vitals analysis</td>
                <td>Frontend</td>
            </tr>
            <tr>
                <td>Apache JMeter</td>
                <td>Load testing</td>
                <td>Backend</td>
            </tr>
        </table>

        <h2>Conclusion</h2>
        <p>Web performance optimization requires a holistic approach covering:</p>
        <ul>
            <li>Backend efficiency</li>
            <li>Database optimization</li>
            <li>Frontend delivery</li>
            <li>Continuous monitoring</li>
        </ul>

        <p>For more information, visit <a href="https://web.dev/performance">web.dev/performance</a>.</p>
        </body>
        </html>
        """
        * 3,  # Multiply to reach ~25 KB
    },
    "large": {
        "name": "Large Document (150 KB equivalent)",
        "html": """
        <html>
        <head><title>Comprehensive Wikipedia-style Article</title></head>
        <body>
        <h1>History of Computing</h1>
        <p>This is a comprehensive article representing a large Wikipedia-style document
        with extensive content, multiple sections, tables, and complex nesting.</p>

        <h2>Ancient Period (3000 BCE - 1450 CE)</h2>
        <p>The history of computing spans thousands of years. Early humans used basic
        mechanical aids for arithmetic, including the abacus.</p>

        <h3>3.0 - 1800s</h3>
        <p>Key developments in this era:</p>
        <ul>
            <li>Abacus (3000 BCE)</li>
            <li>Astrolabe (c. 150 BCE)</li>
            <li>Antikythera mechanism (100 BCE)</li>
        </ul>

        <h3>1800s - Early 1900s</h3>
        <p>Mechanical computation machines emerged:</p>
        <ol>
            <li>Jacquard loom (1804)</li>
            <li>Babbage's Analytical Engine (1837)</li>
            <li>Tabulating machine by Hollerith (1890)</li>
        </ol>

        <h2>Electronic Era (1930s - 1950s)</h2>
        <p>The development of electronic computers represented a major shift.</p>

        <table>
            <tr>
                <th>Computer</th>
                <th>Year</th>
                <th>Technology</th>
                <th>Notable Features</th>
            </tr>
            <tr>
                <td>ENIAC</td>
                <td>1946</td>
                <td>Vacuum tubes</td>
                <td>30 tons, 18,000 tubes</td>
            </tr>
            <tr>
                <td>EDVAC</td>
                <td>1949</td>
                <td>Vacuum tubes</td>
                <td>Stored-program concept</td>
            </tr>
            <tr>
                <td>UNIVAC I</td>
                <td>1951</td>
                <td>Vacuum tubes</td>
                <td>Commercial computer</td>
            </tr>
            <tr>
                <td>IBM 701</td>
                <td>1952</td>
                <td>Vacuum tubes</td>
                <td>Scientific applications</td>
            </tr>
        </table>

        <h2>Second Generation (1955 - 1965)</h2>
        <p>Transistors replaced vacuum tubes, enabling smaller and more reliable machines.</p>

        <blockquote>
            <p>The transistor was invented at Bell Labs in 1947, and by the mid-1950s,
            transistorized computers were becoming common in research institutions.</p>
        </blockquote>

        <h3>Key Computers</h3>
        <ul>
            <li><strong>IBM 7090</strong> (1959) - Transistorized version of IBM 709</li>
            <li><strong>PDP-1</strong> (1960) - First minicomputer</li>
            <li><strong>IBM System/360</strong> (1964) - Influential mainframe</li>
            <li><strong>DEC PDP-8</strong> (1965) - First widely-used minicomputer</li>
        </ul>

        <h2>Third Generation (1965 - 1975)</h2>
        <p>Integrated circuits enabled even greater miniaturization.</p>

        <p>This era saw the development of:</p>
        <ol>
            <li>Operating systems (UNIX, DOS)</li>
            <li>Database management systems</li>
            <li>High-level programming languages (COBOL, FORTRAN, ALGOL, PL/1)</li>
            <li>Computer networks</li>
        </ol>

        <h2>Fourth Generation (1975 - Present)</h2>
        <p>Microprocessors and personal computers revolutionized computing.</p>

        <h3>Microprocessor Evolution</h3>
        <table>
            <tr>
                <th>Year</th>
                <th>Processor</th>
                <th>Company</th>
                <th>Transistors</th>
            </tr>
            <tr>
                <td>1971</td>
                <td>4004</td>
                <td>Intel</td>
                <td>2,250</td>
            </tr>
            <tr>
                <td>1978</td>
                <td>8086</td>
                <td>Intel</td>
                <td>29,000</td>
            </tr>
            <tr>
                <td>1985</td>
                <td>80386</td>
                <td>Intel</td>
                <td>275,000</td>
            </tr>
            <tr>
                <td>2000</td>
                <td>Pentium 4</td>
                <td>Intel</td>
                <td>42,000,000</td>
            </tr>
            <tr>
                <td>2020</td>
                <td>M1</td>
                <td>Apple</td>
                <td>16,000,000,000</td>
            </tr>
        </table>

        <h3>Personal Computers</h3>
        <ul>
            <li>Apple II (1977) - Color graphics, expansion slots</li>
            <li>IBM PC (1981) - Open architecture</li>
            <li>Commodore 64 (1982) - Best-selling computer ever</li>
            <li>Macintosh (1984) - Graphical user interface</li>
        </ul>

        <h2>Modern Era (1990s - Present)</h2>
        <p>The exponential growth of computing power continues to accelerate.</p>

        <h3>Key Developments</h3>
        <ul>
            <li><strong>1989</strong> - World Wide Web invented by Tim Berners-Lee</li>
            <li><strong>1991</strong> - Linux kernel released</li>
            <li><strong>1995</strong> - Java, JavaScript, and Windows 95 released</li>
            <li><strong>2001</strong> - Wikipedia launched</li>
            <li><strong>2008</strong> - Git version control system released</li>
            <li><strong>2010s</strong> - Rise of cloud computing and mobile devices</li>
            <li><strong>2020s</strong> - AI and machine learning become mainstream</li>
        </ul>

        <h3>Quantum Computing</h3>
        <p>Quantum computers represent the next frontier, using quantum bits (qubits)
        instead of classical bits. Key milestones:</p>
        <ol>
            <li>1985 - David Deutsch describes the quantum computer</li>
            <li>1994 - Shor's algorithm for factoring</li>
            <li>1996 - Grover's search algorithm</li>
            <li>2001 - IBM demonstrates Shor's algorithm</li>
            <li>2019 - Google claims quantum supremacy</li>
        </ol>

        <h2>Performance Evolution</h2>
        <p>Computing power has grown exponentially, following Moore's Law:</p>

        <blockquote>
            <p>The number of transistors in an integrated circuit doubles approximately
            every two years. This observation, formulated by Gordon Moore in 1965, has held
            true for decades and remains a key driver of technological progress.</p>
        </blockquote>

        <p>This exponential growth has enabled:</p>
        <ul>
            <li>Scientific simulation</li>
            <li>Artificial intelligence</li>
            <li>Real-time graphics</li>
            <li>Big data analysis</li>
            <li>Machine learning</li>
        </ul>

        <h2>See Also</h2>
        <ul>
            <li><a href="/wiki/Computer_architecture">Computer architecture</a></li>
            <li><a href="/wiki/History_of_programming_languages">History of programming languages</a></li>
            <li><a href="/wiki/Computer_networking">Computer networking</a></li>
            <li><a href="/wiki/Software_engineering">Software engineering</a></li>
        </ul>

        <h2>References</h2>
        <ol>
            <li><a href="https://example.com/ref1">Computer History Museum</a></li>
            <li><a href="https://example.com/ref2">IEEE Computer Society</a></li>
            <li><a href="https://example.com/ref3">ACM Digital Library</a></li>
        </ol>
        </body>
        </html>
        """
        * 3,  # Multiply to reach ~150 KB
    },
}


def format_number(value: float, precision: int = 2) -> str:
    """
    Format a number with thousands separator.

    Args:
        value: Number to format
        precision: Decimal places (default: 2)

    Returns:
        Formatted number string
    """
    return f"{value:,.{precision}f}"


def format_size(size_bytes: int) -> str:
    """
    Format byte size to human-readable format.

    Args:
        size_bytes: Size in bytes

    Returns:
        Human-readable size string
    """
    for unit in ["B", "KB", "MB", "GB"]:
        if size_bytes < 1024:
            return f"{size_bytes:.2f} {unit}"
        size_bytes /= 1024
    return f"{size_bytes:.2f} TB"


def format_duration(seconds: float) -> str:
    """
    Format duration to human-readable format.

    Args:
        seconds: Duration in seconds

    Returns:
        Human-readable duration string
    """
    if seconds < 0.001:
        return f"{seconds * 1_000_000:.2f} μs"
    if seconds < 1:
        return f"{seconds * 1000:.2f} ms"
    if seconds < 60:
        return f"{seconds:.2f} s"
    minutes = seconds / 60
    return f"{minutes:.2f} min"


class TimingMeasurement:
    """Context manager for measuring execution time."""

    def __init__(self, name: str = "Timing") -> None:
        """
        Initialize timing measurement.

        Args:
            name: Operation name for logging
        """
        self.name = name
        self.elapsed = 0.0
        self._start_time = 0.0

    def __enter__(self) -> Self:
        self._start_time = time.perf_counter()
        return self

    def __exit__(self, *args: object) -> None:
        self.elapsed = time.perf_counter() - self._start_time

    @property
    def elapsed_ms(self) -> float:
        """Elapsed time in milliseconds."""
        return self.elapsed * 1000

    @property
    def elapsed_sec(self) -> float:
        """Elapsed time in seconds."""
        return self.elapsed


class MemoryTracker:
    """Context manager for tracking memory usage during operations."""

    def __init__(self, operation_name: str = "Operation") -> None:
        """
        Initialize memory tracker.

        Args:
            operation_name: Operation name for logging
        """
        self.operation_name = operation_name
        self.baseline_memory = 0
        self.peak_memory = 0

    def __enter__(self) -> Self:
        gc.collect()
        tracemalloc.start()
        self.baseline_memory = tracemalloc.get_traced_memory()[0]
        return self

    def __exit__(self, *args: object) -> None:
        _current, peak = tracemalloc.get_traced_memory()
        self.peak_memory = peak
        tracemalloc.stop()

    @property
    def memory_used_kb(self) -> float:
        """Memory used during operation in KB."""
        return self.peak_memory / 1024

    @property
    def memory_used_mb(self) -> float:
        """Memory used during operation in MB."""
        return self.peak_memory / (1024 * 1024)

    @property
    def memory_used_bytes(self) -> int:
        """Memory used during operation in bytes."""
        return self.peak_memory


def measure_conversion_timing(
    html: str,
    fixture_name: str,
    fixture_size: str,
    iterations: int = 1,
) -> dict[str, Any]:
    """
    Measure timing metrics for HTML conversion.

    Args:
        html: HTML content to convert
        fixture_name: Display name of fixture
        fixture_size: Size category (small/medium/large)
        iterations: Number of iterations

    Returns:
        Dictionary with timing metrics
    """
    html_bytes = len(html.encode("utf-8"))

    # Warmup run
    convert(html)

    # Measure conversions
    with TimingMeasurement() as timer:
        for _ in range(iterations):
            convert(html)

    # Calculate metrics
    avg_time_ms = (timer.elapsed / iterations) * 1000
    throughput_docs_sec = iterations / timer.elapsed
    bytes_processed = html_bytes * iterations
    bandwidth_mb_sec = (bytes_processed / (1024 * 1024)) / timer.elapsed

    return {
        "fixture": fixture_name,
        "size_category": fixture_size,
        "html_size_bytes": html_bytes,
        "iterations": iterations,
        "total_time_sec": timer.elapsed,
        "avg_time_ms": avg_time_ms,
        "throughput_docs_sec": throughput_docs_sec,
        "bandwidth_mb_sec": bandwidth_mb_sec,
    }


def print_section_header(title: str, width: int = 80) -> None:
    """
    Print a formatted section header.

    Args:
        title: Section title
        width: Header width (default: 80)
    """
    print("\n" + "=" * width)
    print(f"  {title}")
    print("=" * width)


def print_result_row(
    label: str,
    value: str | float,
    format_spec: str = "",
) -> None:
    """
    Print a formatted result row.

    Args:
        label: Row label
        value: Value to print
        format_spec: Optional format specification
    """
    value_str = format(value, format_spec) if isinstance(value, float) and format_spec else str(value)
    print(f"  {label:<40} {value_str:>30}")

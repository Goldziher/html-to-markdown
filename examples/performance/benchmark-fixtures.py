#!/usr/bin/env python3
"""
Benchmark html-to-markdown conversion on different fixture sizes.

Demonstrates performance characteristics across small, medium, and large documents.
Measures latency (ms), throughput (docs/sec), and bandwidth (MB/s).

Usage:
    python benchmark-fixtures.py --size small
    python benchmark-fixtures.py --size medium
    python benchmark-fixtures.py --size large
    python benchmark-fixtures.py --all
"""

from __future__ import annotations

import argparse
import sys
import time

try:
    from html_to_markdown import ConversionOptions, convert, create_options_handle
except ImportError:
    sys.exit(1)


# Test fixtures
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
    """Format a number with thousands separator."""
    return f"{value:,.{precision}f}"


def run_benchmark(
    html: str,
    fixture_name: str,
    fixture_size: str,
    iterations: int = 50,
) -> dict[str, float]:
    """
    Run benchmark on a single fixture.

    Args:
        html: HTML content to convert
        fixture_name: Display name of fixture
        fixture_size: Size category (small/medium/large)
        iterations: Number of iterations to run

    Returns:
        Dictionary with benchmark results
    """
    html_bytes = len(html.encode("utf-8"))

    # Warmup run (not counted)
    convert(html)

    # Measure conversions
    start = time.perf_counter()
    for _ in range(iterations):
        convert(html)
    elapsed = time.perf_counter() - start

    # Calculate metrics
    avg_time_ms = (elapsed / iterations) * 1000
    throughput_docs_sec = iterations / elapsed
    bytes_processed = html_bytes * iterations
    bandwidth_mb_sec = (bytes_processed / (1024 * 1024)) / elapsed

    return {
        "fixture": fixture_name,
        "size_category": fixture_size,
        "html_size_bytes": html_bytes,
        "iterations": iterations,
        "total_time_sec": elapsed,
        "avg_time_ms": avg_time_ms,
        "throughput_docs_sec": throughput_docs_sec,
        "bandwidth_mb_sec": bandwidth_mb_sec,
    }


def run_with_options_benchmark(
    html: str,
    fixture_name: str,
    fixture_size: str,
    iterations: int = 50,
) -> dict[str, float]:
    """
    Benchmark with ConversionOptions (shows overhead of option handling).

    Args:
        html: HTML content to convert
        fixture_name: Display name of fixture
        fixture_size: Size category (small/medium/large)
        iterations: Number of iterations to run

    Returns:
        Dictionary with benchmark results
    """
    html_bytes = len(html.encode("utf-8"))

    # Create options handle (recommended approach for repeated conversions)
    options = ConversionOptions(sanitize=True)
    handle = create_options_handle(options)

    # Warmup run
    from html_to_markdown import convert_with_handle

    convert_with_handle(html, handle)

    # Measure conversions
    start = time.perf_counter()
    for _ in range(iterations):
        convert_with_handle(html, handle)
    elapsed = time.perf_counter() - start

    # Calculate metrics
    avg_time_ms = (elapsed / iterations) * 1000
    throughput_docs_sec = iterations / elapsed
    bytes_processed = html_bytes * iterations
    bandwidth_mb_sec = (bytes_processed / (1024 * 1024)) / elapsed

    return {
        "fixture": fixture_name,
        "size_category": fixture_size,
        "scenario": "with_options",
        "html_size_bytes": html_bytes,
        "iterations": iterations,
        "total_time_sec": elapsed,
        "avg_time_ms": avg_time_ms,
        "throughput_docs_sec": throughput_docs_sec,
        "bandwidth_mb_sec": bandwidth_mb_sec,
    }


def print_results(results: list[dict[str, float]]) -> None:
    """Print benchmark results in formatted tables."""
    # Group by scenario
    default_results = [r for r in results if "scenario" not in r]
    options_results = [r for r in results if r.get("scenario") == "with_options"]

    if default_results:
        for _result in default_results:
            pass

    if options_results:
        for _result in options_results:
            pass

        for default, options in zip(default_results, options_results, strict=False):
            ((options["avg_time_ms"] - default["avg_time_ms"]) / default["avg_time_ms"]) * 100

    for category in ["small", "medium", "large"]:
        cat_results = [r for r in default_results if r["size_category"] == category]
        if cat_results:
            cat_results[0]


def main() -> None:
    """Main benchmark runner."""
    parser = argparse.ArgumentParser(description="Benchmark html-to-markdown conversion performance")
    parser.add_argument(
        "--size",
        choices=["small", "medium", "large"],
        help="Fixture size to benchmark (default: all)",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Run all benchmarks (default behavior)",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=50,
        help="Number of iterations per fixture (default: 50)",
    )
    parser.add_argument(
        "--with-options",
        action="store_true",
        help="Also benchmark with ConversionOptions",
    )

    args = parser.parse_args()

    # Determine which fixtures to run
    sizes_to_run = []
    sizes_to_run = [args.size] if args.size else list(FIXTURES.keys())

    # Run benchmarks
    results = []
    for size in sizes_to_run:
        fixture = FIXTURES[size]

        result = run_benchmark(
            fixture["html"],
            fixture["name"],
            size,
            iterations=args.iterations,
        )
        results.append(result)

        # Run with options if requested
        if args.with_options:
            result_opts = run_with_options_benchmark(
                fixture["html"],
                fixture["name"],
                size,
                iterations=args.iterations,
            )
            results.append(result_opts)

    # Print results
    print_results(results)


if __name__ == "__main__":
    main()

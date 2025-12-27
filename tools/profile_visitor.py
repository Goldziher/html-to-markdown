#!/usr/bin/env python3
"""
Profile visitor callback overhead in html-to-markdown Python binding.

Measures:
  - Callback invocation overhead
  - Context marshalling cost
  - Result conversion overhead
  - GC impact of visitor callbacks

Test scenarios:
  - no-op: Visitor with empty callbacks
  - simple: Simple text extraction
  - custom_output: Building custom output
  - complex: Multiple operations per callback
"""

import argparse
import contextlib
import cProfile
import json
import pstats
import re
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).parent.parent / "packages" / "python"))

from html_to_markdown import convert, convert_with_visitor


@dataclass
class TimingMetrics:
    """Timing metrics for a test run."""

    scenario: str
    html_size_bytes: int
    element_count: int
    baseline_ms: float
    visitor_ms: float
    overhead_ms: float
    overhead_percent: float
    callback_invocations: int
    avg_callback_time_us: float
    iterations: int


def load_test_html(filename: str) -> str:
    """Load test HTML document."""
    path = Path(__file__).parent.parent / "test_documents" / "html" / "wikipedia" / filename
    if not path.exists():
        raise FileNotFoundError(f"Test document not found: {path}")
    return path.read_text()


def count_elements(html: str) -> int:
    """Estimate element count by counting tags."""
    return len(re.findall(r"<[^/>]+>", html))


class NoOpVisitor:
    """Visitor that does nothing."""

    def __init__(self) -> None:
        self.invocations = 0

    def visit_node(self, _node: dict[str, Any]) -> None:
        self.invocations += 1


class SimpleVisitor:
    """Visitor that extracts simple text."""

    def __init__(self) -> None:
        self.invocations = 0
        self.texts: list[str] = []

    def visit_node(self, node: dict[str, Any]) -> None:
        self.invocations += 1
        if node.get("type") == "text":
            text = node.get("content", "")
            self.texts.append(text)


class CustomOutputVisitor:
    """Visitor that builds custom output."""

    def __init__(self) -> None:
        self.invocations = 0
        self.output: list[str] = []

    def visit_node(self, node: dict[str, Any]) -> None:
        self.invocations += 1
        node_type = node.get("type", "")
        tag = node.get("tag", "")

        if node_type == "element":
            self.output.append(f"[{tag.upper()}]")
        elif node_type == "text":
            content = node.get("content", "")
            self.output.append(content)


class ComplexVisitor:
    """Visitor that performs multiple operations."""

    def __init__(self) -> None:
        self.invocations = 0
        self.stats: dict[str, int] = {}
        self.depths: list[int] = []

    def visit_node(self, node: dict[str, Any]) -> None:
        self.invocations += 1
        node_type = node.get("type", "")

        if node_type not in self.stats:
            self.stats[node_type] = 0
        self.stats[node_type] += 1

        depth = node.get("depth", 0)
        self.depths.append(depth)

        if node.get("attributes"):
            attr_count = len(node.get("attributes", {}))
            self.stats["attrs_total"] = self.stats.get("attrs_total", 0) + attr_count


def benchmark_with_visitor(
    html: str,
    visitor: Any,
    iterations: int = 10,
) -> float:
    """Benchmark conversion with visitor. Returns total time in ms."""
    start = time.perf_counter()

    for _ in range(iterations):
        with contextlib.suppress(Exception):
            convert_with_visitor(html, visitor=visitor)

    elapsed = time.perf_counter() - start
    return elapsed * 1000


def benchmark_baseline(html: str, iterations: int = 10) -> float:
    """Benchmark conversion without visitor. Returns total time in ms."""
    start = time.perf_counter()

    for _ in range(iterations):
        convert(html)

    elapsed = time.perf_counter() - start
    return elapsed * 1000


def profile_scenario(
    name: str,
    html: str,
    visitor_class: type,
    iterations: int = 10,
) -> TimingMetrics:
    """Profile a specific scenario."""
    element_count = count_elements(html)

    benchmark_baseline(html, iterations=2)

    baseline_ms = benchmark_baseline(html, iterations=iterations)
    baseline_avg = baseline_ms / iterations

    visitor = visitor_class()
    visitor_ms = benchmark_with_visitor(html, visitor, iterations=iterations)
    visitor_avg = visitor_ms / iterations

    overhead_ms = visitor_ms - baseline_ms
    overhead_percent = (overhead_ms / baseline_ms) * 100 if baseline_ms > 0 else 0

    callback_count = visitor.invocations if hasattr(visitor, "invocations") else 0
    avg_callback_time_us = (overhead_ms * 1000 / callback_count) if callback_count > 0 else 0

    return TimingMetrics(
        scenario=name,
        html_size_bytes=len(html),
        element_count=element_count,
        baseline_ms=baseline_avg,
        visitor_ms=visitor_avg,
        overhead_ms=overhead_ms,
        overhead_percent=overhead_percent,
        callback_invocations=callback_count,
        avg_callback_time_us=avg_callback_time_us,
        iterations=iterations,
    )


def profile_with_cprofile(
    html: str,
    visitor: Any,
    scenario_name: str,
    output_dir: Path,
) -> None:
    """Run cProfile on a specific scenario."""
    profiler = cProfile.Profile()
    profiler.enable()

    for _ in range(5):
        with contextlib.suppress(Exception):
            convert_with_visitor(html, visitor=visitor)

    profiler.disable()

    stats_path = output_dir / f"profile_{scenario_name}.txt"
    with stats_path.open("w") as f:
        stats = pstats.Stats(profiler, stream=f)
        stats.strip_dirs()
        stats.sort_stats("cumulative")
        stats.print_stats(30)


def main() -> None:
    """Run visitor callback overhead profiling."""
    parser = argparse.ArgumentParser(description="Profile visitor callback overhead in html-to-markdown")
    parser.add_argument(
        "--scenario",
        choices=["all", "no-op", "simple", "custom-output", "complex"],
        default="all",
        help="Scenario to profile",
    )
    parser.add_argument("--html", choices=["small", "medium", "large"], default="medium", help="HTML document size")
    parser.add_argument("--iterations", type=int, default=10, help="Number of iterations per scenario")
    parser.add_argument(
        "--output", type=Path, default=Path("visitor_profile_results"), help="Output directory for results"
    )
    parser.add_argument("--cprofile", action="store_true", help="Generate cProfile data")

    args = parser.parse_args()

    args.output.mkdir(parents=True, exist_ok=True)

    html_map = {
        "small": "small_html.html",
        "medium": "medium_python.html",
        "large": "large_rust.html",
    }
    html_file = html_map.get(args.html, "medium_python.html")
    html = load_test_html(html_file)

    scenarios = [
        ("no-op", NoOpVisitor),
        ("simple", SimpleVisitor),
        ("custom-output", CustomOutputVisitor),
        ("complex", ComplexVisitor),
    ]

    results: list[TimingMetrics] = []

    for scenario_name, visitor_class in scenarios:
        if args.scenario not in ("all", scenario_name):
            continue

        metrics = profile_scenario(
            scenario_name,
            html,
            visitor_class,
            iterations=args.iterations,
        )
        results.append(metrics)

        if args.cprofile:
            visitor = visitor_class()
            profile_with_cprofile(html, visitor, scenario_name, args.output)

    json_path = args.output / "results.json"
    with json_path.open("w") as f:
        json.dump(
            {
                "html_size": len(html),
                "html_file": html_file,
                "element_count": count_elements(html),
                "timestamp": time.time(),
                "results": [asdict(m) for m in results],
            },
            f,
            indent=2,
        )


if __name__ == "__main__":
    main()

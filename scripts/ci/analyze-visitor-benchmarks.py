#!/usr/bin/env python3
"""
Analyze visitor pattern benchmark results for performance regressions.

This script compares benchmark results across different visitor configurations
and detects performance regressions based on configurable overhead thresholds.

Usage:
    python3 scripts/ci/analyze-visitor-benchmarks.py \
        --baseline results/visitor-baseline/results.json \
        --callbacks results/visitor-callbacks/results.json \
        --custom results/visitor-custom/results.json \
        --complex results/visitor-complex/results.json \
        --framework python \
        --thresholds '{"baseline": 10, "callbacks": 30, "complex": 60}'
"""

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass
class BenchmarkResult:
    """Represents a single benchmark result."""

    fixture_id: str
    framework: str
    mean_ns: float
    min_ns: float
    max_ns: float
    stddev_ns: float
    iterations: int


def load_results(path: str) -> dict[str, Any]:
    """Load benchmark results from a JSON file."""
    try:
        with Path(path).open() as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError) as e:
        print(f"Error loading results from {path}: {e}", file=sys.stderr)
        return {}


def extract_benchmark_data(results: dict[str, Any] | list[Any], framework: str) -> dict[str, BenchmarkResult]:
    """Extract benchmark data for a specific framework."""
    data = {}

    # Handle direct array format from benchmark harness
    if isinstance(results, list):
        benchmark_list = results
    # Handle nested "benchmarks" key format
    elif isinstance(results, dict) and "benchmarks" in results:
        benchmark_list = results.get("benchmarks", [])
    else:
        return data

    for benchmark in benchmark_list:
        if benchmark.get("framework") != framework:
            continue

        fixture_id = benchmark.get("fixture_id", "unknown")
        metrics = benchmark.get("metrics", {})

        if not metrics:
            continue

        result = BenchmarkResult(
            fixture_id=fixture_id,
            framework=framework,
            mean_ns=metrics.get("mean", 0),
            min_ns=metrics.get("min", 0),
            max_ns=metrics.get("max", 0),
            stddev_ns=metrics.get("stddev", 0),
            iterations=benchmark.get("iterations", 0),
        )
        data[fixture_id] = result

    return data


def calculate_overhead(baseline_mean: float, comparison_mean: float) -> float:
    """Calculate overhead percentage between two measurements."""
    if baseline_mean == 0:
        return 0.0
    return ((comparison_mean - baseline_mean) / baseline_mean) * 100


def check_regressions(
    baseline_data: dict[str, BenchmarkResult],
    comparison_data: dict[str, BenchmarkResult],
    threshold_percent: float,
    category: str,
) -> tuple[bool, list[str]]:
    """
    Check for performance regressions.

    Returns:
        Tuple of (passed, messages) where passed is True if no regressions found
    """
    regressions = []

    for fixture_id, baseline in baseline_data.items():
        if fixture_id not in comparison_data:
            continue

        comparison = comparison_data[fixture_id]
        overhead = calculate_overhead(baseline.mean_ns, comparison.mean_ns)

        if overhead > threshold_percent:
            regressions.append(
                f"REGRESSION: {category} - {fixture_id}: {overhead:.1f}% overhead (threshold: {threshold_percent}%)"
            )

    return len(regressions) == 0, regressions


def print_summary(
    baseline_data: dict[str, BenchmarkResult],
    comparison_data: dict[str, BenchmarkResult],
    category: str,
    threshold_percent: float,
) -> None:
    """Print a summary of benchmark results."""
    print(f"\n{'=' * 70}")
    print(f"Visitor Benchmark Analysis: {category.upper()}")
    print(f"Threshold: {threshold_percent}% overhead")
    print(f"{'=' * 70}")

    for fixture_id, baseline in baseline_data.items():
        if fixture_id not in comparison_data:
            print(f"⚠️  {fixture_id}: Missing comparison data")
            continue

        comparison = comparison_data[fixture_id]
        overhead = calculate_overhead(baseline.mean_ns, comparison.mean_ns)

        status = "✓" if overhead <= threshold_percent else "✗"
        baseline_ms = baseline.mean_ns / 1_000_000
        comparison_ms = comparison.mean_ns / 1_000_000

        print(f"{status} {fixture_id}:")
        print(f"    Baseline:   {baseline_ms:.2f}ms")
        print(f"    Comparison: {comparison_ms:.2f}ms")
        print(f"    Overhead:   {overhead:+.1f}% {'(OK)' if overhead <= threshold_percent else '(REGRESSION)'}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Analyze visitor benchmark results for performance regressions")
    parser.add_argument("--baseline", required=True, help="Path to baseline results JSON")
    parser.add_argument("--callbacks", required=True, help="Path to callbacks results JSON")
    parser.add_argument("--custom", required=True, help="Path to custom results JSON")
    parser.add_argument("--complex", required=True, help="Path to complex results JSON")
    parser.add_argument("--framework", required=True, help="Framework name (python, ruby, typescript)")
    parser.add_argument(
        "--thresholds",
        default='{"baseline": 10, "callbacks": 30, "complex": 60}',
        help="JSON string with overhead thresholds per category",
    )

    args = parser.parse_args()

    try:
        thresholds = json.loads(args.thresholds)
    except json.JSONDecodeError:
        print("Error: Invalid JSON in --thresholds argument", file=sys.stderr)
        return 1

    baseline_results = load_results(args.baseline)
    callbacks_results = load_results(args.callbacks)
    custom_results = load_results(args.custom)
    complex_results = load_results(args.complex)

    framework = args.framework

    baseline_data = extract_benchmark_data(baseline_results, framework)
    callbacks_data = extract_benchmark_data(callbacks_results, framework)
    custom_data = extract_benchmark_data(custom_results, framework)
    complex_data = extract_benchmark_data(complex_results, framework)

    if not baseline_data:
        print(f"Error: No benchmark data found for framework '{framework}'", file=sys.stderr)
        return 1

    all_passed = True
    messages = []

    passed, regressions = check_regressions(
        baseline_data,
        baseline_data,
        thresholds.get("baseline", 10),
        "baseline",
    )
    print_summary(baseline_data, baseline_data, "no-op visitor (baseline)", thresholds.get("baseline", 10))

    passed, regressions = check_regressions(
        baseline_data, callbacks_data, thresholds.get("callbacks", 30), "simple callbacks"
    )
    all_passed = all_passed and passed
    messages.extend(regressions)
    print_summary(baseline_data, callbacks_data, "simple callbacks", thresholds.get("callbacks", 30))

    passed, regressions = check_regressions(
        baseline_data, custom_data, thresholds.get("custom", 40), "custom transformations"
    )
    all_passed = all_passed and passed
    messages.extend(regressions)
    print_summary(baseline_data, custom_data, "custom transforms", thresholds.get("custom", 40))

    passed, regressions = check_regressions(
        baseline_data, complex_data, thresholds.get("complex", 60), "complex visitors"
    )
    all_passed = all_passed and passed
    messages.extend(regressions)
    print_summary(baseline_data, complex_data, "complex visitors", thresholds.get("complex", 60))

    print(f"\n{'=' * 70}")
    if all_passed:
        print("✓ All visitor benchmarks passed regression checks")
        print(f"{'=' * 70}")
        return 0
    print("✗ Some visitor benchmarks exceeded regression thresholds:")
    for msg in messages:
        print(f"  {msg}")
    print(f"{'=' * 70}")
    return 1


if __name__ == "__main__":
    sys.exit(main())

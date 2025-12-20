#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

from html_to_markdown import (
    ConversionOptions,
    convert_with_handle,
    create_options_handle,
    start_profiling,
    stop_profiling,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Benchmark html-to-markdown Python bindings")
    parser.add_argument("--file", required=True, help="Path to the HTML/HOCR fixture")
    parser.add_argument("--iterations", type=int, default=50, help="Number of iterations")
    parser.add_argument(
        "--format",
        choices=("html", "hocr"),
        default="html",
        help="Fixture format to guide option defaults",
    )
    return parser.parse_args()


def build_options(fixture_format: str) -> ConversionOptions | None:
    if fixture_format == "hocr":
        return ConversionOptions(hocr_spatial_tables=False)
    return None


def main() -> None:
    args = parse_args()
    iterations = max(1, args.iterations)
    fixture = Path(args.file)

    if not fixture.exists():
        raise SystemExit(f"Fixture not found: {fixture}")

    html = fixture.read_text(encoding="utf-8")
    options = build_options(args.format)
    options_handle = create_options_handle(options)

    convert_with_handle(html, options_handle)  # Warmup

    profile_output = os.getenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT")
    profile_frequency = os.getenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY")
    if profile_output:
        freq = int(profile_frequency) if profile_frequency and profile_frequency.isdigit() else 1000
        start_profiling(profile_output, freq)

    start = time.perf_counter()
    for _ in range(iterations):
        convert_with_handle(html, options_handle)
    elapsed = time.perf_counter() - start

    if profile_output:
        stop_profiling()

    bytes_processed = len(html.encode("utf-8")) * iterations
    ops_per_sec = iterations / elapsed
    mb_per_sec = (bytes_processed / (1024 * 1024)) / elapsed

    result = {
        "language": "python",
        "fixture": fixture.name,
        "fixture_path": str(fixture),
        "iterations": iterations,
        "elapsed_seconds": elapsed,
        "ops_per_sec": ops_per_sec,
        "mb_per_sec": mb_per_sec,
        "bytes_processed": bytes_processed,
    }

    sys.stdout.write(json.dumps(result) + "\n")


if __name__ == "__main__":
    main()

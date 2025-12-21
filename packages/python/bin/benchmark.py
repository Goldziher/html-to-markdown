#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

from html_to_markdown import (
    ConversionOptions,
    InlineImageConfig,
    MetadataConfig,
    OptionsHandle,
    convert,
    convert_with_handle,
    convert_with_inline_images,
    convert_with_inline_images_handle,
    convert_with_metadata,
    convert_with_metadata_handle,
    create_options_handle,
    start_profiling,
    stop_profiling,
)

if TYPE_CHECKING:
    from collections.abc import Callable


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Benchmark html-to-markdown Python bindings")
    parser.add_argument("--file", required=True, help="Path to the HTML/HOCR fixture")
    parser.add_argument("--iterations", type=int, default=50, help="Number of iterations")
    parser.add_argument(
        "--scenario",
        default="convert-default",
        choices=(
            "convert-default",
            "convert-options",
            "inline-images-default",
            "inline-images-options",
            "metadata-default",
            "metadata-options",
        ),
        help="Benchmark scenario to run",
    )
    parser.add_argument(
        "--format",
        choices=("html", "hocr"),
        default="html",
        help="Fixture format to guide option defaults",
    )
    return parser.parse_args()


def build_options(fixture_format: str) -> ConversionOptions:
    if fixture_format == "hocr":
        return ConversionOptions(hocr_spatial_tables=False)
    return ConversionOptions()


def require_handle(handle: OptionsHandle | None, scenario: str) -> OptionsHandle:
    if handle is None:
        raise SystemExit(f"Options handle required for {scenario} scenario")
    return handle


def require_metadata(metadata_config: MetadataConfig | None, scenario: str) -> MetadataConfig:
    if metadata_config is None:
        raise SystemExit(f"Metadata config required for {scenario} scenario")
    return metadata_config


def run_scenario(
    html: str,
    scenario: str,
    handle: OptionsHandle | None,
    metadata_config: MetadataConfig | None,
) -> None:
    handlers: dict[str, Callable[[], object]] = {
        "convert-default": lambda: convert(html),
        "convert-options": lambda: convert_with_handle(html, require_handle(handle, "convert-options")),
        "inline-images-default": lambda: convert_with_inline_images(html, None, None, InlineImageConfig()),
        "inline-images-options": lambda: convert_with_inline_images_handle(
            html, require_handle(handle, "inline-images-options"), InlineImageConfig()
        ),
        "metadata-default": lambda: convert_with_metadata(
            html, None, None, require_metadata(metadata_config, "metadata-default")
        ),
        "metadata-options": lambda: convert_with_metadata_handle(
            html,
            require_handle(handle, "metadata-options"),
            require_metadata(metadata_config, "metadata-options"),
        ),
    }
    handler = handlers.get(scenario)
    if handler is None:
        raise SystemExit(f"Unsupported scenario: {scenario}")
    handler()


def main() -> None:
    args = parse_args()
    iterations = max(1, args.iterations)
    fixture = Path(args.file)

    if not fixture.exists():
        raise SystemExit(f"Fixture not found: {fixture}")

    html = fixture.read_text(encoding="utf-8")
    options = build_options(args.format)
    handle = (
        create_options_handle(options)
        if args.scenario in {"convert-options", "inline-images-options", "metadata-options"}
        else None
    )
    metadata_config = MetadataConfig() if "metadata" in args.scenario else None
    run_scenario(html, args.scenario, handle, metadata_config)  # Warmup

    profile_output = os.getenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT")
    profile_frequency = os.getenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY")
    if profile_output:
        freq = int(profile_frequency) if profile_frequency and profile_frequency.isdigit() else 1000
        start_profiling(profile_output, freq)

    start = time.perf_counter()
    for _ in range(iterations):
        run_scenario(html, args.scenario, handle, metadata_config)
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
        "scenario": args.scenario,
        "iterations": iterations,
        "elapsed_seconds": elapsed,
        "ops_per_sec": ops_per_sec,
        "mb_per_sec": mb_per_sec,
        "bytes_processed": bytes_processed,
    }

    sys.stdout.write(json.dumps(result) + "\n")


if __name__ == "__main__":
    main()

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
    convert_with_visitor,
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
    parser.add_argument(
        "--visitor",
        choices=("noop", "simple", "custom", "complex"),
        default=None,
        help="Visitor type to use during conversion",
    )
    return parser.parse_args()


def build_options(fixture_format: str) -> ConversionOptions:
    if fixture_format == "hocr":
        return ConversionOptions(hocr_spatial_tables=False)
    return ConversionOptions()


class NoopVisitor:
    def visit_text(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_heading(self, _ctx: object, _level: int, _text: str, _element_id: str) -> str:
        return "continue"

    def visit_paragraph(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_link(self, _ctx: object, _href: str, _text: str, _title: str) -> str:
        return "continue"

    def visit_image(self, _ctx: object, _src: str, _alt: str, _title: str) -> str:
        return "continue"

    def visit_strong(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_em(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_code(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_br(self, _ctx: object) -> str:
        return "continue"


def create_noop_visitor() -> object:
    """Create a no-op visitor that returns Continue for all methods."""
    return NoopVisitor()


class SimpleVisitor:
    def __init__(self) -> None:
        self.text_count = 0
        self.link_count = 0
        self.image_count = 0

    def visit_text(self, _ctx: object, _text: str) -> str:
        self.text_count += 1
        return "continue"

    def visit_heading(self, _ctx: object, _level: int, _text: str, _element_id: str) -> str:
        return "continue"

    def visit_paragraph(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_link(self, _ctx: object, _href: str, _text: str, _title: str) -> str:
        self.link_count += 1
        return "continue"

    def visit_image(self, _ctx: object, _src: str, _alt: str, _title: str) -> str:
        self.image_count += 1
        return "continue"

    def visit_strong(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_em(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_code(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_br(self, _ctx: object) -> str:
        return "continue"


def create_simple_visitor() -> object:
    """Create a simple visitor with basic callbacks."""
    return SimpleVisitor()


class CustomVisitor:
    def visit_text(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_heading(self, _ctx: object, _level: int, _text: str, _element_id: str) -> str:
        return "continue"

    def visit_paragraph(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_link(self, _ctx: object, _href: str, text: str, _title: str) -> tuple[str, str]:
        return ("custom", f"LINK[{text}]({_href})")

    def visit_image(self, _ctx: object, src: str, alt: str, _title: str) -> tuple[str, str]:
        return ("custom", f"![{alt}]({src})")

    def visit_strong(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_em(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_code(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_br(self, _ctx: object) -> str:
        return "continue"


def create_custom_visitor() -> object:
    """Create a custom output visitor that modifies conversion."""
    return CustomVisitor()


class ComplexVisitor:
    def __init__(self) -> None:
        self.stats: dict[str, int] = {"texts": 0, "links": 0, "images": 0, "headings": 0}

    def visit_text(self, _ctx: object, _text: str) -> str:
        self.stats["texts"] += 1
        return "continue"

    def visit_heading(self, _ctx: object, _level: int, _text: str, _element_id: str) -> str:
        self.stats["headings"] += 1
        return "continue"

    def visit_paragraph(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_link(self, _ctx: object, _href: str, text: str, _title: str) -> tuple[str, str]:
        self.stats["links"] += 1
        return ("custom", f"[{text}]({_href})")

    def visit_image(self, _ctx: object, _src: str, _alt: str, _title: str) -> str:
        self.stats["images"] += 1
        return "skip"

    def visit_strong(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_em(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_code(self, _ctx: object, _text: str) -> str:
        return "continue"

    def visit_br(self, _ctx: object) -> str:
        return "continue"


def create_complex_visitor() -> object:
    """Create a complex visitor with multiple callbacks."""
    return ComplexVisitor()


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
    visitor: object | None = None,
) -> None:
    if visitor is not None:
        # When visitor is provided, use convert_with_visitor for all scenarios
        convert_with_visitor(html, None, visitor)  # type: ignore[arg-type]
        return

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

    # Create visitor if specified
    visitor = None
    if args.visitor:
        visitor_creators = {
            "noop": create_noop_visitor,
            "simple": create_simple_visitor,
            "custom": create_custom_visitor,
            "complex": create_complex_visitor,
        }
        creator = visitor_creators.get(args.visitor)
        if creator:
            visitor = creator()

    run_scenario(html, args.scenario, handle, metadata_config, visitor)  # Warmup

    profile_output = os.getenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT")
    profile_frequency = os.getenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY")
    if profile_output:
        freq = int(profile_frequency) if profile_frequency and profile_frequency.isdigit() else 1000
        start_profiling(profile_output, freq)

    start = time.perf_counter()
    for _ in range(iterations):
        run_scenario(html, args.scenario, handle, metadata_config, visitor)
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

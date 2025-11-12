# ruff: noqa: INP001
from __future__ import annotations

from html_to_markdown import ConversionOptions, convert

HTML = """
<h1>Python Smoke Test</h1>
<p>This script verifies the pip installation.</p>
""".strip()

markdown = convert(HTML, ConversionOptions(heading_style="atx"))

if "# Python Smoke Test" not in markdown:
    raise SystemExit("html-to-markdown did not return the expected heading")

"""CLI wrapper that proxies to Rust CLI binary.

This module provides backwards compatibility for code that imports
from html_to_markdown.cli. The actual CLI implementation is in Rust.
"""

from html_to_markdown.cli_proxy import main

__all__ = ["main"]

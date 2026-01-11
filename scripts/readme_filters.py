"""
Jinja2 filters and utilities for README generation.

Provides reusable filters for template rendering, code block handling,
and performance table generation.
"""

import re
from pathlib import Path
from typing import Any, ClassVar


class CodeBlockHandler:
    """Handles code block extraction and wrapping for snippet inclusion."""

    LANGUAGE_MAP: ClassVar[dict[str, str]] = {
        ".py": "python",
        ".go": "go",
        ".java": "java",
        ".js": "javascript",
        ".ts": "typescript",
        ".rb": "ruby",
        ".php": "php",
        ".cs": "csharp",
        ".rs": "rust",
        ".ex": "elixir",
        ".exs": "elixir",
    }

    @staticmethod
    def extract_code_block(content: str, snippet_path: Path) -> str:
        """
        Extract code block from markdown file.

        Looks for the first code block marked with triple backticks.

        Args:
            content: Markdown file content
            snippet_path: Path to snippet file

        Returns:
            Extracted code block with fences

        Raises:
            ValueError: If no code block found
        """
        pattern = r'```(\w+)?\s*(?:title="[^"]*")?\s*\n(.*?)```'
        match = re.search(pattern, content, re.DOTALL)

        if not match:
            raise ValueError(
                f"No code block found in markdown snippet: {snippet_path}\n"
                "Ensure file contains code wrapped in triple backticks"
            )

        language = match.group(1) or "text"
        code = match.group(2).rstrip()

        return f"```{language}\n{code}\n```\n"

    @staticmethod
    def wrap_code_block(content: str, snippet_path: Path, language: str) -> str:
        """
        Wrap raw code in markdown code fences.

        Args:
            content: Raw code content
            snippet_path: Path to snippet file
            language: Language for syntax highlighting

        Returns:
            Code wrapped in markdown fences
        """
        content_stripped = content.lstrip()
        if content_stripped.startswith("```"):
            return content

        lang_id = CodeBlockHandler.LANGUAGE_MAP.get(language, snippet_path.suffix.lstrip(".") or "text")

        code = content.rstrip()
        return f"```{lang_id}\n{code}\n```\n"


class PerformanceTableRenderer:
    """Renders performance benchmark data as markdown tables."""

    @staticmethod
    def render(perf_data: dict[str, Any], runtime: str) -> str:
        """
        Render performance table from structured data.

        Supports two benchmark formats:
        1. latency/throughput (Python, TypeScript, Ruby, Go)
        2. ops_sec (PHP, Java, C#, Elixir)

        Args:
            perf_data: Dict with 'platform', 'function', 'benchmarks' keys
            runtime: Runtime label (e.g., "Python", "Node.js")

        Returns:
            Markdown table as string
        """
        if not perf_data or "benchmarks" not in perf_data:
            return ""

        platform = perf_data.get("platform", "Unknown")
        function = perf_data.get("function", "convert()")
        note = perf_data.get("note", "")
        benchmarks = perf_data["benchmarks"]

        header = f"{platform} • {note} • `{function}` ({runtime})\n\n"

        if not benchmarks:
            return header + "\n(No benchmarks available)\n"

        first_bench = benchmarks[0]
        uses_latency = "latency" in first_bench
        uses_ops_sec = "ops_sec" in first_bench

        if uses_latency:
            table = PerformanceTableRenderer._render_latency_table(benchmarks)
        elif uses_ops_sec:
            table = PerformanceTableRenderer._render_ops_sec_table(benchmarks)
        else:
            table = "\n(Unknown benchmark format)\n"

        return header + table

    @staticmethod
    def _render_latency_table(benchmarks: list[dict[str, Any]]) -> str:
        """Render latency/throughput format table."""
        table = "| Document | Size | Latency | Throughput |\n"
        table += "| -------- | ---- | ------- | ---------- |\n"
        for bench in benchmarks:
            table += f"| {bench['name']} | {bench['size']} | "
            table += f"{bench['latency']} | {bench['throughput']} |\n"
        return table

    @staticmethod
    def _render_ops_sec_table(benchmarks: list[dict[str, Any]]) -> str:
        """Render ops_sec format table (with optional throughput)."""
        has_throughput = "throughput" in benchmarks[0]

        if has_throughput:
            table = "| Document | Size | Ops/sec | Throughput |\n"
            table += "| -------- | ---- | ------- | ---------- |\n"
            for bench in benchmarks:
                table += f"| {bench['name']} | {bench['size']} | "
                table += f"{bench['ops_sec']:,} | {bench['throughput']} |\n"
        else:
            table = "| Document | Size | Ops/sec |\n"
            table += "| -------- | ---- | ------- |\n"
            for bench in benchmarks:
                table += f"| {bench['name']} | {bench['size']} | {bench['ops_sec']:,} |\n"

        return table


class FilterRegistry:
    """Registers custom Jinja2 filters for README generation."""

    @staticmethod
    def register_snippet_filter(jinja_env: Any, include_snippet_handler: Any) -> None:
        """
        Register code snippet inclusion filter.

        Args:
            jinja_env: Jinja2 Environment instance
            include_snippet_handler: Callable for handling snippet inclusion
        """
        jinja_env.filters["include_snippet"] = include_snippet_handler
        jinja_env.globals["include_snippet"] = include_snippet_handler

    @staticmethod
    def register_performance_filter(jinja_env: Any) -> None:
        """
        Register performance table rendering filter.

        Args:
            jinja_env: Jinja2 Environment instance
        """
        jinja_env.filters["render_performance_table"] = PerformanceTableRenderer.render

    @staticmethod
    def register_migration_filter(jinja_env: Any, has_migration_handler: Any) -> None:
        """
        Register migration guide existence check filter.

        Args:
            jinja_env: Jinja2 Environment instance
            has_migration_handler: Callable for checking migration guide existence
        """
        jinja_env.filters["has_migration"] = has_migration_handler

    @staticmethod
    def register_all(
        jinja_env: Any,
        include_snippet_handler: Any,
        has_migration_handler: Any,
    ) -> None:
        """
        Register all custom filters at once.

        Args:
            jinja_env: Jinja2 Environment instance
            include_snippet_handler: Callable for snippet inclusion
            has_migration_handler: Callable for migration guide checking
        """
        FilterRegistry.register_snippet_filter(jinja_env, include_snippet_handler)
        FilterRegistry.register_performance_filter(jinja_env)
        FilterRegistry.register_migration_filter(jinja_env, has_migration_handler)

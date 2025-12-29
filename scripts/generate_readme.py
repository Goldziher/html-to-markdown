#!/usr/bin/env python3
"""
README generation script for html-to-markdown.

Generates language-specific READMEs from templates and snippets using Jinja2.
Supports validation mode to check if existing READMEs match generated output.

Adapted from kreuzberg.dev's README generation system.
"""

import argparse
import logging
import re
import sys
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    print("Error: PyYAML is required. Install with: pip install pyyaml jinja2")
    sys.exit(1)

try:
    from jinja2 import Environment, FileSystemLoader, TemplateNotFound
except ImportError:
    print("Error: Jinja2 is required. Install with: pip install pyyaml jinja2")
    sys.exit(1)


# Configure logging
logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")
logger = logging.getLogger(__name__)


class ReadmeGenerator:
    """Handles README generation from templates and snippets."""

    def __init__(self, project_root: Path) -> None:
        """Initialize generator with project root path."""
        self.project_root = project_root
        self.scripts_dir = project_root / "scripts"
        self.packages_dir = project_root / "packages"
        self.docs_dir = project_root / "docs"
        self.snippets_dir = self.docs_dir / "snippets"
        self.templates_dir = self.scripts_dir / "readme_templates"

        self.config = {}
        self.jinja_env = None

    def load_config(self) -> dict[str, Any]:
        """Load and parse README configuration from YAML."""
        config_path = self.scripts_dir / "readme_config.yaml"

        if not config_path.exists():
            raise FileNotFoundError(
                f"Configuration file not found: {config_path}\nCreate readme_config.yaml in scripts/ directory."
            )

        try:
            self.config = yaml.safe_load(config_path.read_text(encoding="utf-8"))

            if not self.config:
                raise ValueError("Configuration file is empty")

            logger.info("Loaded configuration with %d languages", len(self.config.get("languages", {})))
            return self.config

        except yaml.YAMLError as e:
            raise ValueError(f"Failed to parse YAML configuration: {e}") from e

    def setup_jinja_env(self) -> Environment:
        """Configure Jinja2 environment with custom filters."""
        if not self.templates_dir.exists():
            raise FileNotFoundError(
                f"Templates directory not found: {self.templates_dir}\nCreate readme_templates/ directory in scripts/"
            )

        self.jinja_env = Environment(
            loader=FileSystemLoader(str(self.templates_dir)),
            keep_trailing_newline=True,
            autoescape=True,
        )

        # Register custom filter - use a lambda to capture self
        self.jinja_env.filters["include_snippet"] = lambda path, lang: self.include_snippet_filter(path, lang)

        # Also register as a global function for potential use in templates
        self.jinja_env.globals["include_snippet"] = lambda path, lang: self.include_snippet_filter(path, lang)

        # html-to-markdown specific filters
        self.jinja_env.filters["render_performance_table"] = self.render_performance_table_filter
        self.jinja_env.filters["has_migration"] = lambda lang, ver: self.has_migration_guide(lang, ver)

        logger.debug("Jinja2 environment configured")
        return self.jinja_env

    def include_snippet_filter(self, path: str, language: str) -> str:
        """
        Include code snippets in generated READMEs via Jinja2 filter.

        Loads snippets from docs/snippets/{language}/{path}
        Handles both .md files (extract code block) and raw code files.

        Args:
            path: Snippet path relative to language folder
            language: Language identifier (python, go, java, etc.)

        Returns:
            Formatted snippet content

        Raises:
            FileNotFoundError: If snippet file not found
            ValueError: If snippet format is invalid
        """
        # Build snippet path
        snippet_path = self.snippets_dir / language / path

        # Try with .md extension first if no extension provided
        if not snippet_path.suffix:
            md_path = snippet_path.with_suffix(".md")
            if md_path.exists():
                snippet_path = md_path

        if not snippet_path.exists():
            raise FileNotFoundError(f"Snippet not found: {snippet_path}\nLooking for: docs/snippets/{language}/{path}")

        try:
            content = snippet_path.read_text(encoding="utf-8")
        except Exception as e:
            raise ValueError(f"Failed to read snippet {snippet_path}: {e}") from e

        # Handle markdown files (extract code block)
        if snippet_path.suffix == ".md":
            return self._extract_code_block(content, snippet_path)

        # Handle raw code files (wrap in code fences)
        return self._wrap_code_block(content, snippet_path, language)

    def _extract_code_block(self, content: str, snippet_path: Path) -> str:
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
        # Match code blocks with language identifier
        pattern = r'```(\w+)?\s*(?:title="[^"]*")?\s*\n(.*?)```'
        match = re.search(pattern, content, re.DOTALL)

        if not match:
            raise ValueError(
                f"No code block found in markdown snippet: {snippet_path}\n"
                "Ensure file contains code wrapped in triple backticks"
            )

        language = match.group(1) or "text"
        code = match.group(2).rstrip()

        # Return the complete code block with fences
        return f"```{language}\n{code}\n```\n"

    def _wrap_code_block(self, content: str, snippet_path: Path, language: str) -> str:
        """
        Wrap raw code in markdown code fences.

        Args:
            content: Raw code content
            snippet_path: Path to snippet file
            language: Language for syntax highlighting

        Returns:
            Code wrapped in markdown fences
        """
        # Check if content already has fence markers to prevent double-wrapping
        content_stripped = content.lstrip()
        if content_stripped.startswith("```"):
            # Content already has fences, return as-is
            return content

        # Determine language from file extension if not provided
        ext_map = {
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

        lang_id = ext_map[language] if language in ext_map else snippet_path.suffix.lstrip(".") or "text"

        code = content.rstrip()
        return f"```{lang_id}\n{code}\n```\n"

    def render_performance_table_filter(self, perf_data: dict[str, Any], runtime: str) -> str:
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

        # Build header
        header = f"{platform} • {note} • `{function}` ({runtime})\n\n"

        # Detect format from first benchmark
        if not benchmarks:
            return header + "\n(No benchmarks available)\n"

        first_bench = benchmarks[0]
        uses_latency = "latency" in first_bench
        uses_ops_sec = "ops_sec" in first_bench

        # Build table based on format
        if uses_latency:
            # latency/throughput format
            table = "| Document | Size | Latency | Throughput |\n"
            table += "| -------- | ---- | ------- | ---------- |\n"
            for bench in benchmarks:
                table += f"| {bench['name']} | {bench['size']} | "
                table += f"{bench['latency']} | {bench['throughput']} |\n"
        elif uses_ops_sec:
            # ops_sec format (with optional throughput)
            has_throughput = "throughput" in first_bench
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
        else:
            # Fallback for unknown format
            table = "\n(Unknown benchmark format)\n"

        return header + table

    def has_migration_guide(self, lang_code: str, version: str) -> bool:
        """Check if migration guide exists for language/version."""
        migration_dir = self.docs_dir / "migration-guides" / lang_code
        guide_path = migration_dir / f"{version}.md"
        return guide_path.exists()

    def inject_migration_guide(self, lang_code: str, version: str) -> str:
        """
        Load migration guide from docs/migration-guides/{lang_code}/{version}.md
        Returns empty string if no guide exists.
        """
        migration_dir = self.docs_dir / "migration-guides" / lang_code
        guide_path = migration_dir / f"{version}.md"

        if not guide_path.exists():
            return ""

        try:
            content = guide_path.read_text(encoding="utf-8")
            logger.debug("Injected migration guide: %s", guide_path)
            return content
        except Exception as e:
            logger.warning("Failed to read migration guide %s: %s", guide_path, e)
            return ""

    def generate_readme(
        self, lang_code: str, lang_config: dict[str, Any], output_path: Path, dry_run: bool = False
    ) -> str:
        """
        Render README from template using language configuration.

        Args:
            lang_code: Language code (python, go, etc.)
            lang_config: Language-specific configuration
            output_path: Where to write the README
            dry_run: If True, don't write to disk

        Returns:
            Generated README content

        Raises:
            TemplateNotFound: If template not found
            Exception: Other rendering errors
        """
        template_name = lang_config.get("template", f"{lang_code}.md.jinja")

        try:
            template = self.jinja_env.get_template(template_name)
        except TemplateNotFound as e:
            raise TemplateNotFound(
                f"Template not found: {template_name}\nExpected at: {self.templates_dir / template_name}"
            ) from e

        # Load migration guide if exists
        current_version = self.config.get("version", "")
        migration_guide = self.inject_migration_guide(lang_code, current_version)

        # Prepare context for template
        context = {
            "language": lang_code,
            "version": current_version,
            "license": self.config.get("license", "MIT"),
            "discord_url": self.config.get("discord_url", ""),
            "banner_url": self.config.get("banner_url", ""),
            "migration_guide": migration_guide,
            **lang_config,
        }

        try:
            content = template.render(**context)
        except Exception as e:
            raise Exception(f"Failed to render template {template_name}: {e}") from e

        # Write to disk unless dry-run
        if not dry_run:
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(content, encoding="utf-8")
            logger.info("Generated: %s", output_path)
        else:
            logger.info("[DRY-RUN] Would generate: %s", output_path)

        return content

    def validate_readme(self, lang_code: str, lang_config: dict[str, Any], readme_path: Path) -> bool:
        """
        Validate that existing README matches generated output.

        Args:
            lang_code: Language code
            lang_config: Language configuration
            readme_path: Path to existing README

        Returns:
            True if README is up-to-date, False otherwise
        """
        if not readme_path.exists():
            logger.warning("README not found: %s", readme_path)
            return False

        try:
            # Generate fresh README content
            generated = self.generate_readme(lang_code, lang_config, readme_path, dry_run=True)
            existing = readme_path.read_text(encoding="utf-8")

            if generated == existing:
                logger.info("Valid: %s", readme_path)
                return True
            logger.warning("Out of date: %s", readme_path)
            return False

        except Exception as e:
            logger.error("Validation error for %s: %s", readme_path, e)
            return False

    def resolve_output_path(self, lang_code: str, lang_config: dict[str, Any]) -> Path:
        """
        Resolve README output path with special handling for Go v2 structure.

        Args:
            lang_code: Language code (e.g., 'python', 'go')
            lang_config: Language configuration dict

        Returns:
            Resolved path for README output
        """
        # Check for custom output_path in config
        if "output_path" in lang_config:
            return self.project_root / lang_config["output_path"]

        # Special case: Go uses v2 subdirectory
        if lang_code == "go":
            return self.packages_dir / "go" / "v2" / "README.md"

        # Standard pattern: packages/{language}/README.md
        return self.packages_dir / lang_code / "README.md"

    def process_all_languages(
        self, language_filter: str | None = None, dry_run: bool = False, validate_only: bool = False
    ) -> bool:
        """
        Process READMEs for all configured languages.

        Args:
            language_filter: Only process specific language (e.g., 'python')
            dry_run: Don't write to disk
            validate_only: Only validate, don't generate

        Returns:
            True if all operations succeeded, False otherwise
        """
        if not self.config:
            logger.error("Configuration not loaded")
            return False

        languages = self.config.get("languages", {})

        if language_filter:
            if language_filter not in languages:
                logger.error("Unknown language: %s", language_filter)
                logger.info("Available: %s", ", ".join(languages.keys()))
                return False
            languages = {language_filter: languages[language_filter]}

        all_ok = True

        for lang_code, lang_config in languages.items():
            # Resolve output path with special handling for Go v2
            readme_path = self.resolve_output_path(lang_code, lang_config)

            try:
                if validate_only:
                    if not self.validate_readme(lang_code, lang_config, readme_path):
                        all_ok = False
                else:
                    self.generate_readme(lang_code, lang_config, readme_path, dry_run)

            except Exception as e:
                logger.error("Failed to process %s: %s", lang_code, e)
                all_ok = False

        return all_ok

    def main(self, args: argparse.Namespace) -> int:
        """
        Handle argument processing and README generation workflow.

        Args:
            args: Parsed command-line arguments

        Returns:
            Exit code (0 for success, 1 for failure)
        """
        try:
            # Load configuration
            self.load_config()

            # Setup Jinja2
            self.setup_jinja_env()

            # Process languages
            success = self.process_all_languages(
                language_filter=args.language, dry_run=args.dry_run, validate_only=args.validate
            )

            if args.validate:
                if success:
                    logger.info("All READMEs are up-to-date")
                else:
                    logger.error("Some READMEs are out of date")
            elif success:
                logger.info("README generation completed successfully")
            else:
                logger.error("README generation completed with errors")

            return 0 if success else 1

        except Exception as e:
            logger.exception("Fatal error: %s", e)
            return 1


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(
        description="Generate language-specific READMEs from templates and snippets",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate all READMEs
  python scripts/generate_readme.py

  # Generate only Python README
  python scripts/generate_readme.py --language python

  # Preview changes without writing
  python scripts/generate_readme.py --dry-run

  # Check if READMEs are up-to-date
  python scripts/generate_readme.py --validate

  # Validate specific language
  python scripts/generate_readme.py --language go --validate
        """,
    )

    parser.add_argument(
        "--language",
        help="Generate README for specific language only",
        metavar="LANG",
    )

    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview generation without writing to disk",
    )

    parser.add_argument(
        "--validate",
        action="store_true",
        help="Validate existing READMEs match generated output",
    )

    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Enable verbose output",
    )

    return parser.parse_args()


def main() -> int:
    """Execute README generation workflow with argument parsing."""
    args = parse_args()

    if args.verbose:
        logger.setLevel(logging.DEBUG)

    # Find project root
    project_root = Path(__file__).parent.parent

    # Create generator and run
    generator = ReadmeGenerator(project_root)
    return generator.main(args)


if __name__ == "__main__":
    sys.exit(main())

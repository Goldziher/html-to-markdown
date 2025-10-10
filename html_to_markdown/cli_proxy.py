"""CLI proxy that calls the Rust CLI binary.

This module provides a Python wrapper around the Rust CLI binary,
allowing the Python package to use the high-performance Rust implementation
for command-line operations. It also provides v1 -> v2 CLI argument translation.
"""

import subprocess
import sys
from pathlib import Path

from html_to_markdown.exceptions import RedundantV1FlagError, RemovedV1FlagError


def find_cli_binary() -> Path:
    """Find the html-to-markdown CLI binary.

    Returns:
        Path to the CLI binary

    Raises:
        FileNotFoundError: If the binary cannot be found
    """
    binary_name = "html-to-markdown.exe" if sys.platform == "win32" else "html-to-markdown"

    possible_locations = [
        Path(__file__).parent.parent / "target" / "release" / binary_name,
        Path(__file__).parent / "bin" / binary_name,
        Path(__file__).parent / binary_name,
    ]

    for location in possible_locations:
        if location.exists() and location.is_file():
            return location

    msg = "html-to-markdown CLI binary not found. Please install or build the package."
    raise FileNotFoundError(msg)


def translate_v1_args_to_v2(argv: list[str]) -> list[str]:
    """Translate v1 CLI arguments to v2 Rust CLI arguments.

    This handles differences between the v1 Python CLI and v2 Rust CLI:
    - Boolean flags: v1 used --flag/--no-flag, v2 uses presence/absence
    - Flag name changes: --preprocess-html -> --preprocess
    - Unsupported flags: --strip, --convert (raise errors)

    Args:
        argv: v1 CLI arguments

    Returns:
        Translated v2 CLI arguments

    Raises:
        RemovedV1FlagError: If a v1 flag has been removed in v2
    """
    translated = []
    i = 0
    while i < len(argv):
        arg = argv[i]

        # Error on removed/unsupported v1 features
        if arg in ("--strip", "--convert"):
            raise RemovedV1FlagError(
                flag=arg,
                reason=f"{arg} option has been removed in v2.",
                migration="Remove this flag from your command. The feature is no longer available.",
            )

        # These flags are redundant (match v2 defaults) but we accept them for v1 compatibility
        # Silently skip - Rust CLI defaults match these flags
        if arg in (
            "--no-escape-asterisks",
            "--no-escape-underscores",
            "--no-escape-misc",
            "--no-wrap",
            "--no-autolinks",
            "--no-extract-metadata",
        ):
            # Skip this flag - matches Rust CLI defaults
            pass

        # Flag name translations
        elif arg == "--preprocess-html":
            translated.append("--preprocess")

        # Positive flags that should be passed through
        elif arg in (
            "--escape-asterisks",
            "--escape-underscores",
            "--escape-misc",
            "--autolinks",
            "--extract-metadata",
            "--wrap",
        ):
            translated.append(arg)

        # All other args pass through unchanged
        else:
            translated.append(arg)

        i += 1

    return translated


def main(argv: list[str]) -> str:
    """Run the Rust CLI with the given arguments.

    Translates v1 CLI arguments to v2 format if needed.
    Exits with non-zero status on errors (FileNotFoundError, UnsupportedV1FeatureError, CLI errors).

    Args:
        argv: Command line arguments (without program name)

    Returns:
        Output from the CLI
    """
    cli_binary = find_cli_binary()

    try:
        translated_args = translate_v1_args_to_v2(argv)
    except (RemovedV1FlagError, RedundantV1FlagError) as e:
        # Format the error nicely for CLI users
        sys.stderr.write(f"\n❌ Error: {e.flag}\n\n")
        sys.stderr.write(f"   {e.reason}\n\n")
        sys.stderr.write(f"   💡 {e.migration}\n\n")
        sys.exit(1)
    except ValueError as e:
        sys.stderr.write(f"Error: {e}\n")
        sys.exit(1)

    result = subprocess.run(  # noqa: S603
        [str(cli_binary), *translated_args],
        capture_output=True,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        sys.exit(result.returncode)

    return result.stdout

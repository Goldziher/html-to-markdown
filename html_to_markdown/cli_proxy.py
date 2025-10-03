"""CLI proxy that calls the Rust CLI binary.

This module provides a Python wrapper around the Rust CLI binary,
allowing the Python package to use the high-performance Rust implementation
for command-line operations. It also provides v1 -> v2 CLI argument translation.
"""

import subprocess
import sys
from pathlib import Path


def find_cli_binary() -> Path:
    """Find the html-to-markdown CLI binary.

    Returns:
        Path to the CLI binary

    Raises:
        FileNotFoundError: If the binary cannot be found
    """
    possible_locations = [
        Path(__file__).parent.parent / "target" / "release" / "html-to-markdown",
        Path(__file__).parent / "bin" / "html-to-markdown",
        Path(__file__).parent / "html-to-markdown",
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
        ValueError: If unsupported v1 flags are used
    """
    translated = []
    i = 0
    while i < len(argv):
        arg = argv[i]

        if arg in ("--strip", "--convert"):
            raise ValueError(f"{arg} option is not supported in v2. Please remove it from your command.")

        if arg == "--preprocess-html":
            translated.append("--preprocess")

        elif arg == "--no-escape-asterisks":
            translated.append("--no-escape-asterisks")
        elif arg == "--escape-asterisks":
            pass

        elif arg == "--no-escape-underscores":
            translated.append("--no-escape-underscores")
        elif arg == "--escape-underscores":
            pass

        elif arg == "--no-escape-misc":
            translated.append("--no-escape-misc")
        elif arg in {"--escape-misc", "--no-autolinks"}:
            pass
        elif arg == "--autolinks":
            translated.append("--autolinks")

        elif arg == "--no-extract-metadata":
            translated.append("--no-extract-metadata")
        elif arg in {"--extract-metadata", "--no-wrap"}:
            pass
        elif arg == "--wrap":
            translated.append("--wrap")

        else:
            translated.append(arg)

        i += 1

    return translated


def main(argv: list[str]) -> str:
    """Run the Rust CLI with the given arguments.

    Translates v1 CLI arguments to v2 format if needed.
    Exits with non-zero status on errors (FileNotFoundError, ValueError, CLI errors).

    Args:
        argv: Command line arguments (without program name)

    Returns:
        Output from the CLI
    """
    cli_binary = find_cli_binary()

    try:
        translated_args = translate_v1_args_to_v2(argv)
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

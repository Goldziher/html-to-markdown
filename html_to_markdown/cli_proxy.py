"""CLI proxy that calls the Rust CLI binary.

This module provides a Python wrapper around the Rust CLI binary,
allowing the Python package to use the high-performance Rust implementation
for command-line operations.
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
    # Try to find the binary in common locations
    possible_locations = [
        # Development: built with cargo
        Path(__file__).parent.parent / "target" / "release" / "html-to-markdown",
        # Installed: bundled with Python package
        Path(__file__).parent / "bin" / "html-to-markdown",
        # macOS bundled binary
        Path(__file__).parent / "html-to-markdown",
    ]

    for location in possible_locations:
        if location.exists() and location.is_file():
            return location

    msg = "html-to-markdown CLI binary not found. Please install or build the package."
    raise FileNotFoundError(msg)


def main(argv: list[str]) -> str:
    """Run the Rust CLI with the given arguments.

    Args:
        argv: Command line arguments (without program name)

    Returns:
        Output from the CLI

    Raises:
        subprocess.CalledProcessError: If the CLI exits with non-zero status
        FileNotFoundError: If the CLI binary cannot be found
    """
    cli_binary = find_cli_binary()

    # Run the Rust CLI binary
    result = subprocess.run(
        [str(cli_binary), *argv],
        capture_output=True,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        # Print stderr and exit with same code
        sys.stderr.write(result.stderr)
        sys.exit(result.returncode)

    return result.stdout

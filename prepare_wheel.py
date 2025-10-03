# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "tomli>=2; python_version<'3.11'",
# ]
# ///
"""Build script to include CLI binary in wheel.

This script is called by maturin during the build process to:
1. Build the html-to-markdown CLI binary
2. Copy it to the .data/scripts directory for wheel inclusion
"""

import shutil
import subprocess
import sys
from pathlib import Path

try:
    import tomllib  # Python 3.11+  # type: ignore[import-not-found]
except ImportError:
    import tomli as tomllib  # Python 3.10  # type: ignore[import-not-found]


def main() -> None:
    """Build CLI binary and prepare for wheel packaging."""
    # Build the CLI binary
    subprocess.run(
        ["cargo", "build", "--release", "--package", "html-to-markdown-cli"],
        check=True,
    )

    # Determine binary name based on platform
    binary_name = "html-to-markdown.exe" if sys.platform == "win32" else "html-to-markdown"
    source = Path("target") / "release" / binary_name

    if not source.exists():
        msg = f"CLI binary not found at {source}"
        raise FileNotFoundError(msg)

    # Get version from Cargo.toml
    with Path("Cargo.toml").open("rb") as f:
        cargo_toml = tomllib.load(f)
    version = cargo_toml["workspace"]["package"]["version"]

    # Copy to .data/scripts for maturin (PEP 427 format)
    data_dir_name = f"html_to_markdown-{version}.data"
    scripts_dir = Path(data_dir_name) / "scripts"
    scripts_dir.mkdir(parents=True, exist_ok=True)

    dest = scripts_dir / binary_name
    shutil.copy(source, dest)

    # Make it executable on Unix
    if sys.platform != "win32":
        dest.chmod(0o755)


if __name__ == "__main__":
    main()

import subprocess
import sys
import warnings
from pathlib import Path

from html_to_markdown.exceptions import RedundantV1FlagError, RemovedV1FlagError


def find_cli_binary() -> Path:
    """Find the html-to-markdown CLI binary in expected locations.

    Returns:
        Path to the CLI binary.

    Raises:
        FileNotFoundError: If the binary cannot be found.
    """
    binary_name = "html-to-markdown.exe" if sys.platform == "win32" else "html-to-markdown"

    module_dir = Path(__file__).resolve().parent
    parent_dirs = list(module_dir.parents)

    search_roots = []
    for parent in parent_dirs:
        candidate = parent / "target" / "release" / binary_name
        search_roots.append(candidate)

    possible_locations = [
        *search_roots,
        module_dir / "bin" / binary_name,
        module_dir / binary_name,
    ]

    for location in possible_locations:
        if location.exists() and location.is_file():
            return location

    msg = "html-to-markdown CLI binary not found. Please install or build the package."
    raise FileNotFoundError(msg)


def translate_v1_args_to_v2(argv: list[str]) -> list[str]:
    """Translate v1 CLI arguments to v2 format.

    Args:
        argv: List of command-line arguments.

    Returns:
        Translated list of arguments compatible with v2.

    Raises:
        RemovedV1FlagError: If a v1 flag has been removed in v2.
    """
    translated = []
    i = 0
    while i < len(argv):
        arg = argv[i]

        if arg in ("--strip", "--convert"):
            raise RemovedV1FlagError(
                flag=arg,
                reason=f"{arg} option has been removed in v2.",
                migration="Remove this flag from your command. The feature is no longer available.",
            )

        if arg in (
            "--no-escape-asterisks",
            "--no-escape-underscores",
            "--no-escape-misc",
            "--no-wrap",
            "--no-autolinks",
            "--no-extract-metadata",
        ):
            warnings.warn(
                f"'{arg}' is deprecated and redundant in v2. "
                f"These options are now disabled by default. Remove this flag.",
                DeprecationWarning,
                stacklevel=2,
            )

        elif arg == "--preprocess-html":
            warnings.warn(
                "'--preprocess-html' is deprecated. Use '--preprocess' instead.",
                DeprecationWarning,
                stacklevel=2,
            )
            translated.append("--preprocess")

        elif arg in (
            "--escape-asterisks",
            "--escape-underscores",
            "--escape-misc",
            "--autolinks",
            "--extract-metadata",
            "--wrap",
        ):
            translated.append(arg)

        else:
            translated.append(arg)

        i += 1

    return translated


def main(argv: list[str]) -> str:
    """Execute the CLI proxy.

    Translates v1 arguments to v2 and invokes the native Rust CLI binary.

    Args:
        argv: Command-line arguments.

    Returns:
        Stdout from the CLI binary.
    """
    cli_binary = find_cli_binary()

    try:
        translated_args = translate_v1_args_to_v2(argv)
    except (RemovedV1FlagError, RedundantV1FlagError) as e:
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

import shutil
import subprocess
import sys
from pathlib import Path

try:
    import tomllib  # Python 3.11+  # type: ignore[import-not-found]
except ImportError:
    import tomli as tomllib  # Python 3.10  # type: ignore[import-not-found]


def main() -> None:
    print("Building html-to-markdown CLI binary...")
    subprocess.run(
        ["cargo", "build", "--release", "--package", "html-to-markdown-cli"],
        check=True,
    )

    binary_name = "html-to-markdown.exe" if sys.platform == "win32" else "html-to-markdown"
    source = Path("target") / "release" / binary_name

    if not source.exists():
        msg = f"CLI binary not found at {source}"
        raise FileNotFoundError(msg)

    print(f"Found CLI binary at {source}")

    package_bin_dir = Path("html_to_markdown") / "bin"
    package_bin_dir.mkdir(parents=True, exist_ok=True)

    dest = package_bin_dir / binary_name
    shutil.copy(source, dest)
    print(f"Copied CLI binary to {dest}")

    if sys.platform != "win32":
        dest.chmod(0o755)
        print(f"Made binary executable: {dest}")

    with Path("Cargo.toml").open("rb") as f:
        cargo_toml = tomllib.load(f)
    version = cargo_toml["workspace"]["package"]["version"]

    data_dir_name = f"html_to_markdown-{version}.data"
    scripts_dir = Path(data_dir_name) / "scripts"
    scripts_dir.mkdir(parents=True, exist_ok=True)

    scripts_dest = scripts_dir / binary_name
    shutil.copy(source, scripts_dest)
    print(f"Copied CLI binary to {scripts_dest} for PATH installation")

    if sys.platform != "win32":
        scripts_dest.chmod(0o755)


if __name__ == "__main__":
    main()

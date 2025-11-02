import shutil
import subprocess
import sys
from pathlib import Path

try:
    import tomllib  # Python 3.11+  # type: ignore[import-not-found]
except ImportError:
    import tomli as tomllib  # Python 3.10  # type: ignore[import-not-found]


def main() -> None:
    repo_root = Path(__file__).resolve().parent.parent

    print("Building html-to-markdown CLI binary...")
    subprocess.run(
        ["cargo", "build", "--release", "--package", "html-to-markdown-cli"],
        check=True,
        cwd=repo_root,
    )

    binary_name = "html-to-markdown.exe" if sys.platform == "win32" else "html-to-markdown"
    source = repo_root / "target" / "release" / binary_name

    if not source.exists():
        msg = f"CLI binary not found at {source}"
        raise FileNotFoundError(msg)

    print(f"Found CLI binary at {source}")

    package_root = repo_root / "packages" / "python" / "html_to_markdown"
    package_bin_dir = package_root / "bin"
    package_bin_dir.mkdir(parents=True, exist_ok=True)

    dest = package_bin_dir / binary_name
    shutil.copy(source, dest)
    print(f"Copied CLI binary to {dest}")

    if sys.platform != "win32":
        dest.chmod(0o755)
        print(f"Made binary executable: {dest}")

    license_src = repo_root / "LICENSE"
    license_dest = repo_root / "packages" / "python" / "LICENSE"
    if license_src.exists():
        shutil.copy(license_src, license_dest)
        print(f"Copied LICENSE to {license_dest}")

    with (repo_root / "Cargo.toml").open("rb") as f:
        cargo_toml = tomllib.load(f)
    version = cargo_toml["workspace"]["package"]["version"]

    data_dir_name = repo_root / "packages" / "python" / f"html_to_markdown-{version}.data"
    scripts_dir = data_dir_name / "scripts"
    scripts_dir.mkdir(parents=True, exist_ok=True)

    scripts_dest = scripts_dir / binary_name
    shutil.copy(source, scripts_dest)
    print(f"Copied CLI binary to {scripts_dest} for PATH installation")

    if sys.platform != "win32":
        scripts_dest.chmod(0o755)


if __name__ == "__main__":
    main()

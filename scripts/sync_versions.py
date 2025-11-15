#!/usr/bin/env python3
"""
Sync version from Cargo.toml workspace to all package manifests.

This script reads the version from Cargo.toml [workspace.package] and updates:
- All package.json files (including optionalDependencies)
- Python pyproject.toml files
- Ruby version.rb file
- Cargo.toml files with hardcoded versions (not using workspace)
"""

import json
import re
import sys
from pathlib import Path


def get_repo_root() -> Path:
    """Get the repository root directory."""
    script_dir = Path(__file__).resolve().parent
    return script_dir.parent


def get_workspace_version(repo_root: Path) -> str:
    """Extract version from Cargo.toml [workspace.package]."""
    cargo_toml = repo_root / "Cargo.toml"
    if not cargo_toml.exists():
        raise FileNotFoundError(f"Cargo.toml not found at {cargo_toml}")

    content = cargo_toml.read_text()
    match = re.search(r'^\[workspace\.package\]\s*\nversion\s*=\s*"([^"]+)"', content, re.MULTILINE)

    if not match:
        raise ValueError("Could not find version in Cargo.toml [workspace.package]")

    return match.group(1)


def update_package_json(file_path: Path, version: str) -> tuple[bool, str, str]:
    """
    Update a package.json file.

    Returns: (changed, old_version, new_version)
    """
    data = json.loads(file_path.read_text())
    old_version = data.get("version", "N/A")
    changed = False

    # Update main version
    if data.get("version") != version:
        data["version"] = version
        changed = True

    # Note: We don't update optionalDependencies for html-to-markdown-node
    # because napi prepublish adds them automatically during publish

    if changed:
        file_path.write_text(json.dumps(data, indent=2) + "\n")

    return changed, old_version, version


def update_pyproject_toml(file_path: Path, version: str) -> tuple[bool, str, str]:
    """
    Update a pyproject.toml file.

    Returns: (changed, old_version, new_version)
    """
    content = file_path.read_text()
    match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    old_version = match.group(1) if match else "NOT FOUND"

    if old_version == version:
        return False, old_version, version

    new_content = re.sub(r'^(version\s*=\s*)"[^"]+"', rf'\1"{version}"', content, count=1, flags=re.MULTILINE)

    file_path.write_text(new_content)
    return True, old_version, version


def update_ruby_version(file_path: Path, version: str) -> tuple[bool, str, str]:
    """
    Update Ruby version.rb file.

    Returns: (changed, old_version, new_version)
    """
    content = file_path.read_text()
    match = re.search(r"VERSION\s*=\s*'([^']+)'", content)
    old_version = match.group(1) if match else "NOT FOUND"

    if old_version == version:
        return False, old_version, version

    new_content = re.sub(r"(VERSION\s*=\s*)'[^']+'", rf"\1'{version}'", content)

    file_path.write_text(new_content)
    return True, old_version, version


def update_cargo_toml(file_path: Path, version: str) -> tuple[bool, str, str]:
    """
    Update a Cargo.toml file that has hardcoded version (not using workspace).

    Returns: (changed, old_version, new_version)
    """
    content = file_path.read_text()
    match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    old_version = match.group(1) if match else "NOT FOUND"

    if old_version == version:
        return False, old_version, version

    # Update the version field
    new_content = re.sub(r'^(version\s*=\s*)"[^"]+"', rf'\1"{version}"', content, count=1, flags=re.MULTILINE)

    file_path.write_text(new_content)
    return True, old_version, version


def update_rust_dependency_versions(file_path: Path, version: str) -> bool:
    """Update html-to-markdown-rs dependency version pins inside Cargo manifests."""
    content = file_path.read_text()

    pattern = re.compile(r'(html-to-markdown-rs\s*=\s*\{\s*version\s*=\s*")([^"]+)(")')

    def repl(match: re.Match[str]) -> str:
        return f"{match.group(1)}{version}{match.group(3)}"

    new_content, count = pattern.subn(repl, content)
    if count == 0:
        return False

    file_path.write_text(new_content)
    return True


def update_gemfile_lock(file_path: Path, version: str) -> tuple[bool, str, str]:
    content = file_path.read_text()
    match = re.search(r"(html-to-markdown\s*\()\s*([^)]+)(\))", content)
    if not match:
        return False, "NOT FOUND", version

    old_version = match.group(2)
    if old_version == version:
        return False, old_version, version

    new_content = re.sub(
        r"(html-to-markdown\s*\()\s*([^)]+)(\))",
        lambda m: f"{m.group(1)}{version}{m.group(3)}",
        content,
        count=1,
    )
    file_path.write_text(new_content)
    return True, old_version, version


def update_python_version_file(file_path: Path, version: str) -> tuple[bool, str, str]:
    content = file_path.read_text()
    match = re.search(r'(__version__\s*=\s*)"([^"]+)"', content)
    old_version = match.group(2) if match else "NOT FOUND"

    if old_version == version:
        return False, old_version, version

    new_content = re.sub(r'(__version__\s*=\s*)"([^"]+)"', rf'\1"{version}"', content, count=1)
    file_path.write_text(new_content)
    return True, old_version, version


def update_node_binding_version(file_path: Path, version: str) -> tuple[bool, str, str]:
    content = file_path.read_text()
    pattern = r"(bindingPackageVersion\s*!==\s*')([0-9]+\.[0-9]+\.[0-9]+)(')"
    new_content, count = re.subn(pattern, rf"\g<1>{version}\g<3>", content)
    new_content, count_expected = re.subn(
        r"(expected\s+)([0-9]+\.[0-9]+\.[0-9]+)(\s+but)", rf"\g<1>{version}\g<3>", new_content
    )
    if count == 0 and count_expected == 0:
        return False, "N/A", version

    file_path.write_text(new_content)
    return True, "Updated node binding checks", version


def update_uv_lock(file_path: Path, version: str) -> tuple[bool, str, str]:
    content = file_path.read_text()
    pattern = re.compile(r'(name\s*=\s*"html-to-markdown"\s+version\s*=\s*)"([^"]+)"')
    match = pattern.search(content)
    if not match:
        return False, "NOT FOUND", version

    old_version = match.group(2)
    if old_version == version:
        return False, old_version, version

    new_content = pattern.sub(lambda m: f'{m.group(1)}"{version}"', content, count=1)
    file_path.write_text(new_content)
    return True, old_version, version


def update_mix_version(file_path: Path, version: str) -> tuple[bool, str, str]:
    """
    Update @version declarations inside mix.exs files.

    Returns: (changed, old_version, new_version)
    """
    content = file_path.read_text()
    match = re.search(r'@version\s*=?\s*"([^"]+)"', content)
    old_version = match.group(1) if match else "NOT FOUND"

    if old_version == version:
        return False, old_version, version

    new_content = re.sub(r'(@version\s*=?\s*)"[^"]+"', rf'\1"{version}"', content, count=1)
    file_path.write_text(new_content)
    return True, old_version, version


def update_csproj_version(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update <Version> tags inside .csproj files."""
    content = file_path.read_text()
    match = re.search(r"<Version>([^<]+)</Version>", content)
    old_version = match.group(1) if match else "NOT FOUND"

    if old_version == version:
        return False, old_version, version

    new_content = re.sub(r"(<Version>)[^<]+(</Version>)", rf"\g<1>{version}\2", content, count=1)
    file_path.write_text(new_content)
    return True, old_version, version


def update_pom_version(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update the primary <version> tag for the Java package."""
    content = file_path.read_text()
    pattern = re.compile(
        r"(<artifactId>\s*html-to-markdown\s*</artifactId>\s*<version>)([^<]+)(</version>)",
        re.IGNORECASE | re.DOTALL,
    )
    match = pattern.search(content)
    old_version = match.group(2).strip() if match else "NOT FOUND"

    if old_version == version:
        return False, old_version, version

    new_content, count = pattern.subn(lambda m: f"{m.group(1)}{version}{m.group(3)}", content, count=1)
    if count == 0:
        return False, old_version, version

    file_path.write_text(new_content)
    return True, old_version, version


def main() -> None:
    repo_root = get_repo_root()

    try:
        version = get_workspace_version(repo_root)
    except (FileNotFoundError, ValueError) as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"\n📦 Syncing version {version} from Cargo.toml\n")

    updated_files: list[str] = []
    unchanged_files: list[str] = []

    # Update package.json files (excluding dist/, node_modules/, .git/)
    for pkg_json in repo_root.rglob("package.json"):
        # Skip build artifacts and dependencies
        if any(part in pkg_json.parts for part in ["node_modules", ".git", "target"]):
            continue
        # Skip WASM dist directories (these are build artifacts)
        if "wasm" in str(pkg_json) and any(part.startswith("dist") for part in pkg_json.parts):
            continue

        changed, old_ver, new_ver = update_package_json(pkg_json, version)
        rel_path = pkg_json.relative_to(repo_root)

        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update Python pyproject.toml files
    for pyproject in [
        repo_root / "packages/python/pyproject.toml",
        repo_root / "crates/html-to-markdown-py/pyproject.toml",
    ]:
        if pyproject.exists():
            changed, old_ver, new_ver = update_pyproject_toml(pyproject, version)
            rel_path = pyproject.relative_to(repo_root)

            if changed:
                print(f"✓ {rel_path}: {old_ver} → {new_ver}")
                updated_files.append(str(rel_path))
            else:
                unchanged_files.append(str(rel_path))

    # Update Ruby version file
    ruby_version = repo_root / "packages/ruby/lib/html_to_markdown/version.rb"
    if ruby_version.exists():
        changed, old_ver, new_ver = update_ruby_version(ruby_version, version)
        rel_path = ruby_version.relative_to(repo_root)

        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update Gemfile.lock for the Ruby gem
    gemfile_lock = repo_root / "packages/ruby/Gemfile.lock"
    if gemfile_lock.exists():
        changed, old_ver, new_ver = update_gemfile_lock(gemfile_lock, version)
        rel_path = gemfile_lock.relative_to(repo_root)

        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update Python package __version__
    python_version_file = repo_root / "packages/python/html_to_markdown/__init__.py"
    if python_version_file.exists():
        changed, old_ver, new_ver = update_python_version_file(python_version_file, version)
        rel_path = python_version_file.relative_to(repo_root)

        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update mix.exs for the Elixir bindings
    mix_exs = repo_root / "packages/elixir/mix.exs"
    if mix_exs.exists():
        changed, old_ver, new_ver = update_mix_version(mix_exs, version)
        rel_path = mix_exs.relative_to(repo_root)
        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update Node binding runtime version checks
    node_binding_index = repo_root / "crates/html-to-markdown-node/index.js"
    if node_binding_index.exists():
        changed, _, _ = update_node_binding_version(node_binding_index, version)
        rel_path = node_binding_index.relative_to(repo_root)
        if changed:
            print(f"✓ {rel_path}: updated binding version guards to {version}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update C# package version
    csproj = repo_root / "packages/csharp/HtmlToMarkdown/HtmlToMarkdown.csproj"
    if csproj.exists():
        changed, old_ver, new_ver = update_csproj_version(csproj, version)
        rel_path = csproj.relative_to(repo_root)
        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update Java pom version
    pom = repo_root / "packages/java/pom.xml"
    if pom.exists():
        changed, old_ver, new_ver = update_pom_version(pom, version)
        rel_path = pom.relative_to(repo_root)
        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update Java smoke test pom to keep dependency versions aligned
    smoke_pom = repo_root / "examples/java-smoke/pom.xml"
    if smoke_pom.exists():
        changed, old_ver, new_ver = update_pom_version(smoke_pom, version)
        rel_path = smoke_pom.relative_to(repo_root)
        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update uv.lock version pin
    uv_lock = repo_root / "uv.lock"
    if uv_lock.exists():
        changed, old_ver, new_ver = update_uv_lock(uv_lock, version)
        rel_path = uv_lock.relative_to(repo_root)
        if changed:
            print(f"✓ {rel_path}: {old_ver} → {new_ver}")
            updated_files.append(str(rel_path))
        else:
            unchanged_files.append(str(rel_path))

    # Update Cargo.toml files that don't use workspace version
    print()
    for cargo_toml in repo_root.rglob("Cargo.toml"):
        # Skip the workspace Cargo.toml
        if cargo_toml == repo_root / "Cargo.toml":
            continue
        # Skip target directories
        if "target" in cargo_toml.parts:
            continue

        content = cargo_toml.read_text()
        # Only process if it has a hardcoded version (not workspace)
        if re.search(r'^version\s*=\s*"[^"]+"', content, re.MULTILINE) and "version.workspace = true" not in content:
            changed, old_ver, new_ver = update_cargo_toml(cargo_toml, version)
            rel_path = cargo_toml.relative_to(repo_root)

            if changed:
                print(f"✓ {rel_path}: {old_ver} → {new_ver}")
                updated_files.append(str(rel_path))
            else:
                unchanged_files.append(str(rel_path))

    # Update html-to-markdown-rs dependency pins across Cargo manifests (including the workspace root)
    for cargo_toml in repo_root.rglob("Cargo.toml"):
        if "target" in cargo_toml.parts:
            continue

        if update_rust_dependency_versions(cargo_toml, version):
            rel_path = cargo_toml.relative_to(repo_root)
            print(f"✓ {rel_path}: updated html-to-markdown-rs dependency → {version}")
            updated_files.append(str(rel_path))

    # Summary
    print("\n📊 Summary:")
    print(f"   Updated: {len(updated_files)} files")
    print(f"   Unchanged: {len(unchanged_files)} files")

    if updated_files:
        print(f"\n✨ Version sync complete! All files now at {version}\n")
    else:
        print(f"\n✨ All files already at {version}\n")


if __name__ == "__main__":
    main()

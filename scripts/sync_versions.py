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
from dataclasses import dataclass
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


@dataclass
class SyncReport:
    updated: list[str]
    unchanged: list[str]

    def record(self, rel_path: Path, changed: bool, detail: str | None = None) -> None:
        if changed:
            if detail:
                print(f"✓ {rel_path}: {detail}")
            else:
                print(f"✓ {rel_path}")
            self.updated.append(str(rel_path))
        else:
            self.unchanged.append(str(rel_path))


def sync_package_jsons(repo_root: Path, version: str, report: SyncReport) -> None:
    """Sync package.json files, skipping build artifacts/deps."""
    for pkg_json in repo_root.rglob("package.json"):
        if any(part in pkg_json.parts for part in ["node_modules", ".git", "target"]):
            continue
        if "wasm" in str(pkg_json) and any(part.startswith("dist") for part in pkg_json.parts):
            continue

        changed, old_ver, new_ver = update_package_json(pkg_json, version)
        report.record(pkg_json.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


def sync_pyprojects(repo_root: Path, version: str, report: SyncReport) -> None:
    for pyproject in [
        repo_root / "packages/python/pyproject.toml",
        repo_root / "crates/html-to-markdown-py/pyproject.toml",
    ]:
        if pyproject.exists():
            changed, old_ver, new_ver = update_pyproject_toml(pyproject, version)
            report.record(pyproject.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


def sync_ruby(repo_root: Path, version: str, report: SyncReport) -> None:
    ruby_version = repo_root / "packages/ruby/lib/html_to_markdown/version.rb"
    if ruby_version.exists():
        changed, old_ver, new_ver = update_ruby_version(ruby_version, version)
        report.record(ruby_version.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")

    gemfile_lock = repo_root / "packages/ruby/Gemfile.lock"
    if gemfile_lock.exists():
        changed, old_ver, new_ver = update_gemfile_lock(gemfile_lock, version)
        report.record(gemfile_lock.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


def sync_python_version_file(repo_root: Path, version: str, report: SyncReport) -> None:
    python_version_file = repo_root / "packages/python/html_to_markdown/__init__.py"
    if python_version_file.exists():
        changed, old_ver, new_ver = update_python_version_file(python_version_file, version)
        report.record(python_version_file.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


def sync_mix(repo_root: Path, version: str, report: SyncReport) -> None:
    mix_exs = repo_root / "packages/elixir/mix.exs"
    if mix_exs.exists():
        changed, old_ver, new_ver = update_mix_version(mix_exs, version)
        report.record(mix_exs.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


def sync_node_binding(repo_root: Path, version: str, report: SyncReport) -> None:
    node_binding_index = repo_root / "crates/html-to-markdown-node/index.js"
    if node_binding_index.exists():
        changed, _, _ = update_node_binding_version(node_binding_index, version)
        report.record(
            node_binding_index.relative_to(repo_root), changed, f"updated binding version guards to {version}"
        )


def sync_csproj(repo_root: Path, version: str, report: SyncReport) -> None:
    csproj = repo_root / "packages/csharp/HtmlToMarkdown/HtmlToMarkdown.csproj"
    if csproj.exists():
        changed, old_ver, new_ver = update_csproj_version(csproj, version)
        report.record(csproj.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


def sync_poms(repo_root: Path, version: str, report: SyncReport) -> None:
    for pom in [repo_root / "packages/java/pom.xml", repo_root / "examples/java-smoke/pom.xml"]:
        if pom.exists():
            changed, old_ver, new_ver = update_pom_version(pom, version)
            report.record(pom.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


def sync_uv_lock(repo_root: Path, version: str, report: SyncReport) -> None:
    uv_lock = repo_root / "uv.lock"
    if uv_lock.exists():
        changed, old_ver, new_ver = update_uv_lock(uv_lock, version)
        report.record(uv_lock.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


def sync_cargo_versions(repo_root: Path, version: str, report: SyncReport) -> None:
    for cargo_toml in repo_root.rglob("Cargo.toml"):
        if "target" in cargo_toml.parts:
            continue
        if cargo_toml == repo_root / "Cargo.toml":
            continue

        content = cargo_toml.read_text()
        has_hardcoded = re.search(r'^version\s*=\s*"[^"]+"', content, re.MULTILINE)
        if has_hardcoded and "version.workspace = true" not in content:
            changed, old_ver, new_ver = update_cargo_toml(cargo_toml, version)
            report.record(cargo_toml.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")

    for cargo_toml in repo_root.rglob("Cargo.toml"):
        if "target" in cargo_toml.parts:
            continue
        if update_rust_dependency_versions(cargo_toml, version):
            report.record(
                cargo_toml.relative_to(repo_root), True, f"updated html-to-markdown-rs dependency → {version}"
            )


def summarize(version: str, report: SyncReport) -> None:
    print("\n📊 Summary:")
    print(f"   Updated: {len(report.updated)} files")
    print(f"   Unchanged: {len(report.unchanged)} files")

    if report.updated:
        print(f"\n✨ Version sync complete! All files now at {version}\n")
    else:
        print(f"\n✨ All files already at {version}\n")


def main() -> None:
    repo_root = get_repo_root()

    try:
        version = get_workspace_version(repo_root)
    except (FileNotFoundError, ValueError) as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"\n📦 Syncing version {version} from Cargo.toml\n")

    report = SyncReport(updated=[], unchanged=[])
    sync_package_jsons(repo_root, version, report)
    sync_pyprojects(repo_root, version, report)
    sync_ruby(repo_root, version, report)
    sync_python_version_file(repo_root, version, report)
    sync_mix(repo_root, version, report)
    sync_node_binding(repo_root, version, report)
    sync_csproj(repo_root, version, report)
    sync_poms(repo_root, version, report)
    sync_uv_lock(repo_root, version, report)
    sync_cargo_versions(repo_root, version, report)
    summarize(version, report)


if __name__ == "__main__":
    main()

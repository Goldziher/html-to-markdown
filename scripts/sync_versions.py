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
from collections.abc import Callable
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


# ============================================================================
# Generic Version Update Helpers
# ============================================================================


def _extract_version_regex(content: str, pattern: str) -> str:
    """Extract version from content using regex pattern with capturing group."""
    match = re.search(pattern, content)
    return match.group(1) if match else "NOT FOUND"


def _update_via_regex(
    file_path: Path, pattern: str, replacement_fn: Callable[[re.Match[str]], str] | str, count: int = 1
) -> tuple[bool, str, str]:
    """
    Update version via regex pattern matching.

    Args:
        file_path: Path to file to update
        pattern: Regex pattern with capture group for old version
        replacement_fn: Either a format string (\\1, \\2, etc.) or callable(match) -> str
        count: Max number of replacements

    Returns: (changed, old_version, new_version)
    """
    content = file_path.read_text()
    old_version = _extract_version_regex(content, pattern)

    if isinstance(replacement_fn, str):
        new_content, num_replaced = re.subn(pattern, replacement_fn, content, count=count)
    else:
        new_content, num_replaced = re.subn(pattern, replacement_fn, content, count=count)

    changed = num_replaced > 0 and new_content != content
    if changed:
        file_path.write_text(new_content)

    return changed, old_version, old_version  # old_version used as placeholder until actual new version is extracted


def _update_single_regex_field(
    file_path: Path, field_pattern: str, version: str, quote_char: str = '"', count: int = 1
) -> tuple[bool, str, str]:
    """
    Update single field via regex (e.g., "version = "X.Y.Z"").

    Args:
        file_path: Path to file
        field_pattern: Pattern like r'^version\\s*=' (capture group added automatically)
        version: New version value
        quote_char: Quote character to use (default: ")
        count: Max replacements

    Returns: (changed, old_version, new_version)
    """
    content = file_path.read_text()
    # Build full pattern with capture group for extraction
    quote_esc = re.escape(quote_char)
    extract_pattern = field_pattern + rf"\s*{quote_esc}([^{quote_esc}]+){quote_esc}"
    old_version = _extract_version_regex(content, extract_pattern)

    if old_version == version:
        return False, old_version, version

    # Build replacement pattern that captures the field and replaces the quoted version
    replacement_pattern = f"({field_pattern})" + rf"\s*{quote_esc}[^{quote_esc}]+{quote_esc}"
    replacement_text = rf"\1{quote_char}{version}{quote_char}"
    new_content = re.sub(replacement_pattern, replacement_text, content, count=count, flags=re.MULTILINE)

    if new_content != content:
        file_path.write_text(new_content)
        return True, old_version, version

    return False, old_version, version


def _update_json_field(file_path: Path, field: str, version: str) -> tuple[bool, str, str]:
    """Update a JSON field (e.g., "version" in package.json)."""
    data = json.loads(file_path.read_text())
    old_version = data.get(field, "N/A")
    changed = False

    if data.get(field) != version:
        data[field] = version
        changed = True

    if changed:
        file_path.write_text(json.dumps(data, indent=2) + "\n")

    return changed, old_version, version


def _update_json_dependency(file_path: Path, package_name: str, version_spec: str) -> None:
    """Update dependency version in JSON files (package.json, composer.json)."""
    data = json.loads(file_path.read_text())

    # Check all possible dependency fields
    for dep_type in ["dependencies", "optionalDependencies", "devDependencies", "require"]:
        if dep_type in data and package_name in data[dep_type]:
            data[dep_type][package_name] = version_spec

    file_path.write_text(json.dumps(data, indent=2) + "\n")


def update_package_json(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update a package.json file."""
    return _update_json_field(file_path, "version", version)


def update_pyproject_toml(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update a pyproject.toml file."""
    return _update_single_regex_field(file_path, r"^version\s*=", version, quote_char='"')


def update_ruby_version(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update Ruby version.rb file."""
    return _update_single_regex_field(file_path, r"VERSION\s*=", version, quote_char="'")


def update_cargo_toml(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update a Cargo.toml file that has hardcoded version (not using workspace)."""
    return _update_single_regex_field(file_path, r"^version\s*=", version, quote_char='"')


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
    """Update html-to-markdown version in Gemfile.lock."""
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
    """Update __version__ in Python __init__.py files."""
    content = file_path.read_text()
    match = re.search(r'(__version__\s*=\s*)"([^"]+)"', content)
    old_version = match.group(2) if match else "NOT FOUND"

    if old_version == version:
        return False, old_version, version

    new_content = re.sub(r'(__version__\s*=\s*)"([^"]+)"', rf'\1"{version}"', content, count=1)
    file_path.write_text(new_content)
    return True, old_version, version


def update_node_binding_version(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update version checks in Node binding index.js file."""
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
    """Update html-to-markdown version in uv.lock file."""
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


def update_composer_json(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update a composer.json file's version field."""
    return _update_json_field(file_path, "version", version)


def update_json_dependency(file_path: Path, package_name: str, version_spec: str) -> None:
    """Update dependency version in JSON files (package.json, composer.json)."""
    _update_json_dependency(file_path, package_name, version_spec)


def update_toml_dependency(file_path: Path, package_name: str, version_spec: str) -> None:
    """Update dependency version in pyproject.toml."""
    content = file_path.read_text()
    pattern = re.compile(rf'({re.escape(package_name)}\s*=\s*)"[^"]+"')
    updated = pattern.sub(rf'\1"{version_spec}"', content)
    file_path.write_text(updated)


def update_gemfile_dependency(gemfile_path: Path, gem_name: str, version: str) -> None:
    """Update gem version in Gemfile."""
    content = gemfile_path.read_text()
    pattern = re.compile(rf"gem\s+['\"]?{re.escape(gem_name)}['\"]?.*")
    replacement = f"gem '{gem_name}', '{version}'"
    updated = pattern.sub(replacement, content)
    gemfile_path.write_text(updated)


def update_go_mod(go_mod_path: Path, module_path: str, version: str) -> None:
    """Update module version in go.mod."""
    content = go_mod_path.read_text()
    pattern = re.compile(rf"{re.escape(module_path)}\s+v[\d\.]+")
    replacement = f"{module_path} v{version}"
    updated = pattern.sub(replacement, content)
    go_mod_path.write_text(updated)


def update_pom_dependency(pom_path: Path, group_id: str, artifact_id: str, version: str) -> None:
    """Update dependency version in pom.xml."""
    content = pom_path.read_text()
    pattern = re.compile(
        rf"(<dependency>.*?<groupId>{re.escape(group_id)}</groupId>.*?<artifactId>{re.escape(artifact_id)}</artifactId>.*?<version>).*?(</version>.*?</dependency>)",
        re.DOTALL,
    )
    replacement = rf"\g<1>{version}\g<2>"
    updated = pattern.sub(replacement, content)
    pom_path.write_text(updated)


def update_csproj_dependency(csproj_path: Path, package_name: str, version: str) -> None:
    """Update PackageReference version in .csproj."""
    content = csproj_path.read_text()
    pattern = re.compile(rf'(<PackageReference\s+Include="{re.escape(package_name)}"\s+Version=")[^"]+(")')
    replacement = rf"\g<1>{version}\g<2>"
    updated = pattern.sub(replacement, content)
    csproj_path.write_text(updated)


def update_mix_dependency(mix_path: Path, package_name: str, version: str) -> None:
    """Update dependency version in mix.exs."""
    content = mix_path.read_text()
    pattern = re.compile(rf'(\{{{re.escape(package_name)},\s*"~>\s*)[^"]+("}})')
    replacement = rf"\g<1>{version}\g<2>"
    updated = pattern.sub(replacement, content)
    mix_path.write_text(updated)


def update_mix_version(file_path: Path, version: str) -> tuple[bool, str, str]:
    """Update @version declarations inside mix.exs files."""
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


def update_test_apps_versions(repo_root: Path, version: str) -> None:
    """Update test_apps package manifests with new version."""
    test_apps_dir = repo_root / "tests" / "test_apps"

    if not test_apps_dir.exists():
        print("⚠️  test_apps directory not found, skipping")
        return

    print("\n📦 Updating test_apps manifests...")

    # Update Python pyproject.toml
    python_toml = test_apps_dir / "python" / "pyproject.toml"
    if python_toml.exists():
        update_toml_dependency(python_toml, "html-to-markdown", f">={version}")
        print(f"  ✓ Python pyproject.toml → html-to-markdown>={version}")

    # Update Node package.json
    node_pkg = test_apps_dir / "node" / "package.json"
    if node_pkg.exists():
        update_json_dependency(node_pkg, "html-to-markdown", f">={version}")
        print(f"  ✓ Node package.json → html-to-markdown>={version}")

    # Update Ruby Gemfile
    ruby_gemfile = test_apps_dir / "ruby" / "Gemfile"
    if ruby_gemfile.exists():
        update_gemfile_dependency(ruby_gemfile, "html-to-markdown", f">= {version}")
        print(f"  ✓ Ruby Gemfile → html-to-markdown>={version}")

    # Update PHP composer.json
    php_composer = test_apps_dir / "php" / "composer.json"
    if php_composer.exists():
        update_json_dependency(php_composer, "kreuzberg-dev/html-to-markdown", f">={version}")
        print(f"  ✓ PHP composer.json → kreuzberg-dev/html-to-markdown>={version}")

    # Update Go go.mod
    go_mod = test_apps_dir / "go" / "go.mod"
    if go_mod.exists():
        update_go_mod(go_mod, "github.com/kreuzberg-dev/html-to-markdown/packages/go/v2", version)
        print(f"  ✓ Go go.mod → v{version}")

    # Update Java pom.xml
    java_pom = test_apps_dir / "java" / "pom.xml"
    if java_pom.exists():
        update_pom_dependency(java_pom, "dev.kreuzberg", "html-to-markdown", version)
        print(f"  ✓ Java pom.xml → {version}")

    # Update C# TestApp.csproj
    csharp_csproj = test_apps_dir / "csharp" / "TestApp.csproj"
    if csharp_csproj.exists():
        update_csproj_dependency(csharp_csproj, "HtmlToMarkdown", version)
        print(f"  ✓ C# TestApp.csproj → {version}")

    # Update Elixir mix.exs
    elixir_mix = test_apps_dir / "elixir" / "mix.exs"
    if elixir_mix.exists():
        update_mix_dependency(elixir_mix, "html_to_markdown", version)
        print(f"  ✓ Elixir mix.exs → ~> {version}")


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


def sync_composer(repo_root: Path, version: str, report: SyncReport) -> None:
    # Update root composer.json (for Packagist)
    root_composer = repo_root / "composer.json"
    if root_composer.exists():
        changed, old_ver, new_ver = update_composer_json(root_composer, version)
        report.record(root_composer.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")

    # Update package composer.json
    composer = repo_root / "packages/php/composer.json"
    if composer.exists():
        changed, old_ver, new_ver = update_composer_json(composer, version)
        report.record(composer.relative_to(repo_root), changed, f"{old_ver} → {new_ver}")


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
    sync_composer(repo_root, version, report)
    sync_cargo_versions(repo_root, version, report)

    # Update test_apps manifests
    update_test_apps_versions(repo_root, version)

    summarize(version, report)


if __name__ == "__main__":
    main()

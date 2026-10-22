"""Sync release pins in maintained consumer manifests without replacing their contents."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

import tomllib

CONSUMER_PINS = (
    ("test_apps/dart/pubspec.yaml", r"(h2m: \^)[^\s]+", 1),
    ("test_apps/elixir/mix.exs", r'(html_to_markdown, "~> )[^\"]+', 1),
    ("test_apps/kotlin_android/build.gradle.kts", r'(io.xberg:html-to-markdown-android:)[^"\s]+', 2),
    ("e2e/go/go.mod", r"(github\.com/xberg-io/html-to-markdown/packages/go/v3[ \t]+v)[^\s]+", 1),
)


def synchronize(root: Path, *, check: bool = False) -> bool:
    """Update all pins after validating every match; return whether pins already matched."""
    version = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    changes = []
    for relative_path, pattern, expected_count in CONSUMER_PINS:
        path = root / relative_path
        original = path.read_text()
        updated, count = re.subn(pattern, lambda match: match[1] + version, original)
        if count != expected_count:
            raise ValueError(f"syncing {relative_path}: expected {expected_count} release pins, found {count}")
        if original != updated:
            changes.append((path, updated))
    for path, updated in changes:
        if check:
            print(f"stale consumer version: {path.relative_to(root)}")
        else:
            path.write_text(updated)
    return not changes


def main() -> int:
    """Apply the canonical version, or report drift for the release gate."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    current = synchronize(Path(__file__).resolve().parent.parent, check=arguments.check)
    return int(arguments.check and not current)


if __name__ == "__main__":
    raise SystemExit(main())

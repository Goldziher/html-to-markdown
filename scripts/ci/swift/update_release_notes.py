"""Preserve release notes while replacing only Swift installation metadata."""

import argparse
import re
import subprocess
import tempfile
from pathlib import Path


def upsert(notes: str, version: str, checksum: str) -> str:
    """Replace marked or legacy Swift metadata without consuming neighboring markers."""
    if re.fullmatch(r"[0-9a-f]{64}", checksum) is None:
        raise ValueError("Expected one lowercase SHA256 checksum")
    if re.fullmatch(r"\d+\.\d+\.\d+(?:-[\w.-]+)?", version) is None:
        raise ValueError("Expected a release version")
    start = "<!-- swift-package-manager -->"
    end = "<!-- /swift-package-manager -->"
    url = f"https://github.com/xberg-io/html-to-markdown/releases/download/v{version}/HtmlToMarkdown-rs.artifactbundle.zip"
    section = f"""{start}
## Swift Package Manager

Add to your `Package.swift`:

```swift
.package(url: "https://github.com/xberg-io/html-to-markdown", from: "{version}")
```

The Swift binding is distributed as a pre-built artifact bundle. No Rust toolchain required.

**Artifact**: [HtmlToMarkdown-rs.artifactbundle.zip]({url})
**Checksum**: `{checksum}`
{end}
"""
    if start in notes or end in notes:
        if notes.count(start) != 1 or notes.count(end) != 1 or notes.index(end) < notes.index(start):
            raise ValueError("Malformed Swift release-notes markers")
        return notes[: notes.index(start)] + section.rstrip("\n") + notes[notes.index(end) + len(end) :]
    legacy = re.search(r"(?m)^## Swift Package Manager\s*$", notes)
    if legacy:
        following = re.search(r"(?m)^(?:#{1,2} |<!--)", notes[legacy.end() :])
        stop = legacy.end() + following.start() if following else len(notes)
        return notes[: legacy.start()] + section + "\n" + notes[stop:]
    return notes + ("\n\n" if notes and not notes.endswith("\n\n") else "") + section


def update(checksum: str, version: str) -> None:
    """Fetch current notes before updating the Swift section."""
    command = ["gh", "release"]
    notes = subprocess.run(
        [*command, "view", f"v{version}", "--repo", "xberg-io/html-to-markdown", "--json", "body", "--jq", ".body"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    updated = upsert(notes, version, checksum)
    if notes == updated:
        return
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "notes.md"
        path.write_text(updated, encoding="utf-8")
        subprocess.run(
            [*command, "edit", f"v{version}", "--repo", "xberg-io/html-to-markdown", "--notes-file", str(path)],
            check=True,
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checksum")
    parser.add_argument("version")
    arguments = parser.parse_args()
    update(arguments.checksum, arguments.version)

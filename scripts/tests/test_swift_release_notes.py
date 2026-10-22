"""Exercise the actual Swift release-notes workflow against local release bodies."""

import os
import shutil
import subprocess
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[2]
ZIG = "<!-- zig-fetch -->\n## Zig\n\nKeep this archive hash.\n"
CHANGELOG = "## Changes\n\nFix #929 and security advisory.\n"


@pytest.mark.parametrize(
    "existing", [CHANGELOG + "\n" + ZIG, CHANGELOG + "\n## Swift Package Manager\n\nOld checksum\n\n" + ZIG]
)
def test_workflow_preserves_notes_and_updates_swift_idempotently(tmp_path: Path, existing: str) -> None:
    """A real workflow execution retains unrelated sections across recovery runs."""
    workflow = yaml.safe_load((ROOT / ".github/workflows/publish.yaml").read_text())
    steps = workflow["jobs"]["update-swift-package-manifest"]["steps"]
    script = next(step["run"] for step in steps if step.get("name") == "Update release notes with artifact URL")
    tooling = tmp_path / ".release-tooling"
    shutil.copytree(ROOT / "scripts", tooling / "scripts")
    shutil.copy(ROOT / "Taskfile.yaml", tooling / "Taskfile.yaml")
    body = tmp_path / "body.md"
    body.write_text(existing)
    executable = tmp_path / "gh"
    executable.write_text(
        "#!/usr/bin/env python3\nimport os, sys\nfrom pathlib import Path\n"
        'body = Path(os.environ["TEST_RELEASE_BODY"])\n'
        'if sys.argv[1:3] == ["release", "view"]:\n    sys.stdout.write(body.read_text())\n'
        'elif sys.argv[1:3] == ["release", "edit"]:\n'
        '    body.write_text(Path(sys.argv[sys.argv.index("--notes-file") + 1]).read_text())\n'
        "else:\n    raise SystemExit(2)\n"
    )
    executable.chmod(0o755)
    checksums = tmp_path / "swift-checksums"
    checksums.mkdir()
    checksum_file = checksums / "bundle.checksum"
    environment = {
        **os.environ,
        "PATH": f"{tmp_path}:{os.environ['PATH']}",
        "VERSION": "3.18.7",
        "TEST_RELEASE_BODY": str(body),
    }
    previous = ""
    for checksum in ("a" * 64, "b" * 64, "b" * 64):
        checksum_file.write_text(checksum)
        result = subprocess.run(
            ["bash", "-euc", script], cwd=tmp_path, env=environment, capture_output=True, text=True, check=False
        )
        assert result.returncode == 0, result.stderr
        actual = body.read_text()
        assert CHANGELOG in actual
        assert ZIG in actual
        assert actual.count("## Swift Package Manager") == 1
        assert f"**Checksum**: `{checksum}`" in actual
        if checksum == "b" * 64 and previous:
            assert "a" * 64 not in actual
        if previous and checksum == "b" * 64 and "b" * 64 in previous:
            assert actual == previous
        previous = actual


@pytest.mark.parametrize(
    ("notes", "version", "checksum"),
    [
        ("<!-- swift-package-manager -->", "3.18.7", "a" * 64),
        ("<!-- /swift-package-manager -->", "3.18.7", "a" * 64),
        ("", "../bad", "a" * 64),
        ("", "3.18.7", "bad"),
    ],
)
def test_upsert_rejects_malformed_metadata(notes: str, version: str, checksum: str) -> None:
    """Invalid input must fail before replacing the release body."""
    from ci.swift.update_release_notes import upsert

    with pytest.raises(ValueError):
        upsert(notes, version, checksum)


def test_marked_swift_update_preserves_surrounding_bytes() -> None:
    """Existing markers make future metadata replacement bounded and repeatable."""
    from ci.swift.update_release_notes import upsert

    original = CHANGELOG + "<!-- swift-package-manager -->\nOld Swift\n<!-- /swift-package-manager -->\n\n" + ZIG
    updated = upsert(original, "3.18.7", "b" * 64)
    assert updated.startswith(CHANGELOG + "<!-- swift-package-manager -->\n")
    assert updated.endswith("<!-- /swift-package-manager -->\n\n" + ZIG)
    assert upsert(updated, "3.18.7", "b" * 64) == updated
    assert "Old Swift" not in updated


def test_swift_notes_wait_for_zig_notes_updater() -> None:
    """Both read-modify-write operations must finish in a fixed order."""
    workflow = yaml.safe_load((ROOT / ".github/workflows/publish.yaml").read_text())
    job = workflow["jobs"]["update-swift-package-manifest"]
    assert "publish-zig" in job["needs"]
    assert "!cancelled()" in job["if"]

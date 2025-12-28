"""Comprehensive tests using shared fixtures."""

import json
from pathlib import Path
from typing import Any

import pytest

from html_to_markdown import convert_html_to_markdown


def load_fixtures(filename: str) -> list[dict[str, Any]]:
    """Load test fixtures from JSON file."""
    fixture_path = Path(__file__).parent.parent / "fixtures" / filename
    with Path(fixture_path).open() as f:
        return json.load(f)


@pytest.fixture
def basic_fixtures() -> list[dict[str, Any]]:
    """Load basic HTML fixtures."""
    return load_fixtures("basic-html.json")


@pytest.mark.parametrize("test_case", load_fixtures("basic-html.json"), ids=lambda tc: tc["name"])
def test_basic_html_conversion(test_case: dict[str, Any]) -> None:
    """Test basic HTML conversions from fixtures."""
    result = convert_html_to_markdown(test_case["html"])
    expected = test_case["expectedMarkdown"]

    # Normalize whitespace for comparison
    assert result.strip() == expected.strip()  # noqa: S101

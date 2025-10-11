import pytest

from html_to_markdown import markdownify


@pytest.mark.filterwarnings("ignore::DeprecationWarning")
def test_legacy_name() -> None:
    assert markdownify("<b>text</b>") == "**text**\n"

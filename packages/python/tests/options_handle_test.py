from html_to_markdown import ConversionOptions, convert_with_handle, create_options_handle


def test_convert_with_handle_uses_reusable_options() -> None:
    handle = create_options_handle(ConversionOptions(heading_style="atx_closed"))
    markdown = convert_with_handle("<h1>Hello</h1>", handle)
    assert "# Hello #" in markdown

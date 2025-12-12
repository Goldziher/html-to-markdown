from html_to_markdown import convert


def test_script_with_angle_brackets_does_not_swallow_following_content() -> None:
    html = """
    <html>
    <script>1 < 2</script>
    <body>Content</body>
    </html>
    """
    result = convert(html)
    assert "Content" in result


def test_script_with_string_angles_is_ignored() -> None:
    html = """
    <div>before</div>
    <script type="text/javascript">const msg = "<tag>";</script>
    <p>after</p>
    """
    result = convert(html)
    assert "before" in result
    assert "after" in result
    assert "<tag>" not in result


def test_bare_angle_brackets_in_html_body() -> None:
    """Test that bare angle brackets like '1<2' in HTML body don't break parsing."""
    html = """
    <html>
    1<2
    Content
    </html>
    """
    result = convert(html)
    assert "Content" in result, f"Expected 'Content' in result, got: {result}"


def test_angle_brackets_in_div() -> None:
    """Test angle brackets inside div tags."""
    html = """
    <html>
    <div>1<2</div>
    <div>Content</div>
    </html>
    """
    result = convert(html)
    assert "Content" in result, f"Expected 'Content' in result, got: {result}"


def test_multiple_angle_brackets() -> None:
    """Test multiple angle brackets in sequence."""
    html = """
    <html>
    1 < 2 < 3
    <p>Content</p>
    </html>
    """
    result = convert(html)
    assert "Content" in result, f"Expected 'Content' in result, got: {result}"


def test_angle_brackets_with_script_tag() -> None:
    """Test the original issue #94 case with script tag."""
    html = """
    <html>
    <script>
    if (1 < 2) {
    // "test";
    }
    </script>
    Content
    </html>
    """
    result = convert(html)
    assert "Content" in result, f"Expected 'Content' in result, got: {result}"


def test_angle_bracket_at_tag_boundary() -> None:
    """Test angle bracket immediately after tag."""
    html = "<p>test</p>1<2"
    result = convert(html)
    assert "test" in result

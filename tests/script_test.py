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

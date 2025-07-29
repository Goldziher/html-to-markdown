"""Tests for media element functionality (audio, video, iframe)."""

from html_to_markdown import convert_to_markdown


def test_audio_basic() -> None:
    """Test basic audio element conversion."""
    html = '<audio src="audio.mp3"></audio>'
    result = convert_to_markdown(html)
    # Audio elements with src could be converted to links
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_audio_with_controls() -> None:
    """Test audio element with controls attribute."""
    html = '<audio src="audio.mp3" controls></audio>'
    result = convert_to_markdown(html)
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_audio_with_all_attributes() -> None:
    """Test audio element with all common attributes."""
    html = '<audio src="audio.mp3" controls autoplay loop muted preload="auto"></audio>'
    result = convert_to_markdown(html)
    # Attributes are not preserved in Markdown
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_audio_with_source_element() -> None:
    """Test audio with source child element."""
    html = """<audio controls>
    <source src="audio.mp3" type="audio/mpeg">
    <source src="audio.ogg" type="audio/ogg">
</audio>"""
    result = convert_to_markdown(html)
    # Takes first source
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_audio_with_fallback_content() -> None:
    """Test audio element with fallback content."""
    html = '<audio src="audio.mp3" controls>Your browser does not support the audio element.</audio>'
    result = convert_to_markdown(html)
    # Shows fallback content with link
    expected = "[audio.mp3](audio.mp3)\n\nYour browser does not support the audio element.\n\n"
    assert result == expected


def test_audio_without_src() -> None:
    """Test audio element without src attribute."""
    html = "<audio controls></audio>"
    result = convert_to_markdown(html)
    # No src, no content
    assert result == ""


def test_video_basic() -> None:
    """Test basic video element conversion."""
    html = '<video src="video.mp4"></video>'
    result = convert_to_markdown(html)
    # Video elements with src converted to links
    assert result == "[video.mp4](video.mp4)\n\n"


def test_video_with_dimensions() -> None:
    """Test video element with width and height."""
    html = '<video src="video.mp4" width="640" height="480"></video>'
    result = convert_to_markdown(html)
    # Dimensions are not preserved in Markdown
    assert result == "[video.mp4](video.mp4)\n\n"


def test_video_with_all_attributes() -> None:
    """Test video element with all common attributes."""
    html = '<video src="video.mp4" width="640" height="480" poster="poster.jpg" controls autoplay loop muted preload="metadata"></video>'
    result = convert_to_markdown(html)
    # All attributes are lost in Markdown
    assert result == "[video.mp4](video.mp4)\n\n"


def test_video_with_source_element() -> None:
    """Test video with source child element."""
    html = """<video controls width="640">
    <source src="video.mp4" type="video/mp4">
    <source src="video.webm" type="video/webm">
</video>"""
    result = convert_to_markdown(html)
    # Takes first source
    assert result == "[video.mp4](video.mp4)\n\n"


def test_video_with_fallback_content() -> None:
    """Test video element with fallback content."""
    html = '<video src="video.mp4" controls>Your browser does not support the video element.</video>'
    result = convert_to_markdown(html)
    # Shows fallback content with link
    expected = "[video.mp4](video.mp4)\n\nYour browser does not support the video element.\n\n"
    assert result == expected


def test_video_with_track_elements() -> None:
    """Test video with track elements (subtitles, captions)."""
    html = """<video src="video.mp4" controls>
    <track src="subtitles_en.vtt" kind="subtitles" srclang="en" label="English">
    <track src="subtitles_es.vtt" kind="subtitles" srclang="es" label="Spanish">
</video>"""
    result = convert_to_markdown(html)
    # Track elements are not preserved
    assert result == "[video.mp4](video.mp4)\n\n"


def test_iframe_basic() -> None:
    """Test basic iframe element conversion."""
    html = '<iframe src="https://example.com"></iframe>'
    result = convert_to_markdown(html)
    # Iframe converted to link
    assert result == "[https://example.com](https://example.com)\n\n"


def test_iframe_with_dimensions() -> None:
    """Test iframe element with width and height."""
    html = '<iframe src="https://example.com" width="800" height="600"></iframe>'
    result = convert_to_markdown(html)
    # Dimensions not preserved
    assert result == "[https://example.com](https://example.com)\n\n"


def test_iframe_with_all_attributes() -> None:
    """Test iframe element with all common attributes."""
    html = '<iframe src="https://example.com" width="800" height="600" title="Example Frame" allow="fullscreen" sandbox="allow-scripts" loading="lazy"></iframe>'
    result = convert_to_markdown(html)
    # All attributes lost, only link remains
    assert result == "[https://example.com](https://example.com)\n\n"


def test_iframe_youtube_embed() -> None:
    """Test iframe for YouTube embed."""
    html = '<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" title="YouTube video player" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>'
    result = convert_to_markdown(html)
    # YouTube embed becomes a link
    assert result == "[https://www.youtube.com/embed/dQw4w9WgXcQ](https://www.youtube.com/embed/dQw4w9WgXcQ)\n\n"


def test_iframe_with_sandbox_boolean() -> None:
    """Test iframe with sandbox as boolean attribute."""
    html = '<iframe src="https://example.com" sandbox></iframe>'
    result = convert_to_markdown(html)
    # Sandbox attribute not preserved
    assert result == "[https://example.com](https://example.com)\n\n"


def test_media_in_paragraphs() -> None:
    """Test media elements within paragraphs."""
    html = """<p>Here is an audio file: <audio src="audio.mp3" controls></audio></p>
<p>Here is a video: <video src="video.mp4" controls></video></p>
<p>Here is an iframe: <iframe src="https://example.com"></iframe></p>"""
    result = convert_to_markdown(html)
    expected = """Here is an audio file: [audio.mp3](audio.mp3)

Here is a video: [video.mp4](video.mp4)

Here is an iframe: [https://example.com](https://example.com)

"""
    assert result == expected


def test_nested_media_elements() -> None:
    """Test media elements with nested content."""
    html = """<article>
    <h2>Media Gallery</h2>
    <section>
        <h3>Audio Section</h3>
        <audio src="audio1.mp3" controls>
            <p>Your browser doesn't support HTML5 audio.</p>
        </audio>
    </section>
    <section>
        <h3>Video Section</h3>
        <video src="video1.mp4" controls width="640" height="480">
            <p>Your browser doesn't support HTML5 video.</p>
        </video>
    </section>
</article>"""
    result = convert_to_markdown(html)
    expected = """Media Gallery
-------------

### Audio Section

[audio1.mp3](audio1.mp3)

Your browser doesn't support HTML5 audio.

### Video Section

[video1.mp4](video1.mp4)

Your browser doesn't support HTML5 video.

"""
    assert result == expected


def test_media_inline_mode() -> None:
    """Test media elements in inline mode."""
    html = '<audio src="audio.mp3" controls></audio>'
    result = convert_to_markdown(html, convert_as_inline=True)
    # In inline mode, media elements become links without newlines
    assert result == "[audio.mp3](audio.mp3)"


def test_empty_media_attributes() -> None:
    """Test media elements with empty attributes."""
    html = '<video src="" width="" height=""></video>'
    result = convert_to_markdown(html)
    # No src means no link to create
    assert result == ""


def test_media_with_metadata() -> None:
    """Test media elements work with metadata extraction."""
    html = """<html>
<head>
    <title>Media Page</title>
    <meta name="description" content="Page with media elements">
</head>
<body>
    <audio src="audio.mp3" controls></audio>
    <video src="video.mp4" controls></video>
    <iframe src="https://example.com"></iframe>
</body>
</html>"""
    result = convert_to_markdown(html)
    expected = """<!--
meta-description: Page with media elements
title: Media Page
-->

[audio.mp3](audio.mp3)

[video.mp4](video.mp4)

[https://example.com](https://example.com)

"""
    assert result == expected


def test_audio_no_boolean_attributes() -> None:
    """Test audio element without boolean attributes."""
    html = '<audio src="audio.mp3" controls="false"></audio>'
    result = convert_to_markdown(html)
    # Converts to link regardless of controls attribute
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_video_poster_only() -> None:
    """Test video element with only poster attribute."""
    html = '<video poster="poster.jpg"></video>'
    result = convert_to_markdown(html)
    # No src, no content to show
    assert result == ""

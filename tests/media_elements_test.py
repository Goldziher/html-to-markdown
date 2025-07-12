"""Tests for media element functionality (audio, video, iframe)."""

from html_to_markdown import convert_to_markdown


def test_audio_basic() -> None:
    """Test basic audio element conversion."""
    html = '<audio src="audio.mp3"></audio>'
    result = convert_to_markdown(html)
    assert result == '<audio src="audio.mp3" />\n\n'


def test_audio_with_controls() -> None:
    """Test audio element with controls attribute."""
    html = '<audio src="audio.mp3" controls></audio>'
    result = convert_to_markdown(html)
    assert result == '<audio src="audio.mp3" controls />\n\n'


def test_audio_with_all_attributes() -> None:
    """Test audio element with all common attributes."""
    html = '<audio src="audio.mp3" controls autoplay loop muted preload="auto"></audio>'
    result = convert_to_markdown(html)
    assert result == '<audio src="audio.mp3" controls autoplay loop muted preload="auto" />\n\n'


def test_audio_with_source_element() -> None:
    """Test audio with source child element."""
    html = """<audio controls>
    <source src="audio.mp3" type="audio/mpeg">
    <source src="audio.ogg" type="audio/ogg">
</audio>"""
    result = convert_to_markdown(html)
    assert result == '<audio src="audio.mp3" controls />\n\n'


def test_audio_with_fallback_content() -> None:
    """Test audio element with fallback content."""
    html = '<audio src="audio.mp3" controls>Your browser does not support the audio element.</audio>'
    result = convert_to_markdown(html)
    expected = '<audio src="audio.mp3" controls>\nYour browser does not support the audio element.\n</audio>\n\n'
    assert result == expected


def test_audio_without_src() -> None:
    """Test audio element without src attribute."""
    html = "<audio controls></audio>"
    result = convert_to_markdown(html)
    assert result == "<audio controls />\n\n"


def test_video_basic() -> None:
    """Test basic video element conversion."""
    html = '<video src="video.mp4"></video>'
    result = convert_to_markdown(html)
    assert result == '<video src="video.mp4" />\n\n'


def test_video_with_dimensions() -> None:
    """Test video element with width and height."""
    html = '<video src="video.mp4" width="640" height="480"></video>'
    result = convert_to_markdown(html)
    assert result == '<video src="video.mp4" width="640" height="480" />\n\n'


def test_video_with_all_attributes() -> None:
    """Test video element with all common attributes."""
    html = '<video src="video.mp4" width="640" height="480" poster="poster.jpg" controls autoplay loop muted preload="metadata"></video>'
    result = convert_to_markdown(html)
    expected = '<video src="video.mp4" width="640" height="480" poster="poster.jpg" controls autoplay loop muted preload="metadata" />\n\n'
    assert result == expected


def test_video_with_source_element() -> None:
    """Test video with source child element."""
    html = """<video controls width="640">
    <source src="video.mp4" type="video/mp4">
    <source src="video.webm" type="video/webm">
</video>"""
    result = convert_to_markdown(html)
    assert result == '<video src="video.mp4" width="640" controls />\n\n'


def test_video_with_fallback_content() -> None:
    """Test video element with fallback content."""
    html = '<video src="video.mp4" controls>Your browser does not support the video element.</video>'
    result = convert_to_markdown(html)
    expected = '<video src="video.mp4" controls>\nYour browser does not support the video element.\n</video>\n\n'
    assert result == expected


def test_video_with_track_elements() -> None:
    """Test video with track elements (subtitles, captions)."""
    html = """<video src="video.mp4" controls>
    <track src="subtitles_en.vtt" kind="subtitles" srclang="en" label="English">
    <track src="subtitles_es.vtt" kind="subtitles" srclang="es" label="Spanish">
</video>"""
    result = convert_to_markdown(html)
    assert result == '<video src="video.mp4" controls />\n\n'


def test_iframe_basic() -> None:
    """Test basic iframe element conversion."""
    html = '<iframe src="https://example.com"></iframe>'
    result = convert_to_markdown(html)
    assert result == '<iframe src="https://example.com"></iframe>\n\n'


def test_iframe_with_dimensions() -> None:
    """Test iframe element with width and height."""
    html = '<iframe src="https://example.com" width="800" height="600"></iframe>'
    result = convert_to_markdown(html)
    assert result == '<iframe src="https://example.com" width="800" height="600"></iframe>\n\n'


def test_iframe_with_all_attributes() -> None:
    """Test iframe element with all common attributes."""
    html = '<iframe src="https://example.com" width="800" height="600" title="Example Frame" allow="fullscreen" sandbox="allow-scripts" loading="lazy"></iframe>'
    result = convert_to_markdown(html)
    expected = '<iframe src="https://example.com" width="800" height="600" title="Example Frame" allow="fullscreen" sandbox="allow-scripts" loading="lazy"></iframe>\n\n'
    assert result == expected


def test_iframe_youtube_embed() -> None:
    """Test iframe for YouTube embed."""
    html = '<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" title="YouTube video player" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>'
    result = convert_to_markdown(html)
    expected = '<iframe src="https://www.youtube.com/embed/dQw4w9WgXcQ" width="560" height="315" title="YouTube video player" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"></iframe>\n\n'
    assert result == expected


def test_iframe_with_sandbox_boolean() -> None:
    """Test iframe with sandbox as boolean attribute."""
    html = '<iframe src="https://example.com" sandbox></iframe>'
    result = convert_to_markdown(html)
    assert result == '<iframe src="https://example.com" sandbox></iframe>\n\n'


def test_media_in_paragraphs() -> None:
    """Test media elements within paragraphs."""
    html = """<p>Here is an audio file: <audio src="audio.mp3" controls></audio></p>
<p>Here is a video: <video src="video.mp4" controls></video></p>
<p>Here is an iframe: <iframe src="https://example.com"></iframe></p>"""
    result = convert_to_markdown(html)
    expected = """Here is an audio file: <audio src="audio.mp3" controls />

Here is a video: <video src="video.mp4" controls />

Here is an iframe: <iframe src="https://example.com"></iframe>

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
    expected = """
Media Gallery
-------------

### Audio Section

<audio src="audio1.mp3" controls>
Your browser doesn't support HTML5 audio.
</audio>

### Video Section

<video src="video1.mp4" width="640" height="480" controls>
Your browser doesn't support HTML5 video.
</video>

"""
    assert result == expected


def test_media_inline_mode() -> None:
    """Test media elements in inline mode."""
    html = '<audio src="audio.mp3" controls></audio>'
    result = convert_to_markdown(html, convert_as_inline=True)

    assert result == '<audio src="audio.mp3" controls />'


def test_empty_media_attributes() -> None:
    """Test media elements with empty attributes."""
    html = '<video src="" width="" height=""></video>'
    result = convert_to_markdown(html)
    assert result == "<video />\n\n"


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

<audio src="audio.mp3" controls />

<video src="video.mp4" controls />

<iframe src="https://example.com"></iframe>

"""
    assert result == expected


def test_audio_no_boolean_attributes() -> None:
    """Test audio element without boolean attributes."""
    html = '<audio src="audio.mp3" controls="false"></audio>'
    result = convert_to_markdown(html)

    assert result == '<audio src="audio.mp3" controls />\n\n'


def test_video_poster_only() -> None:
    """Test video element with only poster attribute."""
    html = '<video poster="poster.jpg"></video>'
    result = convert_to_markdown(html)
    assert result == '<video poster="poster.jpg" />\n\n'

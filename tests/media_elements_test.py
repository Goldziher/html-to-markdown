from html_to_markdown import convert_to_markdown


def test_audio_basic() -> None:
    """Test basic audio element conversion."""
    html = '<audio src="audio.mp3"></audio>'
    result = convert_to_markdown(html)
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_audio_with_controls() -> None:
    html = '<audio src="audio.mp3" controls></audio>'
    result = convert_to_markdown(html)
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_audio_with_all_attributes() -> None:
    html = '<audio src="audio.mp3" controls autoplay loop muted preload="auto"></audio>'
    result = convert_to_markdown(html)
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_audio_with_source_element() -> None:
    html = """<audio controls>
    <source src="audio.mp3" type="audio/mpeg">
    <source src="audio.ogg" type="audio/ogg">
</audio>"""
    result = convert_to_markdown(html)
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_audio_with_fallback_content() -> None:
    html = '<audio src="audio.mp3" controls>Your browser does not support the audio element.</audio>'
    result = convert_to_markdown(html)
    expected = "[audio.mp3](audio.mp3)\n\nYour browser does not support the audio element.\n\n"
    assert result == expected


def test_audio_without_src() -> None:
    html = "<audio controls></audio>"
    result = convert_to_markdown(html)
    assert result == ""


def test_video_basic() -> None:
    html = '<video src="video.mp4"></video>'
    result = convert_to_markdown(html)
    assert result == "[video.mp4](video.mp4)\n\n"


def test_video_with_dimensions() -> None:
    html = '<video src="video.mp4" width="640" height="480"></video>'
    result = convert_to_markdown(html)
    assert result == "[video.mp4](video.mp4)\n\n"


def test_video_with_all_attributes() -> None:
    html = '<video src="video.mp4" width="640" height="480" poster="poster.jpg" controls autoplay loop muted preload="metadata"></video>'
    result = convert_to_markdown(html)
    assert result == "[video.mp4](video.mp4)\n\n"


def test_video_with_source_element() -> None:
    html = """<video controls width="640">
    <source src="video.mp4" type="video/mp4">
    <source src="video.webm" type="video/webm">
</video>"""
    result = convert_to_markdown(html)
    assert result == "[video.mp4](video.mp4)\n\n"


def test_video_with_fallback_content() -> None:
    html = '<video src="video.mp4" controls>Your browser does not support the video element.</video>'
    result = convert_to_markdown(html)
    expected = "[video.mp4](video.mp4)\n\nYour browser does not support the video element.\n\n"
    assert result == expected


def test_video_with_track_elements() -> None:
    html = """<video src="video.mp4" controls>
    <track src="subtitles_en.vtt" kind="subtitles" srclang="en" label="English">
    <track src="subtitles_es.vtt" kind="subtitles" srclang="es" label="Spanish">
</video>"""
    result = convert_to_markdown(html)
    assert result == "[video.mp4](video.mp4)\n\n"


def test_iframe_basic() -> None:
    html = '<iframe src="https://example.com"></iframe>'
    result = convert_to_markdown(html)
    assert result == "[https://example.com](https://example.com)\n\n"


def test_iframe_with_dimensions() -> None:
    html = '<iframe src="https://example.com" width="800" height="600"></iframe>'
    result = convert_to_markdown(html)
    assert result == "[https://example.com](https://example.com)\n\n"


def test_iframe_with_all_attributes() -> None:
    html = '<iframe src="https://example.com" width="800" height="600" title="Example Frame" allow="fullscreen" sandbox="allow-scripts" loading="lazy"></iframe>'
    result = convert_to_markdown(html)
    assert result == "[https://example.com](https://example.com)\n\n"


def test_iframe_youtube_embed() -> None:
    html = '<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" title="YouTube video player" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>'
    result = convert_to_markdown(html)
    assert result == "[https://www.youtube.com/embed/dQw4w9WgXcQ](https://www.youtube.com/embed/dQw4w9WgXcQ)\n\n"


def test_iframe_with_sandbox_boolean() -> None:
    html = '<iframe src="https://example.com" sandbox></iframe>'
    result = convert_to_markdown(html)
    assert result == "[https://example.com](https://example.com)\n\n"


def test_media_in_paragraphs() -> None:
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
    html = '<audio src="audio.mp3" controls></audio>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "[audio.mp3](audio.mp3)"


def test_empty_media_attributes() -> None:
    html = '<video src="" width="" height=""></video>'
    result = convert_to_markdown(html)
    assert result == ""


def test_media_with_metadata() -> None:
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
    html = '<audio src="audio.mp3" controls="false"></audio>'
    result = convert_to_markdown(html)
    assert result == "[audio.mp3](audio.mp3)\n\n"


def test_video_poster_only() -> None:
    html = '<video poster="poster.jpg"></video>'
    result = convert_to_markdown(html)
    assert result == ""

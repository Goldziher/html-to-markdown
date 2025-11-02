from __future__ import annotations

from html_to_markdown import (
    InlineImage,
    InlineImageConfig,
    InlineImageWarning,
    convert_with_inline_images,
)


def test_convert_with_inline_images_extracts_data_uri() -> None:
    html = '<img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/x8AAusB9Y9GeVwAAAAASUVORK5CYII=" alt="Pixel">'

    markdown, images, warnings = convert_with_inline_images(
        html, options=None, image_config=InlineImageConfig(max_decoded_size_bytes=256)
    )

    assert "![Pixel](data:image/png;base64," in markdown
    assert len(images) == 1

    image = images[0]
    assert image["format"] == "png"
    assert image["data"].startswith(b"\x89PNG")
    assert image["filename"] is not None
    assert image["source"] == "img_data_uri"
    assert warnings == []


def test_inline_image_typeddicts_are_exposed() -> None:
    image: InlineImage = {
        "data": b"binary",
        "format": "png",
        "filename": None,
        "description": None,
        "dimensions": None,
        "source": "img_data_uri",
        "attributes": {},
    }

    warning: InlineImageWarning = {"index": 0, "message": "test"}

    assert image["format"] == "png"
    assert warning["index"] == 0

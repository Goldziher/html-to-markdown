"""HTML preprocessing using nh3 (ammonia bindings) for improved quality and performance."""

from __future__ import annotations

import re
from typing import Any

import nh3


def preprocess_html(
    html: str,
    *,
    remove_navigation: bool = True,
    remove_forms: bool = True,
    remove_scripts: bool = True,
    remove_styles: bool = True,
    remove_comments: bool = True,
    preserve_semantic_structure: bool = True,
    preserve_tables: bool = True,
    preserve_media: bool = True,
    custom_tags_to_remove: set[str] | None = None,
    custom_attributes_to_remove: set[str] | None = None,
) -> str:
    """Preprocess HTML to remove unwanted elements and improve quality.

    Args:
        html: Raw HTML content to preprocess.
        remove_navigation: Remove navigation elements and menus.
        remove_forms: Remove form elements (input, button, select, etc.).
        remove_scripts: Remove script tags and content.
        remove_styles: Remove style tags and content.
        remove_comments: Remove HTML comments.
        preserve_semantic_structure: Preserve semantic HTML5 elements.
        preserve_tables: Preserve table structure.
        preserve_media: Preserve media elements (img, video, audio).
        custom_tags_to_remove: Additional tags to remove.
        custom_attributes_to_remove: Additional attributes to remove.

    Returns:
        Cleaned HTML ready for conversion to markdown.
    """
    if not html or not html.strip():  # pragma: no cover
        return html

    html = _remove_class_based_navigation(html, remove_navigation)

    nh3_config = _configure_cleaning_rules(
        remove_navigation=remove_navigation,
        remove_forms=remove_forms,
        remove_scripts=remove_scripts,
        remove_styles=remove_styles,
        remove_comments=remove_comments,
        preserve_semantic_structure=preserve_semantic_structure,
        preserve_tables=preserve_tables,
        preserve_media=preserve_media,
        custom_tags_to_remove=custom_tags_to_remove or set(),
        custom_attributes_to_remove=custom_attributes_to_remove or set(),
    )

    cleaned_html = nh3.clean(
        html,
        tags=nh3_config["tags"],
        attributes=nh3_config["attributes"],
        clean_content_tags=nh3_config["clean_content_tags"],
        strip_comments=nh3_config["strip_comments"],
    )

    cleaned_html = _remove_navigation_patterns(cleaned_html, remove_navigation)
    return _fix_whitespace_issues(cleaned_html)


def _configure_cleaning_rules(
    *,
    remove_navigation: bool,
    remove_forms: bool,
    remove_scripts: bool,
    remove_styles: bool,
    remove_comments: bool,
    preserve_semantic_structure: bool,
    preserve_tables: bool,
    preserve_media: bool,
    custom_tags_to_remove: set[str],
    custom_attributes_to_remove: set[str],
) -> dict[str, Any]:
    """Configure the cleaning rules for nh3."""
    allowed_tags = {
        "p",
        "div",
        "span",
        "br",
        "hr",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "ul",
        "ol",
        "li",
        "dl",
        "dt",
        "dd",
        "strong",
        "b",
        "em",
        "i",
        "u",
        "s",
        "del",
        "ins",
        "mark",
        "small",
        "sub",
        "sup",
        "code",
        "pre",
        "kbd",
        "samp",
        "var",
        "abbr",
        "cite",
        "dfn",
        "time",
        "data",
        "a",
        "blockquote",
        "q",
    }

    if preserve_semantic_structure:
        allowed_tags.update(
            {
                "article",
                "section",
                "aside",
                "header",
                "footer",
                "main",
                "nav",
                "figure",
                "figcaption",
                "details",
                "summary",
            }
        )

    if preserve_tables:
        allowed_tags.update(
            {
                "table",
                "thead",
                "tbody",
                "tfoot",
                "tr",
                "th",
                "td",
                "caption",
                "col",
                "colgroup",
            }
        )

    if preserve_media:
        allowed_tags.update(
            {
                "img",
                "picture",
                "source",
                "audio",
                "video",
                "track",
                "canvas",
                "svg",
                "iframe",
            }
        )

    allowed_tags -= custom_tags_to_remove

    clean_content_tags = set()

    if remove_navigation:
        clean_content_tags.update(
            {
                "nav",
                "menu",
                "menuitem",
                "header",
                "footer",
                "mw-jump-link",
                "vector-header",
                "vector-header-container",
                "vector-main-menu",
                "vector-page-tools",
                "vector-toc",
                "mw-navigation",
                "navbox",
                "navigation-box",
                "sidebar",
            }
        )

    if remove_forms:
        clean_content_tags.update(
            {
                "form",
                "input",
                "button",
                "select",
                "option",
                "optgroup",
                "textarea",
                "fieldset",
                "legend",
                "label",
                "output",
                "progress",
                "meter",
                "datalist",
            }
        )

    if remove_scripts:
        clean_content_tags.update({"script", "noscript"})

    if remove_styles:
        clean_content_tags.update({"style"})

    clean_content_tags.update(custom_tags_to_remove)

    allowed_tags -= clean_content_tags

    allowed_attributes = {
        "*": {"id", "class", "lang", "dir", "title"},
        "a": {"href"},
        "img": {"src", "alt", "width", "height"},
        "th": {"scope", "colspan", "rowspan"},
        "td": {"colspan", "rowspan"},
    }

    if custom_attributes_to_remove:
        for attrs in allowed_attributes.values():
            if isinstance(attrs, set):
                attrs.difference_update(custom_attributes_to_remove)

    return {
        "tags": allowed_tags,
        "attributes": allowed_attributes,
        "clean_content_tags": clean_content_tags,
        "strip_comments": remove_comments,
    }


def _remove_class_based_navigation(html: str, remove_navigation: bool) -> str:
    """Remove elements with navigation-related classes."""
    if not remove_navigation:
        return html

    navigation_classes = [
        r'vector-header[^"]*',
        r'vector-main-menu[^"]*',
        r'vector-page-tools[^"]*',
        r'vector-toc[^"]*',
        r'mw-jump-link[^"]*',
        r'mw-navigation[^"]*',
        r'navbox[^"]*',
        r'navigation-box[^"]*',
        r'sidebar[^"]*',
        r'nav[^"]*',
        r'header[^"]*',
        r'footer[^"]*',
        r'menu[^"]*',
        r'breadcrumb[^"]*',
        r'topbar[^"]*',
        r'toolbar[^"]*',
    ]

    for class_pattern in navigation_classes:
        pattern = rf'<[^>]*class="[^"]*{class_pattern}[^"]*"[^>]*>.*?</[^>]*>'
        html = re.sub(pattern, "", html, flags=re.DOTALL | re.IGNORECASE)

        pattern = rf'<[^>]*class="[^"]*{class_pattern}[^"]*"[^>]*/>'
        html = re.sub(pattern, "", html, flags=re.IGNORECASE)

    return html


def _remove_navigation_patterns(html: str, remove_navigation: bool) -> str:
    """Remove common navigation patterns that nh3 might miss."""
    if not remove_navigation:
        return html

    html = _remove_wikipedia_navigation_lists(html)

    patterns_to_remove = [
        r"\[Jump to content\]\(#[^)]*\)",
        r"\[Jump to content\]",
        r"Jump to content",
        r"Main menu.*?hide.*?Navigation",
        r"move to sidebar.*?hide",
        r"Home\s*[>»]\s*[^<]*[>»]",
        r"\[Skip to [^]]*\]",
        r"\[Skip [^]]*\]",
        r"<label[^>]*>.*?menu.*?</label>",
        r"<button[^>]*>.*?(menu|toggle|expand|collapse|show|hide).*?</button>",
        r"The Free Encyclopedia[^a-zA-Z]*",
        r"<img[^>]*wikipedia[^>]*>",
        r"\[Wikipedia\]\([^)]*\)",
        r'\[Search\]\([^)]*"Search[^)]*"\)',
        r"\[Add links\]\([^)]*\)",
        r"This is a good article\. Click here for more information\.",
        r"From Wikipedia, the free encyclopedia",
        r'<img[^>]*alt=[\'"][\'"][^>]*>',
        r'<img[^>]*src=[\'"][\'"][^>]*>',
        r"div\\>",
        r"</?\w+\\>",
        r"^Main menu\s*$",
        r"^Search\s*$",
        r"^History\s*$",
        r"^ProgrammingTranslatorReferencesExternal links\s*$",
    ]

    for pattern in patterns_to_remove:
        html = re.sub(pattern, "", html, flags=re.IGNORECASE | re.MULTILINE | re.DOTALL)

    return html


def _remove_wikipedia_navigation_lists(html: str) -> str:
    """Remove Wikipedia-style navigation lists that appear at the start."""
    patterns = [
        r"Main menu\s*\n\n(-\s*\[.*?\]\(.*?\).*?\n){3,}",
        r"(-\s*\[[^\]]*\]\(/wiki/[^)]*\).*?\n){5,}",
    ]

    for pattern in patterns:
        html = re.sub(pattern, "", html, flags=re.DOTALL | re.MULTILINE)

    return html


def _fix_whitespace_issues(html: str) -> str:
    """Fix common whitespace issues in HTML."""
    html = re.sub(r"[ \t]{2,}", " ", html)
    html = re.sub(r"\n\s*\n", "\n\n", html)

    return re.sub(r">\s*<", "><", html)


PRESETS: dict[str, dict[str, Any]] = {
    "minimal": {
        "remove_navigation": True,
        "remove_forms": True,
        "remove_scripts": True,
        "remove_styles": True,
        "remove_comments": True,
        "preserve_semantic_structure": False,
        "preserve_tables": True,
        "preserve_media": False,
    },
    "standard": {
        "remove_navigation": True,
        "remove_forms": True,
        "remove_scripts": True,
        "remove_styles": True,
        "remove_comments": True,
        "preserve_semantic_structure": True,
        "preserve_tables": True,
        "preserve_media": True,
    },
    "aggressive": {
        "remove_navigation": True,
        "remove_forms": True,
        "remove_scripts": True,
        "remove_styles": True,
        "remove_comments": True,
        "preserve_semantic_structure": False,
        "preserve_tables": True,
        "preserve_media": False,
        "custom_tags_to_remove": {"aside", "footer", "header"},
    },
}


def create_preprocessor(preset: str = "standard", **overrides: Any) -> dict[str, Any]:
    """Create preprocessor configuration with a preset.

    Args:
        preset: The preset configuration to use (minimal, standard, aggressive).
        **overrides: Any configuration options to override.

    Returns:
        Configuration dict for preprocessor.

    Raises:
        ValueError: If preset is unknown.
    """
    if preset not in PRESETS:
        msg = f"Unknown preset '{preset}'. Available presets: {list(PRESETS.keys())}"
        raise ValueError(msg)

    config: dict[str, Any] = dict(PRESETS[preset])
    config.update(overrides)

    return config

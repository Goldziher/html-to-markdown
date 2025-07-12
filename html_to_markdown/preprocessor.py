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

    # Pre-clean class-based navigation elements before nh3 processing
    html = _remove_class_based_navigation(html, remove_navigation)

    # Configure nh3 settings
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

    # Use nh3 to clean the HTML with proper configuration
    cleaned_html = nh3.clean(
        html,
        tags=nh3_config["tags"],
        attributes=nh3_config["attributes"],
        clean_content_tags=nh3_config["clean_content_tags"],
        strip_comments=nh3_config["strip_comments"],
    )

    # Additional custom cleaning for specific patterns
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
    # Start with basic allowed tags for text extraction
    allowed_tags = {
        # Text formatting
        "p",
        "div",
        "span",
        "br",
        "hr",
        # Headings
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        # Lists
        "ul",
        "ol",
        "li",
        "dl",
        "dt",
        "dd",
        # Text semantics
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
        # Links
        "a",
        # Quotes and citations
        "blockquote",
        "q",
    }

    # Add semantic HTML5 elements if preserving structure
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

    # Add table elements if preserving tables
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

    # Add media elements if preserving media
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

    # Remove custom tags
    allowed_tags -= custom_tags_to_remove

    # Define tags to completely remove (including content)
    clean_content_tags = set()

    if remove_navigation:
        clean_content_tags.update(
            {
                "nav",
                "menu",
                "menuitem",
                "header",
                "footer",
                # Wikipedia-specific navigation elements
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

    # Add custom tags to remove
    clean_content_tags.update(custom_tags_to_remove)

    # Remove clean_content_tags from allowed_tags to avoid conflicts
    allowed_tags -= clean_content_tags

    # Configure allowed attributes (basic set for text extraction)
    # Use a simple approach to avoid conflicts with nh3 defaults
    allowed_attributes = {
        "*": {"id", "class", "lang", "dir", "title"},
        "a": {"href"},
        "img": {"src", "alt", "width", "height"},
        "th": {"scope", "colspan", "rowspan"},
        "td": {"colspan", "rowspan"},
    }

    # Remove custom attributes
    if custom_attributes_to_remove:
        for attrs in allowed_attributes.values():
            if isinstance(attrs, set):
                attrs -= custom_attributes_to_remove

    # Store the configuration
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

    # Define navigation-related class patterns
    navigation_classes = [
        # Wikipedia-specific classes
        r'vector-header[^"]*',
        r'vector-main-menu[^"]*',
        r'vector-page-tools[^"]*',
        r'vector-toc[^"]*',
        r'mw-jump-link[^"]*',
        r'mw-navigation[^"]*',
        r'navbox[^"]*',
        r'navigation-box[^"]*',
        r'sidebar[^"]*',
        # Generic navigation classes
        r'nav[^"]*',
        r'header[^"]*',
        r'footer[^"]*',
        r'menu[^"]*',
        r'breadcrumb[^"]*',
        r'topbar[^"]*',
        r'toolbar[^"]*',
    ]

    # Remove elements with these classes
    for class_pattern in navigation_classes:
        # Match elements with the specified class
        pattern = rf'<[^>]*class="[^"]*{class_pattern}[^"]*"[^>]*>.*?</[^>]*>'
        html = re.sub(pattern, "", html, flags=re.DOTALL | re.IGNORECASE)

        # Also match self-closing elements
        pattern = rf'<[^>]*class="[^"]*{class_pattern}[^"]*"[^>]*/>'
        html = re.sub(pattern, "", html, flags=re.IGNORECASE)

    return html


def _remove_navigation_patterns(html: str, remove_navigation: bool) -> str:
    """Remove common navigation patterns that nh3 might miss."""
    if not remove_navigation:
        return html

    # Remove entire navigation list sections
    html = _remove_wikipedia_navigation_lists(html)

    # Common navigation patterns found in benchmarks
    patterns_to_remove = [
        # Wikipedia-style navigation
        r"\[Jump to content\]\(#[^)]*\)",
        r"\[Jump to content\]",  # Sometimes without link
        r"Jump to content",  # Plain text version
        r"Main menu.*?hide.*?Navigation",
        r"move to sidebar.*?hide",
        # Breadcrumb navigation
        r"Home\s*[>»]\s*[^<]*[>»]",
        # Skip links
        r"\[Skip to [^]]*\]",
        r"\[Skip [^]]*\]",
        # Menu toggles
        r"<label[^>]*>.*?menu.*?</label>",
        # Common UI text
        r"<button[^>]*>.*?(menu|toggle|expand|collapse|show|hide).*?</button>",
        # Wikipedia-specific patterns
        r"The Free Encyclopedia[^a-zA-Z]*",
        # Wikipedia logos and branding
        r"<img[^>]*wikipedia[^>]*>",
        r"\[Wikipedia\]\([^)]*\)",
        # Wikipedia search and navigation
        r'\[Search\]\([^)]*"Search[^)]*"\)',
        r"\[Add links\]\([^)]*\)",
        # Wikipedia article info
        r"This is a good article\. Click here for more information\.",
        r"From Wikipedia, the free encyclopedia",
        # Remove image tags with empty or placeholder content
        r'<img[^>]*alt=[\'"][\'"][^>]*>',
        r'<img[^>]*src=[\'"][\'"][^>]*>',
        # Clean up malformed tags
        r"div\\>",
        r"</?\w+\\>",
        # Remove standalone "Main menu" headers
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
    # Pattern to match navigation lists (multiple consecutive list items with Wikipedia links)
    # This targets the main menu that appears before the article content
    patterns = [
        # Remove navigation list that starts with "Contents" and has multiple Wikipedia links
        r"Main menu\s*\n\n(-\s*\[.*?\]\(.*?\).*?\n){3,}",
        # Remove any list with multiple consecutive Wikipedia navigation links
        r"(-\s*\[[^\]]*\]\(/wiki/[^)]*\).*?\n){5,}",
    ]

    for pattern in patterns:
        html = re.sub(pattern, "", html, flags=re.DOTALL | re.MULTILINE)

    return html


def _fix_whitespace_issues(html: str) -> str:
    """Fix common whitespace issues in HTML."""
    # Only normalize excessive internal whitespace, preserve leading/trailing
    # Replace multiple whitespace characters (except single spaces) with single space
    html = re.sub(r"[ \t]{2,}", " ", html)  # Multiple spaces/tabs -> single space
    html = re.sub(r"\n\s*\n", "\n\n", html)  # Multiple newlines -> double newline
    
    # Clean up whitespace around tags but preserve leading/trailing text whitespace
    html = re.sub(r">\s*<", "><", html)
    
    return html


# Preset configurations for common use cases
PRESETS = {
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

    config: dict[str, Any] = PRESETS[preset].copy()
    config.update(overrides)

    return config

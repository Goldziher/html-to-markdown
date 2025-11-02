from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Literal

import pytest

if TYPE_CHECKING:
    from collections.abc import Callable

from html_to_markdown import ConversionOptions, PreprocessingOptions
from html_to_markdown import convert as convert_api

TEST_DOCUMENTS_DIR = Path(__file__).resolve().parents[3] / "test_documents"


@pytest.fixture
def parser() -> str:
    return "html.parser"


@pytest.fixture
def convert_v2() -> Callable[..., str]:
    def _convert(
        html: str,
        *,
        heading_style: Literal["underlined", "atx", "atx_closed"] = "atx",
        list_indent_type: Literal["spaces", "tabs"] = "spaces",
        list_indent_width: int = 2,
        bullets: str = "-*+",
        strong_em_symbol: Literal["*", "_"] = "*",
        escape_asterisks: bool = False,
        escape_underscores: bool = False,
        escape_misc: bool = False,
        escape_ascii: bool = False,
        code_language: str = "",
        code_block_style: Literal["indented", "backticks", "tildes"] = "backticks",
        autolinks: bool = True,
        default_title: bool = False,
        br_in_tables: bool = False,
        highlight_style: Literal["double-equal", "html", "bold"] = "double-equal",
        extract_metadata: bool = True,
        whitespace_mode: Literal["normalized", "strict"] = "normalized",
        strip_newlines: bool = False,
        wrap: bool = False,
        wrap_width: int = 80,
        convert_as_inline: bool = False,
        sub_symbol: str = "",
        sup_symbol: str = "",
        newline_style: Literal["spaces", "backslash"] = "spaces",
        keep_inline_images_in: set[str] | None = None,
        preprocess: bool = False,
        preprocessing_preset: Literal["minimal", "standard", "aggressive"] = "standard",
        remove_navigation: bool = True,
        remove_forms: bool = True,
        source_encoding: str = "utf-8",
        strip: list[str] | None = None,
        strip_tags: list[str] | None = None,
        preserve_tags: list[str] | None = None,
    ) -> str:
        final_strip_tags = strip_tags or strip

        options = ConversionOptions(
            heading_style=heading_style,
            list_indent_type=list_indent_type,
            list_indent_width=list_indent_width,
            bullets=bullets,
            strong_em_symbol=strong_em_symbol,
            escape_asterisks=escape_asterisks,
            escape_underscores=escape_underscores,
            escape_misc=escape_misc,
            escape_ascii=escape_ascii,
            code_language=code_language,
            code_block_style=code_block_style,
            autolinks=autolinks,
            default_title=default_title,
            br_in_tables=br_in_tables,
            highlight_style=highlight_style,
            extract_metadata=extract_metadata,
            whitespace_mode=whitespace_mode,
            strip_newlines=strip_newlines,
            wrap=wrap,
            wrap_width=wrap_width,
            convert_as_inline=convert_as_inline,
            sub_symbol=sub_symbol,
            sup_symbol=sup_symbol,
            newline_style=newline_style,
            keep_inline_images_in=keep_inline_images_in,
            strip_tags=set(final_strip_tags) if final_strip_tags else None,
            preserve_tags=set(preserve_tags) if preserve_tags else None,
        )

        preprocessing = PreprocessingOptions(
            enabled=preprocess,
            preset=preprocessing_preset,
            remove_navigation=remove_navigation,
            remove_forms=remove_forms,
        )

        options.encoding = source_encoding

        return convert_api(html, options, preprocessing)

    return _convert


@pytest.fixture
def convert(convert_v2: Callable[..., str]) -> Callable[..., str]:
    return convert_v2


@pytest.fixture
def nested_uls() -> str:
    return """
    <ul>
        <li>1
            <ul>
                <li>a
                    <ul>
                        <li>I</li>
                        <li>II</li>
                        <li>III</li>
                    </ul>
                </li>
                <li>b</li>
                <li>c</li>
            </ul>
        </li>
        <li>2</li>
        <li>3</li>
    </ul>"""


@pytest.fixture
def nested_ols() -> str:
    return """
    <ol>
        <li>1
            <ol>
                <li>a
                    <ol>
                        <li>I</li>
                        <li>II</li>
                        <li>III</li>
                    </ol>
                </li>
                <li>b</li>
                <li>c</li>
            </ol>
        </li>
        <li>2</li>
        <li>3</li>
    </ul>"""


@pytest.fixture
def table() -> str:
    return """<table>
    <tr>
        <th>Firstname</th>
        <th>Lastname</th>
        <th>Age</th>
    </tr>
    <tr>
        <td>Jill</td>
        <td>Smith</td>
        <td>50</td>
    </tr>
    <tr>
        <td>Eve</td>
        <td>Jackson</td>
        <td>94</td>
    </tr>
</table>"""


@pytest.fixture
def table_with_html_content() -> str:
    return """<table>
    <tr>
        <th>Firstname</th>
        <th>Lastname</th>
        <th>Age</th>
    </tr>
    <tr>
        <td><b>Jill</b></td>
        <td><i>Smith</i></td>
        <td><a href="#">50</a></td>
    </tr>
    <tr>
        <td>Eve</td>
        <td>Jackson</td>
        <td>94</td>
    </tr>
</table>"""


@pytest.fixture
def table_with_paragraphs() -> str:
    return """<table>
    <tr>
        <th>Firstname</th>
        <th><p>Lastname</p></th>
        <th>Age</th>
    </tr>
    <tr>
        <td><p>Jill</p></td>
        <td><p>Smith</p></td>
        <td><p>50</p></td>
    </tr>
    <tr>
        <td>Eve</td>
        <td>Jackson</td>
        <td>94</td>
    </tr>
</table>"""


@pytest.fixture
def table_with_linebreaks() -> str:
    return """<table>
    <tr>
        <th>Firstname</th>
        <th>Lastname</th>
        <th>Age</th>
    </tr>
    <tr>
        <td>Jill</td>
        <td>Smith
        Jackson</td>
        <td>50</td>
    </tr>
    <tr>
        <td>Eve</td>
        <td>Jackson
        Smith</td>
        <td>94</td>
    </tr>
</table>"""


@pytest.fixture
def table_with_header_column() -> str:
    return """<table>
    <tr>
        <th>Firstname</th>
        <th>Lastname</th>
        <th>Age</th>
    </tr>
    <tr>
        <th>Jill</th>
        <td>Smith</td>
        <td>50</td>
    </tr>
    <tr>
        <th>Eve</th>
        <td>Jackson</td>
        <td>94</td>
    </tr>
</table>"""


@pytest.fixture
def table_head_body() -> str:
    return """<table>
    <thead>
        <tr>
            <th>Firstname</th>
            <th>Lastname</th>
            <th>Age</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>Jill</td>
            <td>Smith</td>
            <td>50</td>
        </tr>
        <tr>
            <td>Eve</td>
            <td>Jackson</td>
            <td>94</td>
        </tr>
    </tbody>
</table>"""


@pytest.fixture
def table_head_body_missing_head() -> str:
    return """<table>
    <thead>
        <tr>
            <td>Firstname</td>
            <td>Lastname</td>
            <td>Age</td>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>Jill</td>
            <td>Smith</td>
            <td>50</td>
        </tr>
        <tr>
            <td>Eve</td>
            <td>Jackson</td>
            <td>94</td>
        </tr>
    </tbody>
</table>"""


@pytest.fixture
def table_missing_text() -> str:
    return """<table>
    <thead>
        <tr>
            <th></th>
            <th>Lastname</th>
            <th>Age</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>Jill</td>
            <td></td>
            <td>50</td>
        </tr>
        <tr>
            <td>Eve</td>
            <td>Jackson</td>
            <td>94</td>
        </tr>
    </tbody>
</table>"""


@pytest.fixture
def table_missing_head() -> str:
    return """<table>
    <tr>
        <td>Firstname</td>
        <td>Lastname</td>
        <td>Age</td>
    </tr>
    <tr>
        <td>Jill</td>
        <td>Smith</td>
        <td>50</td>
    </tr>
    <tr>
        <td>Eve</td>
        <td>Jackson</td>
        <td>94</td>
    </tr>
</table>"""


@pytest.fixture
def table_body() -> str:
    return """<table>
    <tbody>
        <tr>
            <td>Firstname</td>
            <td>Lastname</td>
            <td>Age</td>
        </tr>
        <tr>
            <td>Jill</td>
            <td>Smith</td>
            <td>50</td>
        </tr>
        <tr>
            <td>Eve</td>
            <td>Jackson</td>
            <td>94</td>
        </tr>
    </tbody>
</table>"""


@pytest.fixture
def table_with_caption() -> str:
    return """TEXT<table><caption>Caption</caption>
    <tbody><tr><td>Firstname</td>
            <td>Lastname</td>
            <td>Age</td>
        </tr>
    </tbody>
</table>"""


@pytest.fixture
def table_with_colspan() -> str:
    return """<table>
    <tr>
        <th colspan="2">Name</th>
        <th>Age</th>
    </tr>
    <tr>
        <td colspan="1">Jill</td>
        <td>Smith</td>
        <td>50</td>
    </tr>
    <tr>
        <td>Eve</td>
        <td>Jackson</td>
        <td>94</td>
    </tr>
</table>"""


@pytest.fixture
def table_with_undefined_colspan() -> str:
    return """<table>
    <tr>
        <th colspan="undefined">Name</th>
        <th>Age</th>
    </tr>
    <tr>
        <td colspan="-1">Jill</td>
        <td>Smith</td>
    </tr>
</table>"""

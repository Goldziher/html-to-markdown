from __future__ import annotations

from typing import TYPE_CHECKING, Any

import pytest

if TYPE_CHECKING:
    from collections.abc import Callable


def test_single_tag(convert: Callable[..., str]) -> None:
    assert convert("<span>Hello</span>") == "Hello\n"


def test_soup(convert: Callable[..., str]) -> None:
    assert convert("<div><span>Hello</div></span>") == "Hello\n"


def test_whitespace(convert: Callable[..., str]) -> None:
    assert convert(" a  b \t\t c ") == " a b c\n"


def test_asterisks(convert: Callable[..., str]) -> None:
    assert convert("*hey*dude*") == r"*hey*dude*" + "\n"
    assert convert("*hey*dude*", escape_asterisks=True) == r"\*hey\*dude\*" + "\n"


def test_underscore(convert: Callable[..., str]) -> None:
    assert convert("_hey_dude_") == r"_hey_dude_" + "\n"
    assert convert("_hey_dude_", escape_underscores=True) == r"\_hey\_dude\_" + "\n"


def test_xml_entities(convert: Callable[..., str]) -> None:
    assert convert("&amp;") == "&\n"


def test_named_entities(convert: Callable[..., str]) -> None:
    assert convert("&raquo;") == "»\n"


def test_hexadecimal_entities(convert: Callable[..., str]) -> None:
    assert convert("&#x27;") == "'\n"


def test_single_escaping_entities(convert: Callable[..., str]) -> None:
    assert convert("&amp;amp;") == "&amp;\n"


def test_misc(convert: Callable[..., str]) -> None:
    assert convert("\\*") == "\\*\n"
    assert convert("<foo>") == ""
    assert convert("# foo") == "# foo\n"
    assert convert("> foo") == "> foo\n"
    assert convert("~~foo~~") == "~~foo~~\n"
    assert convert("foo\n===\n") == "foo\n===\n"
    assert convert("---\n") == "---\n"
    assert convert("+ x\n+ y\n") == "+ x\n+ y\n"
    assert convert("`x`") == "`x`\n"
    assert convert("[text](link)") == "[text](link)\n"
    assert convert("1. x") == "1. x\n"
    assert convert("not a number. x") == "not a number. x\n"
    assert convert("1) x") == "1) x\n"
    assert convert("not a number) x") == "not a number) x\n"
    assert convert("|not table|") == "|not table|\n"
    assert convert(r"\ <foo> &amp;amp; | ` `", escape_misc=False, preprocess=True) == "\\ &amp; | ` `\n"

    assert convert("# foo", escape_misc=True) == "\\# foo\n"
    assert convert("> foo", escape_misc=True) == "\\> foo\n"
    assert convert("~~foo~~", escape_misc=True) == "\\~\\~foo\\~\\~\n"


def test_chomp(convert: Callable[..., str]) -> None:
    assert convert(" <b></b> ") == "  \n"
    assert convert(" <b> </b> ") == "  \n"
    assert convert(" <b>  </b> ") == "  \n"
    assert convert(" <b>   </b> ") == "  \n"
    assert convert(" <b>s </b> ") == " **s**\n"
    assert convert(" <b> s</b> ") == "  **s**\n"
    assert convert(" <b> s </b> ") == "  **s**\n"
    assert convert(" <b>  s  </b> ") == "  **s**\n"


def test_nested(convert: Callable[..., str]) -> None:
    text = convert('<p>This is an <a href="http://example.com/">example link</a>.</p>')
    assert text == "This is an [example link](http://example.com/).\n"


def test_ignore_comments(convert: Callable[..., str]) -> None:
    text = convert("<!-- This is a comment -->")
    assert text == ""


def test_ignore_comments_with_other_tags(convert: Callable[..., str]) -> None:
    text = convert("<!-- This is a comment --><a href='http://example.com/'>example link</a>")
    assert text == "[example link](http://example.com/)\n"


def test_code_with_tricky_content(convert: Callable[..., str]) -> None:
    assert convert("<code>></code>") == "`>`\n"
    assert convert("<code>/home/</code><b>username</b>") == "`/home/`**username**\n"
    assert (
        convert("First line <code>blah blah<br />blah blah</code> second line")
        == "First line `blah blah  \nblah blah` second line\n"
    )
    assert (
        convert("First line <code>blah blah<br />blah blah</code> second line", newline_style="backslash")
        == "First line `blah blah\\\nblah blah` second line\n"
    )


def test_special_tags(convert: Callable[..., str]) -> None:
    assert convert("<!DOCTYPE html>") == ""

    # CDATA sections are preserved as-is during HTML parsing
    assert convert("<![CDATA[foobar]]>") == "<![CDATA[foobar]]>\n"


def test_strip(convert: Callable[..., str]) -> None:
    text = convert('<a href="https://github.com/matthewwithanm">Some Text</a>', strip=["a"])
    assert text == "Some Text\n"


def test_do_not_strip(convert: Callable[..., str]) -> None:
    text = convert('<a href="https://github.com/matthewwithanm">Some Text</a>', strip=[])
    assert text == "[Some Text](https://github.com/matthewwithanm)\n"


@pytest.mark.skip(reason="convert parameter removed in v2 - v1 only feature")
def test_convert(convert: Callable[..., str]) -> None:
    text = convert('<a href="https://github.com/matthewwithanm">Some Text</a>', convert=["a"])
    assert text == "[Some Text](https://github.com/matthewwithanm)"


@pytest.mark.skip(reason="convert parameter removed in v2 - v1 only feature")
def test_do_not_convert(convert: Callable[..., str]) -> None:
    text = convert('<a href="https://github.com/matthewwithanm">Some Text</a>', convert=[])
    assert text == "Some Text"


def test_ol(convert: Callable[..., str]) -> None:
    assert convert("<ol><li>a</li><li>b</li></ol>") == "1. a\n2. b\n"
    assert convert('<ol start="3"><li>a</li><li>b</li></ol>') == "3. a\n4. b\n"
    assert convert('<ol start="-1"><li>a</li><li>b</li></ol>') == "1. a\n2. b\n"
    assert convert('<ol start="foo"><li>a</li><li>b</li></ol>') == "1. a\n2. b\n"
    assert convert('<ol start="1.5"><li>a</li><li>b</li></ol>') == "1. a\n2. b\n"


def test_nested_ols(nested_ols: str, convert: Callable[..., str]) -> None:
    assert convert(nested_ols) == "1. 1\n  1. a\n    1. I\n    2. II\n    3. III\n  2. b\n  3. c\n2. 2\n3. 3\n"


def test_ul(convert: Callable[..., str]) -> None:
    assert convert("<ul><li>a</li><li>b</li></ul>") == "- a\n- b\n"
    assert (
        convert(
            """<ul>
         <li>
                 a
         </li>
         <li> b </li>
         <li>   c
         </li>
     </ul>"""
        )
        == "- a\n- b\n- c\n"
    )


def test_inline_ul(convert: Callable[..., str]) -> None:
    assert convert("<p>foo</p><ul><li>a</li><li>b</li></ul><p>bar</p>") == "foo\n\n- a\n- b\n\n\nbar\n"


def test_nested_uls(nested_uls: str, convert: Callable[..., str]) -> None:
    assert convert(nested_uls) == "- 1\n  * a\n    + I\n    + II\n    + III\n  * b\n  * c\n- 2\n- 3\n"


def test_bullets(nested_uls: str, convert: Callable[..., str]) -> None:
    assert (
        convert(nested_uls, bullets="*+-", list_indent_width=4)
        == "* 1\n    + a\n        - I\n        - II\n        - III\n    + b\n    + c\n* 2\n* 3\n"
    )


def test_li_text(convert: Callable[..., str]) -> None:
    assert (
        convert('<ul><li>foo <a href="#">bar</a></li><li>foo bar  </li><li>foo <b>bar</b>   <i>space</i>.</ul>')
        == "- foo [bar](#)\n- foo bar\n- foo **bar** *space*.\n"
    )


def test_table(
    table: str,
    table_with_html_content: str,
    table_with_paragraphs: str,
    table_with_linebreaks: str,
    table_with_header_column: str,
    table_head_body: str,
    table_head_body_missing_head: str,
    table_missing_text: str,
    table_missing_head: str,
    table_body: str,
    table_with_caption: str,
    table_with_colspan: str,
    table_with_undefined_colspan: str,
    convert: Callable[..., str],
) -> None:
    assert (
        convert(table)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert (
        convert(table_with_html_content)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| **Jill** | *Smith* | [50](#) |\n| Eve | Jackson | 94 |\n"
    )
    assert (
        convert(table_with_paragraphs)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert (
        convert(table_with_linebreaks)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith  Jackson | 50 |\n| Eve | Jackson  Smith | 94 |\n"
    )
    assert (
        convert(table_with_header_column)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert (
        convert(table_head_body)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert (
        convert(table_head_body_missing_head)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert (
        convert(table_missing_text)
        == "\n\n|  | Lastname | Age |\n| --- | --- | --- |\n| Jill |  | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert (
        convert(table_missing_head)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert (
        convert(table_body)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert convert(table_with_caption) == "TEXT\n\n*Caption*\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n"
    assert (
        convert(table_with_colspan)
        == "\n\n| Name | | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n"
    )
    assert convert(table_with_undefined_colspan) == "\n\n| Name | Age |\n| --- | --- |\n| Jill | Smith |\n"


def inline_tests(tag: str, markup: str, convert: Callable[..., str]) -> None:
    preserves_whitespace = tag == "code"

    assert convert(f"<{tag}>Hello</{tag}>") == f"{markup}Hello{markup}\n"
    assert convert(f"foo <{tag}>Hello</{tag}> bar") == f"foo {markup}Hello{markup} bar\n"

    if preserves_whitespace:
        assert convert(f"foo<{tag}> Hello</{tag}> bar") == f"foo{markup} Hello{markup} bar\n"
        assert convert(f"foo <{tag}>Hello </{tag}>bar") == f"foo {markup}Hello {markup}bar\n"
    else:
        assert convert(f"foo<{tag}> Hello</{tag}> bar") == f"foo {markup}Hello{markup} bar\n"
        assert convert(f"foo <{tag}>Hello </{tag}>bar") == f"foo {markup}Hello{markup} bar\n"

    assert convert(f"foo <{tag}></{tag}> bar") == "foo  bar\n"


def test_a(convert: Callable[..., str]) -> None:
    assert convert('<a href="https://google.com">Google</a>') == "[Google](https://google.com)\n"
    assert convert('<a href="https://google.com">https://google.com</a>') == "<https://google.com>\n"
    assert (
        convert('<a href="https://community.kde.org/Get_Involved">https://community.kde.org/Get_Involved</a>')
        == "<https://community.kde.org/Get_Involved>\n"
    )
    assert (
        convert(
            '<a href="https://community.kde.org/Get_Involved">https://community.kde.org/Get_Involved</a>',
            autolinks=False,
        )
        == "[https://community.kde.org/Get_Involved](https://community.kde.org/Get_Involved)\n"
    )
    assert (
        convert(
            '<a href="https://community.kde.org/Get_Involved">https://community.kde.org/Get_Involved</a>',
            autolinks=False,
            escape_underscores=True,
        )
        == "[https://community.kde.org/Get\\_Involved](https://community.kde.org/Get_Involved)\n"
    )


def test_a_spaces(convert: Callable[..., str]) -> None:
    assert (
        convert('foo <a href="http://google.com">Google</a> bar', preprocess=True)
        == "foo [Google](http://google.com) bar\n"
    )
    assert (
        convert('foo<a href="http://google.com"> Google</a> bar', preprocess=True)
        == "foo[Google](http://google.com) bar\n"
    )
    assert (
        convert('foo <a href="http://google.com">Google </a>bar', preprocess=True)
        == "foo [Google](http://google.com)bar\n"
    )
    assert convert('foo <a href="http://google.com"></a> bar', preprocess=True) == "foo [](http://google.com) bar\n"
    assert convert("foo <a>text</a> bar") == "foo text bar\n"


def test_a_with_title(convert: Callable[..., str]) -> None:
    text = convert('<a href="http://google.com" title="The &quot;Goog&quot;">Google</a>')
    assert text == r'[Google](http://google.com "The &quot;Goog&quot;")' + "\n"
    assert (
        convert('<a href="https://google.com">https://google.com</a>', default_title=True)
        == '[https://google.com](https://google.com "https://google.com")\n'
    )


def test_a_shortcut(convert: Callable[..., str]) -> None:
    text = convert('<a href="http://google.com">http://google.com</a>')
    assert text == "<http://google.com>\n"


def test_a_no_autolinks(convert: Callable[..., str]) -> None:
    assert (
        convert('<a href="https://google.com">https://google.com</a>', autolinks=False)
        == "[https://google.com](https://google.com)\n"
    )


def test_b(convert: Callable[..., str]) -> None:
    assert convert("<b>Hello</b>") == "**Hello**\n"


def test_b_spaces(convert: Callable[..., str]) -> None:
    assert convert("foo <b>Hello</b> bar") == "foo **Hello** bar\n"
    assert convert("foo<b> Hello</b> bar") == "foo **Hello** bar\n"
    assert convert("foo <b>Hello </b>bar") == "foo **Hello** bar\n"
    assert convert("foo <b></b> bar") == "foo  bar\n"


def test_blockquote(convert: Callable[..., str]) -> None:
    assert convert("<blockquote>Hello</blockquote>", preprocess=True) == "> Hello\n"
    assert convert("<blockquote>\nHello\n</blockquote>", preprocess=True) == "> Hello\n"


def test_blockquote_with_nested_paragraph(convert: Callable[..., str]) -> None:
    assert convert("<blockquote><p>Hello</p></blockquote>", preprocess=True) == "> Hello\n"
    assert (
        convert("<blockquote><p>Hello</p><p>Hello again</p></blockquote>", preprocess=True)
        == "> Hello\n>\n> Hello again\n"
    )


def test_blockquote_with_paragraph(convert: Callable[..., str]) -> None:
    assert convert("<blockquote>Hello</blockquote><p>handsome</p>", preprocess=True) == "> Hello\n\nhandsome\n"


def test_blockquote_nested(convert: Callable[..., str]) -> None:
    text = convert("<blockquote>And she was like <blockquote>Hello</blockquote></blockquote>", preprocess=True)
    assert text == "> And she was like\n>\n>\n> > Hello\n"


def test_br(convert: Callable[..., str]) -> None:
    assert convert("a<br />b<br />c") == "a  \nb  \nc\n"
    assert convert("a<br />b<br />c", newline_style="backslash") == "a\\\nb\\\nc\n"


def test_caption(convert: Callable[..., str]) -> None:
    assert (
        convert("TEXT<figure><figcaption>Caption</figcaption><span>SPAN</span></figure>")
        == "TEXT\n\n*Caption*\n\nSPAN\n"
    )
    assert (
        convert("<figure><span>SPAN</span><figcaption>Caption</figcaption></figure>TEXT")
        == "SPAN\n\n*Caption*\n\nTEXT\n"
    )


def test_code(convert: Callable[..., str]) -> None:
    inline_tests("code", "`", convert)
    assert convert("<code>*this_should_not_escape*</code>") == "`*this_should_not_escape*`\n"
    assert convert("<kbd>*this_should_not_escape*</kbd>") == "`*this_should_not_escape*`\n"
    assert convert("<samp>*this_should_not_escape*</samp>") == "`*this_should_not_escape*`\n"
    assert convert("<code><span>*this_should_not_escape*</span></code>") == "`*this_should_not_escape*`\n"
    assert convert("<div><code>code_with_underscores</code></div>", preprocess=True) == "`code_with_underscores`\n"
    assert convert("<span><code>*asterisks* and _underscores_</code></span>") == "`*asterisks* and _underscores_`\n"
    assert convert("<p><code>foo_bar_baz</code></p>", preprocess=True) == "`foo_bar_baz`\n"
    assert convert("<code>this  should\t\tnormalize</code>") == "`this  should\t\tnormalize`\n"
    assert convert("<code><span>this  should\t\tnormalize</span></code>") == "`this  should\t\tnormalize`\n"
    assert convert("<code>foo<b>bar</b>baz</code>") == "`foobarbaz`\n"
    assert convert("<kbd>foo<i>bar</i>baz</kbd>") == "`foobarbaz`\n"
    assert convert("<samp>foo<del> bar </del>baz</samp>") == "`foo bar baz`\n"
    assert convert("<samp>foo <del>bar</del> baz</samp>") == "`foo bar baz`\n"
    assert convert("<code>foo<em> bar </em>baz</code>") == "`foo bar baz`\n"
    assert convert("<code>foo<code> bar </code>baz</code>") == "`foo bar baz`\n"
    assert convert("<code>foo<strong> bar </strong>baz</code>") == "`foo bar baz`\n"
    assert convert("<code>foo<s> bar </s>baz</code>") == "`foo bar baz`\n"
    assert convert("<code>foo<sup>bar</sup>baz</code>") == "`foobarbaz`\n"
    assert convert("<code>foo<sub>bar</sub>baz</code>") == "`foobarbaz`\n"


def test_del(convert: Callable[..., str]) -> None:
    inline_tests("del", "~~", convert)


def test_div(convert: Callable[..., str]) -> None:
    assert convert("Hello</div> World") == "Hello World\n"


def test_em(convert: Callable[..., str]) -> None:
    inline_tests("em", "*", convert)


def test_header_with_space(convert: Callable[..., str]) -> None:
    assert convert("<h3>\n\nHello</h3>") == "### Hello\n"
    assert convert("<h4>\n\nHello</h4>") == "#### Hello\n"
    assert convert("<h5>\n\nHello</h5>") == "##### Hello\n"
    assert convert("<h5>\n\nHello\n\n</h5>") == "##### Hello\n"
    assert convert("<h5>\n\nHello   \n\n</h5>") == "##### Hello\n"


def test_h1(convert: Callable[..., str]) -> None:
    assert convert("<h1>Hello</h1>") == "# Hello\n"
    assert convert("<h1>Hello</h1>", heading_style="underlined") == "Hello\n=====\n"


def test_h2(convert: Callable[..., str]) -> None:
    assert convert("<h2>Hello</h2>") == "## Hello\n"
    assert convert("<h2>Hello</h2>", heading_style="underlined") == "Hello\n-----\n"


def test_hn(convert: Callable[..., str]) -> None:
    assert convert("<h3>Hello</h3>") == "### Hello\n"
    assert convert("<h4>Hello</h4>") == "#### Hello\n"
    assert convert("<h5>Hello</h5>") == "##### Hello\n"
    assert convert("<h6>Hello</h6>") == "###### Hello\n"


def test_hn_chained(convert: Callable[..., str]) -> None:
    assert (
        convert("<h1>First</h1>\n<h2>Second</h2>\n<h3>Third</h3>", heading_style="atx")
        == "# First\n\n## Second\n\n### Third\n"
    )
    assert convert("X<h1>First</h1>", heading_style="atx") == "X\n\n# First\n"


def test_hn_nested_tag_heading_style(convert: Callable[..., str]) -> None:
    result = convert("<h1>A <p>P</p> C </h1>", heading_style="atx_closed")
    assert result in ["# A P C #\n", "# A #\n\nP\n\n C "]

    result2 = convert("<h1>A <p>P</p> C </h1>", heading_style="atx")
    assert result2 in ["# A P C\n", "# A\n\nP\n\n C "]


def test_hn_eol(convert: Callable[..., str]) -> None:
    assert convert("<p>xxx</p><h3>Hello</h3>", heading_style="atx") == "xxx\n\n### Hello\n"

    assert convert("\n<h3>Hello</h3>", heading_style="atx") == "### Hello\n"
    assert convert("\nx<h3>Hello</h3>", heading_style="atx") == " x\n\n### Hello\n"
    assert convert("\n<span>x<h3>Hello</h3></span>", heading_style="atx") == "x\n\n### Hello\n"
    assert convert("xxx<h3>Hello</h3>", heading_style="atx") == "xxx\n\n### Hello\n"


def test_hn_nested_simple_tag(convert: Callable[..., str]) -> None:
    inline_tag_to_markdown = [
        ("strong", "**strong**"),
        ("b", "**b**"),
        ("em", "*em*"),
        ("i", "*i*"),
        ("a", "a"),
        ("div", "div"),
        ("blockquote", "blockquote"),
    ]

    for tag, markdown in inline_tag_to_markdown:
        assert convert("<h3>A <" + tag + ">" + tag + "</" + tag + "> B</h3>") == "### A " + markdown + " B\n"

    result = convert("<h3>A <p>p</p> B</h3>")
    assert result in ["### A p B\n", "### A\n\np\n\n B"]

    assert convert("<h3>A <br>B</h3>") == "### A  B\n"


def test_hn_nested_img(convert: Callable[..., str]) -> None:
    image_attributes_to_markdown = [
        ("", "", ""),
        ("alt='Alt Text'", "Alt Text", ""),
        ("alt='Alt Text' title='Optional title'", "Alt Text", ' "Optional title"'),
    ]
    for image_attributes, markdown, title in image_attributes_to_markdown:
        assert (
            convert('<h3>A <img src="/path/to/img.jpg" ' + image_attributes + "/> B</h3>")
            == "### A " + markdown + " B\n"
        )
        assert (
            convert(
                '<h3>A <img src="/path/to/img.jpg" ' + image_attributes + "/> B</h3>",
                keep_inline_images_in=["h3"],
            )
            == "### A ![" + markdown + "](/path/to/img.jpg" + title + ") B\n"
        )


def test_hn_atx_headings(convert: Callable[..., str]) -> None:
    assert convert("<h1>Hello</h1>", heading_style="atx") == "# Hello\n"
    assert convert("<h2>Hello</h2>", heading_style="atx") == "## Hello\n"


def test_hn_atx_closed_headings(convert: Callable[..., str]) -> None:
    assert convert("<h1>Hello</h1>", heading_style="atx_closed") == "# Hello #\n"
    assert convert("<h2>Hello</h2>", heading_style="atx_closed") == "## Hello ##\n"


def test_head(convert: Callable[..., str]) -> None:
    assert convert("<head>head</head>") == ""


def test_hr(convert: Callable[..., str]) -> None:
    assert convert("Hello<hr>World") == "Hello\n\n---\nWorld\n"
    assert convert("Hello<hr />World") == "Hello\n\n---\nWorld\n"
    assert convert("<p>Hello</p>\n<hr>\n<p>World</p>") == "Hello\n\n---\n\n\nWorld\n"


def test_i(convert: Callable[..., str]) -> None:
    assert convert("<i>Hello</i>") == "*Hello*\n"


def test_img(convert: Callable[..., str]) -> None:
    assert (
        convert('<img src="/path/to/img.jpg" alt="Alt text" title="Optional title" />')
        == '![Alt text](/path/to/img.jpg "Optional title")\n'
    )
    assert convert('<img src="/path/to/img.jpg" alt="Alt text" />') == "![Alt text](/path/to/img.jpg)\n"
    assert convert('<img src="/path/to/img.jpg" width="100" height="100" />') == "![](/path/to/img.jpg)\n"


def test_kbd(convert: Callable[..., str]) -> None:
    inline_tests("kbd", "`", convert)


def test_p(convert: Callable[..., str]) -> None:
    assert convert("<p>hello</p>") == "hello\n"
    assert convert("<p>123456789 123456789</p>") == "123456789 123456789\n"
    assert convert("<p>123456789 123456789</p>", wrap=True, wrap_width=10) == "123456789\n123456789\n\n"
    assert (
        convert(
            '<p><a href="https://example.com">Some long link</a></p>',
            wrap=True,
            wrap_width=10,
        )
        == "[Some long\nlink](https://example.com)\n\n"
    )
    assert (
        convert("<p>12345<br />67890</p>", wrap=True, wrap_width=10, newline_style="backslash") == "12345\\\n67890\n\n"
    )
    assert (
        convert(
            "<p>12345678901<br />12345</p>",
            wrap=True,
            wrap_width=10,
            newline_style="backslash",
        )
        == "12345678901\\\n12345\n\n"
    )


def test_mark_tag(convert: Callable[..., str]) -> None:
    html = "<mark>highlighted</mark>"
    expected = "==highlighted=="
    assert convert(html).strip() == expected


def test_mark_tag_with_different_styles(convert: Callable[..., str]) -> None:
    html = "<mark>highlighted</mark>"

    assert convert(html, highlight_style="double-equal").strip() == "==highlighted=="

    assert convert(html, highlight_style="bold").strip() == "**highlighted**"

    assert convert(html, highlight_style="html").strip() == "<mark>highlighted</mark>"


def test_mark_tag_in_paragraph(convert: Callable[..., str]) -> None:
    html = "<p>This is <mark>highlighted text</mark> in a paragraph.</p>"
    expected = "This is ==highlighted text== in a paragraph.\n"
    assert convert(html) == expected


def test_mark_tag_with_nested_formatting(convert: Callable[..., str]) -> None:
    html = "<mark>This is <strong>bold highlighted</strong> text</mark>"
    expected = "==This is **bold highlighted** text=="
    assert convert(html).strip() == expected

    html = "<mark>This is <em>italic highlighted</em> text</mark>"
    expected = "==This is *italic highlighted* text=="
    assert convert(html).strip() == expected


def test_multiple_mark_tags(convert: Callable[..., str]) -> None:
    html = "<p>First <mark>highlight</mark> and second <mark>highlight</mark>.</p>"
    expected = "First ==highlight== and second ==highlight==.\n"
    assert convert(html) == expected


def test_nested_mark_tags(convert: Callable[..., str]) -> None:
    html = "<mark>Outer <mark>nested</mark> mark</mark>"
    expected = "==Outer ==nested== mark=="
    assert convert(html).strip() == expected


def test_mark_tag_as_inline(convert: Callable[..., str]) -> None:
    html = "<mark>highlighted</mark>"
    expected = "highlighted"
    assert convert(html, convert_as_inline=True).strip() == expected


def test_mark_tag_with_complex_content(convert: Callable[..., str]) -> None:
    html = """
    <div>
        <h2>Title</h2>
        <p>Regular text with <mark>highlighted portion</mark> and more text.</p>
        <ul>
            <li>Item with <mark>highlighted item text</mark></li>
            <li>Another item</li>
        </ul>
    </div>
    """
    result = convert(html)
    assert "==highlighted portion==" in result
    assert "==highlighted item text==" in result


def test_pre(convert: Callable[..., str]) -> None:
    assert convert("<pre>test\n    foo\nbar</pre>") == "```\ntest\n    foo\nbar\n```\n"
    assert convert("<pre><code>test\n    foo\nbar</code></pre>") == "```\ntest\n    foo\nbar\n```\n"
    assert convert("<pre>*this_should_not_escape*</pre>") == "```\n*this_should_not_escape*\n```\n"
    assert convert("<pre><span>*this_should_not_escape*</span></pre>") == "```\n*this_should_not_escape*\n```\n"
    assert convert("<div><pre>code_with_underscores</pre></div>") == "```\ncode_with_underscores\n```\n"
    assert convert("<div><pre>*asterisks* and _underscores_</pre></div>") == "```\n*asterisks* and _underscores_\n```\n"
    assert convert("<section><pre>foo_bar_baz</pre></section>") == "```\nfoo_bar_baz\n```\n"
    assert convert("<pre>\t\tthis  should\t\tnot  normalize</pre>") == "```\n\t\tthis  should\t\tnot  normalize\n```\n"
    assert (
        convert("<pre><span>\t\tthis  should\t\tnot  normalize</span></pre>")
        == "```\n\t\tthis  should\t\tnot  normalize\n```\n"
    )
    assert convert("<pre>foo<b>\nbar\n</b>baz</pre>") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo<i>\nbar\n</i>baz</pre>") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo\n<i>bar</i>\nbaz</pre>") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo<i>\n</i>baz</pre>") == "```\nfoo\nbaz\n```\n"
    assert convert("<pre>foo<del>\nbar\n</del>baz</pre>") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo<em>\nbar\n</em>baz</pre>") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo<code>\nbar\n</code>baz</pre>") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo<strong>\nbar\n</strong>baz</pre>") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo<s>\nbar\n</s>baz</pre>") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo<sup>\nbar\n</sup>baz</pre>", sup_symbol="^") == "```\nfoo\nbar\nbaz\n```\n"
    assert convert("<pre>foo<sub>\nbar\n</sub>baz</pre>", sub_symbol="^") == "```\nfoo\nbar\nbaz\n```\n"

    assert convert("<pre>test\n    foo\nbar</pre>", code_block_style="indented") == "    test\n        foo\n    bar\n"
    assert (
        convert("<pre><code>test\n    foo\nbar</code></pre>", code_block_style="indented")
        == "    test\n        foo\n    bar\n"
    )


def test_script(convert: Callable[..., str]) -> None:
    assert convert("foo <script>var foo=42;</script> bar") == "foo  bar\n"


def test_style(convert: Callable[..., str]) -> None:
    assert convert("foo <style>h1 { font-size: larger }</style> bar") == "foo  bar\n"


def test_s(convert: Callable[..., str]) -> None:
    inline_tests("s", "~~", convert)


def test_samp(convert: Callable[..., str]) -> None:
    inline_tests("samp", "`", convert)


def test_strong(convert: Callable[..., str]) -> None:
    assert convert("<strong>Hello</strong>") == "**Hello**\n"


def test_strong_em_symbol(convert: Callable[..., str]) -> None:
    assert convert("<strong>Hello</strong>", strong_em_symbol="_") == "__Hello__\n"
    assert convert("<b>Hello</b>", strong_em_symbol="_") == "__Hello__\n"
    assert convert("<em>Hello</em>", strong_em_symbol="_") == "_Hello_\n"
    assert convert("<i>Hello</i>", strong_em_symbol="_") == "_Hello_\n"


def test_sub(convert: Callable[..., str]) -> None:
    assert convert("<sub>foo</sub>") == "foo\n"
    assert convert("<sub>foo</sub>", sub_symbol="~") == "~foo~\n"
    assert convert("<sub>foo</sub>", sub_symbol="<sub>") == "<sub>foo</sub>\n"


def test_sup(convert: Callable[..., str]) -> None:
    assert convert("<sup>foo</sup>") == "foo\n"
    assert convert("<sup>foo</sup>", sup_symbol="^") == "^foo^\n"
    assert convert("<sup>foo</sup>", sup_symbol="<sup>") == "<sup>foo</sup>\n"


def test_lang(convert: Callable[..., str]) -> None:
    assert (
        convert("<pre>test\n    foo\nbar</pre>", code_language="python", code_block_style="backticks")
        == "```python\ntest\n    foo\nbar\n```\n"
    )
    assert (
        convert("<pre><code>test\n    foo\nbar</code></pre>", code_language="javascript", code_block_style="backticks")
        == "```javascript\ntest\n    foo\nbar\n```\n"
    )


@pytest.mark.skip(reason="code_language_callback removed in v2 - use static code_language")
def test_lang_callback(convert: Callable[..., str]) -> None:
    def callback(el: Any) -> str | None:
        return el["class"][0] if el.has_attr("class") else None

    assert (
        convert(
            '<pre class="python">test\n    foo\nbar</pre>',
            code_language_callback=callback,
        )
        == "\n```python\ntest\n    foo\nbar\n```\n"
    )
    assert (
        convert(
            '<pre class="javascript"><code>test\n    foo\nbar</code></pre>',
            code_language_callback=callback,
        )
        == "\n```javascript\ntest\n    foo\nbar\n```\n"
    )
    assert (
        convert(
            '<pre class="javascript"><code class="javascript">test\n    foo\nbar</code></pre>',
            code_language_callback=callback,
        )
        == "\n```javascript\ntest\n    foo\nbar\n```\n"
    )


def test_idempotence(convert: Callable[..., str]) -> None:
    html_text = "<h2>Header&nbsp;</h2><p>Next paragraph.</p>"
    converted = convert(html_text)
    assert converted == convert(converted)


def test_character_encoding(convert: Callable[..., str]) -> None:
    html_with_encoding_issue = (
        '<cite>api_key="your-api-key"</cite> or by defining <cite>GOOGLE_API_KEY="your-api-key"</cite> as an'
    )

    result = convert(html_with_encoding_issue)
    assert result == '*api_key="your-api-key"* or by defining *GOOGLE_API_KEY="your-api-key"* as an\n'

    result_escaped = convert(html_with_encoding_issue, escape_underscores=True, escape_misc=True)
    assert (
        result_escaped
        == '*api\\_key\\="your\\-api\\-key"* or by defining *GOOGLE\\_API\\_KEY\\="your\\-api\\-key"* as an\n'
    )

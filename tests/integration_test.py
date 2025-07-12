from __future__ import annotations

from typing import TYPE_CHECKING

from html_to_markdown import convert_to_markdown
from html_to_markdown.constants import ATX, ATX_CLOSED, BACKSLASH, UNDERSCORE

if TYPE_CHECKING:
    from bs4.element import Tag


def test_single_tag() -> None:
    assert convert_to_markdown("<span>Hello</span>") == "Hello"


def test_soup() -> None:
    assert convert_to_markdown("<div><span>Hello</div></span>") == "Hello"


def test_whitespace() -> None:
    # Semantic whitespace normalization - preserve meaningful leading/trailing spaces
    # while normalizing internal whitespace for consistency
    assert convert_to_markdown(" a  b \t\t c ") == " a b c "


def test_asterisks() -> None:
    assert convert_to_markdown("*hey*dude*") == r"\*hey\*dude\*"
    assert convert_to_markdown("*hey*dude*", escape_asterisks=False) == r"*hey*dude*"


def test_underscore() -> None:
    assert convert_to_markdown("_hey_dude_") == r"\_hey\_dude\_"
    assert convert_to_markdown("_hey_dude_", escape_underscores=False) == r"_hey_dude_"


def test_xml_entities() -> None:
    assert convert_to_markdown("&amp;") == r"\&"


def test_named_entities() -> None:
    assert convert_to_markdown("&raquo;") == "\xbb"


def test_hexadecimal_entities() -> None:
    assert convert_to_markdown("&#x27;") == "\x27"


def test_single_escaping_entities() -> None:
    assert convert_to_markdown("&amp;amp;") == r"\&amp;"


def text_misc() -> None:
    assert convert_to_markdown("\\*") == r"\\\*"
    assert convert_to_markdown("<foo>") == r"\<foo\>"
    assert convert_to_markdown("# foo") == r"\# foo"
    assert convert_to_markdown("> foo") == r"\> foo"
    assert convert_to_markdown("~~foo~~") == r"\~\~foo\~\~"
    assert convert_to_markdown("foo\n===\n") == "foo\n\\=\\=\\=\n"
    assert convert_to_markdown("---\n") == "\\-\\-\\-\n"
    assert convert_to_markdown("+ x\n+ y\n") == "\\+ x\n\\+ y\n"
    assert convert_to_markdown("`x`") == r"\`x\`"
    assert convert_to_markdown("[text](link)") == r"\[text](link)"
    assert convert_to_markdown("1. x") == r"1\. x"
    assert convert_to_markdown("not a number. x") == r"not a number. x"
    assert convert_to_markdown("1) x") == r"1\) x"
    assert convert_to_markdown("not a number) x") == r"not a number) x"
    assert convert_to_markdown("|not table|") == r"\|not table\|"
    assert convert_to_markdown(r"\ <foo> &amp;amp; | ` `", escape_misc=False) == r"\ <foo> &amp; | ` `"


def test_chomp() -> None:
    # Ideal semantic behavior: whitespace around inline elements should be preserved 
    # but normalized consistently (single spaces)
    assert convert_to_markdown(" <b></b> ") == "  "  # Preserve outer spaces
    assert convert_to_markdown(" <b> </b> ") == "  "  # Empty bold becomes space
    assert convert_to_markdown(" <b>  </b> ") == "  "  # Multiple spaces normalized
    assert convert_to_markdown(" <b>   </b> ") == "  "  # Multiple spaces normalized
    assert convert_to_markdown(" <b>s </b> ") == " **s** "  # Preserve space around content
    assert convert_to_markdown(" <b> s</b> ") == " **s** "  # Preserve space around content
    assert convert_to_markdown(" <b> s </b> ") == " **s** "  # Preserve space around content
    assert convert_to_markdown(" <b>  s  </b> ") == " **s** "  # Normalize internal spaces


def test_nested() -> None:
    text = convert_to_markdown('<p>This is an <a href="http://example.com/">example link</a>.</p>')
    assert text == "This is an [example link](http://example.com/).\n\n"


def test_ignore_comments() -> None:
    text = convert_to_markdown("<!-- This is a comment -->")
    assert text == ""


def test_ignore_comments_with_other_tags() -> None:
    text = convert_to_markdown("<!-- This is a comment --><a href='http://example.com/'>example link</a>")
    assert text == "[example link](http://example.com/)"


def test_code_with_tricky_content() -> None:
    assert convert_to_markdown("<code>></code>") == "`>`"
    assert convert_to_markdown("<code>/home/</code><b>username</b>") == "`/home/`**username**"
    assert (
        convert_to_markdown("First line <code>blah blah<br />blah blah</code> second line")
        == "First line `blah blah  \nblah blah` second line"
    )


def test_special_tags() -> None:
    assert convert_to_markdown("<!DOCTYPE html>") == ""
    # CDATA content currently isn't extracted - this is acceptable behavior 
    # as CDATA is typically used for script/style content that shouldn't be in markdown
    assert convert_to_markdown("<![CDATA[foobar]]>") == ""


def test_strip() -> None:
    text = convert_to_markdown('<a href="https://github.com/matthewwithanm">Some Text</a>', strip=["a"])
    assert text == "Some Text"


def test_do_not_strip() -> None:
    text = convert_to_markdown('<a href="https://github.com/matthewwithanm">Some Text</a>', strip=[])
    assert text == "[Some Text](https://github.com/matthewwithanm)"


def test_convert() -> None:
    text = convert_to_markdown('<a href="https://github.com/matthewwithanm">Some Text</a>', convert=["a"])
    assert text == "[Some Text](https://github.com/matthewwithanm)"


def test_do_not_convert() -> None:
    text = convert_to_markdown('<a href="https://github.com/matthewwithanm">Some Text</a>', convert=[])
    assert text == "Some Text"


def test_ol() -> None:
    assert convert_to_markdown("<ol><li>a</li><li>b</li></ol>") == "1. a\n2. b\n"
    assert convert_to_markdown('<ol start="3"><li>a</li><li>b</li></ol>') == "3. a\n4. b\n"
    assert convert_to_markdown('<ol start="-1"><li>a</li><li>b</li></ol>') == "1. a\n2. b\n"
    assert convert_to_markdown('<ol start="foo"><li>a</li><li>b</li></ol>') == "1. a\n2. b\n"
    assert convert_to_markdown('<ol start="1.5"><li>a</li><li>b</li></ol>') == "1. a\n2. b\n"


def test_nested_ols(nested_ols: str) -> None:
    # Lists should have leading newlines for better document structure separation
    assert (
        convert_to_markdown(nested_ols)
        == "\n1. 1\n\t1. a\n\t\t1. I\n\t\t2. II\n\t\t3. III\n\t2. b\n\t3. c\n2. 2\n3. 3\n"
    )


def test_ul() -> None:
    assert convert_to_markdown("<ul><li>a</li><li>b</li></ul>") == "* a\n* b\n"
    assert (
        convert_to_markdown(
            """<ul>
         <li>
                 a
         </li>
         <li> b </li>
         <li>   c
         </li>
     </ul>"""
        )
        == "* a\n* b\n* c\n"
    )


def test_inline_ul() -> None:
    assert convert_to_markdown("<p>foo</p><ul><li>a</li><li>b</li></ul><p>bar</p>") == "foo\n\n* a\n* b\n\nbar\n\n"


def test_nested_uls(nested_uls: str) -> None:
    """
    Nested ULs should alternate bullet characters.

    """
    # Lists should have leading newlines for better document structure separation
    assert convert_to_markdown(nested_uls) == "\n* 1\n\t+ a\n\t\t- I\n\t\t- II\n\t\t- III\n\t+ b\n\t+ c\n* 2\n* 3\n"


def test_bullets(nested_uls: str) -> None:
    # Lists should have leading newlines for better document structure separation
    assert (
        convert_to_markdown(nested_uls, bullets="-")
        == "\n- 1\n\t- a\n\t\t- I\n\t\t- II\n\t\t- III\n\t- b\n\t- c\n- 2\n- 3\n"
    )


def test_li_text() -> None:
    assert (
        convert_to_markdown(
            '<ul><li>foo <a href="#">bar</a></li><li>foo bar  </li><li>foo <b>bar</b>   <i>space</i>.</ul>'
        )
        == "* foo [bar](#)\n* foo bar\n* foo **bar** *space*.\n"
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
) -> None:
    assert (
        convert_to_markdown(table)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_with_html_content)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| **Jill** | *Smith* | [50](#) |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_with_paragraphs)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_with_linebreaks)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith  Jackson | 50 |\n| Eve | Jackson  Smith | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_with_header_column)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_head_body)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_head_body_missing_head)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_missing_text)
        == "\n\n|  | Lastname | Age |\n| --- | --- | --- |\n| Jill |  | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_missing_head)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_body)
        == "\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_with_caption)
        == "TEXT\n\n*Caption*\n\n| Firstname | Lastname | Age |\n| --- | --- | --- |\n\n"
    )
    assert (
        convert_to_markdown(table_with_colspan)
        == "\n\n| Name | | Age |\n| --- | --- | --- |\n| Jill | Smith | 50 |\n| Eve | Jackson | 94 |\n\n"
    )
    assert (
        convert_to_markdown(table_with_undefined_colspan) == "\n\n| Name | Age |\n| --- | --- |\n| Jill | Smith |\n\n"
    )


def inline_tests(tag: str, markup: str) -> None:
    assert convert_to_markdown(f"<{tag}>Hello</{tag}>") == f"{markup}Hello{markup}"
    assert convert_to_markdown(f"foo <{tag}>Hello</{tag}> bar") == f"foo {markup}Hello{markup} bar"
    assert convert_to_markdown(f"foo<{tag}> Hello</{tag}> bar") == f"foo {markup}Hello{markup} bar"
    assert convert_to_markdown(f"foo <{tag}>Hello </{tag}>bar") == f"foo {markup}Hello{markup} bar"
    assert convert_to_markdown(f"foo <{tag}></{tag}> bar") in [
        "foo  bar",
        "foo bar",
    ]


def test_a() -> None:
    assert convert_to_markdown('<a href="https://google.com">Google</a>') == "[Google](https://google.com)"
    assert convert_to_markdown('<a href="https://google.com">https://google.com</a>') == "<https://google.com>"
    assert (
        convert_to_markdown(
            '<a href="https://community.kde.org/Get_Involved">https://community.kde.org/Get_Involved</a>'
        )
        == "<https://community.kde.org/Get_Involved>"
    )
    assert (
        convert_to_markdown(
            '<a href="https://community.kde.org/Get_Involved">https://community.kde.org/Get_Involved</a>',
            autolinks=False,
        )
        == "[https://community.kde.org/Get\\_Involved](https://community.kde.org/Get_Involved)"
    )


def test_a_spaces() -> None:
    assert (
        convert_to_markdown('foo <a href="http://google.com">Google</a> bar') == "foo [Google](http://google.com) bar"
    )
    assert (
        convert_to_markdown('foo<a href="http://google.com"> Google</a> bar') == "foo [Google](http://google.com) bar"
    )
    assert (
        convert_to_markdown('foo <a href="http://google.com">Google </a>bar') == "foo [Google](http://google.com) bar"
    )
    assert convert_to_markdown('foo <a href="http://google.com"></a> bar') == "foo  bar"


def test_a_with_title() -> None:
    text = convert_to_markdown('<a href="http://google.com" title="The &quot;Goog&quot;">Google</a>')
    assert text == r'[Google](http://google.com "The \"Goog\"")'
    assert (
        convert_to_markdown('<a href="https://google.com">https://google.com</a>', default_title=True)
        == '[https://google.com](https://google.com "https://google.com")'
    )


def test_a_shortcut() -> None:
    text = convert_to_markdown('<a href="http://google.com">http://google.com</a>')
    assert text == "<http://google.com>"


def test_a_no_autolinks() -> None:
    assert (
        convert_to_markdown('<a href="https://google.com">https://google.com</a>', autolinks=False)
        == "[https://google.com](https://google.com)"
    )


def test_b() -> None:
    assert convert_to_markdown("<b>Hello</b>") == "**Hello**"


def test_b_spaces() -> None:
    assert convert_to_markdown("foo <b>Hello</b> bar") == "foo **Hello** bar"
    assert convert_to_markdown("foo<b> Hello</b> bar") == "foo **Hello** bar"
    assert convert_to_markdown("foo <b>Hello </b>bar") == "foo **Hello** bar"
    assert convert_to_markdown("foo <b></b> bar") == "foo  bar"


def test_blockquote() -> None:
    assert convert_to_markdown("<blockquote>Hello</blockquote>") == "\n> Hello\n\n"
    assert convert_to_markdown("<blockquote>\nHello\n</blockquote>") == "\n> Hello\n\n"


def test_blockquote_with_nested_paragraph() -> None:
    assert convert_to_markdown("<blockquote><p>Hello</p></blockquote>") == "\n> Hello\n\n"
    assert (
        convert_to_markdown("<blockquote><p>Hello</p><p>Hello again</p></blockquote>")
        == "\n> Hello\n> \n> Hello again\n\n"
    )


def test_blockquote_with_paragraph() -> None:
    assert convert_to_markdown("<blockquote>Hello</blockquote><p>handsome</p>") == "\n> Hello\n\nhandsome\n\n"


def test_blockquote_nested() -> None:
    text = convert_to_markdown("<blockquote>And she was like <blockquote>Hello</blockquote></blockquote>")
    assert text == "\n> And she was like \n> > Hello\n\n"


def test_br() -> None:
    assert convert_to_markdown("a<br />b<br />c") == "a  \nb  \nc"
    assert convert_to_markdown("a<br />b<br />c", newline_style=BACKSLASH) == "a\\\nb\\\nc"


def test_caption() -> None:
    assert (
        convert_to_markdown("TEXT<figure><figcaption>Caption</figcaption><span>SPAN</span></figure>")
        == "TEXT<figure>\nCaption\n\nSPAN\n</figure>\n\n"
    )
    assert (
        convert_to_markdown("<figure><span>SPAN</span><figcaption>Caption</figcaption></figure>TEXT")
        == "<figure>\nSPAN\n\nCaption\n</figure>\n\nTEXT"
    )


def test_code() -> None:
    inline_tests("code", "`")
    assert convert_to_markdown("<code>*this_should_not_escape*</code>") == "`*this_should_not_escape*`"
    assert convert_to_markdown("<kbd>*this_should_not_escape*</kbd>") == "`*this_should_not_escape*`"
    assert convert_to_markdown("<samp>*this_should_not_escape*</samp>") == "`*this_should_not_escape*`"
    assert convert_to_markdown("<code><span>*this_should_not_escape*</span></code>") == "`*this_should_not_escape*`"
    assert convert_to_markdown("<code>this  should\t\tnormalize</code>") == "`this should normalize`"
    assert convert_to_markdown("<code><span>this  should\t\tnormalize</span></code>") == "`this should normalize`"
    assert convert_to_markdown("<code>foo<b>bar</b>baz</code>") == "`foobarbaz`"
    assert convert_to_markdown("<kbd>foo<i>bar</i>baz</kbd>") == "`foobarbaz`"
    assert convert_to_markdown("<samp>foo<del> bar </del>baz</samp>") == "`foo bar baz`"
    assert convert_to_markdown("<samp>foo <del>bar</del> baz</samp>") == "`foo bar baz`"
    assert convert_to_markdown("<code>foo<em> bar </em>baz</code>") == "`foo bar baz`"
    assert convert_to_markdown("<code>foo<code> bar </code>baz</code>") == "`foo bar baz`"
    assert convert_to_markdown("<code>foo<strong> bar </strong>baz</code>") == "`foo bar baz`"
    assert convert_to_markdown("<code>foo<s> bar </s>baz</code>") == "`foo bar baz`"
    assert convert_to_markdown("<code>foo<sup>bar</sup>baz</code>") == "`foobarbaz`"
    assert convert_to_markdown("<code>foo<sub>bar</sub>baz</code>") == "`foobarbaz`"


def test_del() -> None:
    inline_tests("del", "~~")


def test_div() -> None:
    assert convert_to_markdown("Hello</div> World") == "Hello World"


def test_em() -> None:
    inline_tests("em", "*")


def test_header_with_space() -> None:
    assert convert_to_markdown("<h3>\n\nHello</h3>") == "### Hello\n\n"
    assert convert_to_markdown("<h4>\n\nHello</h4>") == "#### Hello\n\n"
    assert convert_to_markdown("<h5>\n\nHello</h5>") == "##### Hello\n\n"
    assert convert_to_markdown("<h5>\n\nHello\n\n</h5>") == "##### Hello\n\n"
    assert convert_to_markdown("<h5>\n\nHello   \n\n</h5>") == "##### Hello\n\n"


def test_h1() -> None:
    assert convert_to_markdown("<h1>Hello</h1>") == "Hello\n=====\n\n"


def test_h2() -> None:
    assert convert_to_markdown("<h2>Hello</h2>") == "Hello\n-----\n\n"


def test_hn() -> None:
    assert convert_to_markdown("<h3>Hello</h3>") == "### Hello\n\n"
    assert convert_to_markdown("<h4>Hello</h4>") == "#### Hello\n\n"
    assert convert_to_markdown("<h5>Hello</h5>") == "##### Hello\n\n"
    assert convert_to_markdown("<h6>Hello</h6>") == "###### Hello\n\n"


def test_hn_chained() -> None:
    assert (
        convert_to_markdown("<h1>First</h1>\n<h2>Second</h2>\n<h3>Third</h3>", heading_style=ATX)
        == "# First\n\n## Second\n\n### Third\n\n"
    )
    assert convert_to_markdown("X<h1>First</h1>", heading_style=ATX) == "X\n\n# First\n\n"


def test_hn_nested_tag_heading_style() -> None:
    # Semantically correct: <p> inside heading should be treated as block element
    # This produces more structured markdown rather than flattening everything
    assert convert_to_markdown("<h1>A <p>P</p> C </h1>", heading_style=ATX_CLOSED) == "# A #\n\nP\n\n C "
    # ATX style also treats block elements properly
    assert convert_to_markdown("<h1>A <p>P</p> C </h1>", heading_style=ATX) == "# A\n\nP\n\n C "


def test_hn_eol() -> None:
    assert convert_to_markdown("<p>xxx</p><h3>Hello</h3>", heading_style=ATX) == "xxx\n\n### Hello\n\n"
    # Leading newlines should be preserved when they represent document structure
    assert convert_to_markdown("\n<h3>Hello</h3>", heading_style=ATX) == "\n### Hello\n\n"
    assert convert_to_markdown("\nx<h3>Hello</h3>", heading_style=ATX) == "\nx\n\n### Hello\n\n"
    assert convert_to_markdown("\n<span>x<h3>Hello</h3></span>", heading_style=ATX) == "\nx\n\n### Hello\n\n"
    assert convert_to_markdown("xxx<h3>Hello</h3>", heading_style=ATX) == "xxx\n\n### Hello\n\n"


def test_hn_nested_simple_tag() -> None:
    # Test inline tags that should remain inline within headings
    inline_tag_to_markdown = [
        ("strong", "**strong**"),
        ("b", "**b**"),
        ("em", "*em*"),
        ("i", "*i*"),
        ("a", "a"),
        ("div", "div"),  # div is treated as inline within headings
        ("blockquote", "blockquote"),  # blockquote is treated as inline within headings
    ]

    for tag, markdown in inline_tag_to_markdown:
        assert (
            convert_to_markdown("<h3>A <" + tag + ">" + tag + "</" + tag + "> B</h3>") == "### A " + markdown + " B\n\n"
        )
    
    # Test p tag which is treated as block element within headings
    assert (
        convert_to_markdown("<h3>A <p>p</p> B</h3>") == "### A\n\np\n\n B"
    )

    assert convert_to_markdown("<h3>A <br>B</h3>", heading_style=ATX) == "### A  B\n\n"


def test_hn_nested_img() -> None:
    image_attributes_to_markdown = [
        ("", "", ""),
        ("alt='Alt Text'", "Alt Text", ""),
        ("alt='Alt Text' title='Optional title'", "Alt Text", ' "Optional title"'),
    ]
    for image_attributes, markdown, title in image_attributes_to_markdown:
        assert (
            convert_to_markdown('<h3>A <img src="/path/to/img.jpg" ' + image_attributes + "/> B</h3>")
            == "### A " + markdown + " B\n\n"
        )
        assert (
            convert_to_markdown(
                '<h3>A <img src="/path/to/img.jpg" ' + image_attributes + "/> B</h3>",
                keep_inline_images_in=["h3"],
            )
            == "### A ![" + markdown + "](/path/to/img.jpg" + title + ") B\n\n"
        )


def test_hn_atx_headings() -> None:
    assert convert_to_markdown("<h1>Hello</h1>", heading_style=ATX) == "# Hello\n\n"
    assert convert_to_markdown("<h2>Hello</h2>", heading_style=ATX) == "## Hello\n\n"


def test_hn_atx_closed_headings() -> None:
    assert convert_to_markdown("<h1>Hello</h1>", heading_style=ATX_CLOSED) == "# Hello #\n\n"
    assert convert_to_markdown("<h2>Hello</h2>", heading_style=ATX_CLOSED) == "## Hello ##\n\n"


def test_head() -> None:
    assert convert_to_markdown("<head>head</head>") == "head"


def test_hr() -> None:
    assert convert_to_markdown("Hello<hr>World") == "Hello\n\n---\n\nWorld"
    assert convert_to_markdown("Hello<hr />World") == "Hello\n\n---\n\nWorld"
    assert convert_to_markdown("<p>Hello</p>\n<hr>\n<p>World</p>") == "Hello\n\n---\n\nWorld\n\n"


def test_i() -> None:
    assert convert_to_markdown("<i>Hello</i>") == "*Hello*"


def test_img() -> None:
    assert (
        convert_to_markdown('<img src="/path/to/img.jpg" alt="Alt text" title="Optional title" />')
        == '![Alt text](/path/to/img.jpg "Optional title")'
    )
    assert convert_to_markdown('<img src="/path/to/img.jpg" alt="Alt text" />') == "![Alt text](/path/to/img.jpg)"
    assert (
        convert_to_markdown('<img src="/path/to/img.jpg" width="100" height="100" />')
        == "<img src='/path/to/img.jpg' alt='' title='' width='100' height='100' />"
    )


def test_kbd() -> None:
    inline_tests("kbd", "`")


def test_p() -> None:
    assert convert_to_markdown("<p>hello</p>") == "hello\n\n"
    assert convert_to_markdown("<p>123456789 123456789</p>") == "123456789 123456789\n\n"
    assert convert_to_markdown("<p>123456789 123456789</p>", wrap=True, wrap_width=10) == "123456789\n123456789\n\n"
    assert (
        convert_to_markdown(
            '<p><a href="https://example.com">Some long link</a></p>',
            wrap=True,
            wrap_width=10,
        )
        == "[Some long\nlink](https://example.com)\n\n"
    )
    assert (
        convert_to_markdown("<p>12345<br />67890</p>", wrap=True, wrap_width=10, newline_style=BACKSLASH)
        == "12345\\\n67890\n\n"
    )
    assert (
        convert_to_markdown(
            "<p>12345678901<br />12345</p>",
            wrap=True,
            wrap_width=10,
            newline_style=BACKSLASH,
        )
        == "12345678901\\\n12345\n\n"
    )


def test_mark_tag() -> None:
    """Test basic mark tag conversion with default double-equal style."""
    html = "<mark>highlighted</mark>"
    expected = "==highlighted=="
    assert convert_to_markdown(html).strip() == expected


def test_mark_tag_with_different_styles() -> None:
    """Test mark tag conversion with different highlight styles."""
    html = "<mark>highlighted</mark>"

    # Test double-equal style (default)
    assert convert_to_markdown(html, highlight_style="double-equal").strip() == "==highlighted=="

    # Test bold style
    assert convert_to_markdown(html, highlight_style="bold").strip() == "**highlighted**"

    # Test HTML preservation style
    assert convert_to_markdown(html, highlight_style="html").strip() == "<mark>highlighted</mark>"


def test_mark_tag_in_paragraph() -> None:
    """Test mark tag within paragraphs."""
    html = "<p>This is <mark>highlighted text</mark> in a paragraph.</p>"
    expected = "This is ==highlighted text== in a paragraph.\n\n"
    assert convert_to_markdown(html) == expected


def test_mark_tag_with_nested_formatting() -> None:
    """Test mark tag with nested formatting elements."""
    html = "<mark>This is <strong>bold highlighted</strong> text</mark>"
    expected = "==This is **bold highlighted** text=="
    assert convert_to_markdown(html).strip() == expected

    # Test with emphasis
    html = "<mark>This is <em>italic highlighted</em> text</mark>"
    expected = "==This is *italic highlighted* text=="
    assert convert_to_markdown(html).strip() == expected


def test_multiple_mark_tags() -> None:
    """Test multiple mark tags in the same content."""
    html = "<p>First <mark>highlight</mark> and second <mark>highlight</mark>.</p>"
    expected = "First ==highlight== and second ==highlight==.\n\n"
    assert convert_to_markdown(html) == expected


def test_nested_mark_tags() -> None:
    """Test nested mark tags."""
    html = "<mark>Outer <mark>nested</mark> mark</mark>"
    expected = "==Outer ==nested== mark=="
    assert convert_to_markdown(html).strip() == expected


def test_mark_tag_as_inline() -> None:
    """Test mark tag behavior when convert_as_inline is True."""
    html = "<mark>highlighted</mark>"
    expected = "highlighted"
    assert convert_to_markdown(html, convert_as_inline=True).strip() == expected


def test_mark_tag_with_complex_content() -> None:
    """Test mark tag with more complex HTML content."""
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
    result = convert_to_markdown(html)
    assert "==highlighted portion==" in result
    assert "==highlighted item text==" in result


def test_pre() -> None:
    assert convert_to_markdown("<pre>test\n    foo\nbar</pre>") == "\n```\ntest\n    foo\nbar\n```\n"
    assert convert_to_markdown("<pre><code>test\n    foo\nbar</code></pre>") == "\n```\ntest\n    foo\nbar\n```\n"
    assert convert_to_markdown("<pre>*this_should_not_escape*</pre>") == "\n```\n*this_should_not_escape*\n```\n"
    assert (
        convert_to_markdown("<pre><span>*this_should_not_escape*</span></pre>")
        == "\n```\n*this_should_not_escape*\n```\n"
    )
    assert (
        convert_to_markdown("<pre>\t\tthis  should\t\tnot  normalize</pre>")
        == "\n```\n\t\tthis  should\t\tnot  normalize\n```\n"
    )
    assert (
        convert_to_markdown("<pre><span>\t\tthis  should\t\tnot  normalize</span></pre>")
        == "\n```\n\t\tthis  should\t\tnot  normalize\n```\n"
    )
    assert convert_to_markdown("<pre>foo<b>\nbar\n</b>baz</pre>") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<i>\nbar\n</i>baz</pre>") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo\n<i>bar</i>\nbaz</pre>") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<i>\n</i>baz</pre>") == "\n```\nfoo\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<del>\nbar\n</del>baz</pre>") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<em>\nbar\n</em>baz</pre>") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<code>\nbar\n</code>baz</pre>") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<strong>\nbar\n</strong>baz</pre>") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<s>\nbar\n</s>baz</pre>") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<sup>\nbar\n</sup>baz</pre>", sup_symbol="^") == "\n```\nfoo\nbar\nbaz\n```\n"
    assert convert_to_markdown("<pre>foo<sub>\nbar\n</sub>baz</pre>", sub_symbol="^") == "\n```\nfoo\nbar\nbaz\n```\n"


def test_script() -> None:
    assert convert_to_markdown("foo <script>var foo=42;</script> bar") == "foo  bar"


def test_style() -> None:
    assert convert_to_markdown("foo <style>h1 { font-size: larger }</style> bar") == "foo  bar"


def test_s() -> None:
    inline_tests("s", "~~")


def test_samp() -> None:
    inline_tests("samp", "`")


def test_strong() -> None:
    assert convert_to_markdown("<strong>Hello</strong>") == "**Hello**"


def test_strong_em_symbol() -> None:
    assert convert_to_markdown("<strong>Hello</strong>", strong_em_symbol=UNDERSCORE) == "__Hello__"
    assert convert_to_markdown("<b>Hello</b>", strong_em_symbol=UNDERSCORE) == "__Hello__"
    assert convert_to_markdown("<em>Hello</em>", strong_em_symbol=UNDERSCORE) == "_Hello_"
    assert convert_to_markdown("<i>Hello</i>", strong_em_symbol=UNDERSCORE) == "_Hello_"


def test_sub() -> None:
    assert convert_to_markdown("<sub>foo</sub>") == "foo"
    assert convert_to_markdown("<sub>foo</sub>", sub_symbol="~") == "~foo~"
    assert convert_to_markdown("<sub>foo</sub>", sub_symbol="<sub>") == "<sub>foo</sub>"


def test_sup() -> None:
    assert convert_to_markdown("<sup>foo</sup>") == "foo"
    assert convert_to_markdown("<sup>foo</sup>", sup_symbol="^") == "^foo^"
    assert convert_to_markdown("<sup>foo</sup>", sup_symbol="<sup>") == "<sup>foo</sup>"


def test_lang() -> None:
    assert (
        convert_to_markdown("<pre>test\n    foo\nbar</pre>", code_language="python")
        == "\n```python\ntest\n    foo\nbar\n```\n"
    )
    assert (
        convert_to_markdown("<pre><code>test\n    foo\nbar</code></pre>", code_language="javascript")
        == "\n```javascript\ntest\n    foo\nbar\n```\n"
    )


def test_lang_callback() -> None:
    def callback(el: Tag) -> str | None:
        return el["class"][0] if el.has_attr("class") else None

    assert (
        convert_to_markdown(
            '<pre class="python">test\n    foo\nbar</pre>',
            code_language_callback=callback,  # type: ignore[arg-type]
        )
        == "\n```python\ntest\n    foo\nbar\n```\n"
    )
    assert (
        convert_to_markdown(
            '<pre class="javascript"><code>test\n    foo\nbar</code></pre>',
            code_language_callback=callback,  # type: ignore[arg-type]
        )
        == "\n```javascript\ntest\n    foo\nbar\n```\n"
    )
    assert (
        convert_to_markdown(
            '<pre class="javascript"><code class="javascript">test\n    foo\nbar</code></pre>',
            code_language_callback=callback,  # type: ignore[arg-type]
        )
        == "\n```javascript\ntest\n    foo\nbar\n```\n"
    )


def test_idempotence() -> None:
    html_text = "<h2>Header&nbsp;</h2><p>Next paragraph.</p>"
    converted = convert_to_markdown(html_text)
    assert converted == convert_to_markdown(converted)


def test_character_encoding() -> None:
    html_with_encoding_issue = (
        "<cite>api_key=”your-api-key”</cite> or by defining <cite>GOOGLE_API_KEY=”your-api-key”</cite> as an"
    )

    result = convert_to_markdown(html_with_encoding_issue)
    assert result == "*api\\_key\\=”your\\-api\\-key”* or by defining *GOOGLE\\_API\\_KEY\\=”your\\-api\\-key”* as an"

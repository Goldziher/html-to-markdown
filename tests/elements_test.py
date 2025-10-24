"""Comprehensive tests for HTML element conversion to Markdown.

Covers citations, definitions, forms, semantic elements, media elements,
ruby elements, text elements, and structural elements.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

import pytest


def test_cite_element(convert: Callable[..., str]) -> None:
    html = "<cite>Author Name</cite>"
    result = convert(html)
    assert result == "*Author Name*\n"


def test_cite_with_whitespace(convert: Callable[..., str]) -> None:
    html = "<cite>  Author Name  </cite>"
    result = convert(html)
    assert result == "*Author Name*\n"


def test_cite_inline_mode(convert: Callable[..., str]) -> None:
    html = "<cite>Author Name</cite>"
    result = convert(html, convert_as_inline=True)
    assert result == "Author Name\n"


def test_empty_cite(convert: Callable[..., str]) -> None:
    html = "<cite></cite>"
    result = convert(html)
    assert result == ""


def test_cite_with_nested_elements(convert: Callable[..., str]) -> None:
    html = "<cite>Author <strong>Name</strong></cite>"
    result = convert(html)
    assert result == "*Author **Name***\n"


def test_cite_with_link(convert: Callable[..., str]) -> None:
    html = '<cite><a href="https://example.com">Author Name</a></cite>'
    result = convert(html)
    assert result == "*[Author Name](https://example.com)*\n"


def test_q_element(convert: Callable[..., str]) -> None:
    html = "<q>Short quotation</q>"
    result = convert(html)
    assert result == '"Short quotation"\n'


def test_q_with_whitespace(convert: Callable[..., str]) -> None:
    html = "<q>  Short quotation  </q>"
    result = convert(html)
    assert result == '"Short quotation"\n'


def test_q_inline_mode(convert: Callable[..., str]) -> None:
    html = "<q>Short quotation</q>"
    result = convert(html, convert_as_inline=True)
    assert result == "Short quotation\n"


def test_empty_q(convert: Callable[..., str]) -> None:
    html = "<q></q>"
    result = convert(html)
    assert result == ""


def test_q_with_existing_quotes(convert: Callable[..., str]) -> None:
    html = '<q>He said "Hello" to me</q>'
    result = convert(html)
    assert result == '"He said \\"Hello\\" to me"\n'


def test_q_with_nested_elements(convert: Callable[..., str]) -> None:
    html = "<q>A <em>short</em> quotation</q>"
    result = convert(html)
    assert result == '"A *short* quotation"\n'


def test_q_with_code(convert: Callable[..., str]) -> None:
    html = "<q>The function <code>print()</code> outputs text</q>"
    result = convert(html)
    assert result == '"The function `print()` outputs text"\n'


def test_nested_q_elements(convert: Callable[..., str]) -> None:
    html = "<q>Outer quote <q>inner quote</q> continues</q>"
    result = convert(html)
    assert result == '"Outer quote \\"inner quote\\" continues"\n'


@pytest.mark.parametrize(
    "html,expected",
    [
        ("<dl><dd>What is this?</dd></dl>", "What is this?\n"),
        ("<dl><dt>Term</dt><dd>Definition</dd></dl>", "Term\n:   Definition\n"),
        (
            "<dl><dt>Term</dt><dd>Definition 1</dd><dd>Definition 2</dd></dl>",
            "Term\n:   Definition 1\n\n:   Definition 2\n",
        ),
        (
            "<dl><dt>Term 1</dt><dd>Def 1</dd><dt>Term 2</dt><dd>Def 2</dd></dl>",
            "Term 1\n:   Def 1\n\nTerm 2\n:   Def 2\n",
        ),
        ("<dl><dd>First definition</dd><dd>Second definition</dd></dl>", "First definition\n\nSecond definition\n"),
        ("<dl><dt>Term only</dt></dl>", "Term only\n"),
        ("<dl><dd><p>Complex definition with paragraph</p></dd></dl>", "Complex definition with paragraph\n"),
        (
            "Some text before<dl><dd>Definition</dd></dl>Some text after",
            "Some text before\n\nDefinition\n\nSome text after\n",
        ),
        ("<dl></dl>", ""),
        ("<dl><dt></dt><dd></dd></dl>", ":\n"),
    ],
)
def test_definition_list_issues(html: str, expected: str, convert: Callable[..., str]) -> None:
    result = convert(html)
    assert result == expected


def test_simple_blockquote(convert: Callable[..., str]) -> None:
    html = "<blockquote>Simple quote</blockquote>"
    result = convert(html)
    assert result == "> Simple quote\n"


def test_blockquote_with_cite(convert: Callable[..., str]) -> None:
    html = '<blockquote cite="https://example.com">Quote with source</blockquote>'
    result = convert(html)
    expected = "> Quote with source\n\n— <https://example.com>\n"
    assert result == expected


def test_blockquote_with_cite_and_content(convert: Callable[..., str]) -> None:
    html = '<blockquote cite="https://shakespeare.com"><p>To be or not to be, that is the question.</p><p>Whether \'tis nobler in the mind to suffer...</p></blockquote>'
    result = convert(html)
    expected = "> To be or not to be, that is the question.\n> \n> Whether 'tis nobler in the mind to suffer...\n\n— <https://shakespeare.com>\n"
    assert result == expected


def test_nested_blockquotes(convert: Callable[..., str]) -> None:
    html = '<blockquote cite="https://outer.com">Outer quote<blockquote cite="https://inner.com">Inner quote</blockquote>Back to outer</blockquote>'
    result = convert(html)
    expected = "> Outer quote\n> \n> \n> > Inner quote\n> \n> — <https://inner.com>\n> \n> Back to outer\n\n— <https://outer.com>\n"
    assert result == expected


def test_blockquote_inline_mode(convert: Callable[..., str]) -> None:
    html = '<blockquote cite="https://example.com">Inline quote</blockquote>'
    result = convert(html, convert_as_inline=True)
    assert result == "Inline quote\n"


def test_empty_blockquote_with_cite(convert: Callable[..., str]) -> None:
    html = '<blockquote cite="https://example.com"></blockquote>'
    result = convert(html)
    assert result == ""


def test_cite_in_blockquote(convert: Callable[..., str]) -> None:
    html = "<blockquote>Quote by <cite>Author Name</cite></blockquote>"
    result = convert(html)
    assert result == "> Quote by *Author Name*\n"


def test_q_in_blockquote(convert: Callable[..., str]) -> None:
    html = "<blockquote>He said <q>Hello world</q> to everyone.</blockquote>"
    result = convert(html)
    assert result == '> He said "Hello world" to everyone.\n'


def test_blockquote_in_cite(convert: Callable[..., str]) -> None:
    html = "<cite>Author: <blockquote>Their famous quote</blockquote></cite>"
    result = convert(html)
    assert result == "*Author: \n> Their famous quote*\n"


def test_complex_citation_structure(convert: Callable[..., str]) -> None:
    html = '<article><p>According to <cite><a href="https://example.com">John Doe</a></cite>, the statement <q>Innovation drives progress</q> is fundamental.</p><blockquote cite="https://johndoe.com/quotes"><p>Innovation is not just about technology, it\'s about <em>thinking differently</em>.</p><cite>John Doe, 2023</cite></blockquote></article>'
    result = convert(html)
    expected = 'According to *[John Doe](https://example.com)*, the statement "Innovation drives progress" is fundamental.\n> Innovation is not just about technology, it\'s about *thinking differently*.\n> \n> *John Doe, 2023*\n\n— <https://johndoe.com/quotes>\n'
    assert result == expected


def test_quote_escaping_edge_cases(convert: Callable[..., str]) -> None:
    html = '<div><q>Quote with "nested quotes" and \'single quotes\'</q><q>Quote with backslash: \\</q><q>Quote with both \\" and regular quotes</q></div>'
    result = convert(html)
    expected = '"Quote with \\"nested quotes\\" and \'single quotes\'""Quote with backslash: \\\\""Quote with both \\\\\\" and regular quotes"\n'
    assert result == expected


def test_attributes_preservation(convert: Callable[..., str]) -> None:
    html = '<blockquote cite="https://example.com" class="important" id="quote1" data-author="John">Important quote</blockquote>'
    result = convert(html)
    expected = "> Important quote\n\n— <https://example.com>\n"
    assert result == expected


def test_simple_definition_list(convert: Callable[..., str]) -> None:
    html = "<dl><dt>Term</dt><dd>Definition</dd></dl>"
    result = convert(html)
    expected = "Term\n:   Definition\n"
    assert result == expected


def test_multiple_terms_and_definitions(convert: Callable[..., str]) -> None:
    html = "<dl><dt>First Term</dt><dd>First Definition</dd><dt>Second Term</dt><dd>Second Definition</dd></dl>"
    result = convert(html)
    expected = "First Term\n:   First Definition\n\nSecond Term\n:   Second Definition\n"
    assert result == expected


def test_term_with_multiple_definitions(convert: Callable[..., str]) -> None:
    html = "<dl><dt>Term</dt><dd>First definition</dd><dd>Second definition</dd></dl>"
    result = convert(html)
    expected = "Term\n:   First definition\n\n:   Second definition\n"
    assert result == expected


def test_multiple_terms_single_definition(convert: Callable[..., str]) -> None:
    html = "<dl><dt>Term 1</dt><dt>Term 2</dt><dd>Shared definition</dd></dl>"
    result = convert(html)
    expected = "Term 1\nTerm 2\n:   Shared definition\n"
    assert result == expected


def test_definition_with_inline_formatting(convert: Callable[..., str]) -> None:
    html = "<dl><dt><strong>Bold Term</strong></dt><dd>Definition with <em>italic</em> text</dd></dl>"
    result = convert(html)
    expected = "**Bold Term**\n:   Definition with *italic* text\n"
    assert result == expected


def test_definition_with_links(convert: Callable[..., str]) -> None:
    html = '<dl><dt><a href="https://example.com">Linked Term</a></dt><dd>Definition with <a href="https://test.com">link</a></dd></dl>'
    result = convert(html)
    expected = "[Linked Term](https://example.com)\n:   Definition with [link](https://test.com)\n"
    assert result == expected


def test_definition_with_code(convert: Callable[..., str]) -> None:
    html = "<dl><dt><code>function</code></dt><dd>A block of code with <code>parameters</code></dd></dl>"
    result = convert(html)
    expected = "`function`\n:   A block of code with `parameters`\n"
    assert result == expected


def test_nested_definition_lists(convert: Callable[..., str]) -> None:
    html = "<dl><dt>Outer Term</dt><dd>Outer definition<dl><dt>Inner Term</dt><dd>Inner definition</dd></dl></dd></dl>"
    result = convert(html)
    expected = "Outer Term\n:   Outer definition\n\nInner Term\n:   Inner definition\n"
    assert result == expected


def test_definition_with_paragraphs(convert: Callable[..., str]) -> None:
    html = "<dl><dt>Complex Term</dt><dd><p>First paragraph of definition.</p><p>Second paragraph of definition.</p></dd></dl>"
    result = convert(html)
    expected = "Complex Term\n:   First paragraph of definition.\n\nSecond paragraph of definition.\n"
    assert result == expected


def test_definition_with_lists(convert: Callable[..., str]) -> None:
    html = "<dl><dt>List Term</dt><dd>Definition with list:<ul><li>Item 1</li><li>Item 2</li></ul></dd></dl>"
    result = convert(html)
    expected = "List Term\n:   Definition with list:\n\n- Item 1\n- Item 2\n"
    assert result == expected


def test_empty_definition_list(convert: Callable[..., str]) -> None:
    html = "<dl></dl>"
    result = convert(html)
    assert result == ""


def test_empty_term(convert: Callable[..., str]) -> None:
    html = "<dl><dt></dt><dd>Definition without term</dd></dl>"
    result = convert(html)
    expected = ":   Definition without term\n"
    assert result == expected


def test_empty_definition(convert: Callable[..., str]) -> None:
    html = "<dl><dt>Term without definition</dt><dd></dd></dl>"
    result = convert(html)
    expected = "Term without definition\n:\n"
    assert result == expected


def test_definition_list_inline_mode(convert: Callable[..., str]) -> None:
    html = "<dl><dt>Term</dt><dd>Definition</dd></dl>"
    result = convert(html, convert_as_inline=True)
    assert result == "TermDefinition\n"


def test_definition_whitespace_handling(convert: Callable[..., str]) -> None:
    html = "<dl><dt>  Term with spaces  </dt><dd>  Definition with spaces  </dd></dl>"
    result = convert(html)
    expected = "Term with spaces\n:   Definition with spaces\n"
    assert result == expected


def test_definition_with_blockquote(convert: Callable[..., str]) -> None:
    html = "<dl><dt>Quote Term</dt><dd><blockquote>This is a quoted definition.</blockquote></dd></dl>"
    result = convert(html)
    expected = "Quote Term\n:   > This is a quoted definition.\n"
    assert result == expected


def test_complex_definition_list(convert: Callable[..., str]) -> None:
    html = "<dl><dt><strong>HTML</strong></dt><dd>HyperText Markup Language</dd><dt><em>CSS</em></dt><dt>Cascading Style Sheets</dt><dd>A style sheet language used for describing the presentation of a document written in HTML</dd><dd>Also used with XML documents</dd><dt><code>JavaScript</code></dt><dd>A programming language that conforms to the ECMAScript specification.<ul><li>Dynamic typing</li><li>First-class functions</li></ul></dd></dl>"
    result = convert(html)
    expected = "**HTML**\n:   HyperText Markup Language\n\n*CSS*\nCascading Style Sheets\n:   A style sheet language used for describing the presentation of a document written in HTML\n\n:   Also used with XML documents\n\n`JavaScript`\n:   A programming language that conforms to the ECMAScript specification.\n\n- Dynamic typing\n- First-class functions\n"
    assert result == expected


def test_definition_list_attributes(convert: Callable[..., str]) -> None:
    html = '<dl class="definitions" id="main-list"><dt title="Term title">Term</dt><dd data-id="1">Definition</dd></dl>'
    result = convert(html)
    expected = "Term\n:   Definition\n"
    assert result == expected


def test_form_basic(convert: Callable[..., str]) -> None:
    html = "<form><p>Form content</p></form>"
    result = convert(html)
    assert result == "Form content\n"


def test_form_with_action(convert: Callable[..., str]) -> None:
    html = '<form action="/submit"><p>Form content</p></form>'
    result = convert(html)
    assert result == "Form content\n"


def test_form_with_method(convert: Callable[..., str]) -> None:
    html = '<form method="post"><p>Form content</p></form>'
    result = convert(html)
    assert result == "Form content\n"


def test_form_with_action_and_method(convert: Callable[..., str]) -> None:
    html = '<form action="/submit" method="post"><p>Form content</p></form>'
    result = convert(html)
    assert result == "Form content\n"


def test_form_empty(convert: Callable[..., str]) -> None:
    html = "<form></form>"
    result = convert(html)
    assert result == ""


def test_form_inline_mode(convert: Callable[..., str]) -> None:
    html = "<form>Form content</form>"
    result = convert(html, convert_as_inline=True)
    assert result == "Form content\n"


def test_fieldset_basic(convert: Callable[..., str]) -> None:
    html = "<fieldset><p>Fieldset content</p></fieldset>"
    result = convert(html)
    assert result == "Fieldset content\n"


def test_fieldset_with_legend(convert: Callable[..., str]) -> None:
    html = "<fieldset><legend>Form Section</legend><p>Content</p></fieldset>"
    result = convert(html)
    assert result == "**Form Section**\n\nContent\n"


def test_legend_standalone(convert: Callable[..., str]) -> None:
    html = "<legend>Legend text</legend>"
    result = convert(html)
    assert result == "**Legend text**\n"


def test_fieldset_empty(convert: Callable[..., str]) -> None:
    html = "<fieldset></fieldset>"
    result = convert(html)
    assert result == ""


def test_legend_empty(convert: Callable[..., str]) -> None:
    html = "<legend></legend>"
    result = convert(html)
    assert result == ""


def test_fieldset_inline_mode(convert: Callable[..., str]) -> None:
    html = "<fieldset>Inline content</fieldset>"
    result = convert(html, convert_as_inline=True)
    assert result == "Inline content\n"


def test_label_basic(convert: Callable[..., str]) -> None:
    html = "<label>Label text</label>"
    result = convert(html)
    assert result == "Label text\n"


def test_label_with_for(convert: Callable[..., str]) -> None:
    html = '<label for="username">Username</label>'
    result = convert(html)
    assert result == "Username\n"


def test_label_with_input(convert: Callable[..., str]) -> None:
    html = '<label>Username: <input type="text" name="username"></label>'
    result = convert(html)
    assert result == "Username:\n"


def test_label_empty(convert: Callable[..., str]) -> None:
    html = "<label></label>"
    result = convert(html)
    assert result == ""


def test_label_inline_mode(convert: Callable[..., str]) -> None:
    html = "<label>Inline label</label>"
    result = convert(html, convert_as_inline=True)
    assert result == "Inline label\n"


def test_input_text(convert: Callable[..., str]) -> None:
    html = '<input type="text" name="username">'
    result = convert(html)
    assert result == ""


def test_input_password(convert: Callable[..., str]) -> None:
    html = '<input type="password" name="password">'
    result = convert(html)
    assert result == ""


def test_input_with_value(convert: Callable[..., str]) -> None:
    html = '<input type="text" name="username" value="john">'
    result = convert(html)
    assert result == ""


def test_input_with_placeholder(convert: Callable[..., str]) -> None:
    html = '<input type="text" name="username" placeholder="Enter username">'
    result = convert(html)
    assert result == ""


def test_input_required(convert: Callable[..., str]) -> None:
    html = '<input type="text" name="username" required>'
    result = convert(html)
    assert result == ""


def test_input_disabled(convert: Callable[..., str]) -> None:
    html = '<input type="text" name="username" disabled>'
    result = convert(html)
    assert result == ""


def test_input_readonly(convert: Callable[..., str]) -> None:
    html = '<input type="text" name="username" readonly>'
    result = convert(html)
    assert result == ""


def test_input_checkbox_unchecked(convert: Callable[..., str]) -> None:
    html = '<input type="checkbox" name="agree">'
    result = convert(html)
    assert result == ""


def test_input_checkbox_checked(convert: Callable[..., str]) -> None:
    html = '<input type="checkbox" name="agree" checked>'
    result = convert(html)
    assert result == ""


def test_input_radio(convert: Callable[..., str]) -> None:
    html = '<input type="radio" name="gender" value="male">'
    result = convert(html)
    assert result == ""


def test_input_submit(convert: Callable[..., str]) -> None:
    html = '<input type="submit" value="Submit">'
    result = convert(html)
    assert result == ""


def test_input_file(convert: Callable[..., str]) -> None:
    html = '<input type="file" name="upload" accept=".jpg,.png">'
    result = convert(html)
    assert result == ""


def test_input_inline_mode(convert: Callable[..., str]) -> None:
    html = '<input type="text" name="username">'
    result = convert(html, convert_as_inline=True)
    assert result == ""


def test_textarea_basic(convert: Callable[..., str]) -> None:
    html = "<textarea>Default text</textarea>"
    result = convert(html)
    assert result == "Default text\n"


def test_textarea_with_name(convert: Callable[..., str]) -> None:
    html = '<textarea name="comment">Comment text</textarea>'
    result = convert(html)
    assert result == "Comment text\n"


def test_textarea_with_placeholder(convert: Callable[..., str]) -> None:
    html = '<textarea placeholder="Enter your comment">Default text</textarea>'
    result = convert(html)
    assert result == "Default text\n"


def test_textarea_with_rows_cols(convert: Callable[..., str]) -> None:
    html = '<textarea rows="5" cols="30">Text</textarea>'
    result = convert(html)
    assert result == "Text\n"


def test_textarea_required(convert: Callable[..., str]) -> None:
    html = "<textarea required>Required text</textarea>"
    result = convert(html)
    assert result == "Required text\n"


def test_textarea_empty(convert: Callable[..., str]) -> None:
    html = "<textarea></textarea>"
    result = convert(html)
    assert result == ""


def test_textarea_inline_mode(convert: Callable[..., str]) -> None:
    html = "<textarea>Inline text</textarea>"
    result = convert(html, convert_as_inline=True)
    assert result == "Inline text\n"


def test_select_basic(convert: Callable[..., str]) -> None:
    html = "<select><option>Option 1</option><option>Option 2</option></select>"
    result = convert(html)
    assert result == "Option 1\nOption 2\n"


def test_select_with_name(convert: Callable[..., str]) -> None:
    html = '<select name="country"><option>USA</option><option>Canada</option></select>'
    result = convert(html)
    assert result == "USA\nCanada\n"


def test_select_multiple(convert: Callable[..., str]) -> None:
    html = "<select multiple><option>Option 1</option><option>Option 2</option></select>"
    result = convert(html)
    assert result == "Option 1\nOption 2\n"


def test_option_with_value(convert: Callable[..., str]) -> None:
    html = '<select><option value="us">United States</option><option value="ca">Canada</option></select>'
    result = convert(html)
    assert result == "United States\nCanada\n"


def test_option_selected(convert: Callable[..., str]) -> None:
    html = "<select><option>Option 1</option><option selected>Option 2</option></select>"
    result = convert(html)
    assert result == "Option 1\n* Option 2\n"


def test_optgroup(convert: Callable[..., str]) -> None:
    html = '<select><optgroup label="Group 1"><option>Option 1</option><option>Option 2</option></optgroup></select>'
    result = convert(html)
    assert result == "**Group 1**\nOption 1\nOption 2\n"


def test_select_empty(convert: Callable[..., str]) -> None:
    html = "<select></select>"
    result = convert(html)
    assert result == ""


def test_option_empty(convert: Callable[..., str]) -> None:
    html = "<select><option></option></select>"
    result = convert(html)
    assert result == ""


def test_select_inline_mode(convert: Callable[..., str]) -> None:
    html = "<select><option>Option</option></select>"
    result = convert(html, convert_as_inline=True)
    assert result == "Option\n"


def test_button_basic(convert: Callable[..., str]) -> None:
    html = "<button>Click me</button>"
    result = convert(html)
    assert result == "Click me\n"


def test_button_with_type(convert: Callable[..., str]) -> None:
    html = '<button type="submit">Submit</button>'
    result = convert(html)
    assert result == "Submit\n"


def test_button_disabled(convert: Callable[..., str]) -> None:
    html = "<button disabled>Disabled</button>"
    result = convert(html)
    assert result == "Disabled\n"


def test_button_with_name_value(convert: Callable[..., str]) -> None:
    html = '<button name="action" value="delete">Delete</button>'
    result = convert(html)
    assert result == "Delete\n"


def test_button_empty(convert: Callable[..., str]) -> None:
    html = "<button></button>"
    result = convert(html)
    assert result == ""


def test_button_inline_mode(convert: Callable[..., str]) -> None:
    html = "<button>Inline button</button>"
    result = convert(html, convert_as_inline=True)
    assert result == "Inline button\n"


def test_progress_basic(convert: Callable[..., str]) -> None:
    html = "<progress>50%</progress>"
    result = convert(html)
    assert result == "50%\n"


def test_progress_with_value_max(convert: Callable[..., str]) -> None:
    html = '<progress value="50" max="100">50%</progress>'
    result = convert(html)
    assert result == "50%\n"


def test_meter_basic(convert: Callable[..., str]) -> None:
    html = "<meter>6 out of 10</meter>"
    result = convert(html)
    assert result == "6 out of 10\n"


def test_meter_with_attributes(convert: Callable[..., str]) -> None:
    html = '<meter value="6" min="0" max="10" low="2" high="8" optimum="5">6 out of 10</meter>'
    result = convert(html)
    assert result == "6 out of 10\n"


def test_progress_empty(convert: Callable[..., str]) -> None:
    html = "<progress></progress>"
    result = convert(html)
    assert result == ""


def test_meter_empty(convert: Callable[..., str]) -> None:
    html = "<meter></meter>"
    result = convert(html)
    assert result == ""


def test_progress_inline_mode(convert: Callable[..., str]) -> None:
    html = "<progress>50%</progress>"
    result = convert(html, convert_as_inline=True)
    assert result == "50%\n"


def test_meter_inline_mode(convert: Callable[..., str]) -> None:
    html = "<meter>6/10</meter>"
    result = convert(html, convert_as_inline=True)
    assert result == "6/10\n"


def test_output_basic(convert: Callable[..., str]) -> None:
    html = "<output>Result: 42</output>"
    result = convert(html)
    assert result == "Result: 42\n"


def test_output_with_for(convert: Callable[..., str]) -> None:
    html = '<output for="input1 input2">Sum: 15</output>'
    result = convert(html)
    assert result == "Sum: 15\n"


def test_output_with_name(convert: Callable[..., str]) -> None:
    html = '<output name="result">42</output>'
    result = convert(html)
    assert result == "42\n"


def test_datalist_basic(convert: Callable[..., str]) -> None:
    html = "<datalist><option>Option 1</option><option>Option 2</option></datalist>"
    result = convert(html)
    assert result == "Option 1\nOption 2\n"


def test_datalist_with_id(convert: Callable[..., str]) -> None:
    html = '<datalist id="browsers"><option>Chrome</option><option>Firefox</option></datalist>'
    result = convert(html)
    assert result == "Chrome\nFirefox\n"


def test_output_empty(convert: Callable[..., str]) -> None:
    html = "<output></output>"
    result = convert(html)
    assert result == ""


def test_datalist_empty(convert: Callable[..., str]) -> None:
    html = "<datalist></datalist>"
    result = convert(html)
    assert result == ""


def test_output_inline_mode(convert: Callable[..., str]) -> None:
    html = "<output>Result</output>"
    result = convert(html, convert_as_inline=True)
    assert result == "Result\n"


def test_datalist_inline_mode(convert: Callable[..., str]) -> None:
    html = "<datalist><option>Option</option></datalist>"
    result = convert(html, convert_as_inline=True)
    assert result == "Option\n"


def test_complete_form_example(convert: Callable[..., str]) -> None:
    html = """<form action="/submit" method="post">
        <fieldset>
            <legend>Personal Information</legend>
            <label for="name">Name:</label>
            <input type="text" id="name" name="name" required>
            <label for="email">Email:</label>
            <input type="email" id="email" name="email" required>
        </fieldset>
        <fieldset>
            <legend>Preferences</legend>
            <label>
                <input type="checkbox" name="newsletter" checked>
                Subscribe to newsletter
            </label>
            <label for="country">Country:</label>
            <select id="country" name="country">
                <option value="us">United States</option>
                <option value="ca">Canada</option>
            </select>
        </fieldset>
        <button type="submit">Submit</button>
    </form>"""
    result = convert(html)
    expected = """**Personal Information**

Name:

Email:

**Preferences**

Subscribe to newsletter

Country:

United States
Canada

Submit
"""
    assert result == expected


def test_form_with_progress_and_meter(convert: Callable[..., str]) -> None:
    html = """<form>
        <label>Upload Progress:</label>
        <progress value="75" max="100">75%</progress>
        <label>Rating:</label>
        <meter value="4" min="1" max="5">4 out of 5</meter>
        <output for="rating">Current rating: 4/5</output>
    </form>"""
    result = convert(html)
    expected = """Upload Progress:

75%

Rating:

4 out of 5

Current rating: 4/5
"""
    assert result == expected


def test_form_with_inputs_inline_mode(convert: Callable[..., str]) -> None:
    html = '<form><label>Name:</label> <input type="text" name="name"> <button>Submit</button></form>'
    result = convert(html, convert_as_inline=True)
    assert result == "Name:  Submit\n"


def test_article_element(convert: Callable[..., str]) -> None:
    html = "<article>This is an article</article>"
    result = convert(html)
    assert result == "This is an article\n"


def test_section_element(convert: Callable[..., str]) -> None:
    html = "<section>This is a section</section>"
    result = convert(html)
    assert result == "This is a section\n"


def test_nav_element(convert: Callable[..., str]) -> None:
    html = "<nav>This is navigation</nav>"
    result = convert(html)
    assert result == "This is navigation\n"


def test_aside_element(convert: Callable[..., str]) -> None:
    html = "<aside>This is an aside</aside>"
    result = convert(html)
    assert result == "This is an aside\n"


def test_header_element(convert: Callable[..., str]) -> None:
    html = "<header>This is a header</header>"
    result = convert(html)
    assert result == "This is a header\n"


def test_footer_element(convert: Callable[..., str]) -> None:
    html = "<footer>This is a footer</footer>"
    result = convert(html)
    assert result == "This is a footer\n"


def test_main_element(convert: Callable[..., str]) -> None:
    html = "<main>This is main content</main>"
    result = convert(html)
    assert result == "This is main content\n"


def test_article_with_sections(convert: Callable[..., str]) -> None:
    html = "<article><header>Article Header</header><section><h2>Section Title</h2><p>Section content</p></section><footer>Article Footer</footer></article>"
    result = convert(html, heading_style="atx")
    expected = "Article Header\n\n## Section Title\n\nSection content\n\nArticle Footer\n"
    assert result == expected


def test_semantic_elements_with_other_content(convert: Callable[..., str]) -> None:
    html = '<nav><ul><li><a href="#home">Home</a></li><li><a href="#about">About</a></li></ul></nav><main><article><h1>Article Title</h1><p>Article content</p></article></main>'
    result = convert(html, heading_style="atx")
    expected = "- [Home](#home)\n- [About](#about)\n\n# Article Title\n\nArticle content\n"
    assert result == expected


def test_empty_article_element(convert: Callable[..., str]) -> None:
    html = "<article></article>"
    result = convert(html)
    assert result == ""


def test_article_inline_mode(convert: Callable[..., str]) -> None:
    html = "<article>This is inline content</article>"
    result = convert(html, convert_as_inline=True)
    assert result == "This is inline content\n"


def test_semantic_elements_with_whitespace(convert: Callable[..., str]) -> None:
    html = "<section>  \n  Content with whitespace  \n  </section>"
    result = convert(html)
    assert result == " Content with whitespace \n"


def test_details_element(convert: Callable[..., str]) -> None:
    html = "<details>This is details content</details>"
    result = convert(html)
    assert result == "This is details content\n"


def test_summary_element(convert: Callable[..., str]) -> None:
    html = "<summary>Summary text</summary>"
    result = convert(html)
    assert result == "**Summary text**\n"


def test_details_with_summary(convert: Callable[..., str]) -> None:
    html = "<details><summary>Click to expand</summary><p>Hidden content here</p></details>"
    result = convert(html)
    expected = "**Click to expand**\n\nHidden content here\n"
    assert result == expected


def test_nested_details(convert: Callable[..., str]) -> None:
    html = "<details><summary>Level 1</summary><details><summary>Level 2</summary><p>Nested content</p></details></details>"
    result = convert(html)
    expected = "**Level 1**\n\n**Level 2**\n\nNested content\n"
    assert result == expected


def test_details_with_complex_content(convert: Callable[..., str]) -> None:
    html = '<details><summary>Code Example</summary><pre><code>def hello():\n    print("Hello, World!")</code></pre><p>This is a Python function.</p></details>'
    result = convert(html)
    expected = '**Code Example**\n\n```\ndef hello():\n    print("Hello, World!")\n```\nThis is a Python function.\n'
    assert result == expected


def test_empty_details(convert: Callable[..., str]) -> None:
    html = "<details></details>"
    result = convert(html)
    assert result == ""


def test_empty_summary(convert: Callable[..., str]) -> None:
    html = "<summary></summary>"
    result = convert(html)
    assert result == ""


def test_details_inline_mode(convert: Callable[..., str]) -> None:
    html = "<details>Inline details</details>"
    result = convert(html, convert_as_inline=True)
    assert result == "Inline details\n"


def test_summary_inline_mode(convert: Callable[..., str]) -> None:
    html = "<summary>Inline summary</summary>"
    result = convert(html, convert_as_inline=True)
    assert result == "Inline summary\n"


def test_details_with_attributes(convert: Callable[..., str]) -> None:
    html = "<details open><summary>Always open</summary><p>Content</p></details>"
    result = convert(html)
    expected = "**Always open**\n\nContent\n"
    assert result == expected


def test_audio_basic(convert: Callable[..., str]) -> None:
    html = '<audio src="audio.mp3"></audio>'
    result = convert(html)
    assert result == "[audio.mp3](audio.mp3)\n"


def test_audio_with_controls(convert: Callable[..., str]) -> None:
    html = '<audio src="audio.mp3" controls></audio>'
    result = convert(html)
    assert result == "[audio.mp3](audio.mp3)\n"


def test_audio_with_all_attributes(convert: Callable[..., str]) -> None:
    html = '<audio src="audio.mp3" controls autoplay loop muted preload="auto"></audio>'
    result = convert(html)
    assert result == "[audio.mp3](audio.mp3)\n"


def test_audio_with_source_element(convert: Callable[..., str]) -> None:
    html = """<audio controls>
    <source src="audio.mp3" type="audio/mpeg">
    <source src="audio.ogg" type="audio/ogg">
</audio>"""
    result = convert(html)
    assert result == "[audio.mp3](audio.mp3)\n"


def test_audio_with_fallback_content(convert: Callable[..., str]) -> None:
    html = '<audio src="audio.mp3" controls>Your browser does not support the audio element.</audio>'
    result = convert(html)
    expected = "[audio.mp3](audio.mp3)\n\nYour browser does not support the audio element.\n"
    assert result == expected


def test_audio_without_src(convert: Callable[..., str]) -> None:
    html = "<audio controls></audio>"
    result = convert(html)
    assert result == ""


def test_video_basic(convert: Callable[..., str]) -> None:
    html = '<video src="video.mp4"></video>'
    result = convert(html)
    assert result == "[video.mp4](video.mp4)\n"


def test_video_with_dimensions(convert: Callable[..., str]) -> None:
    html = '<video src="video.mp4" width="640" height="480"></video>'
    result = convert(html)
    assert result == "[video.mp4](video.mp4)\n"


def test_video_with_all_attributes(convert: Callable[..., str]) -> None:
    html = '<video src="video.mp4" width="640" height="480" poster="poster.jpg" controls autoplay loop muted preload="metadata"></video>'
    result = convert(html)
    assert result == "[video.mp4](video.mp4)\n"


def test_video_with_source_element(convert: Callable[..., str]) -> None:
    html = """<video controls width="640">
    <source src="video.mp4" type="video/mp4">
    <source src="video.webm" type="video/webm">
</video>"""
    result = convert(html)
    assert result == "[video.mp4](video.mp4)\n"


def test_video_with_fallback_content(convert: Callable[..., str]) -> None:
    html = '<video src="video.mp4" controls>Your browser does not support the video element.</video>'
    result = convert(html)
    expected = "[video.mp4](video.mp4)\n\nYour browser does not support the video element.\n"
    assert result == expected


def test_video_with_track_elements(convert: Callable[..., str]) -> None:
    html = """<video src="video.mp4" controls>
    <track src="subtitles_en.vtt" kind="subtitles" srclang="en" label="English">
    <track src="subtitles_es.vtt" kind="subtitles" srclang="es" label="Spanish">
</video>"""
    result = convert(html)
    assert result == "[video.mp4](video.mp4)\n"


def test_iframe_basic(convert: Callable[..., str]) -> None:
    html = '<iframe src="https://example.com"></iframe>'
    result = convert(html)
    assert result == "[https://example.com](https://example.com)\n"


def test_iframe_with_dimensions(convert: Callable[..., str]) -> None:
    html = '<iframe src="https://example.com" width="800" height="600"></iframe>'
    result = convert(html)
    assert result == "[https://example.com](https://example.com)\n"


def test_iframe_with_all_attributes(convert: Callable[..., str]) -> None:
    html = '<iframe src="https://example.com" width="800" height="600" title="Example Frame" allow="fullscreen" sandbox="allow-scripts" loading="lazy"></iframe>'
    result = convert(html)
    assert result == "[https://example.com](https://example.com)\n"


def test_iframe_youtube_embed(convert: Callable[..., str]) -> None:
    html = '<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" title="YouTube video player" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>'
    result = convert(html)
    assert result == "[https://www.youtube.com/embed/dQw4w9WgXcQ](https://www.youtube.com/embed/dQw4w9WgXcQ)\n"


def test_iframe_with_sandbox_boolean(convert: Callable[..., str]) -> None:
    html = '<iframe src="https://example.com" sandbox></iframe>'
    result = convert(html)
    assert result == "[https://example.com](https://example.com)\n"


def test_blockquote_with_single_newline_end(convert: Callable[..., str]) -> None:
    html = "<blockquote>Test content\n</blockquote>"
    result = convert(html)
    assert result == "> Test content\n"


def test_list_with_empty_lines_multiline_content(convert: Callable[..., str]) -> None:
    html = """<ul>
    <li><p>First paragraph</p>

    <p>Second paragraph</p></li>
</ul>"""
    result = convert(html)
    assert "First paragraph\n\n  Second paragraph" in result


def test_list_with_empty_lines(convert: Callable[..., str]) -> None:
    html = """<ul>
    <li>
        First item

        Second paragraph
    </li>
</ul>"""
    result = convert(html)
    expected = "- First item\n\n    Second paragraph\n"
    assert result == expected


def test_media_in_paragraphs(convert: Callable[..., str]) -> None:
    html = """<p>Here is an audio file: <audio src="audio.mp3" controls></audio></p>
<p>Here is a video: <video src="video.mp4" controls></video></p>
<p>Here is an iframe: <iframe src="https://example.com"></iframe></p>"""
    result = convert(html)
    expected = """Here is an audio file: [audio.mp3](audio.mp3)

Here is a video: [video.mp4](video.mp4)

Here is an iframe: [https://example.com](https://example.com)
"""
    assert result == expected


def test_nested_media_elements(convert: Callable[..., str]) -> None:
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
    result = convert(html)
    expected = """## Media Gallery

### Audio Section

[audio1.mp3](audio1.mp3)

Your browser doesn't support HTML5 audio.

### Video Section

[video1.mp4](video1.mp4)

Your browser doesn't support HTML5 video.
"""
    assert result == expected


def test_media_inline_mode(convert: Callable[..., str]) -> None:
    html = '<audio src="audio.mp3" controls></audio>'
    result = convert(html, convert_as_inline=True)
    assert result == "[audio.mp3](audio.mp3)\n"


def test_empty_media_attributes(convert: Callable[..., str]) -> None:
    html = '<video src="" width="" height=""></video>'
    result = convert(html)
    assert result == ""


def test_media_with_metadata(convert: Callable[..., str]) -> None:
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
    result = convert(html)
    expected = """---
meta-description: Page with media elements
title: Media Page
---

[audio.mp3](audio.mp3)

[video.mp4](video.mp4)

[https://example.com](https://example.com)
"""
    assert result == expected


def test_audio_no_boolean_attributes(convert: Callable[..., str]) -> None:
    html = '<audio src="audio.mp3" controls="false"></audio>'
    result = convert(html)
    assert result == "[audio.mp3](audio.mp3)\n"


def test_video_poster_only(convert: Callable[..., str]) -> None:
    html = '<video poster="poster.jpg"></video>'
    result = convert(html)
    assert result == ""


def test_ruby_basic(convert: Callable[..., str]) -> None:
    html = "<ruby>漢字<rt>kanji</rt></ruby>"
    result = convert(html)
    assert result == "漢字(kanji)\n"


def test_ruby_with_rb(convert: Callable[..., str]) -> None:
    html = "<ruby><rb>漢字</rb><rt>kanji</rt></ruby>"
    result = convert(html)
    assert result == "漢字(kanji)\n"


def test_ruby_with_fallback_rp(convert: Callable[..., str]) -> None:
    html = "<ruby>漢字<rp>(</rp><rt>kanji</rt><rp>)</rp></ruby>"
    result = convert(html)
    assert result == "漢字(kanji)\n"


def test_ruby_complex_structure(convert: Callable[..., str]) -> None:
    html = "<ruby><rb>東京</rb><rp>(</rp><rt>とうきょう</rt><rp>)</rp></ruby>"
    result = convert(html)
    assert result == "東京(とうきょう)\n"


def test_ruby_multiple_readings(convert: Callable[..., str]) -> None:
    html = "<ruby><rb>漢</rb><rt>kan</rt><rb>字</rb><rt>ji</rt></ruby>"
    result = convert(html)
    assert result == "漢(kan)字(ji)\n"


def test_ruby_inline_mode(convert: Callable[..., str]) -> None:
    html = "<ruby>漢字<rt>kanji</rt></ruby>"
    result = convert(html, convert_as_inline=True)
    assert result == "漢字(kanji)\n"


def test_ruby_block_mode(convert: Callable[..., str]) -> None:
    html = "<ruby>漢字<rt>kanji</rt></ruby>"
    result = convert(html, convert_as_inline=False)
    assert result == "漢字(kanji)\n"


def test_ruby_nested_in_paragraph(convert: Callable[..., str]) -> None:
    html = "<p>This is <ruby>漢字<rt>kanji</rt></ruby> text.</p>"
    result = convert(html)
    assert result == "This is 漢字(kanji) text.\n"


def test_ruby_with_whitespace(convert: Callable[..., str]) -> None:
    html = "<ruby> 漢字 <rt> kanji </rt> </ruby>"
    result = convert(html)
    assert result == "漢字(kanji)\n"


def test_ruby_empty_elements(convert: Callable[..., str]) -> None:
    html = "<ruby><rb></rb><rt></rt></ruby>"
    result = convert(html)
    assert result == "()\n"


def test_ruby_only_base_text(convert: Callable[..., str]) -> None:
    html = "<ruby>漢字</ruby>"
    result = convert(html)
    assert result == "漢字\n"


def test_ruby_only_annotation(convert: Callable[..., str]) -> None:
    html = "<ruby><rt>kanji</rt></ruby>"
    result = convert(html)
    assert result == "(kanji)\n"


def test_ruby_with_formatting(convert: Callable[..., str]) -> None:
    html = "<ruby><strong>漢字</strong><rt><em>kanji</em></rt></ruby>"
    result = convert(html)
    assert result == "**漢字**(*kanji*)\n"


def test_ruby_multiple_in_sentence(convert: Callable[..., str]) -> None:
    html = "I love <ruby>寿司<rt>sushi</rt></ruby> and <ruby>刺身<rt>sashimi</rt></ruby>!"
    result = convert(html)
    assert result == "I love 寿司(sushi) and 刺身(sashimi)!\n"


def test_ruby_with_mixed_content(convert: Callable[..., str]) -> None:
    html = "<ruby>東<rb>京</rb>都<rt>とう<strong>きょう</strong>と</rt></ruby>"
    result = convert(html)
    assert result == "東京都(とう**きょう**と)\n"


def test_rb_standalone(convert: Callable[..., str]) -> None:
    html = "<rb>漢字</rb>"
    result = convert(html)
    assert result == "漢字\n"


def test_rb_inline_mode(convert: Callable[..., str]) -> None:
    html = "<rb>漢字</rb>"
    result = convert(html, convert_as_inline=True)
    assert result == "漢字\n"


def test_rb_block_mode(convert: Callable[..., str]) -> None:
    html = "<rb>漢字</rb>"
    result = convert(html, convert_as_inline=False)
    assert result == "漢字\n"


def test_rt_standalone(convert: Callable[..., str]) -> None:
    html = "<rt>kanji</rt>"
    result = convert(html)
    assert result == "(kanji)\n"


def test_rt_with_surrounding_rp(convert: Callable[..., str]) -> None:
    html = "<rp>(</rp><rt>kanji</rt><rp>)</rp>"
    result = convert(html)
    assert result == "(kanji)\n"


def test_rt_inline_mode(convert: Callable[..., str]) -> None:
    html = "<rt>kanji</rt>"
    result = convert(html, convert_as_inline=True)
    assert result == "(kanji)\n"


def test_rt_block_mode(convert: Callable[..., str]) -> None:
    html = "<rt>kanji</rt>"
    result = convert(html, convert_as_inline=False)
    assert result == "(kanji)\n"


def test_rp_standalone(convert: Callable[..., str]) -> None:
    html = "<rp>(</rp>"
    result = convert(html)
    assert result == "(\n"


def test_rp_inline_mode(convert: Callable[..., str]) -> None:
    html = "<rp>)</rp>"
    result = convert(html, convert_as_inline=True)
    assert result == ")\n"


def test_rp_block_mode(convert: Callable[..., str]) -> None:
    html = "<rp>(</rp>"
    result = convert(html, convert_as_inline=False)
    assert result == "(\n"


def test_rtc_standalone(convert: Callable[..., str]) -> None:
    html = "<rtc>annotation</rtc>"
    result = convert(html)
    assert result == "annotation\n"


def test_rtc_inline_mode(convert: Callable[..., str]) -> None:
    html = "<rtc>annotation</rtc>"
    result = convert(html, convert_as_inline=True)
    assert result == "annotation\n"


def test_rtc_block_mode(convert: Callable[..., str]) -> None:
    html = "<rtc>annotation</rtc>"
    result = convert(html, convert_as_inline=False)
    assert result == "annotation\n"


def test_nested_ruby_elements(convert: Callable[..., str]) -> None:
    html = "<ruby><ruby>漢<rt>kan</rt></ruby><rt>字</rt></ruby>"
    result = convert(html)
    assert result == "漢(kan)(字)\n"


def test_ruby_with_line_breaks(convert: Callable[..., str]) -> None:
    html = "<ruby>\n漢字\n<rt>\nkanji\n</rt>\n</ruby>"
    result = convert(html)
    assert result == "漢字(kanji)\n"


def test_ruby_with_special_characters(convert: Callable[..., str]) -> None:
    html = "<ruby>*test*<rt>_annotation_</rt></ruby>"
    result = convert(html)
    assert result == "*test*(_annotation_)\n"


def test_ruby_with_links(convert: Callable[..., str]) -> None:
    html = '<ruby><a href="https://example.com">漢字</a><rt>kanji</rt></ruby>'
    result = convert(html)
    assert result == "[漢字](https://example.com)(kanji)\n"


def test_ruby_in_table(convert: Callable[..., str]) -> None:
    html = "<table><tr><td><ruby>漢字<rt>kanji</rt></ruby></td></tr></table>"
    result = convert(html)
    assert "漢字(kanji)" in result


def test_ruby_in_list(convert: Callable[..., str]) -> None:
    html = "<ul><li><ruby>漢字<rt>kanji</rt></ruby></li></ul>"
    result = convert(html)
    assert "- 漢字(kanji)" in result


def test_multiple_rt_elements(convert: Callable[..., str]) -> None:
    html = "<ruby>漢字<rt>kan</rt><rt>ji</rt></ruby>"
    result = convert(html)
    assert result == "漢字(kan)(ji)\n"


def test_ruby_with_rtc_and_rt(convert: Callable[..., str]) -> None:
    html = "<ruby>漢字<rt>kanji</rt><rtc>Chinese characters</rtc></ruby>"
    result = convert(html)
    assert result == "漢字(kanji)Chinese characters\n"


def test_complex_ruby_structure(convert: Callable[..., str]) -> None:
    html = """<ruby>
        <rb>漢</rb>
        <rb>字</rb>
        <rp>(</rp>
        <rt>kan</rt>
        <rt>ji</rt>
        <rp>)</rp>
        <rtc>Chinese characters</rtc>
    </ruby>"""
    result = convert(html)
    assert result == "漢字((kan)(ji))Chinese characters\n"


def test_ruby_with_empty_rt(convert: Callable[..., str]) -> None:
    html = "<ruby>漢字<rt></rt></ruby>"
    result = convert(html)
    assert result == "漢字()\n"


def test_ruby_with_only_spaces(convert: Callable[..., str]) -> None:
    html = "<ruby>   <rt>   </rt>   </ruby>"
    result = convert(html)
    assert result == "()\n"


def test_abbr_basic(convert: Callable[..., str]) -> None:
    html = "<abbr>HTML</abbr>"
    result = convert(html)
    assert result == "HTML\n"


def test_abbr_with_title(convert: Callable[..., str]) -> None:
    html = '<abbr title="HyperText Markup Language">HTML</abbr>'
    result = convert(html)
    assert result == "HTML (HyperText Markup Language)\n"


def test_abbr_with_empty_title(convert: Callable[..., str]) -> None:
    html = '<abbr title="">HTML</abbr>'
    result = convert(html)
    assert result == "HTML\n"


def test_abbr_inline_mode(convert: Callable[..., str]) -> None:
    html = '<abbr title="HyperText Markup Language">HTML</abbr>'
    result = convert(html, convert_as_inline=True)
    assert result == "HTML (HyperText Markup Language)\n"


def test_abbr_nested_content(convert: Callable[..., str]) -> None:
    html = '<p>Learn <abbr title="HyperText Markup Language">HTML</abbr> today!</p>'
    result = convert(html)
    assert result == "Learn HTML (HyperText Markup Language) today!\n"


def test_time_basic(convert: Callable[..., str]) -> None:
    html = "<time>2023-12-25</time>"
    result = convert(html)
    assert result == "2023-12-25\n"


def test_time_with_datetime(convert: Callable[..., str]) -> None:
    html = '<time datetime="2023-12-25T10:30:00">Christmas Day</time>'
    result = convert(html)
    assert result == "Christmas Day\n"


def test_time_inline_mode(convert: Callable[..., str]) -> None:
    html = '<time datetime="2023-12-25">Christmas</time>'
    result = convert(html, convert_as_inline=True)
    assert result == "Christmas\n"


def test_time_in_paragraph(convert: Callable[..., str]) -> None:
    html = '<p>The event is on <time datetime="2023-12-25">Christmas Day</time>.</p>'
    result = convert(html)
    assert result == "The event is on Christmas Day.\n"


def test_data_basic(convert: Callable[..., str]) -> None:
    html = "<data>Product Name</data>"
    result = convert(html)
    assert result == "Product Name\n"


def test_data_with_value(convert: Callable[..., str]) -> None:
    html = '<data value="12345">Product Name</data>'
    result = convert(html)
    assert result == "Product Name\n"


def test_data_with_empty_value(convert: Callable[..., str]) -> None:
    html = '<data value="">Product</data>'
    result = convert(html)
    assert result == "Product\n"


def test_data_inline_mode(convert: Callable[..., str]) -> None:
    html = '<data value="12345">Product</data>'
    result = convert(html, convert_as_inline=True)
    assert result == "Product\n"


def test_data_in_list(convert: Callable[..., str]) -> None:
    html = '<ul><li><data value="A001">Product A</data></li><li><data value="B002">Product B</data></li></ul>'
    result = convert(html)
    assert result == "- Product A\n- Product B\n"


def test_ins_basic(convert: Callable[..., str]) -> None:
    html = "<ins>This text was added</ins>"
    result = convert(html)
    assert result == "==This text was added==\n"


def test_ins_with_cite(convert: Callable[..., str]) -> None:
    html = '<ins cite="https://example.com">Added text</ins>'
    result = convert(html)
    assert result == "==Added text==\n"


def test_ins_with_datetime(convert: Callable[..., str]) -> None:
    html = '<ins datetime="2023-12-25">Added on Christmas</ins>'
    result = convert(html)
    assert result == "==Added on Christmas==\n"


def test_ins_inline_mode(convert: Callable[..., str]) -> None:
    html = "<ins>Added text</ins>"
    result = convert(html, convert_as_inline=True)
    assert result == "==Added text==\n"


def test_ins_in_paragraph(convert: Callable[..., str]) -> None:
    html = "<p>Original text <ins>with addition</ins> and more.</p>"
    result = convert(html)
    assert result == "Original text ==with addition== and more.\n"


def test_var_basic(convert: Callable[..., str]) -> None:
    html = "<var>x</var>"
    result = convert(html)
    assert result == "*x*\n"


def test_var_in_code(convert: Callable[..., str]) -> None:
    html = "<p>Set <var>username</var> to your login name.</p>"
    result = convert(html)
    assert result == "Set *username* to your login name.\n"


def test_var_mathematical(convert: Callable[..., str]) -> None:
    html = "<p>If <var>x</var> = 5, then <var>y</var> = <var>x</var> + 3.</p>"
    result = convert(html)
    assert result == "If *x* = 5, then *y* = *x* + 3.\n"


def test_var_inline_mode(convert: Callable[..., str]) -> None:
    html = "<var>variable</var>"
    result = convert(html, convert_as_inline=True)
    assert result == "*variable*\n"


def test_dfn_basic(convert: Callable[..., str]) -> None:
    html = "<dfn>API</dfn>"
    result = convert(html)
    assert result == "*API*\n"


def test_dfn_with_title(convert: Callable[..., str]) -> None:
    html = '<dfn title="Application Programming Interface">API</dfn>'
    result = convert(html)
    assert result == "*API*\n"


def test_dfn_in_definition_list(convert: Callable[..., str]) -> None:
    html = "<dl><dt><dfn>API</dfn></dt><dd>Application Programming Interface</dd></dl>"
    result = convert(html)
    assert result == "*API*\n:   Application Programming Interface\n"


def test_dfn_inline_mode(convert: Callable[..., str]) -> None:
    html = "<dfn>term</dfn>"
    result = convert(html, convert_as_inline=True)
    assert result == "*term*\n"


def test_bdi_basic(convert: Callable[..., str]) -> None:
    html = "<bdi>عربي</bdi>"
    result = convert(html)
    assert result == "عربي\n"


def test_bdo_basic(convert: Callable[..., str]) -> None:
    html = '<bdo dir="rtl">English text</bdo>'
    result = convert(html)
    assert result == "English text\n"


def test_bdi_mixed_text(convert: Callable[..., str]) -> None:
    html = "<p>User <bdi>إيان</bdi> scored 90 points.</p>"
    result = convert(html)
    assert result == "User إيان scored 90 points.\n"


def test_bdo_with_direction(convert: Callable[..., str]) -> None:
    html = '<p>The title is <bdo dir="rtl">مرحبا</bdo> in Arabic.</p>'
    result = convert(html)
    assert result == "The title is مرحبا in Arabic.\n"


def test_bdi_inline_mode(convert: Callable[..., str]) -> None:
    html = "<bdi>نص عربي</bdi>"
    result = convert(html, convert_as_inline=True)
    assert result == "نص عربي\n"


def test_small_basic(convert: Callable[..., str]) -> None:
    html = "<small>Fine print</small>"
    result = convert(html)
    assert result == "Fine print\n"


def test_small_copyright(convert: Callable[..., str]) -> None:
    html = "<p>© 2023 Company Name. <small>All rights reserved.</small></p>"
    result = convert(html)
    assert result == "© 2023 Company Name. All rights reserved.\n"


def test_small_inline_mode(convert: Callable[..., str]) -> None:
    html = "<small>Legal disclaimer</small>"
    result = convert(html, convert_as_inline=True)
    assert result == "Legal disclaimer\n"


def test_u_basic(convert: Callable[..., str]) -> None:
    html = "<u>Underlined text</u>"
    result = convert(html)
    assert result == "Underlined text\n"


def test_u_misspelling(convert: Callable[..., str]) -> None:
    html = "<p>This word is <u>mispelled</u>.</p>"  # codespell:ignore mispelled
    result = convert(html)
    assert result == "This word is mispelled.\n"  # codespell:ignore mispelled


def test_u_inline_mode(convert: Callable[..., str]) -> None:
    html = "<u>underlined</u>"
    result = convert(html, convert_as_inline=True)
    assert result == "underlined\n"


def test_wbr_basic(convert: Callable[..., str]) -> None:
    html = "super<wbr>cali<wbr>fragilistic"
    result = convert(html)
    assert result == "supercalifragilistic\n"


def test_wbr_long_url(convert: Callable[..., str]) -> None:
    html = "<p>Visit https://www.<wbr>example.<wbr>com/very/<wbr>long/<wbr>path</p>"
    result = convert(html)
    assert result == "Visit https://www.example.com/very/long/path\n"


def test_wbr_inline_mode(convert: Callable[..., str]) -> None:
    html = "long<wbr>word"
    result = convert(html, convert_as_inline=True)
    assert result == "longword\n"


def test_mixed_semantic_elements(convert: Callable[..., str]) -> None:
    html = """<article>
        <h2>Programming Concepts</h2>
        <p>An <dfn>API</dfn> (<abbr title="Application Programming Interface">API</abbr>)
        allows different software components to communicate.</p>
        <p>When you set the variable <var>timeout</var> to <data value="5000">5 seconds</data>,
        <ins>added in version 2.0</ins>, the system will wait.</p>
        <p><small>Last updated: <time datetime="2023-12-25">December 25, 2023</time></small></p>
    </article>"""
    result = convert(html, heading_style="atx")
    expected = """## Programming Concepts

An *API* (API (Application Programming Interface))
 allows different software components to communicate.

When you set the variable *timeout* to 5 seconds, ==added in version 2.0==, the system will wait.

Last updated: December 25, 2023
"""
    assert result == expected


def test_complex_nested_semantic_elements(convert: Callable[..., str]) -> None:
    html = '<p>The <dfn><abbr title="Application Programming Interface">API</abbr></dfn> documentation has been <ins>updated with <var>new_parameter</var></ins>.</p>'
    result = convert(html)
    assert (
        result
        == "The *API (Application Programming Interface)* documentation has been ==updated with *new_parameter*==.\n"
    )


def test_mixed_semantic_elements_inline_mode(convert: Callable[..., str]) -> None:
    html = '<abbr title="HyperText Markup Language">HTML</abbr> and <var>css</var> with <ins>updates</ins>'
    result = convert(html, convert_as_inline=True)
    assert result == "HTML (HyperText Markup Language) and *css* with ==updates==\n"


def test_multiple_empty_semantic_elements(convert: Callable[..., str]) -> None:
    html = "<p>Empty elements: <abbr></abbr> <var></var> <ins></ins> <dfn></dfn></p>"
    result = convert(html)
    assert result == "Empty elements: \n"


def test_whitespace_handling_semantic(convert: Callable[..., str]) -> None:
    html = "<p>Spaces around <var>  variable  </var> and <abbr title='  title  '>  abbr  </abbr></p>"
    result = convert(html)
    assert result == "Spaces around  *variable* and abbr (title)\n"


def test_figure_basic(convert: Callable[..., str]) -> None:
    html = '<figure><img src="image.jpg" alt="Test image"></figure>'
    result = convert(html)
    assert result == "![Test image](image.jpg)\n"


def test_figure_with_caption(convert: Callable[..., str]) -> None:
    html = '<figure><img src="test.jpg"><figcaption>Image caption</figcaption></figure>'
    result = convert(html)
    expected = "![](test.jpg)\n\n*Image caption*\n"
    assert result == expected


def test_figure_with_id(convert: Callable[..., str]) -> None:
    html = '<figure id="fig1"><img src="chart.png"></figure>'
    result = convert(html)
    assert result == "![](chart.png)\n"


def test_figure_with_class(convert: Callable[..., str]) -> None:
    html = '<figure class="photo"><img src="photo.jpg"></figure>'
    result = convert(html)
    assert result == "![](photo.jpg)\n"


def test_figure_with_multiple_attributes(convert: Callable[..., str]) -> None:
    html = '<figure id="fig2" class="diagram"><img src="diagram.svg"></figure>'
    result = convert(html)
    assert result == "![](diagram.svg)\n"


def test_figure_empty(convert: Callable[..., str]) -> None:
    html = "<figure></figure>"
    result = convert(html)
    assert result == ""


def test_figure_inline_mode(convert: Callable[..., str]) -> None:
    html = '<figure><img src="inline.jpg" alt="Inline image"></figure>'
    result = convert(html, convert_as_inline=True)
    assert result == "Inline image\n"


def test_figure_with_complex_content(convert: Callable[..., str]) -> None:
    html = """<figure>
        <img src="main.jpg" alt="Main image">
        <figcaption>
            <strong>Figure 1:</strong> This is a complex caption with <em>emphasis</em>.
        </figcaption>
    </figure>"""
    result = convert(html)
    expected = "![Main image](main.jpg)\n\n***Figure 1:** This is a complex caption with *emphasis*.*\n"
    assert result == expected


def test_figure_with_multiple_images(convert: Callable[..., str]) -> None:
    html = """<figure>
        <img src="before.jpg" alt="Before">
        <img src="after.jpg" alt="After">
        <figcaption>Before and after comparison</figcaption>
    </figure>"""
    result = convert(html)
    expected = "![Before](before.jpg)![After](after.jpg)\n\n*Before and after comparison*\n"
    assert result == expected


def test_figure_with_nested_elements(convert: Callable[..., str]) -> None:
    html = """<figure id="stats">
        <table>
            <tr><th>Year</th><th>Sales</th></tr>
            <tr><td>2023</td><td>100</td></tr>
        </table>
        <figcaption>Annual sales data</figcaption>
    </figure>"""
    result = convert(html)
    expected = """| Year | Sales |\n| --- | --- |\n| 2023 | 100 |\n\n*Annual sales data*\n"""
    assert result == expected


def test_hgroup_basic(convert: Callable[..., str]) -> None:
    html = "<hgroup><h1>Main Title</h1><h2>Subtitle</h2></hgroup>"
    result = convert(html)
    expected = "# Main Title\n\n## Subtitle\n"
    assert result == expected


def test_hgroup_multiple_headings(convert: Callable[..., str]) -> None:
    html = "<hgroup><h1>Title</h1><h2>Subtitle</h2><h3>Section</h3></hgroup>"
    result = convert(html)
    expected = "# Title\n\n## Subtitle\n\n### Section\n"
    assert result == expected


def test_hgroup_empty(convert: Callable[..., str]) -> None:
    html = "<hgroup></hgroup>"
    result = convert(html)
    assert result == ""


def test_hgroup_inline_mode(convert: Callable[..., str]) -> None:
    html = "<hgroup><h1>Inline Title</h1></hgroup>"
    result = convert(html, convert_as_inline=True)
    assert result == "Inline Title\n"


def test_hgroup_with_atx_headings(convert: Callable[..., str]) -> None:
    html = "<hgroup><h1>Main</h1><h2>Sub</h2></hgroup>"
    result = convert(html, heading_style="atx")
    expected = "# Main\n\n## Sub\n"
    assert result == expected


def test_hgroup_excessive_spacing(convert: Callable[..., str]) -> None:
    html = "<hgroup><h1>Title</h1><p></p><p></p><h2>Subtitle</h2></hgroup>"
    result = convert(html)
    expected = "# Title\n\n## Subtitle\n"
    assert result == expected


def test_hgroup_with_formatted_headings(convert: Callable[..., str]) -> None:
    html = "<hgroup><h1>The <em>Amazing</em> Title</h1><h2>A <strong>Bold</strong> Subtitle</h2></hgroup>"
    result = convert(html)
    expected = "# The *Amazing* Title\n\n## A **Bold** Subtitle\n"
    assert result == expected


def test_picture_basic(convert: Callable[..., str]) -> None:
    html = '<picture><img src="image.jpg" alt="Test"></picture>'
    result = convert(html)
    assert result == "![Test](image.jpg)\n"


def test_picture_with_source(convert: Callable[..., str]) -> None:
    html = """<picture>
        <source srcset="large.jpg" media="(min-width: 800px)">
        <img src="small.jpg" alt="Responsive image">
    </picture>"""
    result = convert(html)
    assert result == "![Responsive image](small.jpg)\n"


def test_picture_multiple_sources(convert: Callable[..., str]) -> None:
    html = """<picture>
        <source srcset="image.webp" type="image/webp">
        <source srcset="image.jpg" type="image/jpeg">
        <img src="fallback.jpg" alt="Multi-format">
    </picture>"""
    result = convert(html)
    assert result == "![Multi-format](fallback.jpg)\n"


def test_picture_complex_srcset(convert: Callable[..., str]) -> None:
    html = """<picture>
        <source srcset="small.jpg 480w, medium.jpg 800w, large.jpg 1200w"
                media="(min-width: 600px)">
        <img src="default.jpg">
    </picture>"""
    result = convert(html)
    assert result == "![](default.jpg)\n"


def test_picture_no_img(convert: Callable[..., str]) -> None:
    html = '<picture><source srcset="test.jpg"></picture>'
    result = convert(html)
    assert result == ""


def test_picture_empty(convert: Callable[..., str]) -> None:
    html = "<picture></picture>"
    result = convert(html)
    assert result == ""


def test_picture_inline_mode(convert: Callable[..., str]) -> None:
    html = """<picture>
        <source srcset="large.jpg" media="(min-width: 800px)">
        <img src="small.jpg" alt="Test">
    </picture>"""
    result = convert(html, convert_as_inline=True)
    assert result == "Test\n"


def test_picture_with_sizes(convert: Callable[..., str]) -> None:
    html = """<picture>
        <source srcset="img-480.jpg 480w, img-800.jpg 800w"
                sizes="(max-width: 600px) 480px, 800px">
        <img src="default.jpg">
    </picture>"""
    result = convert(html)
    assert result == "![](default.jpg)\n"


def test_figure_in_article(convert: Callable[..., str]) -> None:
    html = """<article>
        <h1>Article Title</h1>
        <figure id="main-image">
            <img src="hero.jpg" alt="Hero image">
            <figcaption>The main article image</figcaption>
        </figure>
        <p>Article content...</p>
    </article>"""
    result = convert(html)
    expected = "# Article Title\n\n![Hero image](hero.jpg)\n\n*The main article image*\n\nArticle content...\n"
    assert result == expected


def test_hgroup_in_header(convert: Callable[..., str]) -> None:
    html = """<header>
        <hgroup>
            <h1>Site Title</h1>
            <h2>Site Tagline</h2>
        </hgroup>
        <nav>Navigation here</nav>
    </header>"""
    result = convert(html)
    expected = "# Site Title\n\n## Site Tagline\n\nNavigation here\n"
    assert result == expected


def test_picture_in_figure(convert: Callable[..., str]) -> None:
    html = """<figure>
        <picture>
            <source srcset="large.webp" type="image/webp">
            <img src="fallback.jpg" alt="Test image">
        </picture>
        <figcaption>A responsive image in a figure</figcaption>
    </figure>"""
    result = convert(html)
    expected = "![Test image](fallback.jpg)\n\n*A responsive image in a figure*\n"
    assert result == expected


def test_multiple_figures(convert: Callable[..., str]) -> None:
    html = """
    <figure id="fig1">
        <img src="image1.jpg">
        <figcaption>First figure</figcaption>
    </figure>
    <figure id="fig2">
        <img src="image2.jpg">
        <figcaption>Second figure</figcaption>
    </figure>
    """
    result = convert(html)
    expected = "![](image1.jpg)\n\n*First figure*\n\n![](image2.jpg)\n\n*Second figure*\n"
    assert result == expected


def test_nested_structural_elements(convert: Callable[..., str]) -> None:
    html = """<section>
        <hgroup>
            <h1>Section Title</h1>
            <h2>Section Subtitle</h2>
        </hgroup>
        <figure>
            <picture>
                <source srcset="chart.svg" type="image/svg+xml">
                <img src="chart.png" alt="Data chart">
            </picture>
            <figcaption>Quarterly results</figcaption>
        </figure>
    </section>"""
    result = convert(html)
    expected = "# Section Title\n\n## Section Subtitle\n\n![Data chart](chart.png)\n\n*Quarterly results*\n"
    assert result == expected


def test_figure_with_special_characters(convert: Callable[..., str]) -> None:
    html = '<figure><img src="test.jpg"><figcaption>Caption with *asterisks* and _underscores_</figcaption></figure>'
    result = convert(html)
    expected = "![](test.jpg)\n\n*Caption with *asterisks* and _underscores_*\n"
    assert result == expected


def test_hgroup_single_heading(convert: Callable[..., str]) -> None:
    html = "<hgroup><h1>Only Title</h1></hgroup>"
    result = convert(html)
    expected = "# Only Title\n"
    assert result == expected


def test_picture_malformed_source(convert: Callable[..., str]) -> None:
    html = """<picture>
        <source>
        <source srcset="">
        <img src="valid.jpg">
    </picture>"""
    result = convert(html)
    assert result == "![](valid.jpg)\n"


def test_figure_whitespace_handling(convert: Callable[..., str]) -> None:
    html = """<figure>

        <img src="test.jpg">

        <figcaption>
            Caption text
        </figcaption>

    </figure>"""
    result = convert(html)
    expected = "![](test.jpg)\n\n*Caption text*\n"
    assert result == expected


def test_empty_elements_with_attributes(convert: Callable[..., str]) -> None:
    html1 = '<figure id="empty-fig"></figure>'
    assert convert(html1) == ""

    html2 = '<hgroup class="empty"></hgroup>'
    assert convert(html2) == ""

    html3 = '<picture id="empty-pic"></picture>'
    assert convert(html3) == ""


def test_figure_with_pre_content(convert: Callable[..., str]) -> None:
    html = """<figure>
        <pre><code>function example() {
  return 42;
}</code></pre>
        <figcaption>Code example</figcaption>
    </figure>"""
    result = convert(html)
    expected = "```\nfunction example() {\n  return 42;\n}\n```\n\n*Code example*\n"
    assert result == expected


@pytest.mark.parametrize(
    "html,expected",
    [
        ("<cite>Author</cite>", "*Author*"),
        ("<q>Quote</q>", '"Quote"'),
        ('<abbr title="Title">Text</abbr>', "Text (Title)"),
        ("<var>variable</var>", "*variable*"),
        ("<ins>inserted</ins>", "==inserted=="),
        ("<dfn>definition</dfn>", "*definition*"),
        ("<small>small text</small>", "small text"),
        ("<u>underlined</u>", "underlined"),
        ("word<wbr>break", "wordbreak"),
        ("<ruby>漢字<rt>kanji</rt></ruby>", "漢字(kanji)"),
    ],
)
def test_element_patterns(html: str, expected: str, convert: Callable[..., str]) -> None:
    result = convert(html)
    assert expected in result


@pytest.mark.parametrize(
    "html,expected_empty",
    [
        ("<cite></cite>", True),
        ("<q></q>", True),
        ("<abbr></abbr>", True),
        ("<var></var>", True),
        ("<ins></ins>", True),
        ("<dfn></dfn>", True),
        ("<form></form>", True),
        ("<fieldset></fieldset>", True),
        ("<legend></legend>", True),
        ("<label></label>", True),
        ("<button></button>", True),
        ("<progress></progress>", True),
        ("<meter></meter>", True),
        ("<output></output>", True),
        ("<datalist></datalist>", True),
        ("<select></select>", True),
        ("<textarea></textarea>", True),
        ("<dl></dl>", True),
        ("<article></article>", True),
        ("<section></section>", True),
        ("<nav></nav>", True),
        ("<aside></aside>", True),
        ("<header></header>", True),
        ("<footer></footer>", True),
        ("<main></main>", True),
        ("<details></details>", True),
        ("<summary></summary>", True),
        ("<figure></figure>", True),
        ("<hgroup></hgroup>", True),
        ("<picture></picture>", True),
    ],
)
def test_empty_elements(html: str, expected_empty: bool, convert: Callable[..., str]) -> None:
    result = convert(html)
    assert (result == "") == expected_empty


def test_blockquote_with_cite_in_list(convert: Callable[..., str]) -> None:
    html = """<ul>
        <li>
            Item with blockquote:
            <blockquote cite="https://example.com">Quote with citation</blockquote>
        </li>
    </ul>"""

    result = convert(html)
    assert "Item with blockquote:" in result
    assert "Quote with citation" in result
    assert "— <https://example.com>" in result


def test_mark_with_unsupported_highlight_style(convert: Callable[..., str]) -> None:
    html = "<mark>highlighted text</mark>"
    result = convert(html, highlight_style="unsupported")
    assert result == "highlighted text\n"


def test_empty_pre_element(convert: Callable[..., str]) -> None:
    html = "<pre></pre>"
    result = convert(html)
    assert result == ""


def test_media_element_without_src_but_with_text(convert: Callable[..., str]) -> None:
    html = "<video>Your browser doesn't support video</video>"
    result = convert(html)
    assert "Your browser doesn't support video" in result

    html = "<audio>Audio not supported</audio>"
    result = convert(html)
    assert "Audio not supported" in result


def test_paragraph_directly_in_list(convert: Callable[..., str]) -> None:
    html = """<ul>
        <p>Line 1\n\nLine 2</p>
    </ul>"""
    result = convert(html)
    assert "Line 1" in result
    assert "Line 2" in result


def test_paragraph_in_list_with_blank_lines(convert: Callable[..., str]) -> None:
    html = """<ul>
        <li>
            <p>First line\n\nSecond line\n\n\nThird line</p>
        </li>
    </ul>"""
    result = convert(html)
    assert "First line" in result
    assert "Second line" in result
    assert "Third line" in result


def test_blockquote_directly_under_list(convert: Callable[..., str]) -> None:
    html = """<ul>
        <blockquote>Quote\n\nWith blank lines</blockquote>
    </ul>"""
    result = convert(html)
    assert "Quote" in result


def test_blockquote_deeply_nested_in_li_needs_traversal(convert: Callable[..., str]) -> None:
    html = """<ul>
        <li>
            <div>
                <span>
                    <blockquote>Nested quote</blockquote>
                </span>
            </div>
        </li>
    </ul>"""
    result = convert(html)
    assert "> Nested quote" in result


def test_checkbox_with_string_content(convert: Callable[..., str]) -> None:
    html = '<ul><li><input type="checkbox">checkbox text content</input> List item text</li></ul>'
    result = convert(html)
    assert "[ ]" in result
    assert "List item text" in result


def test_paragraph_in_deeply_nested_li(convert: Callable[..., str]) -> None:
    html = """<ul>
        <li>
            <div>
                <span>
                    <p>First paragraph</p>
                    <p>Second paragraph with\n\nempty lines</p>
                </span>
            </div>
        </li>
    </ul>"""
    result = convert(html)
    assert "First paragraph" in result
    assert "Second paragraph" in result


def test_paragraph_deeply_nested_needs_traversal(convert: Callable[..., str]) -> None:
    html = """<ul>
        <li>
            <div>
                <section>
                    <article>
                        <p>Deeply nested paragraph</p>
                    </article>
                </section>
            </div>
        </li>
    </ul>"""
    result = convert(html)
    assert "Deeply nested paragraph" in result


def test_blockquote_in_list_with_empty_lines(convert: Callable[..., str]) -> None:
    html = """<ul>
        <li>
            <blockquote>Line 1\n\nLine 2\n\n\nLine 3</blockquote>
        </li>
    </ul>"""
    result = convert(html)
    assert "Line 1" in result
    assert "Line 2" in result
    assert "Line 3" in result


def test_iframe_inline_mode(convert: Callable[..., str]) -> None:
    html = '<iframe src="https://example.com/embed"></iframe>'
    result = convert(html, convert_as_inline=True)
    assert result == "[https://example.com/embed](https://example.com/embed)\n"


def test_time_element_empty(convert: Callable[..., str]) -> None:
    html = "<time></time>"
    result = convert(html)
    assert result == ""


def test_data_element_empty(convert: Callable[..., str]) -> None:
    html = "<data></data>"
    result = convert(html)
    assert result == ""


def test_optgroup_inline_mode(convert: Callable[..., str]) -> None:
    html = '<optgroup label="Group"><option>Option 1</option></optgroup>'
    result = convert(html, convert_as_inline=True)
    assert "Option 1" in result


def test_optgroup_without_label(convert: Callable[..., str]) -> None:
    html = "<optgroup><option>Option 1</option><option>Option 2</option></optgroup>"
    result = convert(html)
    assert "Option 1" in result
    assert "Option 2" in result


def test_optgroup_empty(convert: Callable[..., str]) -> None:
    html = "<optgroup>  </optgroup>"
    result = convert(html)
    assert result.strip() == ""


def test_ruby_element_empty(convert: Callable[..., str]) -> None:
    html = "<ruby>  </ruby>"
    result = convert(html)
    assert result == ""


def test_rp_element_empty(convert: Callable[..., str]) -> None:
    html = "<rp>  </rp>"
    result = convert(html)
    assert result == ""


def test_rtc_element_empty(convert: Callable[..., str]) -> None:
    html = "<rtc>  </rtc>"
    result = convert(html)
    assert result.strip() == ""


def test_legend_inline_mode(convert: Callable[..., str]) -> None:
    html = "<legend>Form Legend</legend>"
    result = convert(html, convert_as_inline=True)
    assert result == "Form Legend\n"


def test_iframe_without_src(convert: Callable[..., str]) -> None:
    html = "<iframe></iframe>"
    result = convert(html)
    assert result == ""

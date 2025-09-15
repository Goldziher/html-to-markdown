"""Comprehensive tests for HTML element conversion to Markdown.

Covers citations, definitions, forms, semantic elements, media elements,
ruby elements, text elements, and structural elements.
"""

from __future__ import annotations

import pytest

from html_to_markdown import convert_to_markdown


def test_cite_element() -> None:
    html = "<cite>Author Name</cite>"
    result = convert_to_markdown(html)
    assert result == "*Author Name*"


def test_cite_with_whitespace() -> None:
    html = "<cite>  Author Name  </cite>"
    result = convert_to_markdown(html)
    assert result == "*Author Name*"


def test_cite_inline_mode() -> None:
    html = "<cite>Author Name</cite>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Author Name"


def test_empty_cite() -> None:
    html = "<cite></cite>"
    result = convert_to_markdown(html)
    assert result == ""


def test_cite_with_nested_elements() -> None:
    html = "<cite>Author <strong>Name</strong></cite>"
    result = convert_to_markdown(html)
    assert result == "*Author **Name***"


def test_cite_with_link() -> None:
    html = '<cite><a href="https://example.com">Author Name</a></cite>'
    result = convert_to_markdown(html)
    assert result == "*[Author Name](https://example.com)*"


def test_q_element() -> None:
    html = "<q>Short quotation</q>"
    result = convert_to_markdown(html)
    assert result == '"Short quotation"'


def test_q_with_whitespace() -> None:
    html = "<q>  Short quotation  </q>"
    result = convert_to_markdown(html)
    assert result == '"Short quotation"'


def test_q_inline_mode() -> None:
    html = "<q>Short quotation</q>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Short quotation"


def test_empty_q() -> None:
    html = "<q></q>"
    result = convert_to_markdown(html)
    assert result == ""


def test_q_with_existing_quotes() -> None:
    html = '<q>He said "Hello" to me</q>'
    result = convert_to_markdown(html)
    assert result == '"He said \\"Hello\\" to me"'


def test_q_with_nested_elements() -> None:
    html = "<q>A <em>short</em> quotation</q>"
    result = convert_to_markdown(html)
    assert result == '"A *short* quotation"'


def test_q_with_code() -> None:
    html = "<q>The function <code>print()</code> outputs text</q>"
    result = convert_to_markdown(html)
    assert result == '"The function `print()` outputs text"'


def test_nested_q_elements() -> None:
    html = "<q>Outer quote <q>inner quote</q> continues</q>"
    result = convert_to_markdown(html)
    assert result == '"Outer quote \\"inner quote\\" continues"'


@pytest.mark.parametrize(
    "html,expected",
    [
        ("<dl><dd>What is this?</dd></dl>", "What is this?\n\n"),
        ("<dl><dt>Term</dt><dd>Definition</dd></dl>", "**Term**\n\n:   Definition\n\n"),
        (
            "<dl><dt>Term</dt><dd>Definition 1</dd><dd>Definition 2</dd></dl>",
            "**Term**\n\n:   Definition 1\n\n:   Definition 2\n\n",
        ),
        (
            "<dl><dt>Term 1</dt><dd>Def 1</dd><dt>Term 2</dt><dd>Def 2</dd></dl>",
            "**Term 1**\n\n:   Def 1\n\n**Term 2**\n\n:   Def 2\n\n",
        ),
        ("<dl><dd>First definition</dd><dd>Second definition</dd></dl>", "First definition\n\nSecond definition\n\n"),
        ("<dl><dt>Term only</dt></dl>", "**Term only**\n\n"),
        ("<dl><dd><p>Complex definition with paragraph</p></dd></dl>", "Complex definition with paragraph\n\n"),
        (
            "Some text before<dl><dd>Definition</dd></dl>Some text after",
            "Some text before\n\nDefinition\n\nSome text after",
        ),
        ("<dl></dl>", ""),
        ("<dl><dt></dt><dd></dd></dl>", "****\n\n:\n\n"),
    ],
)
def test_definition_list_issues(html: str, expected: str) -> None:
    """Test definition list conversion - Issue #68."""
    result = convert_to_markdown(html)
    assert result == expected


def test_simple_blockquote() -> None:
    html = "<blockquote>Simple quote</blockquote>"
    result = convert_to_markdown(html)
    assert result == "\n> Simple quote\n\n"


def test_blockquote_with_cite() -> None:
    html = '<blockquote cite="https://example.com">Quote with source</blockquote>'
    result = convert_to_markdown(html)
    expected = "\n> Quote with source\n\n— <https://example.com>\n\n"
    assert result == expected


def test_blockquote_with_cite_and_content() -> None:
    html = '<blockquote cite="https://shakespeare.com"><p>To be or not to be, that is the question.</p><p>Whether \'tis nobler in the mind to suffer...</p></blockquote>'
    result = convert_to_markdown(html)
    expected = "\n> To be or not to be, that is the question.\n> \n> Whether 'tis nobler in the mind to suffer...\n\n— <https://shakespeare.com>\n\n"
    assert result == expected


def test_nested_blockquotes() -> None:
    html = '<blockquote cite="https://outer.com">Outer quote<blockquote cite="https://inner.com">Inner quote</blockquote>Back to outer</blockquote>'
    result = convert_to_markdown(html)
    expected = "\n> Outer quote\n> \n> \n> > Inner quote\n> \n> — <https://inner.com>\n> \n> Back to outer\n\n— <https://outer.com>\n\n"
    assert result == expected


def test_blockquote_inline_mode() -> None:
    html = '<blockquote cite="https://example.com">Inline quote</blockquote>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline quote"


def test_empty_blockquote_with_cite() -> None:
    html = '<blockquote cite="https://example.com"></blockquote>'
    result = convert_to_markdown(html)
    assert result == ""


def test_cite_in_blockquote() -> None:
    html = "<blockquote>Quote by <cite>Author Name</cite></blockquote>"
    result = convert_to_markdown(html)
    assert result == "\n> Quote by *Author Name*\n\n"


def test_q_in_blockquote() -> None:
    html = "<blockquote>He said <q>Hello world</q> to everyone.</blockquote>"
    result = convert_to_markdown(html)
    assert result == '\n> He said "Hello world" to everyone.\n\n'


def test_blockquote_in_cite() -> None:
    html = "<cite>Author: <blockquote>Their famous quote</blockquote></cite>"
    result = convert_to_markdown(html)
    assert result == "*Author: \n\n> Their famous quote*"


def test_complex_citation_structure() -> None:
    html = '<article><p>According to <cite><a href="https://example.com">John Doe</a></cite>, the statement <q>Innovation drives progress</q> is fundamental.</p><blockquote cite="https://johndoe.com/quotes"><p>Innovation is not just about technology, it\'s about <em>thinking differently</em>.</p><cite>John Doe, 2023</cite></blockquote></article>'
    result = convert_to_markdown(html)
    expected = 'According to *[John Doe](https://example.com)*, the statement "Innovation drives progress" is fundamental.\n\n> Innovation is not just about technology, it\'s about *thinking differently*.\n> \n> *John Doe, 2023*\n\n— <https://johndoe.com/quotes>\n\n'
    assert result == expected


def test_quote_escaping_edge_cases() -> None:
    html = '<div><q>Quote with "nested quotes" and \'single quotes\'</q><q>Quote with backslash: \\</q><q>Quote with both \\" and regular quotes</q></div>'
    result = convert_to_markdown(html)
    expected = '"Quote with \\"nested quotes\\" and \'single quotes\'""Quote with backslash: \\\\""Quote with both \\\\\\" and regular quotes"\n\n'
    assert result == expected


def test_attributes_preservation() -> None:
    html = '<blockquote cite="https://example.com" class="important" id="quote1" data-author="John">Important quote</blockquote>'
    result = convert_to_markdown(html)
    expected = "\n> Important quote\n\n— <https://example.com>\n\n"
    assert result == expected


def test_simple_definition_list() -> None:
    html = "<dl><dt>Term</dt><dd>Definition</dd></dl>"
    result = convert_to_markdown(html)
    expected = "Term\n:   Definition\n\n"
    assert result == expected


def test_multiple_terms_and_definitions() -> None:
    html = "<dl><dt>First Term</dt><dd>First Definition</dd><dt>Second Term</dt><dd>Second Definition</dd></dl>"
    result = convert_to_markdown(html)
    expected = "First Term\n:   First Definition\n\nSecond Term\n:   Second Definition\n\n"
    assert result == expected


def test_term_with_multiple_definitions() -> None:
    html = "<dl><dt>Term</dt><dd>First definition</dd><dd>Second definition</dd></dl>"
    result = convert_to_markdown(html)
    expected = "Term\n:   First definition\n\n:   Second definition\n\n"
    assert result == expected


def test_multiple_terms_single_definition() -> None:
    html = "<dl><dt>Term 1</dt><dt>Term 2</dt><dd>Shared definition</dd></dl>"
    result = convert_to_markdown(html)
    expected = "Term 1\nTerm 2\n:   Shared definition\n\n"
    assert result == expected


def test_definition_with_inline_formatting() -> None:
    html = "<dl><dt><strong>Bold Term</strong></dt><dd>Definition with <em>italic</em> text</dd></dl>"
    result = convert_to_markdown(html)
    expected = "**Bold Term**\n:   Definition with *italic* text\n\n"
    assert result == expected


def test_definition_with_links() -> None:
    html = '<dl><dt><a href="https://example.com">Linked Term</a></dt><dd>Definition with <a href="https://test.com">link</a></dd></dl>'
    result = convert_to_markdown(html)
    expected = "[Linked Term](https://example.com)\n:   Definition with [link](https://test.com)\n\n"
    assert result == expected


def test_definition_with_code() -> None:
    html = "<dl><dt><code>function</code></dt><dd>A block of code with <code>parameters</code></dd></dl>"
    result = convert_to_markdown(html)
    expected = "`function`\n:   A block of code with `parameters`\n\n"
    assert result == expected


def test_nested_definition_lists() -> None:
    html = "<dl><dt>Outer Term</dt><dd>Outer definition<dl><dt>Inner Term</dt><dd>Inner definition</dd></dl></dd></dl>"
    result = convert_to_markdown(html)
    expected = "Outer Term\n:   Outer definition\n\nInner Term\n:   Inner definition\n\n"
    assert result == expected


def test_definition_with_paragraphs() -> None:
    html = "<dl><dt>Complex Term</dt><dd><p>First paragraph of definition.</p><p>Second paragraph of definition.</p></dd></dl>"
    result = convert_to_markdown(html)
    expected = "Complex Term\n:   First paragraph of definition.\n\nSecond paragraph of definition.\n\n"
    assert result == expected


def test_definition_with_lists() -> None:
    html = "<dl><dt>List Term</dt><dd>Definition with list:<ul><li>Item 1</li><li>Item 2</li></ul></dd></dl>"
    result = convert_to_markdown(html)
    expected = "List Term\n:   Definition with list:\n\n* Item 1\n* Item 2\n\n"
    assert result == expected


def test_empty_definition_list() -> None:
    html = "<dl></dl>"
    result = convert_to_markdown(html)
    assert result == ""


def test_empty_term() -> None:
    html = "<dl><dt></dt><dd>Definition without term</dd></dl>"
    result = convert_to_markdown(html)
    expected = ":   Definition without term\n\n"
    assert result == expected


def test_empty_definition() -> None:
    html = "<dl><dt>Term without definition</dt><dd></dd></dl>"
    result = convert_to_markdown(html)
    expected = "Term without definition\n\n"
    assert result == expected


def test_definition_list_inline_mode() -> None:
    html = "<dl><dt>Term</dt><dd>Definition</dd></dl>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "TermDefinition"


def test_definition_whitespace_handling() -> None:
    html = "<dl><dt>  Term with spaces  </dt><dd>  Definition with spaces  </dd></dl>"
    result = convert_to_markdown(html)
    expected = "Term with spaces\n:   Definition with spaces\n\n"
    assert result == expected


def test_definition_with_blockquote() -> None:
    html = "<dl><dt>Quote Term</dt><dd><blockquote>This is a quoted definition.</blockquote></dd></dl>"
    result = convert_to_markdown(html)
    expected = "Quote Term\n:   > This is a quoted definition.\n\n"
    assert result == expected


def test_complex_definition_list() -> None:
    html = "<dl><dt><strong>HTML</strong></dt><dd>HyperText Markup Language</dd><dt><em>CSS</em></dt><dt>Cascading Style Sheets</dt><dd>A style sheet language used for describing the presentation of a document written in HTML</dd><dd>Also used with XML documents</dd><dt><code>JavaScript</code></dt><dd>A programming language that conforms to the ECMAScript specification.<ul><li>Dynamic typing</li><li>First-class functions</li></ul></dd></dl>"
    result = convert_to_markdown(html)
    expected = "**HTML**\n:   HyperText Markup Language\n\n*CSS*\nCascading Style Sheets\n:   A style sheet language used for describing the presentation of a document written in HTML\n\n:   Also used with XML documents\n\n`JavaScript`\n:   A programming language that conforms to the ECMAScript specification.\n\n* Dynamic typing\n* First\\-class functions\n\n"
    assert result == expected


def test_definition_list_attributes() -> None:
    html = '<dl class="definitions" id="main-list"><dt title="Term title">Term</dt><dd data-id="1">Definition</dd></dl>'
    result = convert_to_markdown(html)
    expected = "Term\n:   Definition\n\n"
    assert result == expected


def test_form_basic() -> None:
    html = "<form><p>Form content</p></form>"
    result = convert_to_markdown(html)
    assert result == "Form content\n\n"


def test_form_with_action() -> None:
    html = '<form action="/submit"><p>Form content</p></form>'
    result = convert_to_markdown(html)
    assert result == "Form content\n\n"


def test_form_with_method() -> None:
    html = '<form method="post"><p>Form content</p></form>'
    result = convert_to_markdown(html)
    assert result == "Form content\n\n"


def test_form_with_action_and_method() -> None:
    html = '<form action="/submit" method="post"><p>Form content</p></form>'
    result = convert_to_markdown(html)
    assert result == "Form content\n\n"


def test_form_empty() -> None:
    html = "<form></form>"
    result = convert_to_markdown(html)
    assert result == ""


def test_form_inline_mode() -> None:
    html = "<form>Form content</form>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Form content"


def test_fieldset_basic() -> None:
    html = "<fieldset><p>Fieldset content</p></fieldset>"
    result = convert_to_markdown(html)
    assert result == "Fieldset content\n\n"


def test_fieldset_with_legend() -> None:
    html = "<fieldset><legend>Form Section</legend><p>Content</p></fieldset>"
    result = convert_to_markdown(html)
    assert result == "**Form Section**\n\nContent\n\n"


def test_legend_standalone() -> None:
    html = "<legend>Legend text</legend>"
    result = convert_to_markdown(html)
    assert result == "**Legend text**\n\n"


def test_fieldset_empty() -> None:
    html = "<fieldset></fieldset>"
    result = convert_to_markdown(html)
    assert result == ""


def test_legend_empty() -> None:
    html = "<legend></legend>"
    result = convert_to_markdown(html)
    assert result == ""


def test_fieldset_inline_mode() -> None:
    html = "<fieldset>Inline content</fieldset>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline content"


def test_label_basic() -> None:
    html = "<label>Label text</label>"
    result = convert_to_markdown(html)
    assert result == "Label text\n\n"


def test_label_with_for() -> None:
    html = '<label for="username">Username</label>'
    result = convert_to_markdown(html)
    assert result == "Username\n\n"


def test_label_with_input() -> None:
    html = '<label>Username: <input type="text" name="username"></label>'
    result = convert_to_markdown(html)
    assert result == "Username:\n\n"


def test_label_empty() -> None:
    html = "<label></label>"
    result = convert_to_markdown(html)
    assert result == ""


def test_label_inline_mode() -> None:
    html = "<label>Inline label</label>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline label"


def test_input_text() -> None:
    html = '<input type="text" name="username">'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_password() -> None:
    html = '<input type="password" name="password">'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_with_value() -> None:
    html = '<input type="text" name="username" value="john">'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_with_placeholder() -> None:
    html = '<input type="text" name="username" placeholder="Enter username">'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_required() -> None:
    html = '<input type="text" name="username" required>'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_disabled() -> None:
    html = '<input type="text" name="username" disabled>'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_readonly() -> None:
    html = '<input type="text" name="username" readonly>'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_checkbox_unchecked() -> None:
    html = '<input type="checkbox" name="agree">'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_checkbox_checked() -> None:
    html = '<input type="checkbox" name="agree" checked>'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_radio() -> None:
    html = '<input type="radio" name="gender" value="male">'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_submit() -> None:
    html = '<input type="submit" value="Submit">'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_file() -> None:
    html = '<input type="file" name="upload" accept=".jpg,.png">'
    result = convert_to_markdown(html)
    assert result == ""


def test_input_inline_mode() -> None:
    html = '<input type="text" name="username">'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == ""


def test_textarea_basic() -> None:
    html = "<textarea>Default text</textarea>"
    result = convert_to_markdown(html)
    assert result == "Default text\n\n"


def test_textarea_with_name() -> None:
    html = '<textarea name="comment">Comment text</textarea>'
    result = convert_to_markdown(html)
    assert result == "Comment text\n\n"


def test_textarea_with_placeholder() -> None:
    html = '<textarea placeholder="Enter your comment">Default text</textarea>'
    result = convert_to_markdown(html)
    assert result == "Default text\n\n"


def test_textarea_with_rows_cols() -> None:
    html = '<textarea rows="5" cols="30">Text</textarea>'
    result = convert_to_markdown(html)
    assert result == "Text\n\n"


def test_textarea_required() -> None:
    html = "<textarea required>Required text</textarea>"
    result = convert_to_markdown(html)
    assert result == "Required text\n\n"


def test_textarea_empty() -> None:
    html = "<textarea></textarea>"
    result = convert_to_markdown(html)
    assert result == ""


def test_textarea_inline_mode() -> None:
    html = "<textarea>Inline text</textarea>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline text"


def test_select_basic() -> None:
    html = "<select><option>Option 1</option><option>Option 2</option></select>"
    result = convert_to_markdown(html)
    assert result == "Option 1\nOption 2\n\n"


def test_select_with_name() -> None:
    html = '<select name="country"><option>USA</option><option>Canada</option></select>'
    result = convert_to_markdown(html)
    assert result == "USA\nCanada\n\n"


def test_select_multiple() -> None:
    html = "<select multiple><option>Option 1</option><option>Option 2</option></select>"
    result = convert_to_markdown(html)
    assert result == "Option 1\nOption 2\n\n"


def test_option_with_value() -> None:
    html = '<select><option value="us">United States</option><option value="ca">Canada</option></select>'
    result = convert_to_markdown(html)
    assert result == "United States\nCanada\n\n"


def test_option_selected() -> None:
    html = "<select><option>Option 1</option><option selected>Option 2</option></select>"
    result = convert_to_markdown(html)
    assert result == "Option 1\n* Option 2\n\n"


def test_optgroup() -> None:
    html = '<select><optgroup label="Group 1"><option>Option 1</option><option>Option 2</option></optgroup></select>'
    result = convert_to_markdown(html)
    assert result == "**Group 1**\nOption 1\nOption 2\n\n"


def test_select_empty() -> None:
    html = "<select></select>"
    result = convert_to_markdown(html)
    assert result == ""


def test_option_empty() -> None:
    html = "<select><option></option></select>"
    result = convert_to_markdown(html)
    assert result == ""


def test_select_inline_mode() -> None:
    html = "<select><option>Option</option></select>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Option"


def test_button_basic() -> None:
    html = "<button>Click me</button>"
    result = convert_to_markdown(html)
    assert result == "Click me\n\n"


def test_button_with_type() -> None:
    html = '<button type="submit">Submit</button>'
    result = convert_to_markdown(html)
    assert result == "Submit\n\n"


def test_button_disabled() -> None:
    html = "<button disabled>Disabled</button>"
    result = convert_to_markdown(html)
    assert result == "Disabled\n\n"


def test_button_with_name_value() -> None:
    html = '<button name="action" value="delete">Delete</button>'
    result = convert_to_markdown(html)
    assert result == "Delete\n\n"


def test_button_empty() -> None:
    html = "<button></button>"
    result = convert_to_markdown(html)
    assert result == ""


def test_button_inline_mode() -> None:
    html = "<button>Inline button</button>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline button"


def test_progress_basic() -> None:
    html = "<progress>50%</progress>"
    result = convert_to_markdown(html)
    assert result == "50%\n\n"


def test_progress_with_value_max() -> None:
    html = '<progress value="50" max="100">50%</progress>'
    result = convert_to_markdown(html)
    assert result == "50%\n\n"


def test_meter_basic() -> None:
    html = "<meter>6 out of 10</meter>"
    result = convert_to_markdown(html)
    assert result == "6 out of 10\n\n"


def test_meter_with_attributes() -> None:
    html = '<meter value="6" min="0" max="10" low="2" high="8" optimum="5">6 out of 10</meter>'
    result = convert_to_markdown(html)
    assert result == "6 out of 10\n\n"


def test_progress_empty() -> None:
    html = "<progress></progress>"
    result = convert_to_markdown(html)
    assert result == ""


def test_meter_empty() -> None:
    html = "<meter></meter>"
    result = convert_to_markdown(html)
    assert result == ""


def test_progress_inline_mode() -> None:
    html = "<progress>50%</progress>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "50%"


def test_meter_inline_mode() -> None:
    html = "<meter>6/10</meter>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "6/10"


def test_output_basic() -> None:
    html = "<output>Result: 42</output>"
    result = convert_to_markdown(html)
    assert result == "Result: 42\n\n"


def test_output_with_for() -> None:
    html = '<output for="input1 input2">Sum: 15</output>'
    result = convert_to_markdown(html)
    assert result == "Sum: 15\n\n"


def test_output_with_name() -> None:
    html = '<output name="result">42</output>'
    result = convert_to_markdown(html)
    assert result == "42\n\n"


def test_datalist_basic() -> None:
    html = "<datalist><option>Option 1</option><option>Option 2</option></datalist>"
    result = convert_to_markdown(html)
    assert result == "Option 1\nOption 2\n\n"


def test_datalist_with_id() -> None:
    html = '<datalist id="browsers"><option>Chrome</option><option>Firefox</option></datalist>'
    result = convert_to_markdown(html)
    assert result == "Chrome\nFirefox\n\n"


def test_output_empty() -> None:
    html = "<output></output>"
    result = convert_to_markdown(html)
    assert result == ""


def test_datalist_empty() -> None:
    html = "<datalist></datalist>"
    result = convert_to_markdown(html)
    assert result == ""


def test_output_inline_mode() -> None:
    html = "<output>Result</output>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Result"


def test_datalist_inline_mode() -> None:
    html = "<datalist><option>Option</option></datalist>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Option"


def test_complete_form_example() -> None:
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
    result = convert_to_markdown(html)
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


def test_form_with_progress_and_meter() -> None:
    html = """<form>
        <label>Upload Progress:</label>
        <progress value="75" max="100">75%</progress>
        <label>Rating:</label>
        <meter value="4" min="1" max="5">4 out of 5</meter>
        <output for="rating">Current rating: 4/5</output>
    </form>"""
    result = convert_to_markdown(html)
    expected = """Upload Progress:

75%

Rating:

4 out of 5

Current rating: 4/5

"""
    assert result == expected


def test_form_with_inputs_inline_mode() -> None:
    html = '<form><label>Name:</label> <input type="text" name="name"> <button>Submit</button></form>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Name:  Submit"


def test_article_element() -> None:
    html = "<article>This is an article</article>"
    result = convert_to_markdown(html)
    assert result == "This is an article\n\n"


def test_section_element() -> None:
    html = "<section>This is a section</section>"
    result = convert_to_markdown(html)
    assert result == "This is a section\n\n"


def test_nav_element() -> None:
    html = "<nav>This is navigation</nav>"
    result = convert_to_markdown(html)
    assert result == "This is navigation\n\n"


def test_aside_element() -> None:
    html = "<aside>This is an aside</aside>"
    result = convert_to_markdown(html)
    assert result == "This is an aside\n\n"


def test_header_element() -> None:
    html = "<header>This is a header</header>"
    result = convert_to_markdown(html)
    assert result == "This is a header\n\n"


def test_footer_element() -> None:
    html = "<footer>This is a footer</footer>"
    result = convert_to_markdown(html)
    assert result == "This is a footer\n\n"


def test_main_element() -> None:
    html = "<main>This is main content</main>"
    result = convert_to_markdown(html)
    assert result == "This is main content\n\n"


def test_article_with_sections() -> None:
    html = "<article><header>Article Header</header><section><h2>Section Title</h2><p>Section content</p></section><footer>Article Footer</footer></article>"
    result = convert_to_markdown(html, heading_style="atx")
    expected = "Article Header\n\n## Section Title\n\nSection content\n\nArticle Footer\n\n"
    assert result == expected


def test_semantic_elements_with_other_content() -> None:
    html = '<nav><ul><li><a href="#home">Home</a></li><li><a href="#about">About</a></li></ul></nav><main><article><h1>Article Title</h1><p>Article content</p></article></main>'
    result = convert_to_markdown(html, heading_style="atx")
    expected = "* [Home](#home)\n* [About](#about)\n\n# Article Title\n\nArticle content\n\n"
    assert result == expected


def test_empty_article_element() -> None:
    html = "<article></article>"
    result = convert_to_markdown(html)
    assert result == ""


def test_article_inline_mode() -> None:
    html = "<article>This is inline content</article>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "This is inline content"


def test_semantic_elements_with_whitespace() -> None:
    html = "<section>  \n  Content with whitespace  \n  </section>"
    result = convert_to_markdown(html)
    assert result == " Content with whitespace \n\n"


def test_details_element() -> None:
    html = "<details>This is details content</details>"
    result = convert_to_markdown(html)
    assert result == "This is details content\n\n"


def test_summary_element() -> None:
    html = "<summary>Summary text</summary>"
    result = convert_to_markdown(html)
    assert result == "**Summary text**\n\n"


def test_details_with_summary() -> None:
    html = "<details><summary>Click to expand</summary><p>Hidden content here</p></details>"
    result = convert_to_markdown(html)
    expected = "**Click to expand**\n\nHidden content here\n\n"
    assert result == expected


def test_nested_details() -> None:
    html = "<details><summary>Level 1</summary><details><summary>Level 2</summary><p>Nested content</p></details></details>"
    result = convert_to_markdown(html)
    expected = "**Level 1**\n\n**Level 2**\n\nNested content\n\n"
    assert result == expected


def test_details_with_complex_content() -> None:
    html = '<details><summary>Code Example</summary><pre><code>def hello():\n    print("Hello, World!")</code></pre><p>This is a Python function.</p></details>'
    result = convert_to_markdown(html)
    expected = '**Code Example**\n\n```\ndef hello():\n    print("Hello, World!")\n```\nThis is a Python function.\n\n'
    assert result == expected


def test_empty_details() -> None:
    html = "<details></details>"
    result = convert_to_markdown(html)
    assert result == ""


def test_empty_summary() -> None:
    html = "<summary></summary>"
    result = convert_to_markdown(html)
    assert result == ""


def test_details_inline_mode() -> None:
    html = "<details>Inline details</details>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline details"


def test_summary_inline_mode() -> None:
    html = "<summary>Inline summary</summary>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline summary"


def test_details_with_attributes() -> None:
    html = "<details open><summary>Always open</summary><p>Content</p></details>"
    result = convert_to_markdown(html)
    expected = "**Always open**\n\nContent\n\n"
    assert result == expected


def test_audio_basic() -> None:
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


def test_blockquote_with_single_newline_end() -> None:
    html = "<blockquote>Test content\n</blockquote>"
    result = convert_to_markdown(html)
    assert result == "\n> Test content\n\n"


def test_list_with_empty_lines_multiline_content() -> None:
    html = """<ul>
    <li><p>First paragraph</p>

    <p>Second paragraph</p></li>
</ul>"""
    result = convert_to_markdown(html)
    assert "First paragraph\n\n    Second paragraph" in result


def test_list_with_empty_lines() -> None:
    html = """<ul>
    <li>
        First item

        Second paragraph
    </li>
</ul>"""
    result = convert_to_markdown(html)
    expected = "* First item\n\n Second paragraph\n"
    assert result == expected


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


def test_ruby_basic() -> None:
    html = "<ruby>漢字<rt>kanji</rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字(kanji)"


def test_ruby_with_rb() -> None:
    html = "<ruby><rb>漢字</rb><rt>kanji</rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字(kanji)"


def test_ruby_with_fallback_rp() -> None:
    html = "<ruby>漢字<rp>(</rp><rt>kanji</rt><rp>)</rp></ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字(kanji)"


def test_ruby_complex_structure() -> None:
    html = "<ruby><rb>東京</rb><rp>(</rp><rt>とうきょう</rt><rp>)</rp></ruby>"
    result = convert_to_markdown(html)
    assert result == "東京(とうきょう)"


def test_ruby_multiple_readings() -> None:
    html = "<ruby><rb>漢</rb><rt>kan</rt><rb>字</rb><rt>ji</rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "漢(kan)字(ji)"


def test_ruby_inline_mode() -> None:
    html = "<ruby>漢字<rt>kanji</rt></ruby>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "漢字(kanji)"


def test_ruby_block_mode() -> None:
    html = "<ruby>漢字<rt>kanji</rt></ruby>"
    result = convert_to_markdown(html, convert_as_inline=False)
    assert result == "漢字(kanji)"


def test_ruby_nested_in_paragraph() -> None:
    html = "<p>This is <ruby>漢字<rt>kanji</rt></ruby> text.</p>"
    result = convert_to_markdown(html)
    assert result == "This is 漢字(kanji) text.\n\n"


def test_ruby_with_whitespace() -> None:
    html = "<ruby> 漢字 <rt> kanji </rt> </ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字 (kanji)"


def test_ruby_empty_elements() -> None:
    html = "<ruby><rb></rb><rt></rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "()"


def test_ruby_only_base_text() -> None:
    html = "<ruby>漢字</ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字"


def test_ruby_only_annotation() -> None:
    html = "<ruby><rt>kanji</rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "(kanji)"


def test_ruby_with_formatting() -> None:
    html = "<ruby><strong>漢字</strong><rt><em>kanji</em></rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "**漢字**(*kanji*)"


def test_ruby_multiple_in_sentence() -> None:
    html = "I love <ruby>寿司<rt>sushi</rt></ruby> and <ruby>刺身<rt>sashimi</rt></ruby>!"
    result = convert_to_markdown(html)
    assert result == "I love 寿司(sushi) and 刺身(sashimi)!"


def test_ruby_with_mixed_content() -> None:
    html = "<ruby>東<rb>京</rb>都<rt>とう<strong>きょう</strong>と</rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "東京都(とう**きょう**と)"


def test_rb_standalone() -> None:
    html = "<rb>漢字</rb>"
    result = convert_to_markdown(html)
    assert result == "漢字"


def test_rb_inline_mode() -> None:
    html = "<rb>漢字</rb>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "漢字"


def test_rb_block_mode() -> None:
    html = "<rb>漢字</rb>"
    result = convert_to_markdown(html, convert_as_inline=False)
    assert result == "漢字"


def test_rt_standalone() -> None:
    html = "<rt>kanji</rt>"
    result = convert_to_markdown(html)
    assert result == "(kanji)"


def test_rt_with_surrounding_rp() -> None:
    html = "<rp>(</rp><rt>kanji</rt><rp>)</rp>"
    result = convert_to_markdown(html)
    assert result == "(kanji)"


def test_rt_inline_mode() -> None:
    html = "<rt>kanji</rt>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "(kanji)"


def test_rt_block_mode() -> None:
    html = "<rt>kanji</rt>"
    result = convert_to_markdown(html, convert_as_inline=False)
    assert result == "(kanji)"


def test_rp_standalone() -> None:
    html = "<rp>(</rp>"
    result = convert_to_markdown(html)
    assert result == "("


def test_rp_inline_mode() -> None:
    html = "<rp>)</rp>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == ")"


def test_rp_block_mode() -> None:
    html = "<rp>(</rp>"
    result = convert_to_markdown(html, convert_as_inline=False)
    assert result == "("


def test_rtc_standalone() -> None:
    html = "<rtc>annotation</rtc>"
    result = convert_to_markdown(html)
    assert result == "annotation"


def test_rtc_inline_mode() -> None:
    html = "<rtc>annotation</rtc>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "annotation"


def test_rtc_block_mode() -> None:
    html = "<rtc>annotation</rtc>"
    result = convert_to_markdown(html, convert_as_inline=False)
    assert result == "annotation"


def test_nested_ruby_elements() -> None:
    html = "<ruby><ruby>漢<rt>kan</rt></ruby><rt>字</rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "漢(kan)(字)"


def test_ruby_with_line_breaks() -> None:
    html = "<ruby>\n漢字\n<rt>\nkanji\n</rt>\n</ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字(kanji)"


def test_ruby_with_special_characters() -> None:
    html = "<ruby>*test*<rt>_annotation_</rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "\\*test\\*(\\_annotation\\_)"


def test_ruby_with_links() -> None:
    html = '<ruby><a href="https://example.com">漢字</a><rt>kanji</rt></ruby>'
    result = convert_to_markdown(html)
    assert result == "[漢字](https://example.com)(kanji)"


def test_ruby_in_table() -> None:
    html = "<table><tr><td><ruby>漢字<rt>kanji</rt></ruby></td></tr></table>"
    result = convert_to_markdown(html)
    assert "漢字(kanji)" in result


def test_ruby_in_list() -> None:
    html = "<ul><li><ruby>漢字<rt>kanji</rt></ruby></li></ul>"
    result = convert_to_markdown(html)
    assert "* 漢字(kanji)" in result


def test_multiple_rt_elements() -> None:
    html = "<ruby>漢字<rt>kan</rt><rt>ji</rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字(kan)(ji)"


def test_ruby_with_rtc_and_rt() -> None:
    html = "<ruby>漢字<rt>kanji</rt><rtc>Chinese characters</rtc></ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字(kanji)Chinese characters"


def test_complex_ruby_structure() -> None:
    html = """<ruby>
        <rb>漢</rb>
        <rb>字</rb>
        <rp>(</rp>
        <rt>kan</rt>
        <rt>ji</rt>
        <rp>)</rp>
        <rtc>Chinese characters</rtc>
    </ruby>"""
    result = convert_to_markdown(html)
    assert result == "漢字((kan)(ji))Chinese characters"


def test_ruby_with_empty_rt() -> None:
    html = "<ruby>漢字<rt></rt></ruby>"
    result = convert_to_markdown(html)
    assert result == "漢字()"


def test_ruby_with_only_spaces() -> None:
    html = "<ruby>   <rt>   </rt>   </ruby>"
    result = convert_to_markdown(html)
    assert result == "()"


def test_abbr_basic() -> None:
    html = "<abbr>HTML</abbr>"
    result = convert_to_markdown(html)
    assert result == "HTML"


def test_abbr_with_title() -> None:
    html = '<abbr title="HyperText Markup Language">HTML</abbr>'
    result = convert_to_markdown(html)
    assert result == "HTML (HyperText Markup Language)"


def test_abbr_with_empty_title() -> None:
    html = '<abbr title="">HTML</abbr>'
    result = convert_to_markdown(html)
    assert result == "HTML"


def test_abbr_inline_mode() -> None:
    html = '<abbr title="HyperText Markup Language">HTML</abbr>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "HTML (HyperText Markup Language)"


def test_abbr_nested_content() -> None:
    html = '<p>Learn <abbr title="HyperText Markup Language">HTML</abbr> today!</p>'
    result = convert_to_markdown(html)
    assert result == "Learn HTML (HyperText Markup Language) today!\n\n"


def test_time_basic() -> None:
    html = "<time>2023-12-25</time>"
    result = convert_to_markdown(html)
    assert result == "2023\\-12\\-25"


def test_time_with_datetime() -> None:
    html = '<time datetime="2023-12-25T10:30:00">Christmas Day</time>'
    result = convert_to_markdown(html)
    assert result == "Christmas Day"


def test_time_with_empty_datetime() -> None:
    html = '<time datetime="">December 25</time>'
    result = convert_to_markdown(html)
    assert result == "December 25"


def test_time_inline_mode() -> None:
    html = '<time datetime="2023-12-25">Christmas</time>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Christmas"


def test_time_in_paragraph() -> None:
    html = '<p>The event is on <time datetime="2023-12-25">Christmas Day</time>.</p>'
    result = convert_to_markdown(html)
    assert result == "The event is on Christmas Day.\n\n"


def test_data_basic() -> None:
    html = "<data>Product Name</data>"
    result = convert_to_markdown(html)
    assert result == "Product Name"


def test_data_with_value() -> None:
    html = '<data value="12345">Product Name</data>'
    result = convert_to_markdown(html)
    assert result == "Product Name"


def test_data_with_empty_value() -> None:
    html = '<data value="">Product</data>'
    result = convert_to_markdown(html)
    assert result == "Product"


def test_data_inline_mode() -> None:
    html = '<data value="12345">Product</data>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Product"


def test_data_in_list() -> None:
    html = '<ul><li><data value="A001">Product A</data></li><li><data value="B002">Product B</data></li></ul>'
    result = convert_to_markdown(html)
    assert result == "* Product A\n* Product B\n"


def test_ins_basic() -> None:
    html = "<ins>This text was added</ins>"
    result = convert_to_markdown(html)
    assert result == "==This text was added=="


def test_ins_with_cite() -> None:
    html = '<ins cite="https://example.com">Added text</ins>'
    result = convert_to_markdown(html)
    assert result == "==Added text=="


def test_ins_with_datetime() -> None:
    html = '<ins datetime="2023-12-25">Added on Christmas</ins>'
    result = convert_to_markdown(html)
    assert result == "==Added on Christmas=="


def test_ins_inline_mode() -> None:
    html = "<ins>Added text</ins>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "==Added text=="


def test_ins_in_paragraph() -> None:
    html = "<p>Original text <ins>with addition</ins> and more.</p>"
    result = convert_to_markdown(html)
    assert result == "Original text ==with addition== and more.\n\n"


def test_var_basic() -> None:
    html = "<var>x</var>"
    result = convert_to_markdown(html)
    assert result == "*x*"


def test_var_in_code() -> None:
    html = "<p>Set <var>username</var> to your login name.</p>"
    result = convert_to_markdown(html)
    assert result == "Set *username* to your login name.\n\n"


def test_var_mathematical() -> None:
    html = "<p>If <var>x</var> = 5, then <var>y</var> = <var>x</var> + 3.</p>"
    result = convert_to_markdown(html)
    assert result == "If *x* \\= 5, then *y* \\= *x* \\+ 3\\.\n\n"


def test_var_inline_mode() -> None:
    html = "<var>variable</var>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "*variable*"


def test_dfn_basic() -> None:
    html = "<dfn>API</dfn>"
    result = convert_to_markdown(html)
    assert result == "*API*"


def test_dfn_with_title() -> None:
    html = '<dfn title="Application Programming Interface">API</dfn>'
    result = convert_to_markdown(html)
    assert result == "*API*"


def test_dfn_in_definition_list() -> None:
    html = "<dl><dt><dfn>API</dfn></dt><dd>Application Programming Interface</dd></dl>"
    result = convert_to_markdown(html)
    assert result == "*API*\n:   Application Programming Interface\n\n"


def test_dfn_inline_mode() -> None:
    html = "<dfn>term</dfn>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "*term*"


def test_bdi_basic() -> None:
    html = "<bdi>عربي</bdi>"
    result = convert_to_markdown(html)
    assert result == "عربي"


def test_bdo_basic() -> None:
    html = '<bdo dir="rtl">English text</bdo>'
    result = convert_to_markdown(html)
    assert result == "English text"


def test_bdi_mixed_text() -> None:
    html = "<p>User <bdi>إيان</bdi> scored 90 points.</p>"
    result = convert_to_markdown(html)
    assert result == "User إيان scored 90 points.\n\n"


def test_bdo_with_direction() -> None:
    html = '<p>The title is <bdo dir="rtl">مرحبا</bdo> in Arabic.</p>'
    result = convert_to_markdown(html)
    assert result == "The title is مرحبا in Arabic.\n\n"


def test_bdi_inline_mode() -> None:
    html = "<bdi>نص عربي</bdi>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "نص عربي"


def test_small_basic() -> None:
    html = "<small>Fine print</small>"
    result = convert_to_markdown(html)
    assert result == "Fine print"


def test_small_copyright() -> None:
    html = "<p>© 2023 Company Name. <small>All rights reserved.</small></p>"
    result = convert_to_markdown(html)
    assert result == "© 2023 Company Name. All rights reserved.\n\n"


def test_small_inline_mode() -> None:
    html = "<small>Legal disclaimer</small>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Legal disclaimer"


def test_u_basic() -> None:
    html = "<u>Underlined text</u>"
    result = convert_to_markdown(html)
    assert result == "Underlined text"


def test_u_misspelling() -> None:
    html = "<p>This word is <u>mispelled</u>.</p>"
    result = convert_to_markdown(html)
    assert result == "This word is mispelled.\n\n"


def test_u_inline_mode() -> None:
    html = "<u>underlined</u>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "underlined"


def test_wbr_basic() -> None:
    html = "super<wbr>cali<wbr>fragilistic"
    result = convert_to_markdown(html)
    assert result == "supercalifragilistic"


def test_wbr_long_url() -> None:
    html = "<p>Visit https://www.<wbr>example.<wbr>com/very/<wbr>long/<wbr>path</p>"
    result = convert_to_markdown(html)
    assert result == "Visit https://www.example.com/very/long/path\n\n"


def test_wbr_inline_mode() -> None:
    html = "long<wbr>word"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "longword"


def test_mixed_semantic_elements() -> None:
    html = """<article>
        <h2>Programming Concepts</h2>
        <p>An <dfn>API</dfn> (<abbr title="Application Programming Interface">API</abbr>)
        allows different software components to communicate.</p>
        <p>When you set the variable <var>timeout</var> to <data value="5000">5 seconds</data>,
        <ins>added in version 2.0</ins>, the system will wait.</p>
        <p><small>Last updated: <time datetime="2023-12-25">December 25, 2023</time></small></p>
    </article>"""
    result = convert_to_markdown(html, heading_style="atx")
    expected = """## Programming Concepts

An *API* (API (Application Programming Interface))
 allows different software components to communicate.

When you set the variable *timeout* to 5 seconds, ==added in version 2\\.0==, the system will wait.

Last updated: December 25, 2023

"""
    assert result == expected


def test_complex_nested_semantic_elements() -> None:
    html = '<p>The <dfn><abbr title="Application Programming Interface">API</abbr></dfn> documentation has been <ins>updated with <var>new_parameter</var></ins>.</p>'
    result = convert_to_markdown(html)
    assert (
        result
        == "The *API (Application Programming Interface)* documentation has been ==updated with *new\\_parameter*==.\n\n"
    )


def test_mixed_semantic_elements_inline_mode() -> None:
    html = '<abbr title="HyperText Markup Language">HTML</abbr> and <var>css</var> with <ins>updates</ins>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "HTML (HyperText Markup Language) and *css* with ==updates=="


def test_multiple_empty_semantic_elements() -> None:
    html = "<p>Empty elements: <abbr></abbr> <var></var> <ins></ins> <dfn></dfn></p>"
    result = convert_to_markdown(html)
    assert result == "Empty elements: \n\n"


def test_whitespace_handling_semantic() -> None:
    html = "<p>Spaces around <var>  variable  </var> and <abbr title='  title  '>  abbr  </abbr></p>"
    result = convert_to_markdown(html)
    assert result == "Spaces around  *variable*  and abbr (title)\n\n"


def test_figure_basic() -> None:
    html = '<figure><img src="image.jpg" alt="Test image"></figure>'
    result = convert_to_markdown(html)
    assert result == "![Test image](image.jpg)\n\n"


def test_figure_with_caption() -> None:
    html = '<figure><img src="test.jpg"><figcaption>Image caption</figcaption></figure>'
    result = convert_to_markdown(html)
    expected = "![](test.jpg)\n\n*Image caption*\n\n"
    assert result == expected


def test_figure_with_id() -> None:
    html = '<figure id="fig1"><img src="chart.png"></figure>'
    result = convert_to_markdown(html)
    assert result == "![](chart.png)\n\n"


def test_figure_with_class() -> None:
    html = '<figure class="photo"><img src="photo.jpg"></figure>'
    result = convert_to_markdown(html)
    assert result == "![](photo.jpg)\n\n"


def test_figure_with_multiple_attributes() -> None:
    html = '<figure id="fig2" class="diagram"><img src="diagram.svg"></figure>'
    result = convert_to_markdown(html)
    assert result == "![](diagram.svg)\n\n"


def test_figure_empty() -> None:
    html = "<figure></figure>"
    result = convert_to_markdown(html)
    assert result == ""


def test_figure_inline_mode() -> None:
    html = '<figure><img src="inline.jpg" alt="Inline image"></figure>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline image"


def test_figure_with_complex_content() -> None:
    html = """<figure>
        <img src="main.jpg" alt="Main image">
        <figcaption>
            <strong>Figure 1:</strong> This is a complex caption with <em>emphasis</em>.
        </figcaption>
    </figure>"""
    result = convert_to_markdown(html)
    expected = """![Main image](main.jpg)\n\n***Figure 1:** This is a complex caption with *emphasis*.*\n\n"""
    assert result == expected


def test_figure_with_multiple_images() -> None:
    html = """<figure>
        <img src="before.jpg" alt="Before">
        <img src="after.jpg" alt="After">
        <figcaption>Before and after comparison</figcaption>
    </figure>"""
    result = convert_to_markdown(html)
    expected = """![Before](before.jpg)![After](after.jpg)\n\n*Before and after comparison*\n\n"""
    assert result == expected


def test_figure_with_nested_elements() -> None:
    html = """<figure id="stats">
        <table>
            <tr><th>Year</th><th>Sales</th></tr>
            <tr><td>2023</td><td>100</td></tr>
        </table>
        <figcaption>Annual sales data</figcaption>
    </figure>"""
    result = convert_to_markdown(html)
    expected = """| Year | Sales |\n| --- | --- |\n| 2023 | 100 |\n\n*Annual sales data*\n\n"""
    assert result == expected


def test_hgroup_basic() -> None:
    html = "<hgroup><h1>Main Title</h1><h2>Subtitle</h2></hgroup>"
    result = convert_to_markdown(html)
    expected = "Main Title\n==========\n\nSubtitle\n--------\n\n"
    assert result == expected


def test_hgroup_multiple_headings() -> None:
    html = "<hgroup><h1>Title</h1><h2>Subtitle</h2><h3>Section</h3></hgroup>"
    result = convert_to_markdown(html)
    expected = "Title\n=====\n\nSubtitle\n--------\n\n### Section\n\n"
    assert result == expected


def test_hgroup_empty() -> None:
    html = "<hgroup></hgroup>"
    result = convert_to_markdown(html)
    assert result == ""


def test_hgroup_inline_mode() -> None:
    html = "<hgroup><h1>Inline Title</h1></hgroup>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline Title"


def test_hgroup_with_atx_headings() -> None:
    html = "<hgroup><h1>Main</h1><h2>Sub</h2></hgroup>"
    result = convert_to_markdown(html, heading_style="atx")
    expected = "# Main\n\n## Sub\n\n"
    assert result == expected


def test_hgroup_excessive_spacing() -> None:
    html = "<hgroup><h1>Title</h1><p></p><p></p><h2>Subtitle</h2></hgroup>"
    result = convert_to_markdown(html)
    expected = "Title\n=====\n\nSubtitle\n--------\n\n"
    assert result == expected


def test_hgroup_with_formatted_headings() -> None:
    html = "<hgroup><h1>The <em>Amazing</em> Title</h1><h2>A <strong>Bold</strong> Subtitle</h2></hgroup>"
    result = convert_to_markdown(html)
    expected = "The *Amazing* Title\n===================\n\nA **Bold** Subtitle\n-------------------\n\n"
    assert result == expected


def test_picture_basic() -> None:
    html = '<picture><img src="image.jpg" alt="Test"></picture>'
    result = convert_to_markdown(html)
    assert result == "![Test](image.jpg)"


def test_picture_with_source() -> None:
    html = """<picture>
        <source srcset="large.jpg" media="(min-width: 800px)">
        <img src="small.jpg" alt="Responsive image">
    </picture>"""
    result = convert_to_markdown(html)
    assert result == "![Responsive image](small.jpg)"


def test_picture_multiple_sources() -> None:
    html = """<picture>
        <source srcset="image.webp" type="image/webp">
        <source srcset="image.jpg" type="image/jpeg">
        <img src="fallback.jpg" alt="Multi-format">
    </picture>"""
    result = convert_to_markdown(html)
    assert result == "![Multi-format](fallback.jpg)"


def test_picture_complex_srcset() -> None:
    html = """<picture>
        <source srcset="small.jpg 480w, medium.jpg 800w, large.jpg 1200w"
                media="(min-width: 600px)">
        <img src="default.jpg">
    </picture>"""
    result = convert_to_markdown(html)
    assert result == "![](default.jpg)"


def test_picture_no_img() -> None:
    html = '<picture><source srcset="test.jpg"></picture>'
    result = convert_to_markdown(html)
    assert result == ""


def test_picture_empty() -> None:
    html = "<picture></picture>"
    result = convert_to_markdown(html)
    assert result == ""


def test_picture_inline_mode() -> None:
    html = """<picture>
        <source srcset="large.jpg" media="(min-width: 800px)">
        <img src="small.jpg" alt="Test">
    </picture>"""
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Test"


def test_picture_with_sizes() -> None:
    html = """<picture>
        <source srcset="img-480.jpg 480w, img-800.jpg 800w"
                sizes="(max-width: 600px) 480px, 800px">
        <img src="default.jpg">
    </picture>"""
    result = convert_to_markdown(html)
    assert result == "![](default.jpg)"


def test_figure_in_article() -> None:
    html = """<article>
        <h1>Article Title</h1>
        <figure id="main-image">
            <img src="hero.jpg" alt="Hero image">
            <figcaption>The main article image</figcaption>
        </figure>
        <p>Article content...</p>
    </article>"""
    result = convert_to_markdown(html)
    expected = """Article Title\n=============\n\n![Hero image](hero.jpg)\n\n*The main article image*\n\nArticle content...\n\n"""
    assert result == expected


def test_hgroup_in_header() -> None:
    html = """<header>
        <hgroup>
            <h1>Site Title</h1>
            <h2>Site Tagline</h2>
        </hgroup>
        <nav>Navigation here</nav>
    </header>"""
    result = convert_to_markdown(html)
    expected = """Site Title\n==========\n\nSite Tagline\n------------\n\nNavigation here\n\n"""
    assert result == expected


def test_picture_in_figure() -> None:
    html = """<figure>
        <picture>
            <source srcset="large.webp" type="image/webp">
            <img src="fallback.jpg" alt="Test image">
        </picture>
        <figcaption>A responsive image in a figure</figcaption>
    </figure>"""
    result = convert_to_markdown(html)
    expected = """![Test image](fallback.jpg)\n\n*A responsive image in a figure*\n\n"""
    assert result == expected


def test_multiple_figures() -> None:
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
    result = convert_to_markdown(html)
    expected = """\n    ![](image1.jpg)\n\n*First figure*\n\n![](image2.jpg)\n\n*Second figure*\n\n"""
    assert result == expected


def test_nested_structural_elements() -> None:
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
    result = convert_to_markdown(html)
    expected = """Section Title\n=============\n\nSection Subtitle\n----------------\n\n![Data chart](chart.png)\n\n*Quarterly results*\n\n"""
    assert result == expected


def test_figure_with_special_characters() -> None:
    html = '<figure><img src="test.jpg"><figcaption>Caption with *asterisks* and _underscores_</figcaption></figure>'
    result = convert_to_markdown(html)
    expected = "![](test.jpg)\n\n*Caption with \\*asterisks\\* and \\_underscores\\_*\n\n"
    assert result == expected


def test_hgroup_single_heading() -> None:
    html = "<hgroup><h1>Only Title</h1></hgroup>"
    result = convert_to_markdown(html)
    expected = "Only Title\n==========\n\n"
    assert result == expected


def test_picture_malformed_source() -> None:
    html = """<picture>
        <source>
        <source srcset="">
        <img src="valid.jpg">
    </picture>"""
    result = convert_to_markdown(html)
    assert result == "![](valid.jpg)"


def test_figure_whitespace_handling() -> None:
    html = """<figure>

        <img src="test.jpg">

        <figcaption>
            Caption text
        </figcaption>

    </figure>"""
    result = convert_to_markdown(html)
    expected = "![](test.jpg)\n\n*Caption text*\n\n"
    assert result == expected


def test_empty_elements_with_attributes() -> None:
    html1 = '<figure id="empty-fig"></figure>'
    assert convert_to_markdown(html1) == ""

    html2 = '<hgroup class="empty"></hgroup>'
    assert convert_to_markdown(html2) == ""

    html3 = '<picture id="empty-pic"></picture>'
    assert convert_to_markdown(html3) == ""


def test_figure_with_pre_content() -> None:
    html = """<figure>
        <pre><code>function example() {
  return 42;
}</code></pre>
        <figcaption>Code example</figcaption>
    </figure>"""
    result = convert_to_markdown(html)
    expected = """```\nfunction example() {\n  return 42;\n}\n```\n\n*Code example*\n\n"""
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
def test_element_patterns(html: str, expected: str) -> None:
    result = convert_to_markdown(html)
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
def test_empty_elements(html: str, expected_empty: bool) -> None:
    result = convert_to_markdown(html)
    assert (result == "") == expected_empty


def test_blockquote_with_cite_in_list() -> None:
    html = """<ul>
        <li>
            Item with blockquote:
            <blockquote cite="https://example.com">Quote with citation</blockquote>
        </li>
    </ul>"""

    result = convert_to_markdown(html)
    assert "Item with blockquote:" in result
    assert "Quote with citation" in result
    assert "— <https://example.com>" in result


def test_mark_with_unsupported_highlight_style() -> None:
    html = "<mark>highlighted text</mark>"
    result = convert_to_markdown(html, highlight_style="unsupported")
    assert result == "highlighted text"


def test_empty_pre_element() -> None:
    html = "<pre></pre>"
    result = convert_to_markdown(html)
    assert result == ""


def test_media_element_without_src_but_with_text() -> None:
    html = "<video>Your browser doesn't support video</video>"
    result = convert_to_markdown(html)
    assert "Your browser doesn't support video" in result

    html = "<audio>Audio not supported</audio>"
    result = convert_to_markdown(html)
    assert "Audio not supported" in result


def test_paragraph_directly_in_list() -> None:
    html = """<ul>
        <p>Line 1\n\nLine 2</p>
    </ul>"""
    result = convert_to_markdown(html)
    assert "Line 1" in result
    assert "Line 2" in result


def test_paragraph_in_list_with_blank_lines() -> None:
    html = """<ul>
        <li>
            <p>First line\n\nSecond line\n\n\nThird line</p>
        </li>
    </ul>"""
    result = convert_to_markdown(html)
    assert "First line" in result
    assert "Second line" in result
    assert "Third line" in result


def test_blockquote_directly_under_list() -> None:
    html = """<ul>
        <blockquote>Quote\n\nWith blank lines</blockquote>
    </ul>"""
    result = convert_to_markdown(html)
    assert "Quote" in result


def test_blockquote_deeply_nested_in_li_needs_traversal() -> None:
    html = """<ul>
        <li>
            <div>
                <span>
                    <blockquote>Nested quote</blockquote>
                </span>
            </div>
        </li>
    </ul>"""
    result = convert_to_markdown(html)
    assert "> Nested quote" in result


def test_checkbox_with_string_content() -> None:
    html = '<ul><li><input type="checkbox">checkbox text content</input> List item text</li></ul>'
    result = convert_to_markdown(html)
    assert "[ ]" in result
    assert "List item text" in result


def test_paragraph_in_deeply_nested_li() -> None:
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
    result = convert_to_markdown(html)
    assert "First paragraph" in result
    assert "Second paragraph" in result


def test_paragraph_deeply_nested_needs_traversal() -> None:
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
    result = convert_to_markdown(html)
    assert "Deeply nested paragraph" in result


def test_blockquote_in_list_with_empty_lines() -> None:
    html = """<ul>
        <li>
            <blockquote>Line 1\n\nLine 2\n\n\nLine 3</blockquote>
        </li>
    </ul>"""
    result = convert_to_markdown(html)
    assert "Line 1" in result
    assert "Line 2" in result
    assert "Line 3" in result


def test_iframe_inline_mode() -> None:
    html = '<iframe src="https://example.com/embed"></iframe>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "[https://example.com/embed](https://example.com/embed)"


def test_time_element_empty() -> None:
    html = "<time></time>"
    result = convert_to_markdown(html)
    assert result == ""


def test_data_element_empty() -> None:
    html = "<data></data>"
    result = convert_to_markdown(html)
    assert result == ""


def test_optgroup_inline_mode() -> None:
    html = '<optgroup label="Group"><option>Option 1</option></optgroup>'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert "Option 1" in result


def test_optgroup_without_label() -> None:
    html = "<optgroup><option>Option 1</option><option>Option 2</option></optgroup>"
    result = convert_to_markdown(html)
    assert "Option 1" in result
    assert "Option 2" in result


def test_optgroup_empty() -> None:
    html = "<optgroup>  </optgroup>"
    result = convert_to_markdown(html)
    assert result == ""


def test_ruby_element_empty() -> None:
    html = "<ruby>  </ruby>"
    result = convert_to_markdown(html)
    assert result == ""


def test_rp_element_empty() -> None:
    html = "<rp>  </rp>"
    result = convert_to_markdown(html)
    assert result == ""


def test_rtc_element_empty() -> None:
    html = "<rtc>  </rtc>"
    result = convert_to_markdown(html)
    assert result == ""


def test_legend_inline_mode() -> None:
    html = "<legend>Form Legend</legend>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Form Legend"


def test_iframe_without_src() -> None:
    html = "<iframe></iframe>"
    result = convert_to_markdown(html)
    assert result == ""

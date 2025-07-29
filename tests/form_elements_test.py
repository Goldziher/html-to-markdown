"""Tests for form elements functionality."""

from html_to_markdown import convert_to_markdown


class TestFormElements:
    """Test form element conversion."""

    def test_form_basic(self) -> None:
        """Test basic form conversion."""
        html = "<form><p>Form content</p></form>"
        result = convert_to_markdown(html)
        # Forms are just containers, only their content is converted
        assert result == "Form content\n\n"

    def test_form_with_action(self) -> None:
        """Test form with action attribute."""
        html = '<form action="/submit"><p>Form content</p></form>'
        result = convert_to_markdown(html)
        # Form attributes are not preserved in Markdown
        assert result == "Form content\n\n"

    def test_form_with_method(self) -> None:
        """Test form with method attribute."""
        html = '<form method="post"><p>Form content</p></form>'
        result = convert_to_markdown(html)
        assert result == "Form content\n\n"

    def test_form_with_action_and_method(self) -> None:
        """Test form with both action and method attributes."""
        html = '<form action="/submit" method="post"><p>Form content</p></form>'
        result = convert_to_markdown(html)
        assert result == "Form content\n\n"

    def test_form_empty(self) -> None:
        """Test empty form."""
        html = "<form></form>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_form_inline_mode(self) -> None:
        """Test form in inline mode."""
        html = "<form>Form content</form>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Form content"


class TestFieldsetAndLegend:
    """Test fieldset and legend element conversion."""

    def test_fieldset_basic(self) -> None:
        """Test basic fieldset conversion."""
        html = "<fieldset><p>Fieldset content</p></fieldset>"
        result = convert_to_markdown(html)
        # Fieldsets are semantic groupings, convert content only
        assert result == "Fieldset content\n\n"

    def test_fieldset_with_legend(self) -> None:
        """Test fieldset with legend."""
        html = "<fieldset><legend>Form Section</legend><p>Content</p></fieldset>"
        result = convert_to_markdown(html)
        # Legend becomes a heading-like element
        assert result == "**Form Section**\n\nContent\n\n"

    def test_legend_standalone(self) -> None:
        """Test legend element standalone."""
        html = "<legend>Legend text</legend>"
        result = convert_to_markdown(html)
        # Legend as a title/heading
        assert result == "**Legend text**\n\n"

    def test_fieldset_empty(self) -> None:
        """Test empty fieldset."""
        html = "<fieldset></fieldset>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_legend_empty(self) -> None:
        """Test empty legend."""
        html = "<legend></legend>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_fieldset_inline_mode(self) -> None:
        """Test fieldset in inline mode."""
        html = "<fieldset>Inline content</fieldset>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline content"


class TestLabelElement:
    """Test label element conversion."""

    def test_label_basic(self) -> None:
        """Test basic label conversion."""
        html = "<label>Label text</label>"
        result = convert_to_markdown(html)
        # Labels convert to their text content
        assert result == "Label text\n\n"

    def test_label_with_for(self) -> None:
        """Test label with for attribute."""
        html = '<label for="username">Username</label>'
        result = convert_to_markdown(html)
        # For attribute is not preserved in Markdown
        assert result == "Username\n\n"

    def test_label_with_input(self) -> None:
        """Test label containing input."""
        html = '<label>Username: <input type="text" name="username"></label>'
        result = convert_to_markdown(html)
        # Input is removed, only label text remains
        assert result == "Username:\n\n"

    def test_label_empty(self) -> None:
        """Test empty label."""
        html = "<label></label>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_label_inline_mode(self) -> None:
        """Test label in inline mode."""
        html = "<label>Inline label</label>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline label"


class TestInputElement:
    """Test input element conversion."""

    def test_input_text(self) -> None:
        """Test text input."""
        html = '<input type="text" name="username">'
        result = convert_to_markdown(html)
        # Input elements have no content and no Markdown equivalent
        assert result == ""

    def test_input_password(self) -> None:
        """Test password input."""
        html = '<input type="password" name="password">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_with_value(self) -> None:
        """Test input with value."""
        html = '<input type="text" name="username" value="john">'
        result = convert_to_markdown(html)
        # Even with value, inputs are not shown in Markdown
        assert result == ""

    def test_input_with_placeholder(self) -> None:
        """Test input with placeholder."""
        html = '<input type="text" name="username" placeholder="Enter username">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_required(self) -> None:
        """Test required input."""
        html = '<input type="text" name="username" required>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_disabled(self) -> None:
        """Test disabled input."""
        html = '<input type="text" name="username" disabled>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_readonly(self) -> None:
        """Test readonly input."""
        html = '<input type="text" name="username" readonly>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_checkbox_unchecked(self) -> None:
        """Test unchecked checkbox input."""
        html = '<input type="checkbox" name="agree">'
        result = convert_to_markdown(html)
        # Checkboxes outside lists are removed
        assert result == ""

    def test_input_checkbox_checked(self) -> None:
        """Test checked checkbox input."""
        html = '<input type="checkbox" name="agree" checked>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_radio(self) -> None:
        """Test radio input."""
        html = '<input type="radio" name="gender" value="male">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_submit(self) -> None:
        """Test submit input."""
        html = '<input type="submit" value="Submit">'
        result = convert_to_markdown(html)
        # Submit buttons could show their value, but we treat all inputs consistently
        assert result == ""

    def test_input_file(self) -> None:
        """Test file input."""
        html = '<input type="file" name="upload" accept=".jpg,.png">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_inline_mode(self) -> None:
        """Test input in inline mode."""
        html = '<input type="text" name="username">'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == ""


class TestTextareaElement:
    """Test textarea element conversion."""

    def test_textarea_basic(self) -> None:
        """Test basic textarea conversion."""
        html = "<textarea>Default text</textarea>"
        result = convert_to_markdown(html)
        # Textarea converts to its text content
        assert result == "Default text\n\n"

    def test_textarea_with_name(self) -> None:
        """Test textarea with name attribute."""
        html = '<textarea name="comment">Comment text</textarea>'
        result = convert_to_markdown(html)
        assert result == "Comment text\n\n"

    def test_textarea_with_placeholder(self) -> None:
        """Test textarea with placeholder."""
        html = '<textarea placeholder="Enter your comment">Default text</textarea>'
        result = convert_to_markdown(html)
        # Placeholder is not shown in Markdown
        assert result == "Default text\n\n"

    def test_textarea_with_rows_cols(self) -> None:
        """Test textarea with rows and cols."""
        html = '<textarea rows="5" cols="30">Text</textarea>'
        result = convert_to_markdown(html)
        # Rows and cols are not preserved in Markdown
        assert result == "Text\n\n"

    def test_textarea_required(self) -> None:
        """Test required textarea."""
        html = "<textarea required>Required text</textarea>"
        result = convert_to_markdown(html)
        # Required attribute is not preserved in Markdown
        assert result == "Required text\n\n"

    def test_textarea_empty(self) -> None:
        """Test empty textarea."""
        html = "<textarea></textarea>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_textarea_inline_mode(self) -> None:
        """Test textarea in inline mode."""
        html = "<textarea>Inline text</textarea>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline text"


class TestSelectAndOptionElements:
    """Test select, option, and optgroup element conversion."""

    def test_select_basic(self) -> None:
        """Test basic select conversion."""
        html = "<select><option>Option 1</option><option>Option 2</option></select>"
        result = convert_to_markdown(html)
        # Select shows options as a list
        assert result == "Option 1\nOption 2\n\n"

    def test_select_with_name(self) -> None:
        """Test select with name attribute."""
        html = '<select name="country"><option>USA</option><option>Canada</option></select>'
        result = convert_to_markdown(html)
        # Name attribute is not preserved
        assert result == "USA\nCanada\n\n"

    def test_select_multiple(self) -> None:
        """Test multiple select."""
        html = "<select multiple><option>Option 1</option><option>Option 2</option></select>"
        result = convert_to_markdown(html)
        # Multiple attribute is not preserved
        assert result == "Option 1\nOption 2\n\n"

    def test_option_with_value(self) -> None:
        """Test option with value attribute."""
        html = '<select><option value="us">United States</option><option value="ca">Canada</option></select>'
        result = convert_to_markdown(html)
        # Value attributes are not preserved
        assert result == "United States\nCanada\n\n"

    def test_option_selected(self) -> None:
        """Test selected option."""
        html = "<select><option>Option 1</option><option selected>Option 2</option></select>"
        result = convert_to_markdown(html)
        # Selected option is marked with *
        assert result == "Option 1\n* Option 2\n\n"

    def test_optgroup(self) -> None:
        """Test optgroup element."""
        html = (
            '<select><optgroup label="Group 1"><option>Option 1</option><option>Option 2</option></optgroup></select>'
        )
        result = convert_to_markdown(html)
        # Optgroup label becomes a heading
        assert result == "**Group 1**\nOption 1\nOption 2\n\n"

    def test_select_empty(self) -> None:
        """Test empty select."""
        html = "<select></select>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_option_empty(self) -> None:
        """Test empty option."""
        html = "<select><option></option></select>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_select_inline_mode(self) -> None:
        """Test select in inline mode."""
        html = "<select><option>Option</option></select>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Option"


class TestButtonElement:
    """Test button element conversion."""

    def test_button_basic(self) -> None:
        """Test basic button conversion."""
        html = "<button>Click me</button>"
        result = convert_to_markdown(html)
        # Buttons convert to their text content
        assert result == "Click me\n\n"

    def test_button_with_type(self) -> None:
        """Test button with type attribute."""
        html = '<button type="submit">Submit</button>'
        result = convert_to_markdown(html)
        # Type attribute is not preserved
        assert result == "Submit\n\n"

    def test_button_disabled(self) -> None:
        """Test disabled button."""
        html = "<button disabled>Disabled</button>"
        result = convert_to_markdown(html)
        # Disabled attribute is not preserved
        assert result == "Disabled\n\n"

    def test_button_with_name_value(self) -> None:
        """Test button with name and value."""
        html = '<button name="action" value="delete">Delete</button>'
        result = convert_to_markdown(html)
        # Name and value attributes are not preserved
        assert result == "Delete\n\n"

    def test_button_empty(self) -> None:
        """Test empty button."""
        html = "<button></button>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_button_inline_mode(self) -> None:
        """Test button in inline mode."""
        html = "<button>Inline button</button>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline button"


class TestProgressAndMeterElements:
    """Test progress and meter element conversion."""

    def test_progress_basic(self) -> None:
        """Test basic progress conversion."""
        html = "<progress>50%</progress>"
        result = convert_to_markdown(html)
        # Progress converts to its text content
        assert result == "50%\n\n"

    def test_progress_with_value_max(self) -> None:
        """Test progress with value and max."""
        html = '<progress value="50" max="100">50%</progress>'
        result = convert_to_markdown(html)
        # Attributes are not preserved
        assert result == "50%\n\n"

    def test_meter_basic(self) -> None:
        """Test basic meter conversion."""
        html = "<meter>6 out of 10</meter>"
        result = convert_to_markdown(html)
        # Meter converts to its text content
        assert result == "6 out of 10\n\n"

    def test_meter_with_attributes(self) -> None:
        """Test meter with attributes."""
        html = '<meter value="6" min="0" max="10" low="2" high="8" optimum="5">6 out of 10</meter>'
        result = convert_to_markdown(html)
        # Attributes are not preserved
        assert result == "6 out of 10\n\n"

    def test_progress_empty(self) -> None:
        """Test empty progress."""
        html = "<progress></progress>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_meter_empty(self) -> None:
        """Test empty meter."""
        html = "<meter></meter>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_progress_inline_mode(self) -> None:
        """Test progress in inline mode."""
        html = "<progress>50%</progress>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "50%"

    def test_meter_inline_mode(self) -> None:
        """Test meter in inline mode."""
        html = "<meter>6/10</meter>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "6/10"


class TestOutputAndDatalistElements:
    """Test output and datalist element conversion."""

    def test_output_basic(self) -> None:
        """Test basic output conversion."""
        html = "<output>Result: 42</output>"
        result = convert_to_markdown(html)
        # Output converts to its text content
        assert result == "Result: 42\n\n"

    def test_output_with_for(self) -> None:
        """Test output with for attribute."""
        html = '<output for="input1 input2">Sum: 15</output>'
        result = convert_to_markdown(html)
        # For attribute is not preserved
        assert result == "Sum: 15\n\n"

    def test_output_with_name(self) -> None:
        """Test output with name attribute."""
        html = '<output name="result">42</output>'
        result = convert_to_markdown(html)
        # Name attribute is not preserved
        assert result == "42\n\n"

    def test_datalist_basic(self) -> None:
        """Test basic datalist conversion."""
        html = "<datalist><option>Option 1</option><option>Option 2</option></datalist>"
        result = convert_to_markdown(html)
        # Datalist shows options as a list
        assert result == "Option 1\nOption 2\n\n"

    def test_datalist_with_id(self) -> None:
        """Test datalist with id attribute."""
        html = '<datalist id="browsers"><option>Chrome</option><option>Firefox</option></datalist>'
        result = convert_to_markdown(html)
        # ID attribute is not preserved
        assert result == "Chrome\nFirefox\n\n"

    def test_output_empty(self) -> None:
        """Test empty output."""
        html = "<output></output>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_datalist_empty(self) -> None:
        """Test empty datalist."""
        html = "<datalist></datalist>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_output_inline_mode(self) -> None:
        """Test output in inline mode."""
        html = "<output>Result</output>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Result"

    def test_datalist_inline_mode(self) -> None:
        """Test datalist in inline mode."""
        html = "<datalist><option>Option</option></datalist>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Option"


class TestComplexFormExamples:
    """Test complex form examples."""

    def test_complete_form_example(self) -> None:
        """Test complete form with various elements."""
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
        # Expected output: pure Markdown with no HTML tags
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

    def test_form_with_progress_and_meter(self) -> None:
        """Test form with progress and meter elements."""
        html = """<form>
            <label>Upload Progress:</label>
            <progress value="75" max="100">75%</progress>
            <label>Rating:</label>
            <meter value="4" min="1" max="5">4 out of 5</meter>
            <output for="rating">Current rating: 4/5</output>
        </form>"""
        result = convert_to_markdown(html)
        # Expected output: pure Markdown with no HTML tags
        expected = """Upload Progress:

75%

Rating:

4 out of 5

Current rating: 4/5

"""
        assert result == expected

    def test_form_inline_mode(self) -> None:
        """Test form elements in inline mode."""
        html = '<form><label>Name:</label> <input type="text" name="name"> <button>Submit</button></form>'
        result = convert_to_markdown(html, convert_as_inline=True)
        # Input elements have no Markdown representation and are removed
        assert result == "Name:  Submit"

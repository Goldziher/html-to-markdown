from html_to_markdown import convert_to_markdown


class TestFormElements:
    """Test form element conversion."""

    def test_form_basic(self) -> None:
        """Test basic form conversion."""
        html = "<form><p>Form content</p></form>"
        result = convert_to_markdown(html)
        assert result == "Form content\n\n"

    def test_form_with_action(self) -> None:
        html = '<form action="/submit"><p>Form content</p></form>'
        result = convert_to_markdown(html)
        assert result == "Form content\n\n"

    def test_form_with_method(self) -> None:
        html = '<form method="post"><p>Form content</p></form>'
        result = convert_to_markdown(html)
        assert result == "Form content\n\n"

    def test_form_with_action_and_method(self) -> None:
        html = '<form action="/submit" method="post"><p>Form content</p></form>'
        result = convert_to_markdown(html)
        assert result == "Form content\n\n"

    def test_form_empty(self) -> None:
        html = "<form></form>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_form_inline_mode(self) -> None:
        html = "<form>Form content</form>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Form content"


class TestFieldsetAndLegend:
    def test_fieldset_basic(self) -> None:
        html = "<fieldset><p>Fieldset content</p></fieldset>"
        result = convert_to_markdown(html)
        assert result == "Fieldset content\n\n"

    def test_fieldset_with_legend(self) -> None:
        html = "<fieldset><legend>Form Section</legend><p>Content</p></fieldset>"
        result = convert_to_markdown(html)
        assert result == "**Form Section**\n\nContent\n\n"

    def test_legend_standalone(self) -> None:
        html = "<legend>Legend text</legend>"
        result = convert_to_markdown(html)
        assert result == "**Legend text**\n\n"

    def test_fieldset_empty(self) -> None:
        html = "<fieldset></fieldset>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_legend_empty(self) -> None:
        html = "<legend></legend>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_fieldset_inline_mode(self) -> None:
        html = "<fieldset>Inline content</fieldset>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline content"


class TestLabelElement:
    def test_label_basic(self) -> None:
        html = "<label>Label text</label>"
        result = convert_to_markdown(html)
        assert result == "Label text\n\n"

    def test_label_with_for(self) -> None:
        html = '<label for="username">Username</label>'
        result = convert_to_markdown(html)
        assert result == "Username\n\n"

    def test_label_with_input(self) -> None:
        html = '<label>Username: <input type="text" name="username"></label>'
        result = convert_to_markdown(html)
        assert result == "Username:\n\n"

    def test_label_empty(self) -> None:
        html = "<label></label>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_label_inline_mode(self) -> None:
        html = "<label>Inline label</label>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline label"


class TestInputElement:
    def test_input_text(self) -> None:
        html = '<input type="text" name="username">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_password(self) -> None:
        html = '<input type="password" name="password">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_with_value(self) -> None:
        html = '<input type="text" name="username" value="john">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_with_placeholder(self) -> None:
        html = '<input type="text" name="username" placeholder="Enter username">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_required(self) -> None:
        html = '<input type="text" name="username" required>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_disabled(self) -> None:
        html = '<input type="text" name="username" disabled>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_readonly(self) -> None:
        html = '<input type="text" name="username" readonly>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_checkbox_unchecked(self) -> None:
        html = '<input type="checkbox" name="agree">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_checkbox_checked(self) -> None:
        html = '<input type="checkbox" name="agree" checked>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_radio(self) -> None:
        html = '<input type="radio" name="gender" value="male">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_submit(self) -> None:
        html = '<input type="submit" value="Submit">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_file(self) -> None:
        html = '<input type="file" name="upload" accept=".jpg,.png">'
        result = convert_to_markdown(html)
        assert result == ""

    def test_input_inline_mode(self) -> None:
        html = '<input type="text" name="username">'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == ""


class TestTextareaElement:
    def test_textarea_basic(self) -> None:
        html = "<textarea>Default text</textarea>"
        result = convert_to_markdown(html)
        assert result == "Default text\n\n"

    def test_textarea_with_name(self) -> None:
        html = '<textarea name="comment">Comment text</textarea>'
        result = convert_to_markdown(html)
        assert result == "Comment text\n\n"

    def test_textarea_with_placeholder(self) -> None:
        html = '<textarea placeholder="Enter your comment">Default text</textarea>'
        result = convert_to_markdown(html)
        assert result == "Default text\n\n"

    def test_textarea_with_rows_cols(self) -> None:
        html = '<textarea rows="5" cols="30">Text</textarea>'
        result = convert_to_markdown(html)
        assert result == "Text\n\n"

    def test_textarea_required(self) -> None:
        html = "<textarea required>Required text</textarea>"
        result = convert_to_markdown(html)
        assert result == "Required text\n\n"

    def test_textarea_empty(self) -> None:
        html = "<textarea></textarea>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_textarea_inline_mode(self) -> None:
        html = "<textarea>Inline text</textarea>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline text"


class TestSelectAndOptionElements:
    def test_select_basic(self) -> None:
        html = "<select><option>Option 1</option><option>Option 2</option></select>"
        result = convert_to_markdown(html)
        assert result == "Option 1\nOption 2\n\n"

    def test_select_with_name(self) -> None:
        html = '<select name="country"><option>USA</option><option>Canada</option></select>'
        result = convert_to_markdown(html)
        assert result == "USA\nCanada\n\n"

    def test_select_multiple(self) -> None:
        html = "<select multiple><option>Option 1</option><option>Option 2</option></select>"
        result = convert_to_markdown(html)
        assert result == "Option 1\nOption 2\n\n"

    def test_option_with_value(self) -> None:
        html = '<select><option value="us">United States</option><option value="ca">Canada</option></select>'
        result = convert_to_markdown(html)
        assert result == "United States\nCanada\n\n"

    def test_option_selected(self) -> None:
        html = "<select><option>Option 1</option><option selected>Option 2</option></select>"
        result = convert_to_markdown(html)
        assert result == "Option 1\n* Option 2\n\n"

    def test_optgroup(self) -> None:
        html = (
            '<select><optgroup label="Group 1"><option>Option 1</option><option>Option 2</option></optgroup></select>'
        )
        result = convert_to_markdown(html)
        assert result == "**Group 1**\nOption 1\nOption 2\n\n"

    def test_select_empty(self) -> None:
        html = "<select></select>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_option_empty(self) -> None:
        html = "<select><option></option></select>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_select_inline_mode(self) -> None:
        html = "<select><option>Option</option></select>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Option"


class TestButtonElement:
    def test_button_basic(self) -> None:
        html = "<button>Click me</button>"
        result = convert_to_markdown(html)
        assert result == "Click me\n\n"

    def test_button_with_type(self) -> None:
        html = '<button type="submit">Submit</button>'
        result = convert_to_markdown(html)
        assert result == "Submit\n\n"

    def test_button_disabled(self) -> None:
        html = "<button disabled>Disabled</button>"
        result = convert_to_markdown(html)
        assert result == "Disabled\n\n"

    def test_button_with_name_value(self) -> None:
        html = '<button name="action" value="delete">Delete</button>'
        result = convert_to_markdown(html)
        assert result == "Delete\n\n"

    def test_button_empty(self) -> None:
        html = "<button></button>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_button_inline_mode(self) -> None:
        html = "<button>Inline button</button>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline button"


class TestProgressAndMeterElements:
    def test_progress_basic(self) -> None:
        html = "<progress>50%</progress>"
        result = convert_to_markdown(html)
        assert result == "50%\n\n"

    def test_progress_with_value_max(self) -> None:
        html = '<progress value="50" max="100">50%</progress>'
        result = convert_to_markdown(html)
        assert result == "50%\n\n"

    def test_meter_basic(self) -> None:
        html = "<meter>6 out of 10</meter>"
        result = convert_to_markdown(html)
        assert result == "6 out of 10\n\n"

    def test_meter_with_attributes(self) -> None:
        html = '<meter value="6" min="0" max="10" low="2" high="8" optimum="5">6 out of 10</meter>'
        result = convert_to_markdown(html)
        assert result == "6 out of 10\n\n"

    def test_progress_empty(self) -> None:
        html = "<progress></progress>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_meter_empty(self) -> None:
        html = "<meter></meter>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_progress_inline_mode(self) -> None:
        html = "<progress>50%</progress>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "50%"

    def test_meter_inline_mode(self) -> None:
        html = "<meter>6/10</meter>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "6/10"


class TestOutputAndDatalistElements:
    def test_output_basic(self) -> None:
        html = "<output>Result: 42</output>"
        result = convert_to_markdown(html)
        assert result == "Result: 42\n\n"

    def test_output_with_for(self) -> None:
        html = '<output for="input1 input2">Sum: 15</output>'
        result = convert_to_markdown(html)
        assert result == "Sum: 15\n\n"

    def test_output_with_name(self) -> None:
        html = '<output name="result">42</output>'
        result = convert_to_markdown(html)
        assert result == "42\n\n"

    def test_datalist_basic(self) -> None:
        html = "<datalist><option>Option 1</option><option>Option 2</option></datalist>"
        result = convert_to_markdown(html)
        assert result == "Option 1\nOption 2\n\n"

    def test_datalist_with_id(self) -> None:
        html = '<datalist id="browsers"><option>Chrome</option><option>Firefox</option></datalist>'
        result = convert_to_markdown(html)
        assert result == "Chrome\nFirefox\n\n"

    def test_output_empty(self) -> None:
        html = "<output></output>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_datalist_empty(self) -> None:
        html = "<datalist></datalist>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_output_inline_mode(self) -> None:
        html = "<output>Result</output>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Result"

    def test_datalist_inline_mode(self) -> None:
        html = "<datalist><option>Option</option></datalist>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Option"


class TestComplexFormExamples:
    def test_complete_form_example(self) -> None:
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

    def test_form_with_progress_and_meter(self) -> None:
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

    def test_form_inline_mode(self) -> None:
        html = '<form><label>Name:</label> <input type="text" name="name"> <button>Submit</button></form>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Name:  Submit"

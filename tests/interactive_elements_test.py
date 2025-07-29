"""Tests for interactive HTML elements (dialog, menu)."""

from html_to_markdown import convert_to_markdown


class TestDialogElement:
    """Test dialog element conversion."""

    def test_dialog_basic(self) -> None:
        """Test basic dialog conversion."""
        html = "<dialog>Simple dialog content</dialog>"
        result = convert_to_markdown(html)
        # Dialog is a semantic container, only content is converted
        assert result == "Simple dialog content\n\n"

    def test_dialog_open(self) -> None:
        """Test dialog with open attribute."""
        html = "<dialog open>This dialog is open</dialog>"
        result = convert_to_markdown(html)
        # Open attribute is not preserved in Markdown
        assert result == "This dialog is open\n\n"

    def test_dialog_with_id(self) -> None:
        """Test dialog with id attribute."""
        html = '<dialog id="myDialog">Dialog with ID</dialog>'
        result = convert_to_markdown(html)
        # ID attribute is not preserved in Markdown
        assert result == "Dialog with ID\n\n"

    def test_dialog_open_with_id(self) -> None:
        """Test dialog with both open and id attributes."""
        html = '<dialog open id="openDialog">Open dialog with ID</dialog>'
        result = convert_to_markdown(html)
        # Attributes are not preserved in Markdown
        assert result == "Open dialog with ID\n\n"

    def test_dialog_empty(self) -> None:
        """Test empty dialog."""
        html = "<dialog></dialog>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_dialog_whitespace_only(self) -> None:
        """Test dialog with only whitespace."""
        html = "<dialog>   \n  \t  </dialog>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_dialog_inline_mode(self) -> None:
        """Test dialog in inline mode."""
        html = "<dialog>Inline dialog content</dialog>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline dialog content"

    def test_dialog_with_nested_elements(self) -> None:
        """Test dialog with nested HTML elements."""
        html = "<dialog><h2>Dialog Title</h2><p>Dialog content with <strong>bold</strong> text.</p></dialog>"
        result = convert_to_markdown(html)
        expected = """Dialog Title\n------------\n\nDialog content with **bold** text.\n\n"""
        assert result == expected

    def test_dialog_multiline_content(self) -> None:
        """Test dialog with multiline content."""
        html = """<dialog>
            <p>First paragraph</p>
            <p>Second paragraph</p>
        </dialog>"""
        result = convert_to_markdown(html)
        expected = """First paragraph\n\nSecond paragraph\n\n"""
        assert result == expected

    def test_dialog_with_buttons(self) -> None:
        """Test dialog with form elements."""
        html = """<dialog>
            <p>Are you sure?</p>
            <button>Yes</button>
            <button>No</button>
        </dialog>"""
        result = convert_to_markdown(html)
        expected = """Are you sure?\n\nYes\n\nNo\n\n"""
        assert result == expected


class TestMenuElement:
    """Test menu element conversion."""

    def test_menu_basic(self) -> None:
        """Test basic menu conversion."""
        html = "<menu><li>Item 1</li><li>Item 2</li></menu>"
        result = convert_to_markdown(html)
        # Menu is converted as a list
        assert result == "- Item 1\n- Item 2\n\n"

    def test_menu_toolbar(self) -> None:
        """Test menu with toolbar type."""
        html = '<menu type="toolbar"><li>Cut</li><li>Copy</li><li>Paste</li></menu>'
        result = convert_to_markdown(html)
        # Type attribute is not preserved in Markdown
        assert result == "- Cut\n- Copy\n- Paste\n\n"

    def test_menu_context(self) -> None:
        """Test menu with context type."""
        html = '<menu type="context"><li>Delete</li><li>Rename</li></menu>'
        result = convert_to_markdown(html)
        # Type attribute is not preserved in Markdown
        assert result == "- Delete\n- Rename\n\n"

    def test_menu_with_label(self) -> None:
        """Test menu with label attribute."""
        html = '<menu label="File Operations"><li>Open</li><li>Save</li></menu>'
        result = convert_to_markdown(html)
        # Label attribute is not preserved in Markdown
        assert result == "- Open\n- Save\n\n"

    def test_menu_toolbar_with_label(self) -> None:
        """Test toolbar menu with label."""
        html = '<menu type="toolbar" label="Edit Tools"><li>Bold</li><li>Italic</li></menu>'
        result = convert_to_markdown(html)
        # Attributes are not preserved in Markdown
        assert result == "- Bold\n- Italic\n\n"

    def test_menu_with_id(self) -> None:
        """Test menu with id attribute."""
        html = '<menu id="mainMenu"><li>Home</li><li>About</li></menu>'
        result = convert_to_markdown(html)
        # ID attribute is not preserved in Markdown
        assert result == "- Home\n- About\n\n"

    def test_menu_all_attributes(self) -> None:
        """Test menu with all supported attributes."""
        html = '<menu type="context" label="Context Actions" id="contextMenu"><li>Edit</li><li>Delete</li></menu>'
        result = convert_to_markdown(html)
        # All attributes are not preserved in Markdown
        assert result == "- Edit\n- Delete\n\n"

    def test_menu_type_list_omitted(self) -> None:
        """Test that type='list' is omitted from output."""
        html = '<menu type="list"><li>Item 1</li><li>Item 2</li></menu>'
        result = convert_to_markdown(html)
        # Type attribute is not preserved in Markdown
        assert result == "- Item 1\n- Item 2\n\n"

    def test_menu_empty(self) -> None:
        """Test empty menu."""
        html = "<menu></menu>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_menu_whitespace_only(self) -> None:
        """Test menu with only whitespace."""
        html = "<menu>   \n  \t  </menu>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_menu_inline_mode(self) -> None:
        """Test menu in inline mode."""
        html = "<menu><li>Inline item</li></menu>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "- Inline item"

    def test_menu_with_nested_elements(self) -> None:
        """Test menu with nested HTML elements."""
        html = "<menu><li><strong>Bold Item</strong></li><li><em>Italic Item</em></li></menu>"
        result = convert_to_markdown(html)
        expected = "- **Bold Item**\n- *Italic Item*\n\n"
        assert result == expected

    def test_menu_with_buttons(self) -> None:
        """Test menu with button elements."""
        html = """<menu type="toolbar">
            <button>New</button>
            <button>Open</button>
            <button>Save</button>
        </menu>"""
        result = convert_to_markdown(html)
        # Buttons convert to their text content
        expected = """New\n\nOpen\n\nSave\n\n"""
        assert result == expected

    def test_menu_mixed_content(self) -> None:
        """Test menu with mixed content types."""
        html = """<menu>
            <li>List item</li>
            <button>Button item</button>
            <li>Another list item</li>
        </menu>"""
        result = convert_to_markdown(html)
        expected = """- List item\nButton item\n\n- Another list item\n\n"""
        assert result == expected


class TestInteractiveElementsIntegration:
    """Test interactive elements in various contexts."""

    def test_dialog_in_paragraph(self) -> None:
        """Test dialog nested in paragraph (should not happen but test anyway)."""
        html = "<p>Click here: <dialog>Modal content</dialog> to see dialog.</p>"
        result = convert_to_markdown(html)
        expected = "Click here: Modal content\n\n to see dialog.\n\n"
        assert result == expected

    def test_menu_in_navigation(self) -> None:
        """Test menu inside navigation element."""
        html = """<nav>
            <menu>
                <li><a href="/home">Home</a></li>
                <li><a href="/about">About</a></li>
            </menu>
        </nav>"""
        result = convert_to_markdown(html)
        expected = "- [Home](/home)\n- [About](/about)\n\n"
        assert result == expected

    def test_nested_interactive_elements(self) -> None:
        """Test interactive elements nested in other structures."""
        html = """<div>
            <details>
                <summary>Show Menu</summary>
                <menu>
                    <li>Option 1</li>
                    <li>Option 2</li>
                </menu>
            </details>
        </div>"""
        result = convert_to_markdown(html)
        # Details/summary have no Markdown equivalent, convert to semantic content
        expected = """**Show Menu**\n\n- Option 1\n- Option 2\n\n"""
        assert result == expected

    def test_dialog_with_form(self) -> None:
        """Test dialog containing form elements."""
        html = """<dialog open>
            <form>
                <label>Name: <input type="text" name="name"></label>
                <button type="submit">Submit</button>
            </form>
        </dialog>"""
        result = convert_to_markdown(html)
        expected = """Name:\n\nSubmit\n\n"""
        assert result == expected

    def test_multiple_dialogs(self) -> None:
        """Test multiple dialog elements."""
        html = """
        <dialog id="dialog1">First dialog</dialog>
        <dialog id="dialog2" open>Second dialog</dialog>
        """
        result = convert_to_markdown(html)
        expected = """\n        First dialog\n\nSecond dialog\n\n"""
        assert result == expected

    def test_menu_with_submenus(self) -> None:
        """Test menu with nested menu structures."""
        html = """<menu>
            <li>File
                <menu>
                    <li>New</li>
                    <li>Open</li>
                </menu>
            </li>
            <li>Edit</li>
        </menu>"""
        result = convert_to_markdown(html)
        # Nested menus are flattened in the current implementation
        expected = """- File - New\n- Open\n- Edit\n\n"""
        assert result == expected


class TestInteractiveElementsEdgeCases:
    """Test edge cases for interactive elements."""

    def test_dialog_with_special_characters(self) -> None:
        """Test dialog with special Markdown characters."""
        html = "<dialog>This has *asterisks* and _underscores_ and [brackets]</dialog>"
        result = convert_to_markdown(html)
        expected = "This has \\*asterisks\\* and \\_underscores\\_ and \\[brackets]\n\n"
        assert result == expected

    def test_menu_with_special_characters(self) -> None:
        """Test menu with special Markdown characters."""
        html = "<menu><li>Item with *bold* text</li><li>Item with _italic_ text</li></menu>"
        result = convert_to_markdown(html)
        expected = "- Item with \\*bold\\* text\n- Item with \\_italic\\_ text\n\n"
        assert result == expected

    def test_dialog_attribute_values_with_quotes(self) -> None:
        """Test dialog with attribute values containing quotes."""
        html = '<dialog id="my-dialog" class="special">Content</dialog>'
        result = convert_to_markdown(html)
        # Attributes are not preserved in Markdown
        assert result == "Content\n\n"

    def test_menu_with_complex_attributes(self) -> None:
        """Test menu with complex attribute combinations."""
        html = '<menu type="toolbar" label="Tools &amp; Options" id="toolbar-1"><li>Cut</li></menu>'
        result = convert_to_markdown(html)
        # Attributes are not preserved, only content matters
        assert result == "- Cut\n\n"

    def test_empty_dialog_with_attributes(self) -> None:
        """Test empty dialog with attributes."""
        html = '<dialog open id="empty"></dialog>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_empty_menu_with_attributes(self) -> None:
        """Test empty menu with attributes."""
        html = '<menu type="toolbar" label="Empty"></menu>'
        result = convert_to_markdown(html)
        assert result == ""

"""Tests for interactive HTML elements (dialog, menu)."""

from html_to_markdown import convert_to_markdown


class TestDialogElement:
    """Test dialog element conversion."""

    def test_dialog_basic(self) -> None:
        """Test basic dialog conversion."""
        html = "<dialog>Simple dialog content</dialog>"
        result = convert_to_markdown(html)
        assert result == "<dialog>\nSimple dialog content\n</dialog>\n\n"

    def test_dialog_open(self) -> None:
        """Test dialog with open attribute."""
        html = "<dialog open>This dialog is open</dialog>"
        result = convert_to_markdown(html)
        assert result == "<dialog open>\nThis dialog is open\n</dialog>\n\n"

    def test_dialog_with_id(self) -> None:
        """Test dialog with id attribute."""
        html = '<dialog id="myDialog">Dialog with ID</dialog>'
        result = convert_to_markdown(html)
        assert result == '<dialog id="myDialog">\nDialog with ID\n</dialog>\n\n'

    def test_dialog_open_with_id(self) -> None:
        """Test dialog with both open and id attributes."""
        html = '<dialog open id="openDialog">Open dialog with ID</dialog>'
        result = convert_to_markdown(html)
        assert result == '<dialog open id="openDialog">\nOpen dialog with ID\n</dialog>\n\n'

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
        assert "Dialog Title" in result
        assert "**bold**" in result
        assert "<dialog>" in result
        assert "</dialog>" in result

    def test_dialog_multiline_content(self) -> None:
        """Test dialog with multiline content."""
        html = """<dialog>
            <p>First paragraph</p>
            <p>Second paragraph</p>
        </dialog>"""
        result = convert_to_markdown(html)
        assert "<dialog>" in result
        assert "First paragraph" in result
        assert "Second paragraph" in result

    def test_dialog_with_buttons(self) -> None:
        """Test dialog with form elements."""
        html = """<dialog>
            <p>Are you sure?</p>
            <button>Yes</button>
            <button>No</button>
        </dialog>"""
        result = convert_to_markdown(html)
        assert "<dialog>" in result
        assert "Are you sure?" in result
        assert "<button>Yes</button>" in result


class TestMenuElement:
    """Test menu element conversion."""

    def test_menu_basic(self) -> None:
        """Test basic menu conversion."""
        html = "<menu><li>Item 1</li><li>Item 2</li></menu>"
        result = convert_to_markdown(html)
        assert result == "<menu>\n- Item 1\n- Item 2\n</menu>\n\n"

    def test_menu_toolbar(self) -> None:
        """Test menu with toolbar type."""
        html = '<menu type="toolbar"><li>Cut</li><li>Copy</li><li>Paste</li></menu>'
        result = convert_to_markdown(html)
        assert result == '<menu type="toolbar">\n- Cut\n- Copy\n- Paste\n</menu>\n\n'

    def test_menu_context(self) -> None:
        """Test menu with context type."""
        html = '<menu type="context"><li>Delete</li><li>Rename</li></menu>'
        result = convert_to_markdown(html)
        assert result == '<menu type="context">\n- Delete\n- Rename\n</menu>\n\n'

    def test_menu_with_label(self) -> None:
        """Test menu with label attribute."""
        html = '<menu label="File Operations"><li>Open</li><li>Save</li></menu>'
        result = convert_to_markdown(html)
        assert result == '<menu label="File Operations">\n- Open\n- Save\n</menu>\n\n'

    def test_menu_toolbar_with_label(self) -> None:
        """Test toolbar menu with label."""
        html = '<menu type="toolbar" label="Edit Tools"><li>Bold</li><li>Italic</li></menu>'
        result = convert_to_markdown(html)
        assert result == '<menu type="toolbar" label="Edit Tools">\n- Bold\n- Italic\n</menu>\n\n'

    def test_menu_with_id(self) -> None:
        """Test menu with id attribute."""
        html = '<menu id="mainMenu"><li>Home</li><li>About</li></menu>'
        result = convert_to_markdown(html)
        assert result == '<menu id="mainMenu">\n- Home\n- About\n</menu>\n\n'

    def test_menu_all_attributes(self) -> None:
        """Test menu with all supported attributes."""
        html = '<menu type="context" label="Context Actions" id="contextMenu"><li>Edit</li><li>Delete</li></menu>'
        result = convert_to_markdown(html)
        assert result == '<menu type="context" label="Context Actions" id="contextMenu">\n- Edit\n- Delete\n</menu>\n\n'

    def test_menu_type_list_omitted(self) -> None:
        """Test that type='list' is omitted from output."""
        html = '<menu type="list"><li>Item 1</li><li>Item 2</li></menu>'
        result = convert_to_markdown(html)
        assert result == "<menu>\n- Item 1\n- Item 2\n</menu>\n\n"

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
        assert "**Bold Item**" in result
        assert "*Italic Item*" in result
        assert "<menu>" in result

    def test_menu_with_buttons(self) -> None:
        """Test menu with button elements."""
        html = """<menu type="toolbar">
            <button>New</button>
            <button>Open</button>
            <button>Save</button>
        </menu>"""
        result = convert_to_markdown(html)
        assert '<menu type="toolbar">' in result
        assert "<button>New</button>" in result
        assert "<button>Open</button>" in result

    def test_menu_mixed_content(self) -> None:
        """Test menu with mixed content types."""
        html = """<menu>
            <li>List item</li>
            <button>Button item</button>
            <li>Another list item</li>
        </menu>"""
        result = convert_to_markdown(html)
        assert "<menu>" in result
        assert "- List item" in result
        assert "<button>Button item</button>" in result
        assert "- Another list item" in result


class TestInteractiveElementsIntegration:
    """Test interactive elements in various contexts."""

    def test_dialog_in_paragraph(self) -> None:
        """Test dialog nested in paragraph (should not happen but test anyway)."""
        html = "<p>Click here: <dialog>Modal content</dialog> to see dialog.</p>"
        result = convert_to_markdown(html)

        assert "Click here:" in result
        assert "Modal content" in result

    def test_menu_in_navigation(self) -> None:
        """Test menu inside navigation element."""
        html = """<nav>
            <menu>
                <li><a href="/home">Home</a></li>
                <li><a href="/about">About</a></li>
            </menu>
        </nav>"""
        result = convert_to_markdown(html)
        assert "<menu>" in result
        assert "[Home](/home)" in result
        assert "[About](/about)" in result

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
        assert "<details>" in result
        assert "<summary>Show Menu</summary>" in result
        assert "<menu>" in result

    def test_dialog_with_form(self) -> None:
        """Test dialog containing form elements."""
        html = """<dialog open>
            <form>
                <label>Name: <input type="text" name="name"></label>
                <button type="submit">Submit</button>
            </form>
        </dialog>"""
        result = convert_to_markdown(html)
        assert "<dialog open>" in result
        assert "<form>" in result
        assert 'Name: <input type="text" name="name" />' in result

    def test_multiple_dialogs(self) -> None:
        """Test multiple dialog elements."""
        html = """
        <dialog id="dialog1">First dialog</dialog>
        <dialog id="dialog2" open>Second dialog</dialog>
        """
        result = convert_to_markdown(html)
        assert '<dialog id="dialog1">' in result
        assert '<dialog open id="dialog2">' in result
        assert "First dialog" in result
        assert "Second dialog" in result

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
        assert result.count("<menu>") == 2
        assert result.count("</menu>") == 2
        assert "- File" in result
        assert "- New" in result
        assert "- Open" in result
        assert "- Edit" in result


class TestInteractiveElementsEdgeCases:
    """Test edge cases for interactive elements."""

    def test_dialog_with_special_characters(self) -> None:
        """Test dialog with special Markdown characters."""
        html = "<dialog>This has *asterisks* and _underscores_ and [brackets]</dialog>"
        result = convert_to_markdown(html)
        assert "<dialog>" in result
        assert "\\*asterisks\\*" in result
        assert "\\_underscores\\_" in result
        assert "\\[brackets]" in result

    def test_menu_with_special_characters(self) -> None:
        """Test menu with special Markdown characters."""
        html = "<menu><li>Item with *bold* text</li><li>Item with _italic_ text</li></menu>"
        result = convert_to_markdown(html)
        assert "<menu>" in result
        assert "\\*bold\\*" in result
        assert "\\_italic\\_" in result

    def test_dialog_attribute_values_with_quotes(self) -> None:
        """Test dialog with attribute values containing quotes."""
        html = '<dialog id="my-dialog" class="special">Content</dialog>'
        result = convert_to_markdown(html)

        assert '<dialog id="my-dialog">' in result

    def test_menu_with_complex_attributes(self) -> None:
        """Test menu with complex attribute combinations."""
        html = '<menu type="toolbar" label="Tools &amp; Options" id="toolbar-1"><li>Cut</li></menu>'
        result = convert_to_markdown(html)
        assert 'type="toolbar"' in result
        assert 'label="Tools & Options"' in result
        assert 'id="toolbar-1"' in result

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

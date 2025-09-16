from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable


def test_dialog_basic(convert: Callable[[str, ...], str]) -> None:
    """Test basic dialog conversion."""
    html = "<dialog>Simple dialog content</dialog>"
    result = convert(html)
    assert result == "Simple dialog content\n\n"


def test_dialog_open(convert: Callable[[str, ...], str]) -> None:
    html = "<dialog open>This dialog is open</dialog>"
    result = convert(html)
    assert result == "This dialog is open\n\n"


def test_dialog_with_id(convert: Callable[[str, ...], str]) -> None:
    html = '<dialog id="myDialog">Dialog with ID</dialog>'
    result = convert(html)
    assert result == "Dialog with ID\n\n"


def test_dialog_open_with_id(convert: Callable[[str, ...], str]) -> None:
    html = '<dialog open id="openDialog">Open dialog with ID</dialog>'
    result = convert(html)
    assert result == "Open dialog with ID\n\n"


def test_dialog_empty(convert: Callable[[str, ...], str]) -> None:
    html = "<dialog></dialog>"
    result = convert(html)
    assert result == ""


def test_dialog_whitespace_only(convert: Callable[[str, ...], str]) -> None:
    html = "<dialog>   \n  \t  </dialog>"
    result = convert(html)
    assert result == ""


def test_dialog_inline_mode(convert: Callable[[str, ...], str]) -> None:
    html = "<dialog>Inline dialog content</dialog>"
    result = convert(html, convert_as_inline=True)
    assert result == "Inline dialog content"


def test_dialog_with_nested_elements(convert: Callable[[str, ...], str]) -> None:
    html = "<dialog><h2>Dialog Title</h2><p>Dialog content with <strong>bold</strong> text.</p></dialog>"
    result = convert(html)
    expected = """Dialog Title\n------------\n\nDialog content with **bold** text.\n\n"""
    assert result == expected


def test_dialog_multiline_content(convert: Callable[[str, ...], str]) -> None:
    html = """<dialog>
        <p>First paragraph</p>
        <p>Second paragraph</p>
    </dialog>"""
    result = convert(html)
    expected = """First paragraph\n\nSecond paragraph\n\n"""
    assert result == expected


def test_dialog_with_buttons(convert: Callable[[str, ...], str]) -> None:
    html = """<dialog>
        <p>Are you sure?</p>
        <button>Yes</button>
        <button>No</button>
    </dialog>"""
    result = convert(html)
    expected = """Are you sure?\n\nYes\n\nNo\n\n"""
    assert result == expected


def test_menu_basic(convert: Callable[[str, ...], str]) -> None:
    html = "<menu><li>Item 1</li><li>Item 2</li></menu>"
    result = convert(html)
    assert result == "- Item 1\n- Item 2\n\n"


def test_menu_toolbar(convert: Callable[[str, ...], str]) -> None:
    html = '<menu type="toolbar"><li>Cut</li><li>Copy</li><li>Paste</li></menu>'
    result = convert(html)
    assert result == "- Cut\n- Copy\n- Paste\n\n"


def test_menu_context(convert: Callable[[str, ...], str]) -> None:
    html = '<menu type="context"><li>Delete</li><li>Rename</li></menu>'
    result = convert(html)
    assert result == "- Delete\n- Rename\n\n"


def test_menu_with_label(convert: Callable[[str, ...], str]) -> None:
    html = '<menu label="File Operations"><li>Open</li><li>Save</li></menu>'
    result = convert(html)
    assert result == "- Open\n- Save\n\n"


def test_menu_toolbar_with_label(convert: Callable[[str, ...], str]) -> None:
    html = '<menu type="toolbar" label="Edit Tools"><li>Bold</li><li>Italic</li></menu>'
    result = convert(html)
    assert result == "- Bold\n- Italic\n\n"


def test_menu_with_id(convert: Callable[[str, ...], str]) -> None:
    html = '<menu id="mainMenu"><li>Home</li><li>About</li></menu>'
    result = convert(html)
    assert result == "- Home\n- About\n\n"


def test_menu_all_attributes(convert: Callable[[str, ...], str]) -> None:
    html = '<menu type="context" label="Context Actions" id="contextMenu"><li>Edit</li><li>Delete</li></menu>'
    result = convert(html)
    assert result == "- Edit\n- Delete\n\n"


def test_menu_type_list_omitted(convert: Callable[[str, ...], str]) -> None:
    html = '<menu type="list"><li>Item 1</li><li>Item 2</li></menu>'
    result = convert(html)
    assert result == "- Item 1\n- Item 2\n\n"


def test_menu_empty(convert: Callable[[str, ...], str]) -> None:
    html = "<menu></menu>"
    result = convert(html)
    assert result == ""


def test_menu_whitespace_only(convert: Callable[[str, ...], str]) -> None:
    html = "<menu>   \n  \t  </menu>"
    result = convert(html)
    assert result == ""


def test_menu_inline_mode(convert: Callable[[str, ...], str]) -> None:
    html = "<menu><li>Inline item</li></menu>"
    result = convert(html, convert_as_inline=True)
    assert result == "- Inline item"


def test_menu_with_nested_elements(convert: Callable[[str, ...], str]) -> None:
    html = "<menu><li><strong>Bold Item</strong></li><li><em>Italic Item</em></li></menu>"
    result = convert(html)
    expected = "- **Bold Item**\n- *Italic Item*\n\n"
    assert result == expected


def test_menu_with_buttons(convert: Callable[[str, ...], str]) -> None:
    html = """<menu type="toolbar">
        <button>New</button>
        <button>Open</button>
        <button>Save</button>
    </menu>"""
    result = convert(html)
    expected = """New\n\nOpen\n\nSave\n\n"""
    assert result == expected


def test_menu_mixed_content(convert: Callable[[str, ...], str]) -> None:
    html = """<menu>
        <li>List item</li>
        <button>Button item</button>
        <li>Another list item</li>
    </menu>"""
    result = convert(html)
    expected = """- List item\nButton item\n\n- Another list item\n\n"""
    assert result == expected


def test_dialog_in_paragraph(convert: Callable[[str, ...], str]) -> None:
    html = "<p>Click here: <dialog>Modal content</dialog> to see dialog.</p>"
    result = convert(html)
    expected = "Click here: Modal content\n\n to see dialog.\n\n"
    assert result == expected


def test_menu_in_navigation(convert: Callable[[str, ...], str]) -> None:
    html = """<nav>
        <menu>
            <li><a href="/home">Home</a></li>
            <li><a href="/about">About</a></li>
        </menu>
    </nav>"""
    result = convert(html)
    expected = "- [Home](/home)\n- [About](/about)\n\n"
    assert result == expected


def test_nested_interactive_elements(convert: Callable[[str, ...], str]) -> None:
    html = """<div>
        <details>
            <summary>Show Menu</summary>
            <menu>
                <li>Option 1</li>
                <li>Option 2</li>
            </menu>
        </details>
    </div>"""
    result = convert(html)
    expected = """**Show Menu**\n\n- Option 1\n- Option 2\n\n"""
    assert result == expected


def test_dialog_with_form(convert: Callable[[str, ...], str]) -> None:
    html = """<dialog open>
        <form>
            <label>Name: <input type="text" name="name"></label>
            <button type="submit">Submit</button>
        </form>
    </dialog>"""
    result = convert(html)
    expected = """Name:\n\nSubmit\n\n"""
    assert result == expected


def test_multiple_dialogs(convert: Callable[[str, ...], str]) -> None:
    html = """
    <dialog id="dialog1">First dialog</dialog>
    <dialog id="dialog2" open>Second dialog</dialog>
    """
    result = convert(html)
    expected = """First dialog\n\nSecond dialog\n\n"""
    assert result == expected


def test_menu_with_submenus(convert: Callable[[str, ...], str]) -> None:
    html = """<menu>
        <li>File
            <menu>
                <li>New</li>
                <li>Open</li>
            </menu>
        </li>
        <li>Edit</li>
    </menu>"""
    result = convert(html)
    expected = """- File - New\n- Open\n- Edit\n\n"""
    assert result == expected


def test_dialog_with_special_characters(convert: Callable[[str, ...], str]) -> None:
    html = "<dialog>This has *asterisks* and _underscores_ and [brackets]</dialog>"
    result = convert(html)
    expected = "This has \\*asterisks\\* and \\_underscores\\_ and \\[brackets]\n\n"
    assert result == expected


def test_menu_with_special_characters(convert: Callable[[str, ...], str]) -> None:
    html = "<menu><li>Item with *bold* text</li><li>Item with _italic_ text</li></menu>"
    result = convert(html)
    expected = "- Item with \\*bold\\* text\n- Item with \\_italic\\_ text\n\n"
    assert result == expected


def test_dialog_attribute_values_with_quotes(convert: Callable[[str, ...], str]) -> None:
    html = '<dialog id="my-dialog" class="special">Content</dialog>'
    result = convert(html)
    assert result == "Content\n\n"


def test_dialog_content_ending_with_single_newline(convert: Callable[[str, ...], str]) -> None:
    html = "<dialog>Content\n</dialog>"
    result = convert(html)
    assert result == "Content\n\n"


def test_menu_with_complex_attributes(convert: Callable[[str, ...], str]) -> None:
    html = '<menu type="toolbar" label="Tools &amp; Options" id="toolbar-1"><li>Cut</li></menu>'
    result = convert(html)
    assert result == "- Cut\n\n"


def test_empty_dialog_with_attributes(convert: Callable[[str, ...], str]) -> None:
    html = '<dialog open id="empty"></dialog>'
    result = convert(html)
    assert result == ""


def test_empty_menu_with_attributes(convert: Callable[[str, ...], str]) -> None:
    html = '<menu type="toolbar" label="Empty"></menu>'
    result = convert(html)
    assert result == ""

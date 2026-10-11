// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Form controls (issues 752 and 757): the text of each control is a word of its own, and a
//! checkbox writes task brackets only as the marker of a task item.

use html_to_markdown_rs::options::PreprocessingOptions;
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, NewlineStyle, TierStrategy, WarningKind, convert};

/// Options that let the fast converter run, with forms kept.
fn options(tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        tier_strategy,
        preprocessing: PreprocessingOptions {
            remove_forms: false,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    }
}

fn markdown(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn fast_converter(html: &str) -> Result<String, BailReason> {
    tier1::run(html, &PrescanReport::default(), &options(TierStrategy::Tier1))
}

/// Asserts `expected` for `html` on every path: the full converter, the fast converter with its
/// fallback, the router, and the default options with forms kept. The fast converter alone must
/// write the same text or hand the input over.
fn assert_all_paths(html: &str, expected: &str) {
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Tier1, TierStrategy::Auto] {
        assert_eq!(
            markdown(html, options(tier_strategy)),
            expected,
            "{tier_strategy:?}: {html}"
        );
    }
    let defaults = ConversionOptions {
        preprocessing: options(TierStrategy::Auto).preprocessing,
        ..ConversionOptions::default()
    };
    assert_eq!(markdown(html, defaults), expected, "default options: {html}");
    if let Ok(fast) = fast_converter(html) {
        assert_eq!(fast, expected, "fast converter alone: {html}");
    }
}

fn render(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.table = true;
    options.extension.tasklist = true;
    comrak::markdown_to_html(markdown, &options)
}

#[test]
fn should_separate_the_options_of_a_select_list() {
    for (html, expected) in [
        (
            "<p>before</p><select><option>Quickstart</option><option>Installation</option><option>Ruby 101</option></select><p>after</p>",
            "before\n\nQuickstart Installation Ruby 101\n\nafter\n",
        ),
        (
            "<p>before</p>\n<select>\n  <option>Quickstart</option>\n  <option>Installation</option>\n  <option>Ruby 101</option>\n</select>\n<p>after</p>",
            "before\n\nQuickstart Installation Ruby 101\n\nafter\n",
        ),
        (
            "<select><option>One<option>Two<option>Three</select>",
            "One Two Three\n",
        ),
        (
            "<select multiple><option selected>One</option><option>Two</option><option selected>Three</option></select>",
            "One Two Three\n",
        ),
        (
            "<select><option></option><option>One</option><option> </option><option>Two</option></select>",
            "One Two\n",
        ),
        (
            r#"<select><optgroup label="Fruit"><option>Apple</option><option>Pear</option></optgroup><optgroup label="Veg"><option>Leek</option></optgroup><option>Other</option></select>"#,
            "Apple Pear Leek Other\n",
        ),
        (
            r#"<input list="l"><datalist id="l"><option value="v1">One</option><option value="v2">Two</option></datalist>"#,
            "One Two\n",
        ),
        (
            r#"<p>Browser <input list="l"><datalist id="l"><option>One</option><option>Two</option></datalist> end</p>"#,
            "Browser One Two end\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

/// Issue 776: the `label` attribute of an option group is not text of the page. A browser shows
/// it only inside the open list.
#[test]
fn should_not_write_the_label_of_an_option_group() {
    for (html, expected) in [
        (
            r#"<p>Before.</p><select><optgroup label="Getting Started"><option>Quickstart</option><option>Installation</option></optgroup></select><p>After.</p>"#,
            "Before.\n\nQuickstart Installation\n\nAfter.\n",
        ),
        (
            r#"<p>Before.</p><select><optgroup label="Getting Started"><option>Quickstart</option></optgroup><optgroup label="Build"><option>Commands</option></optgroup></select><p>After.</p>"#,
            "Before.\n\nQuickstart Commands\n\nAfter.\n",
        ),
        (
            "<p>Before.</p>\n<select>\n  <optgroup label=\"Getting Started\"><option>Quickstart</option></optgroup>\n  <optgroup label=\"Build\"><option>Commands</option></optgroup>\n</select>\n<p>After.</p>",
            "Before.\n\nQuickstart Commands\n\nAfter.\n",
        ),
        (
            r#"<p>Pick <select><optgroup label="Fruit"><option>Apple</option></optgroup></select> now.</p>"#,
            "Pick Apple now.\n",
        ),
        (
            r#"<p>a</p><select><optgroup label="Empty"></optgroup></select><p>b</p>"#,
            "a\n\nb\n",
        ),
        (
            r#"<datalist><optgroup label="Fruit"><option>Apple</option></optgroup></datalist>"#,
            "Apple\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_keep_a_select_list_a_word_of_its_own_in_the_line() {
    for (html, expected) in [
        (
            "<p>Pick <select><option>One</option><option>Two</option></select> now.</p>",
            "Pick One Two now.\n",
        ),
        // ~keep No space before punctuation and none at the end of a block.
        (
            "<p>Pick <select><option>One</option><option>Two</option></select>.</p>",
            "Pick One Two.\n",
        ),
        (
            "<p>(<select><option>One</option><option>Two</option></select>)</p>",
            "(One Two)\n",
        ),
        (
            "<p><select><option>10</option><option>25</option></select><span>items</span></p>",
            "10 25 items\n",
        ),
        (
            "<p><select><option>10</option><option>25</option></select><strong>items</strong></p>",
            "10 25 **items**\n",
        ),
        (
            "<p><label><select><option>10</option><option>25</option></select></label>items</p><p>next</p>",
            "10 25 items\n\nnext\n",
        ),
        (
            "<div><select><option>10</option><option>25</option></select></div><p>items</p>",
            "10 25\n\nitems\n",
        ),
        (
            "<select><option>One</option></select><select><option>Two</option></select>",
            "One Two\n",
        ),
        (
            "<label>Size</label><select><option>S</option><option>M</option></select>",
            "Size S M\n",
        ),
        (
            "<label>Size <select><option>S</option><option>M</option></select></label>",
            "Size S M\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_keep_a_select_list_inline_in_a_cell_a_heading_a_link_and_a_list_item() {
    for (html, expected) in [
        (
            "<table><tr><th>Pick</th></tr><tr><td><select><option>One</option><option>Two</option></select></td></tr></table>",
            "| Pick    |\n| ------- |\n| One Two |\n",
        ),
        (
            "<h2>Version <select><option>One</option><option>Two</option></select></h2>",
            "## Version One Two\n",
        ),
        (
            r#"<a href="/u"><select><option>One</option><option>Two</option></select></a>"#,
            "[One Two](/u)\n",
        ),
        (
            "<ul><li><select><option>One</option><option>Two</option></select></li></ul>",
            "- One Two\n",
        ),
        // ~keep The line break of issue 711 stays a line break.
        ("<select><option>x</option></select><br>y", "x  \ny\n"),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_separate_the_text_of_the_controls_of_a_form() {
    for (html, expected) in [
        (
            r#"<form><label for="n">Name</label><input id="n"><select><option>One</option><option>Two</option></select><textarea>area words</textarea><button>Send</button></form>"#,
            "Name One Two area words\n\nSend\n",
        ),
        (
            "<form>\n  <label for=\"n\">Name</label>\n  <input id=\"n\">\n  <select>\n    <option>One</option>\n    <option>Two</option>\n  </select>\n  <textarea>area words</textarea>\n  <button>Send</button>\n</form>",
            "Name One Two area words\n\nSend\n",
        ),
        // ~keep An input writes no text, so it adds no space: a browser shows these labels as one word.
        (
            r#"<label for="n">Name</label><input id="n"><label for="m">Mail</label><input id="m">"#,
            "NameMail\n",
        ),
        (
            "<label for=\"n\">Name</label>\n<input id=\"n\">\n<label for=\"m\">Mail</label>\n<input id=\"m\">",
            "Name Mail\n",
        ),
        (
            r#"<label>Name <input name="n"></label><label>Mail <input name="m"></label>"#,
            "Name Mail\n",
        ),
        (
            "<fieldset><legend>Who</legend><label>Name <input></label><label>Mail <input></label></fieldset>",
            "**Who**\n\nName Mail\n",
        ),
        (
            r#"<label><input type="radio" name="r" checked> Yes</label><label><input type="radio" name="r"> No</label>"#,
            "Yes No\n",
        ),
        (
            "<label>Note</label><textarea>area words</textarea>",
            "Note area words\n",
        ),
        ("<p>Name <button>Go</button></p>", "Name Go\n"),
        (
            "<p>Say <textarea>area words</textarea> now</p>",
            "Say area words\n\nnow\n",
        ),
        (
            r#"<output>42</output><meter value="0.5">half</meter><progress value="1" max="3">one of three</progress>"#,
            "42\n\nhalf\n\none of three\n",
        ),
        ("<p>Total<output>42</output></p>", "Total 42\n"),
        (
            "<table><tr><th>Name</th><th>Size</th></tr><tr><td><input name=\"n\"></td><td><select><option>S</option><option>M</option></select></td></tr></table>",
            "| Name | Size |\n| ---- | ---- |\n|      | S M  |\n",
        ),
        // ~keep A label is text, as a span is: two labels with no white space between them join.
        ("<label>First</label><label>Second</label>", "FirstSecond\n"),
        ("<label>First</label><br>Second", "First  \nSecond\n"),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_read_the_text_after_a_control_through_what_writes_nothing() {
    for (html, expected) in [
        (
            "<p>Pick <select><option>One</option></select>, then</p>",
            "Pick One, then\n",
        ),
        (
            "<p>(Pick <select><option>One</option></select>) then</p>",
            "(Pick One) then\n",
        ),
        ("<p>Pick <select><option>One</option></select>?</p>", "Pick One?\n"),
        (
            "<p>Pick <select><option>One</option></select>(now)</p>",
            "Pick One (now)\n",
        ),
        (
            "<p><select><option>One</option></select><script>var a;</script>items</p>",
            "One items\n",
        ),
        (
            "<p><select><option>One</option></select><!-- c -->items</p>",
            "One items\n",
        ),
        (
            "<p><select><option>One</option></select><span></span>items</p>",
            "One items\n",
        ),
        (
            "<p><select><option>One</option></select><input>items</p>",
            "One items\n",
        ),
        (
            r#"<p><select><option>One</option></select><input type="hidden">items</p>"#,
            "One items\n",
        ),
        (
            "<p><select><option>One</option></select><button>Go</button></p>",
            "One Go\n",
        ),
        (
            "<p><select><option>One</option></select><em>items</em></p>",
            "One *items*\n",
        ),
        (
            r#"<p><select><option>One</option></select><a href="/u">items</a></p>"#,
            "One [items](/u)\n",
        ),
        (
            "<p><span><select><option>One</option></select></span>items</p>",
            "One items\n",
        ),
        // ~keep A block and a line break end the line: no space is written before them.
        (
            "<div><select><option>One</option></select><div>items</div></div>",
            "One\n\nitems\n",
        ),
        (
            "<div><div><select><option>One</option></select></div>items</div>",
            "One\n\nitems\n",
        ),
        (
            "<p><select><option>One</option></select><br>items</p>",
            "One  \nitems\n",
        ),
        (
            r#"<p><input type="checkbox"><img src="i.png" alt="pic"></p>"#,
            "![pic](i.png)\n",
        ),
        (
            "<select><option>One</option> <option>Two</option></select>",
            "One Two\n",
        ),
        ("<p>text<option>One</option></p>", "text One\n"),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_separate_a_control_in_a_line_that_goes_on_after_it() {
    for (html, expected) in [
        ("<h2>Total<output>42</output>items</h2>", "## Total 42 items\n"),
        ("<h2>Go<button>now</button>then</h2>", "## Go now then\n"),
        ("<h2>Go <button>now</button> then</h2>", "## Go now then\n"),
        ("<h2>Total<output>42</output>.</h2>", "## Total 42.\n"),
    ] {
        assert_all_paths(html, expected);
    }
}

/// In a heading the line goes on after a button. An input between the button and the next word
/// writes no text, so the button is separated from that word.
#[test]
fn should_read_past_an_input_to_the_word_after_a_button() {
    for input in [r#"type="checkbox""#, r#"type="hidden""#, r#"type="text""#] {
        let controls = format!("aaa<button>Xone</button><input {input}>bbb");
        assert_all_paths(&format!("<h1>{controls}</h1>"), "# aaa Xone bbb\n");
    }
}

/// A button, an output, a meter and a progress bar end their line only where a block holds
/// them. In an inline element a browser keeps them in the line, so the text after the element
/// is a word of the same line.
#[test]
fn should_keep_a_control_in_an_inline_element_in_its_line() {
    for control in ["button", "output", "meter", "progress", "textarea"] {
        for (html, expected) in [
            ("<label><c>One</c></label>items", "One items\n"),
            ("<label><c>One</c></label> items", "One items\n"),
            ("<label><c>One</c></label>.", "One.\n"),
            ("<p>a<label><c>One</c></label>items</p>", "a One items\n"),
            ("<p>a<label><c>One</c></label></p><p>next</p>", "a One\n\nnext\n"),
            ("<label><c>One</c></label><label><c>One</c></label>", "One One\n"),
            ("<p><label><c>One</c>after</label>items</p>", "One afteritems\n"),
            ("<p><label>Pick <c>One</c></label>items</p>", "Pick One items\n"),
            ("<b><c>One</c></b>items", "**One** items\n"),
            ("<b><c>One</c></b> items", "**One** items\n"),
            ("<b><c>One</c></b>.", "**One**.\n"),
            ("<p>a<b><c>One</c></b>items</p>", "a**One** items\n"),
            ("<p>a<b><c>One</c></b></p><p>next</p>", "a**One**\n\nnext\n"),
            ("<b><c>One</c></b><b><c>One</c></b>", "**One** **One**\n"),
            ("<p><b><c>One</c>after</b>items</p>", "**One after**items\n"),
            ("<em><c>One</c></em>items", "*One* items\n"),
            ("<p>a<em><c>One</c></em></p><p>next</p>", "a*One*\n\nnext\n"),
            ("<span><c>One</c></span>items", "One items\n"),
            ("<span><c>One</c></span> items", "One items\n"),
            ("<span><c>One</c></span>.", "One.\n"),
            ("<p>a<span><c>One</c></span>items</p>", "a One items\n"),
            ("<p>a<span><c>One</c></span></p><p>next</p>", "a One\n\nnext\n"),
            ("<span><c>One</c></span><span><c>One</c></span>", "One One\n"),
            ("<p><span><c>One</c>after</span>items</p>", "One afteritems\n"),
            ("<my-tag><c>One</c></my-tag>items", "One items\n"),
            ("<p>a<my-tag><c>One</c></my-tag></p><p>next</p>", "a One\n\nnext\n"),
            ("<p><span><my-tag><c>One</c></my-tag></span>items</p>", "One items\n"),
            // ~keep An element the converter does not know is written as its children alone.
            ("<p><font><c>One</c></font>items</p>", "One items\n"),
            (
                "<table><tr><th>h</th></tr><tr><td>a<label><c>One</c></label>items</td></tr></table>",
                "| h           |\n| ----------- |\n| a One items |\n",
            ),
            ("<ul><li>a<label><c>One</c></label>items</li></ul>", "- a One items\n"),
        ] {
            let html = html
                .replace("<c>", &format!("<{control}>"))
                .replace("</c>", &format!("</{control}>"));
            assert_all_paths(&html, expected);
        }
        // ~keep In a link label the control is written as a select list is. In a code span it is
        // ~keep written as its text is: code adds no space.
        for (wrapper, same_as) in [
            ("<code>|</code>", "One"),
            (r#"<a href="/u">|</a>"#, "<select><option>One</option></select>"),
        ] {
            for shape in [
                "|items",
                "| items",
                "|.",
                "<p>a|items</p>",
                "<p>a|</p><p>next</p>",
                "||",
            ] {
                let line_end = wrapper.replace('|', &format!("<{control}>One</{control}>"));
                let html = shape.replace('|', &line_end);
                let reference = shape.replace('|', &wrapper.replace('|', same_as));
                let expected = markdown(&reference, options(TierStrategy::Tier2));
                assert!(expected.contains("One"), "{expected:?}");
                assert_all_paths(&html, &expected);
            }
        }
        // ~keep A block still ends the line of the control.
        for (html, expected) in [
            ("<div><c>One</c>items</div>", "One\n\nitems\n"),
            ("<p>a<c>One</c>items</p>", "a One\n\nitems\n"),
            ("<label><div><c>One</c></div></label><p>items</p>", "One\n\nitems\n"),
        ] {
            let html = html
                .replace("<c>", &format!("<{control}>"))
                .replace("</c>", &format!("</{control}>"));
            assert_all_paths(&html, expected);
        }
    }
}

/// The fast converter has not read the text after a control, so it hands over a control whose
/// line goes on after it and keeps one that a block holds.
#[test]
fn should_hand_a_control_in_an_inline_element_to_the_full_converter() {
    for control in ["button", "output", "meter", "progress"] {
        for wrapper in ["label", "b", "code", "em", "span", "my-tag"] {
            let html = format!("<{wrapper}><{control}>One</{control}></{wrapper}>items");
            let result = fast_converter(&html);
            assert!(matches!(result, Err(BailReason::FormControl)), "{html}: {result:?}");
        }
        let html = format!("<div><{control}>One</{control}>items</div>");
        assert_eq!(
            fast_converter(&html).map_err(|reason| reason.to_string()),
            Ok("One\n\nitems\n".to_string()),
            "{html}"
        );
    }
}

#[test]
fn should_separate_a_label_that_starts_with_a_control_as_the_control_is() {
    for (html, expected) in [
        (r#"<p>Name<label> <input type="checkbox"> ok</label></p>"#, "Name ok\n"),
        (
            "<p>Name<label><!-- c --><select><option>One</option></select></label></p>",
            "Name One\n",
        ),
        ("<p>Name<label><input> Mail</label></p>", "Name Mail\n"),
        // ~keep An input adds no space where the source has none.
        ("<p>Name<label><input>Mail</label></p>", "NameMail\n"),
        // ~keep A label that starts with text, or with an input nobody sees, is text.
        (r#"<p>Name<label>x<input type="checkbox"></label></p>"#, "Namex\n"),
        (
            r#"<p>Name<label><input type="hidden" value="t">x</label></p>"#,
            "Namex\n",
        ),
        // ~keep An inline element at the start of a label is not a control.
        ("<p>Name<label><span>ok</span></label></p>", "Nameok\n"),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_look_for_the_task_checkbox_only_before_the_first_content_of_an_item() {
    for (html, expected) in [
        (r#"<ul><li><!-- c --><input type="checkbox"> a</li></ul>"#, "- [ ] a\n"),
        (
            r#"<ul><li><script>var a;</script><input type="checkbox"> a</li></ul>"#,
            "- [ ] a\n",
        ),
        (
            r#"<ul><li><span></span><input type="checkbox"> a</li></ul>"#,
            "- [ ] a\n",
        ),
        (
            r#"<ul><li><input type="text"><input type="checkbox"> a</li></ul>"#,
            "- [ ] a\n",
        ),
        (r#"<ul><li><p><input type="checkbox"> a</p></li></ul>"#, "- [ ] a\n"),
        (
            r#"<ul><li><span>x</span><input type="checkbox"> a</li></ul>"#,
            "- x a\n",
        ),
        (r#"<ul><li><hr><input type="checkbox"> a</li></ul>"#, "- ___\n\n  a\n"),
        (
            r#"<ul><li><ul><li>x</li></ul><input type="checkbox"> a</li></ul>"#,
            "- * x\n\n  a\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_read_the_first_role_of_a_checkbox() {
    // ~keep Only a table cell shows the answer: a checkbox writes its state there.
    for (role, expected) in [
        ("presentation button", "| h |\n| --- |\n|   |\n"),
        ("MenuItemCheckbox", "| h   |\n| --- |\n| [x] |\n"),
        ("", "| h   |\n| --- |\n| [x] |\n"),
    ] {
        let html = format!(
            r#"<table><tr><th>h</th></tr><tr><td><input type="checkbox" role="{role}" checked></td></tr></table>"#
        );
        assert_all_paths(&html, expected);
        let html = format!(r#"<p>a <input type="checkbox" role="{role}" checked> b</p>"#);
        assert_all_paths(&html, "a b\n");
    }
}

#[test]
fn should_write_a_button_as_a_word_of_its_own_in_the_fast_converter_too() {
    for (html, expected) in [
        ("<label>Name</label><button>Go</button>", "Name Go\n"),
        ("<button>One</button><button>Two</button>", "One\n\nTwo\n"),
        ("<label>Name</label><button></button>tail", "Nametail\n"),
        ("<label>Name</label><button> Go</button>", "Name Go\n"),
        ("<label>Total</label><output>42</output>", "Total 42\n"),
        ("<label>Total</label><progress>42</progress>", "Total 42\n"),
        ("<progress>a</progress>tail", "a\n\ntail\n"),
        ("<progress>a</progress><output>b</output>", "a\n\nb\n"),
        (
            r#"<output>42</output><meter value="0.5">half</meter><progress value="1" max="3">one of three</progress>"#,
            "42\n\nhalf\n\none of three\n",
        ),
    ] {
        assert_all_paths(html, expected);
        // ~keep The fast converter writes a button itself: the fallback must not hide its output.
        assert_eq!(
            fast_converter(html).map_err(|reason| reason.to_string()),
            Ok(expected.to_string()),
            "{html}"
        );
    }
}

#[test]
fn should_hand_a_control_with_separated_text_to_the_full_converter() {
    for html in [
        "<select><option>One</option></select>",
        "<option>One</option>",
        r#"<optgroup label="Fruit"></optgroup>"#,
        "<datalist><option>One</option></datalist>",
        "<h2>Total<output>42</output>.</h2>",
        "<h2>Go<button>now</button>then</h2>",
    ] {
        let result = fast_converter(html);
        assert!(matches!(result, Err(BailReason::FormControl)), "{html}: {result:?}");
    }
    // ~keep An input writes nothing and adds no space in both converters, so the fast converter
    // ~keep keeps the input.
    for (html, expected) in [
        ("<p>Name<input>Mail</p>", "NameMail\n"),
        (r#"<p>Name<input type="text" value="typed">Mail</p>"#, "NameMail\n"),
        (r#"<p>Name<input type="radio" checked>Mail</p>"#, "NameMail\n"),
        ("<p>Name <input> Mail</p>", "Name  Mail\n"),
        ("<p><input>Mail</p>", "Mail\n"),
        (r#"<p>a<input type="hidden" value="t">b</p>"#, "ab\n"),
    ] {
        assert_eq!(
            fast_converter(html).map_err(|reason| reason.to_string()),
            Ok(expected.to_string()),
            "{html}"
        );
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_write_task_brackets_only_for_a_list_item_that_starts_with_a_checkbox() {
    for (html, expected) in [
        (
            r#"<ul><li><input type="checkbox"> open task</li><li><input type="checkbox" checked> done task</li></ul>"#,
            "- [ ] open task\n- [x] done task\n",
        ),
        (
            r#"<ul><li><input type="checkbox"> outer<ul><li><input type="checkbox" checked> inner</li></ul></li></ul>"#,
            "- [ ] outer\n  - [x] inner\n",
        ),
        (
            r#"<ul><li><label><input type="checkbox" checked> Item</label></li></ul>"#,
            "- [x] Item\n",
        ),
        (
            r#"<ol><li><input type="checkbox" checked> first</li><li>second</li></ol>"#,
            "1. [x] first\n2. second\n",
        ),
        // ~keep A second checkbox in a task item, and a checkbox after other content of an
        // ~keep item, are controls in the text.
        (
            r#"<ul><li><input type="checkbox"> a <input type="checkbox" checked> b</li></ul>"#,
            "- [ ] a b\n",
        ),
        (
            r#"<ul><li>Text <input type="checkbox"> more</li></ul>"#,
            "- Text more\n",
        ),
        (r#"<ul><li>Text <input type="checkbox" checked></li></ul>"#, "- Text\n"),
        (
            r#"<ul><li><img src="i.png" alt="pic"><input type="checkbox" checked> shown</li></ul>"#,
            "- ![pic](i.png) shown\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_write_nothing_for_a_checkbox_outside_a_task_item() {
    for (html, expected) in [
        (
            r#"<p>Agree <input type="checkbox" name="a"> to the terms</p>"#,
            "Agree to the terms\n",
        ),
        (
            r#"<input type="checkbox" id="nav-toggle"><label for="nav-toggle">Menu</label><p>text</p>"#,
            "Menu\n\ntext\n",
        ),
        (
            "<input type=\"checkbox\" id=\"nav-toggle\">\n<label for=\"nav-toggle\">Menu</label>\n<p>text</p>",
            "Menu\n\ntext\n",
        ),
        (
            "<p>Agree<input type=\"checkbox\">to the terms</p>",
            "Agreeto the terms\n",
        ),
        (r#"<h2><input type="checkbox" checked> Title</h2>"#, "## Title\n"),
        (
            r#"<a href="/u"><input type="checkbox" checked> label</a>"#,
            "[label](/u)\n",
        ),
        (
            r#"<label><input type="checkbox" checked> Remember me</label>"#,
            "Remember me\n",
        ),
        (
            r#"<label for="c">Remember me</label><input type="checkbox" id="c">"#,
            "Remember me\n",
        ),
        (r#"<p><input type="checkbox"> starts</p>"#, "starts\n"),
        (r#"<p>a <INPUT TYPE="CHECKBOX" CHECKED> b</p>"#, "a b\n"),
        (r#"<p><input type="checkbox"><input type="checkbox" checked></p>"#, ""),
        (
            r#"<blockquote><input type="checkbox"> quoted</blockquote>"#,
            "> quoted\n",
        ),
        (r#"<p>a <input type="checkbox" disabled checked> b</p>"#, "a b\n"),
        (
            r#"<p>Dark mode<input type="checkbox" role="switch" checked>.</p>"#,
            "Dark mode.\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_write_nothing_for_a_checkbox_that_has_the_role_of_a_button() {
    for (html, expected) in [
        (
            r#"<input type="checkbox" id="m" role="button" aria-haspopup="true" aria-label="Main menu"><label for="m">Main menu</label>"#,
            "Main menu\n",
        ),
        (
            r#"<ul><li><input type="checkbox" role="button"> Menu</li></ul>"#,
            "- Menu\n",
        ),
        // ~keep The white space on both sides of a checkbox that writes nothing is one space.
        (r#"<p>a <input type="checkbox" role="button"> b</p>"#, "a b\n"),
        (r#"<p>a <input type="checkbox"><!-- c --> b</p>"#, "a b\n"),
        (
            r#"<pre>a <input type="checkbox" role="button"> b</pre>"#,
            "```\na  b\n```\n",
        ),
        (
            r#"<table><tr><td><input type="checkbox" role="button"></td></tr></table>"#,
            "",
        ),
        (
            r#"<table><tr><td><input type="checkbox" role="checkbox"></td></tr></table>"#,
            "| [ ] |\n| --- |\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_not_turn_a_checkbox_into_a_link() {
    // ~keep With brackets each of these rendered as a link: `[x](optional)` and `[x]` beside
    // ~keep a `[x]:` definition. A checkbox beside text writes nothing.
    for (html, expected, rendered_text) in [
        (
            r#"<p><input type="checkbox" checked>(optional)</p>"#,
            "(optional)\n",
            "<p>(optional)</p>\n",
        ),
        (
            r#"<p><input type="checkbox" checked> on</p><p>[x]: /target</p>"#,
            "on\n\n[x]: /target\n",
            "<p>on</p>\n",
        ),
        (
            r#"<table><tr><th>Done</th></tr><tr><td><input type="checkbox" checked>(late)</td></tr></table>"#,
            "| Done   |\n| ------ |\n| (late) |\n",
            "<table>\n<thead>\n<tr>\n<th>Done</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>(late)</td>\n</tr>\n</tbody>\n</table>\n",
        ),
    ] {
        assert_all_paths(html, expected);
        let rendered = render(expected);
        assert_eq!(rendered, rendered_text, "{html}");
        assert!(!rendered.contains("<a "), "{html}: {rendered}");
    }
    // ~keep The renderer does make a link of the bracket form, so the assertion above can fail.
    assert!(render("[x](optional)\n").contains("<a "));
}

#[test]
fn should_read_past_an_element_that_writes_nothing_to_the_text_after_a_control() {
    for (html, expected) in [
        (
            "<p><select><option>One</option></select><template>.</template>items</p>",
            "One items\n",
        ),
        (
            "<p><select><option>One</option></select><noscript>.</noscript>items</p>",
            "One items\n",
        ),
        (
            r#"<p><select><option>One</option></select><script type="application/ld+json">{"a":1}</script>items</p>"#,
            "One items\n",
        ),
        (r#"<p><input type="checkbox"><button></button>items</p>"#, "items\n"),
        (r#"<p><input type="checkbox"><textarea></textarea>items</p>"#, "items\n"),
        (r#"<p><input type="checkbox"><select></select>items</p>"#, "items\n"),
        (
            "<p><select><option>One</option></select><my-tag>items</my-tag></p>",
            "One items\n",
        ),
        (
            "<p><select><option>One</option></select><my-tag>.</my-tag></p>",
            "One.\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_separate_a_control_in_a_custom_element_from_the_text_after_the_element() {
    for (html, expected) in [
        (
            "<p><my-tag><select><option>One</option></select></my-tag>items</p>",
            "One items\n",
        ),
        (r#"<p><my-tag><input type="checkbox"></my-tag>items</p>"#, "items\n"),
        (
            "<p><x-a><x-b><select><option>One</option></select></x-b></x-a>items</p>",
            "One items\n",
        ),
        (
            "<p><span><my-tag><select><option>One</option></select></my-tag></span>items</p>",
            "One items\n",
        ),
        (
            "<p><my-tag><span><select><option>One</option></select></span></my-tag>items</p>",
            "One items\n",
        ),
        (
            "<p><my-tag><select><option>One</option></select></my-tag><my-tag>items</my-tag></p>",
            "One items\n",
        ),
        ("<h2><my-tag><output>42</output></my-tag>items</h2>", "## 42 items\n"),
        ("<h2><my-tag><button>Go</button></my-tag>now</h2>", "## Go now\n"),
        (
            "<ul><li><my-tag><select><option>One</option></select></my-tag>items</li></ul>",
            "- One items\n",
        ),
        (
            "<table><tr><th>h</th></tr><tr><td><my-tag><select><option>One</option></select></my-tag>items</td></tr></table>",
            "| h         |\n| --------- |\n| One items |\n",
        ),
        // ~keep An element with no hyphen in its name that the converter does not know is the same case.
        (
            "<p><font><select><option>One</option></select></font>items</p>",
            "One items\n",
        ),
        (
            "<p><blink><select><option>One</option></select></blink>items</p>",
            "One items\n",
        ),
        // ~keep Punctuation, white space and the end of the block still take no space.
        (
            "<p><my-tag><select><option>One</option></select></my-tag>.</p>",
            "One.\n",
        ),
        (
            "<p><my-tag><select><option>One</option></select></my-tag> items</p>",
            "One items\n",
        ),
        (
            "<div><my-tag><select><option>One</option></select></my-tag></div><p>items</p>",
            "One\n\nitems\n",
        ),
        (
            "<my-tag><select><option>One</option></select></my-tag><p>items</p>",
            "One\n\nitems\n",
        ),
        // ~keep A table cell and a list item end the line of a control too.
        (
            "<table><tr><th>h</th><th>k</th></tr><tr><td><select><option>One</option></select></td><td>items</td></tr></table>",
            "| h   | k     |\n| --- | ----- |\n| One | items |\n",
        ),
        (
            "<ul><li><select><option>One</option></select></li><li>items</li></ul>",
            "- One\n- items\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

/// A block parent ends the line of a control: what the block writes after itself separates it
/// from the text that follows, and the control adds nothing.
#[test]
fn should_end_the_line_of_a_control_at_a_block_parent() {
    // ~keep A preformatted block in a cell is a code span. A space from the control would be inside it.
    assert_all_paths(
        "<table><tr><th>h</th></tr><tr><td><pre><select><option>One</option></select></pre>items</td></tr></table>",
        "| h           |\n| ----------- |\n| `One` items |\n",
    );
    // ~keep In a heading a block is written in the line. The text of a control in the block is
    // ~keep separated from the text after the block as plain text in that block is.
    for block in ["p", "section", "form"] {
        let control = format!("<h2><{block}><select><option>One</option></select></{block}>items</h2>");
        let text = format!("<h2><{block}>One</{block}>items</h2>");
        assert_eq!(
            markdown(&control, options(TierStrategy::Tier2)),
            markdown(&text, options(TierStrategy::Tier2)),
            "{control}"
        );
    }
}

/// The search for the text after a control reads a bounded number of nodes, so a line of many
/// controls is read in linear time. Past the bound the control writes no space.
#[test]
fn should_stop_the_search_for_the_text_after_a_control_at_the_node_limit() {
    for (empty_elements, expected) in [(10, "a b\n"), (200, "ab\n")] {
        let html = format!(
            "<p><select><option>a</option></select>{}b</p>",
            "<span></span>".repeat(empty_elements)
        );
        assert_all_paths(&html, expected);
    }
}

#[test]
fn should_write_no_space_between_a_control_and_a_line_break() {
    let html = "<p><select><option>One</option></select><br>items</p>";
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Auto] {
        let backslash = ConversionOptions {
            newline_style: NewlineStyle::Backslash,
            ..options(tier_strategy)
        };
        assert_eq!(markdown(html, backslash), "One\\\nitems\n", "{tier_strategy:?}");
    }
}

#[test]
fn should_look_past_what_a_reader_does_not_see_for_the_task_checkbox() {
    for (html, expected) in [
        (
            r#"<ul><li><template>word</template><input type="checkbox"> a</li></ul>"#,
            "- [ ] a\n",
        ),
        (
            r#"<ul><li><script type="application/ld+json">{"a":1}</script><input type="checkbox"> a</li></ul>"#,
            "- [ ] a\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_stop_the_search_for_the_task_checkbox_at_the_depth_limit() {
    let html = format!(
        r#"<ul><li>{}<input type="checkbox"> a{}</li></ul>"#,
        "<span>".repeat(12),
        "</span>".repeat(12)
    );
    let shallow = ConversionOptions {
        max_depth: Some(6),
        ..options(TierStrategy::Tier2)
    };
    let result = convert(&html, Some(shallow)).expect("conversion must succeed");
    let output = result.content.unwrap_or_default();
    assert!(!output.contains("[ ]"), "task brackets in {output:?}");
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.kind == WarningKind::DepthLimitExceeded),
        "{:?}",
        result.warnings
    );
}

/// The text of a control is one level below the control for the depth limit.
#[test]
fn should_count_a_control_as_one_level_for_the_depth_limit() {
    for (html, text_level) in [
        ("<p><button>One</button></p>", 2),
        ("<p><output>One</output></p>", 2),
        ("<p><select><option>One</option></select></p>", 3),
        ("<p><datalist><option>One</option></datalist></p>", 3),
    ] {
        for (max_depth, expected) in [(text_level, ""), (text_level + 1, "One\n")] {
            let limited = ConversionOptions {
                max_depth: Some(max_depth),
                ..options(TierStrategy::Tier2)
            };
            assert_eq!(markdown(html, limited), expected, "max_depth {max_depth}: {html}");
        }
    }
}

#[test]
fn should_write_no_checkbox_for_the_menu_switches_of_a_saved_wikipedia_page() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/benchmark-harness/fixtures/mdream/wikipedia-small.html");
    let html = std::fs::read_to_string(&path).expect("the fixture is in the repository");
    assert_eq!(
        html.matches(r#"type="checkbox""#).count(),
        8,
        "checkboxes in the fixture"
    );
    assert_eq!(html.matches(r#"role="button" aria-haspopup="true""#).count(), 8);

    let keep_everything = ConversionOptions {
        preprocessing: PreprocessingOptions {
            enabled: false,
            remove_navigation: false,
            remove_forms: false,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    };
    let output = markdown(&html, keep_everything);
    assert!(output.contains("Main menu"), "the menu labels are in the output");
    assert_eq!(output.matches("[ ]").count(), 0, "task brackets");
}

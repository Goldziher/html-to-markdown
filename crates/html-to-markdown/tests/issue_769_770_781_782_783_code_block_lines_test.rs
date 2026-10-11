#![allow(missing_docs)]

//! Regression tests for issues #769, #770, #781, #782 and #783: a code block is the lines a reader
//! sees in the page, character for character.
//!
//! Each expected block was compared with the lines that Chrome renders for the same `pre`.

use html_to_markdown_rs::options::WhitespaceMode;
use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, TierStrategy, convert};

struct Case {
    name: &'static str,
    html: &'static str,
    expected: &'static str,
    /// Tier-1 renders this input without a fallback to Tier-2. Only the test kit can ask Tier-1.
    #[cfg(feature = "testkit")]
    tier1_renders: bool,
}

const fn case(name: &'static str, html: &'static str, expected: &'static str) -> Case {
    Case {
        name,
        html,
        expected,
        #[cfg(feature = "testkit")]
        tier1_renders: false,
    }
}

const fn tier1_case(name: &'static str, html: &'static str, expected: &'static str) -> Case {
    Case {
        name,
        html,
        expected,
        #[cfg(feature = "testkit")]
        tier1_renders: true,
    }
}

const CASES: &[Case] = &[
    tier1_case(
        "issue 769, a blank line in a block in a bullet item",
        "<ul><li><p>Run this:</p><pre>Title\n=====\n\n.. toctree::\n\n   sub\n\nAfter\n=====\n</pre></li></ul>",
        "- Run this:\n\n  ```\n  Title\n  =====\n\n  .. toctree::\n\n     sub\n\n  After\n  =====\n  ```\n",
    ),
    tier1_case(
        "issue 769, Python in an ordered item",
        "<ol><li><p>Save this:</p><pre>def f(x):\n    if x:\n\n        return 1\n\n    return 2\n</pre></li></ol>",
        "1. Save this:\n\n   ```\n   def f(x):\n       if x:\n\n           return 1\n\n       return 2\n   ```\n",
    ),
    case(
        "the block is the first content of the item",
        "<ul><li><pre>a\n\n    b\n\nc\n</pre></li></ul>",
        "- ```\n  a\n\n      b\n\n  c\n  ```\n",
    ),
    tier1_case(
        "the block follows text in the item",
        "<ul><li>Text<pre>a\n\n    b\n\nc\n</pre></li></ul>",
        "- Text\n\n  ```\n  a\n\n      b\n\n  c\n  ```\n",
    ),
    tier1_case(
        "an item at the second level",
        "<ul><li>one<ul><li><p>two</p><pre>a\n\n    b\n\nc\n</pre></li></ul></li></ul>",
        "- one\n  * two\n\n    ```\n    a\n\n        b\n\n    c\n    ```\n",
    ),
    case(
        "an item at the third level under ordered items",
        "<ol><li>one<ol><li>two<ul><li><p>three</p><pre>a\n\n    b\n\nc\n</pre></li></ul></li></ol></li></ol>",
        "1. one\n   1. two\n      - three\n\n        ```\n        a\n\n            b\n\n        c\n        ```\n",
    ),
    tier1_case(
        "an item whose marker is four columns wide",
        "<ol start=\"10\"><li><p>x</p><pre>a\n\n    b\n\nc\n</pre></li></ol>",
        "10. x\n\n    ```\n    a\n\n        b\n\n    c\n    ```\n",
    ),
    tier1_case(
        "tabs after a blank line",
        "<ul><li><p>Run:</p><pre>a:\n\n\tb\n\n\t\tc\n</pre></li></ul>",
        "- Run:\n\n  ```\n  a:\n\n  \tb\n\n  \t\tc\n  ```\n",
    ),
    tier1_case(
        "an item inside a block quote",
        "<blockquote><ul><li><p>Run:</p><pre>a\n\n    b\n\nc\n</pre></li></ul></blockquote>",
        "> - Run:\n>\n>   ```\n>   a\n>\n>       b\n>\n>   c\n>   ```\n",
    ),
    case(
        "a block quote inside an item",
        "<ul><li><blockquote><pre>a\n\n    b\n\nc\n</pre></blockquote></li></ul>",
        "- > ```\n  > a\n  >\n  >     b\n  >\n  > c\n  > ```\n",
    ),
    case(
        "a highlighter wrapper inside an item",
        "<ul><li><div>Run:</div><div class=\"highlight\"><pre>a\n\n    b\n\nc\n</pre></div></li></ul>",
        "- Run:\n\n  ```\n  a\n\n      b\n\n  c\n  ```\n",
    ),
    case(
        "two blocks and a paragraph in one item",
        "<ul><li><p>Run:</p><pre>a\n\n    b\n</pre><p>Then:</p><pre>c\n\n        d\n</pre></li></ul>",
        "- Run:\n\n  ```\n  a\n\n      b\n  ```\n\n  Then:\n\n  ```\n  c\n\n          d\n  ```\n",
    ),
    case(
        "a task item",
        "<ul><li><input type=\"checkbox\"> x<pre>a\n\n    b\n\nc\n</pre></li></ul>",
        "- [ ] x\n\n  ```\n  a\n\n      b\n\n  c\n  ```\n",
    ),
    tier1_case(
        "a block quote",
        "<blockquote><p>Run:</p><pre>a\n\n    b\n\nc\n</pre></blockquote>",
        "> Run:\n>\n> ```\n> a\n>\n>     b\n>\n> c\n> ```\n",
    ),
    tier1_case(
        "a table cell",
        "<table><tr><th>h</th></tr><tr><td><pre>a\n\n    b\n\nc\n</pre></td></tr></table>",
        "| h             |\n| ------------- |\n| `a      b  c` |\n",
    ),
    tier1_case(
        "issue 770, one element with a line break for each line",
        "<pre><code><div class=\"token-line\"><span>npm</span><span> install</span><br></div>\
         <div class=\"token-line\"><span>cd site</span><br></div>\
         <div class=\"token-line\"><span>npm start</span><br></div></code></pre>",
        "```\nnpm install\ncd site\nnpm start\n```\n",
    ),
    tier1_case(
        "one block element for each line and no line break",
        "<pre><code><div class=\"line\">a</div><div class=\"line\">b</div><div class=\"line\">c</div></code></pre>",
        "```\na\nb\nc\n```\n",
    ),
    tier1_case(
        "a line feed between two block elements is an empty line in a browser",
        "<pre><code><div class=\"line\">a</div>\n<div class=\"line\">b</div>\n</code></pre>",
        "```\na\n\nb\n```\n",
    ),
    tier1_case(
        "one inline element for each line and a line feed between them",
        "<pre><code><span class=\"line\"><span>a</span></span>\n<span class=\"line\"></span>\n<span class=\"line\"><span>  b</span></span></code></pre>",
        "```\na\n\n  b\n```\n",
    ),
    tier1_case(
        "one inline element with a line break for each line",
        "<pre><code><span class=\"line\">a<br></span><span class=\"line\"><br></span><span class=\"line\">b<br></span></code></pre>",
        "```\na\n\nb\n```\n",
    ),
    tier1_case(
        "an empty block element is no line",
        "<pre><code><div class=\"line\">a</div><div class=\"line\"></div><div class=\"line\">b</div></code></pre>",
        "```\na\nb\n```\n",
    ),
    tier1_case(
        "a block element with only a line break is an empty line",
        "<pre><code><div class=\"token-line\">def f():<br></div><div class=\"token-line\"><span>    </span>return 1<br></div>\
         <div class=\"token-line\"><br></div><div class=\"token-line\">\treturn 2<br></div></code></pre>",
        "```\ndef f():\n    return 1\n\n\treturn 2\n```\n",
    ),
    tier1_case(
        "two line breaks at the end of a block element",
        "<pre><div>a<br><br></div><div>b</div></pre>",
        "```\na\n\nb\n```\n",
    ),
    tier1_case(
        "a line feed at the end of each block element",
        "<pre><div class=\"ec-line\"><div class=\"code\">a\n</div></div><div class=\"ec-line\"><div class=\"code\">\n</div></div>\
         <div class=\"ec-line\"><div class=\"code\">b\n</div></div></pre>",
        "```\na\n\nb\n```\n",
    ),
    tier1_case(
        "line breaks alone",
        "<pre>a<br>b<br><br>c</pre>",
        "```\na\nb\n\nc\n```\n",
    ),
    tier1_case(
        "text between block elements",
        "<pre>x<div class=\"line\">a</div>y<div class=\"line\">b</div>z</pre>",
        "```\nx\na\ny\nb\nz\n```\n",
    ),
    tier1_case(
        "a line feed before and after a block element",
        "<pre>x\n<div>a</div>\ny</pre>",
        "```\nx\na\n\ny\n```\n",
    ),
    tier1_case(
        "no code element, and indentation that all lines share",
        "<pre><div class=\"line\">    a</div><div class=\"line\">  b</div></pre>",
        "```\n    a\n  b\n```\n",
    ),
    tier1_case(
        "issue 782, four spaces that all lines share",
        "<pre>    a\n      b\n    c\n</pre>",
        "```\n    a\n      b\n    c\n```\n",
    ),
    tier1_case(
        "issue 782, one space that all lines share",
        "<pre> createuser -S wikiuser\n createdb -O wikiuser my_wiki\n</pre>",
        "```\n createuser -S wikiuser\n createdb -O wikiuser my_wiki\n```\n",
    ),
    tier1_case(
        "issue 782, two spaces that all lines share",
        "<pre>  {%- for path in scripts %}\n    &lt;script src=x&gt;\n  {%- endfor %}\n</pre>",
        "```\n  {%- for path in scripts %}\n    <script src=x>\n  {%- endfor %}\n```\n",
    ),
    tier1_case(
        "issue 782, shared indentation in a list item",
        "<ul><li><p>x</p><pre>    a\n      b\n</pre></li></ul>",
        "- x\n\n  ```\n      a\n        b\n  ```\n",
    ),
    tier1_case(
        "issue 783, two blank lines",
        "<pre>import a\n\n\nclass B:\n    pass\n</pre>",
        "```\nimport a\n\n\nclass B:\n    pass\n```\n",
    ),
    tier1_case(
        "issue 783, three blank lines in pre and code",
        "<pre><code>a\n\n\n\nb\n</code></pre>",
        "```\na\n\n\n\nb\n```\n",
    ),
    tier1_case(
        "issue 783, two blank lines in a list item",
        "<ul><li><p>x</p><pre>a\n\n\nb\n</pre></li></ul>",
        "- x\n\n  ```\n  a\n\n\n  b\n  ```\n",
    ),
    case(
        "issue 783, two blank lines in a block on the line of a bullet",
        "<ul><li><pre>a  \n\n\nb\n</pre></li></ul>",
        "- ```\n  a  \n\n\n  b\n  ```\n",
    ),
    case(
        "issue 783, two blank lines in a block on the line of a number",
        "<ol><li><pre>a  \n\n\nb\n</pre></li></ol>",
        "1. ```\n   a  \n\n\n   b\n   ```\n",
    ),
    case(
        "issue 783, two blank lines in a block on the line of a bullet in a block quote",
        "<blockquote><ul><li><pre>a  \n\n\nb\n</pre></li></ul></blockquote>",
        "> - ```\n>   a  \n>\n>\n>   b\n>   ```\n",
    ),
    tier1_case(
        "issue 783, two blank lines in a block quote",
        "<blockquote><pre>a\n\n\nb\n</pre></blockquote>",
        "> ```\n> a\n>\n>\n> b\n> ```\n",
    ),
    tier1_case(
        "issue 783, blank lines fold outside the code of the same document",
        "<p>x</p><br><br><br><pre>a\n\n\nb\n</pre><br><br><br><p>y</p>",
        "x\n\n```\na\n\n\nb\n```\n\ny\n",
    ),
    tier1_case(
        "spaces and a tab at line ends",
        "<pre>a \nb   \n\t\nc</pre><p>x</p>",
        "```\na \nb   \n\t\nc\n```\n\nx\n",
    ),
    tier1_case(
        "a line of code that is a shorter fence",
        "<pre>```\n\n\na  \n```\n</pre>",
        "````\n```\n\n\na  \n```\n````\n",
    ),
    case(
        "issue 781, a link in a block",
        "<pre>author: Guido &lt;<a href=\"/mail\">guido</a>&gt;</pre>",
        "```\nauthor: Guido <guido>\n```\n",
    ),
    case(
        "issue 781, a link in pre and code",
        "<pre><code>see <a href=\"/api\">the API</a> first</code></pre>",
        "```\nsee the API first\n```\n",
    ),
    case(
        "issue 781, a link in a code span",
        "<p><code>see <a href=\"/api\">the API</a></code></p>",
        "`see the API`\n",
    ),
    case(
        "issue 781, an image in a block",
        "<pre>a <img src=\"x.png\" alt=\"pic\"> b</pre>",
        "```\na  b\n```\n",
    ),
    case(
        "issue 781, an inline graphic with text in a block",
        "<pre>a <svg width=\"1\" height=\"1\"><title>T</title><text>x</text></svg> b</pre>",
        "```\na T x b\n```\n",
    ),
    case(
        "issue 781, an inline graphic with text in a code span",
        "<p><code>a <svg width=\"1\" height=\"1\"><text>x</text></svg> b</code></p>",
        "`a x b`\n",
    ),
    case(
        "issue 781, an inline graphic with no text in a block",
        "<pre>a <svg width=\"1\" height=\"1\"><path d=\"M0 0\"/></svg> b</pre>",
        "```\na  b\n```\n",
    ),
    case(
        "issue 781, a link around an inline graphic in a block",
        "<pre>go <a href=\"/p\"><svg width=\"1\" height=\"1\"><text>there</text></svg></a> now</pre>",
        "```\ngo there now\n```\n",
    ),
    case(
        "issue 781, highlighted text and an abbreviation in a block",
        "<pre>a <mark>c</mark> <abbr title=\"t\">e</abbr> <sub>1</sub><sup>2</sup> <ins>d</ins></pre>",
        "```\na c e 12 d\n```\n",
    ),
    tier1_case(
        "bold, emphasis and deleted text in a block",
        "<pre>a <b>b</b> <em>c</em> <del>d</del></pre>",
        "```\na b c d\n```\n",
    ),
    tier1_case(
        "blank lines at the start and at the end",
        "<pre>\n\na\n\n\n</pre>",
        "```\n\na\n```\n",
    ),
    tier1_case(
        "paragraphs keep the empty line of their margin",
        "<pre><p>a</p><p>b</p></pre>",
        "```\na\n\nb\n```\n",
    ),
    tier1_case(
        "sectioning elements are lines too",
        "<pre><section>a</section><article>b</article></pre>",
        "```\na\nb\n```\n",
    ),
    tier1_case(
        "character references in block elements",
        "<pre><div>&lt;a&gt; &amp;&amp; b</div><div>c &#x27;d&#x27;</div></pre>",
        "```\n<a> && b\nc 'd'\n```\n",
    ),
    case(
        "line elements in a block in an ordered item",
        "<ol><li><p>Run:</p><pre><code><div class=\"token-line\">a<br></div><div class=\"token-line\"><br></div>\
         <div class=\"token-line\">    b<br></div></code></pre></li></ol>",
        "1. Run:\n\n   ```\n   a\n\n       b\n   ```\n",
    ),
    tier1_case(
        "line elements in a block in a block quote",
        "<blockquote><pre><code><div class=\"line\">a<br></div><div class=\"line\">  b<br></div></code></pre></blockquote>",
        "> ```\n> a\n>   b\n> ```\n",
    ),
    case(
        "line elements in a block in a table cell",
        "<table><tr><th>h</th></tr><tr><td><pre><code><div class=\"line\">a<br></div><div class=\"line\">b<br></div></code></pre></td></tr></table>",
        "| h     |\n| ----- |\n| `a b` |\n",
    ),
    tier1_case("a block with no content", "<p>x</p><pre></pre><p>y</p>", "x\n\ny\n"),
    tier1_case(
        "one line feed at the start of the block is not a line",
        "<pre>\na\nb\n</pre>",
        "```\na\nb\n```\n",
    ),
    tier1_case(
        "a second line feed at the start of the block is a blank first line",
        "<pre>\n\na\nb\n</pre>",
        "```\n\na\nb\n```\n",
    ),
    tier1_case(
        "three line feeds at the start of the block are two blank lines",
        "<pre>\n\n\na</pre>",
        "```\n\n\na\n```\n",
    ),
    tier1_case(
        "a blank first line of a block after text in a list item",
        "<ul><li>t<pre>\n\na\nb\n</pre></li></ul>",
        "- t\n\n  ```\n\n  a\n  b\n  ```\n",
    ),
    case(
        "a blank first line of a block that starts a list item",
        "<ul><li><pre>\n\na\nb\n</pre></li></ul>",
        "- ```\n\n  a\n  b\n  ```\n",
    ),
    tier1_case(
        "a blank first line of a block in a block quote",
        "<blockquote><pre>\n\na\nb\n</pre></blockquote>",
        "> ```\n>\n> a\n> b\n> ```\n",
    ),
    tier1_case(
        "a language class on the code element",
        "<pre><code class=\"language-python\">x = 1\n</code></pre>",
        "```python\nx = 1\n```\n",
    ),
    case(
        "a link in code whose text is its address",
        "<pre>see <a href=\"https://example.com/a_b\">https://example.com/a_b</a></pre>",
        "```\nsee https://example.com/a_b\n```\n",
    ),
    tier1_case(
        "a line of three backticks in the code, then a second block",
        "<pre>a\n```\nb\n\n\nc\n</pre><p>x</p><pre>d\n\n\ne\n</pre>",
        "````\na\n```\nb\n\n\nc\n````\n\nx\n\n```\nd\n\n\ne\n```\n",
    ),
    tier1_case(
        "a line of five backticks in the code",
        "<pre>`````\na\n\n\nb\n</pre>",
        "``````\n`````\na\n\n\nb\n``````\n",
    ),
    tier1_case(
        "a line of tildes in the code of a backtick fence",
        "<pre>~~~\na\n\n\nb\n</pre><p>x</p><pre>c\n\n\nd\n</pre>",
        "```\n~~~\na\n\n\nb\n```\n\nx\n\n```\nc\n\n\nd\n```\n",
    ),
];

fn options(tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        tier_strategy,
        ..ConversionOptions::default()
    }
}

fn converted(html: &str, options: Option<ConversionOptions>) -> String {
    convert(html, options)
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn assert_none(failures: &[String], checked: usize) {
    assert!(
        failures.is_empty(),
        "{} of {checked} checks differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn should_keep_the_indentation_of_a_line_after_a_blank_line_in_a_list_item() {
    let html =
        "<ol><li><p>Save this:</p><pre>def f(x):\n    if x:\n\n        return 1\n\n    return 2\n</pre></li></ol>";
    assert_eq!(
        converted(html, None),
        "1. Save this:\n\n   ```\n   def f(x):\n       if x:\n\n           return 1\n\n       return 2\n   ```\n"
    );
}

#[test]
fn should_write_one_line_for_a_line_element_that_ends_with_a_line_break() {
    let html = "<pre><code><div class=\"token-line\"><span>npm</span><span> install</span><br></div>\
                <div class=\"token-line\"><span>cd site</span><br></div>\
                <div class=\"token-line\"><span>npm start</span><br></div></code></pre>";
    assert_eq!(converted(html, None), "```\nnpm install\ncd site\nnpm start\n```\n");
}

#[test]
fn should_write_each_code_block_as_the_lines_of_the_page() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for case in CASES {
        for (label, options) in [
            ("default options", None),
            ("tier 2", Some(options(TierStrategy::Tier2))),
        ] {
            checked += 1;
            let actual = converted(case.html, options);
            if actual != case.expected {
                failures.push(format!(
                    "{} ({label}):\n  expected {:?}\n  actual   {actual:?}",
                    case.name, case.expected
                ));
            }
        }
    }
    assert_eq!(checked, 2 * CASES.len());
    assert_none(&failures, checked);
}

#[test]
fn should_indent_every_line_of_an_indented_code_block_in_a_list_item() {
    let html = "<ol><li><p>Save this:</p><pre>def f(x):\n    if x:\n\n        return 1\n</pre></li></ol>";
    let options = ConversionOptions {
        code_block_style: CodeBlockStyle::Indented,
        ..ConversionOptions::default()
    };
    assert_eq!(
        converted(html, Some(options)),
        "1. Save this:\n\n       def f(x):\n           if x:\n\n               return 1\n"
    );
}

/// The tiers a test can ask for by name: Tier-1 only where the test kit exposes it.
fn tiers() -> Vec<TierStrategy> {
    vec![
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
    ]
}

/// Converts each input in `style` on every tier and reports the outputs that differ.
fn assert_style(style: CodeBlockStyle, cases: &[(&str, &str)]) {
    let mut failures = Vec::new();
    let mut checked = 0;
    for (html, expected) in cases {
        for tier_strategy in tiers() {
            checked += 1;
            let options = ConversionOptions {
                code_block_style: style,
                ..options(tier_strategy)
            };
            let actual = converted(html, Some(options));
            if actual != *expected {
                failures.push(format!(
                    "{html:?} ({style:?}, {tier_strategy:?}):\n  expected {expected:?}\n  actual   {actual:?}"
                ));
            }
        }
    }
    assert_eq!(checked, cases.len() * tiers().len());
    assert_none(&failures, checked);
}

#[test]
fn should_keep_the_lines_of_an_indented_code_block() {
    assert_style(
        CodeBlockStyle::Indented,
        &[
            (
                "<pre>a   \n\n\nb\n</pre><p>x</p><pre>c \t\n\n\n\nd   \n</pre>",
                "    a   \n\n\n    b\n\nx\n\n    c \t\n\n\n\n    d   \n",
            ),
            (
                "<pre>a   \n```\n\n\nb\n</pre><p>x</p><pre>c   \n~~~~\n\n\nd\n</pre>",
                "    a   \n    ```\n\n\n    b\n\nx\n\n    c   \n    ~~~~\n\n\n    d\n",
            ),
            (
                "<blockquote><pre>a   \n\n\nb\n</pre></blockquote>",
                ">     a   \n>\n>\n>     b\n",
            ),
            (
                "<blockquote><pre>a   \n\n\nb   \n</pre></blockquote>",
                ">     a   \n>\n>\n>     b   \n",
            ),
            (
                "<blockquote><p>t</p><pre>a   \n\n\nb\n</pre><p>u</p></blockquote>",
                "> t\n>\n>     a   \n>\n>\n>     b\n>\n> u\n",
            ),
            (
                "<ul><li><p>t</p><pre>a   \n\n\nb\n</pre></li></ul>",
                "- t\n\n      a   \n\n\n      b\n",
            ),
            (
                "<ul><li><pre>a   \n\n\nb\n</pre></li></ul>",
                "-     a   \n\n\n      b\n",
            ),
            (
                "<blockquote><ul><li><pre>a   \n\n\nb\n</pre></li></ul></blockquote>",
                "> -     a   \n>\n>\n>       b\n",
            ),
        ],
    );
}

#[test]
fn should_keep_a_line_of_spaces_of_a_code_block_in_a_block_quote() {
    let html = "<blockquote><pre>a\n   \nb\n</pre></blockquote>";
    assert_style(CodeBlockStyle::Indented, &[(html, ">     a\n>        \n>     b\n")]);
    assert_style(CodeBlockStyle::Backticks, &[(html, "> ```\n> a\n>    \n> b\n> ```\n")]);
}

#[test]
fn should_keep_the_first_line_of_an_indented_code_block_in_a_wrapper_in_a_block_quote() {
    assert_style(
        CodeBlockStyle::Indented,
        &[
            (
                "<blockquote>\n<div><pre>a\nb\n</pre></div></blockquote>",
                ">     a\n>     b\n",
            ),
            (
                "<blockquote><blockquote><pre>a\nb\n</pre></blockquote></blockquote>",
                "> >     a\n> >     b\n",
            ),
        ],
    );
}

#[test]
fn should_not_read_text_that_starts_with_spaces_in_a_block_quote_as_code() {
    for tier_strategy in tiers() {
        let options = ConversionOptions {
            whitespace_mode: WhitespaceMode::Strict,
            code_block_style: CodeBlockStyle::Indented,
            ..options(tier_strategy)
        };
        let actual = converted("<blockquote><p>      text</p></blockquote>", Some(options));
        assert_eq!(actual, "> text\n", "{tier_strategy:?}");
    }
}

#[test]
fn should_not_read_text_before_a_code_block_in_a_block_quote_as_code() {
    for tier_strategy in tiers() {
        let options = ConversionOptions {
            whitespace_mode: WhitespaceMode::Strict,
            code_block_style: CodeBlockStyle::Indented,
            ..options(tier_strategy)
        };
        let actual = converted("<blockquote>      text<pre>a\n</pre></blockquote>", Some(options));
        assert_eq!(actual.lines().next(), Some("> text"), "{tier_strategy:?}: {actual:?}");
        assert_eq!(actual.lines().last(), Some(">     a"), "{tier_strategy:?}: {actual:?}");
    }
}

#[test]
fn should_keep_the_first_line_of_an_indented_code_block_in_a_definition() {
    assert_style(
        CodeBlockStyle::Indented,
        &[
            (
                "<dl><dt>t</dt><dd><pre>a   \n\n\nb\n</pre></dd></dl>",
                "t\n\n    a   \n\n\n    b\n",
            ),
            (
                "<dl><dt>t</dt><dd><div><pre>a\nb   \n</pre></div></dd></dl>",
                "t\n\n    a\n    b   \n",
            ),
            (
                "<dl><dt>t</dt><dd><pre>a\n</pre><p>u</p></dd></dl>",
                "t\n\n    a\n\nu\n",
            ),
            ("<dl><dd><pre>a\nb\n</pre></dd></dl>", "    a\n    b\n"),
        ],
    );
}

#[test]
fn should_not_read_text_that_starts_with_spaces_in_a_definition_as_code() {
    for tier_strategy in tiers() {
        let options = ConversionOptions {
            whitespace_mode: WhitespaceMode::Strict,
            code_block_style: CodeBlockStyle::Indented,
            ..options(tier_strategy)
        };
        let actual = converted("<dl><dt>t</dt><dd><p>      text</p></dd></dl>", Some(options));
        let lines: Vec<&str> = actual.lines().filter(|line| !line.is_empty()).collect();
        assert_eq!(lines, ["t", "text"], "{tier_strategy:?}");
    }
}

#[test]
fn should_not_close_a_fenced_block_on_a_longer_fence_line_of_the_code_in_a_container() {
    assert_style(
        CodeBlockStyle::Backticks,
        &[
            (
                "<blockquote><pre>a   \n````\nb   \n\n\nc\n</pre></blockquote>",
                "> `````\n> a   \n> ````\n> b   \n>\n>\n> c\n> `````\n",
            ),
            (
                "<ul><li><p>t</p><pre>a   \n````\nb   \n\n\nc\n</pre></li></ul>",
                "- t\n\n  `````\n  a   \n  ````\n  b   \n\n\n  c\n  `````\n",
            ),
            (
                "<blockquote><ul><li><pre>a   \n````\nb   \n\n\nc\n</pre></li></ul></blockquote>",
                "> - `````\n>   a   \n>   ````\n>   b   \n>\n>\n>   c\n>   `````\n",
            ),
        ],
    );
}

#[test]
fn should_not_close_a_fenced_block_on_a_fence_line_of_the_code() {
    assert_style(
        CodeBlockStyle::Backticks,
        &[
            (
                "<pre>a   \n```\nb   \n\n\nc\n````\nd   \n\n\ne\n</pre><p>x</p><pre>f   \n\n\ng\n</pre>",
                "`````\na   \n```\nb   \n\n\nc\n````\nd   \n\n\ne\n`````\n\nx\n\n```\nf   \n\n\ng\n```\n",
            ),
            (
                "<blockquote><pre>a   \n```\nb   \n\n\nc\n</pre></blockquote>",
                "> ````\n> a   \n> ```\n> b   \n>\n>\n> c\n> ````\n",
            ),
            (
                "<ul><li><p>t</p><pre>a   \n```\nb   \n\n\nc\n</pre></li></ul>",
                "- t\n\n  ````\n  a   \n  ```\n  b   \n\n\n  c\n  ````\n",
            ),
        ],
    );
    assert_style(
        CodeBlockStyle::Tildes,
        &[(
            "<pre>a   \n~~~\nb   \n\n\nc\n~~~~\nd   \n\n\ne\n</pre><p>x</p><pre>f   \n\n\ng\n</pre>",
            "~~~~~\na   \n~~~\nb   \n\n\nc\n~~~~\nd   \n\n\ne\n~~~~~\n\nx\n\n~~~\nf   \n\n\ng\n~~~\n",
        )],
    );
}

#[test]
fn should_keep_the_lines_of_a_fenced_block_in_a_block_quote_and_in_a_list_item() {
    for (style, fence) in [(CodeBlockStyle::Backticks, "```"), (CodeBlockStyle::Tildes, "~~~")] {
        let quote = format!("> {fence}\n> a   \n>\n>\n> b\n> {fence}\n");
        let item = format!("- t\n\n  {fence}\n  a   \n\n\n  b\n  {fence}\n");
        let first_in_item = format!("- {fence}\n  a   \n\n\n  b\n  {fence}\n");
        let item_in_quote = format!("> - {fence}\n>   a   \n>\n>\n>   b\n>   {fence}\n");
        assert_style(
            style,
            &[
                ("<blockquote><pre>a   \n\n\nb\n</pre></blockquote>", &quote),
                ("<ul><li><p>t</p><pre>a   \n\n\nb\n</pre></li></ul>", &item),
                ("<ul><li><pre>a   \n\n\nb\n</pre></li></ul>", &first_in_item),
                (
                    "<blockquote><ul><li><pre>a   \n\n\nb\n</pre></li></ul></blockquote>",
                    &item_in_quote,
                ),
            ],
        );
    }
}

#[test]
fn should_not_open_a_block_on_a_code_span_that_holds_a_fence() {
    assert_style(
        CodeBlockStyle::Backticks,
        &[
            // ~keep One backtick delimits a span that holds a run of three (CommonMark 6.1).
            (
                "<p><code>```</code></p><pre>a   \n\n\nb\n</pre><p><code>```</code> y</p><pre>c   \n\n\nd\n</pre>",
                "` ``` `\n\n```\na   \n\n\nb\n```\n\n` ``` ` y\n\n```\nc   \n\n\nd\n```\n",
            ),
            // ~keep The span holds runs of one and two, so three backticks start its line. A
            // ~keep fence cannot have a backtick after its run (CommonMark 4.5): the line is text.
            (
                "<p><code>a`b``c</code></p><pre>a   \n\n\nb\n</pre><p>x   </p>",
                "```a`b``c```\n\n```\na   \n\n\nb\n```\n\nx\n",
            ),
        ],
    );
    assert_style(
        CodeBlockStyle::Tildes,
        &[(
            "<p><code>~~~</code></p><pre>a   \n\n\nb\n</pre>",
            "`~~~`\n\n~~~\na   \n\n\nb\n~~~\n",
        )],
    );
}

#[test]
fn should_grow_a_tilde_fence_past_the_tildes_in_the_code() {
    let cases = [
        (
            "<pre>a\n~~~~\nb\n\n\nc\n</pre><p>x</p><pre>d\n\n\ne\n</pre>",
            "~~~~~\na\n~~~~\nb\n\n\nc\n~~~~~\n\nx\n\n~~~\nd\n\n\ne\n~~~\n",
        ),
        (
            "<pre>```\na\n\n\nb\n</pre><p>x</p><pre>c\n\n\nd\n</pre>",
            "~~~\n```\na\n\n\nb\n~~~\n\nx\n\n~~~\nc\n\n\nd\n~~~\n",
        ),
    ];
    for (html, expected) in cases {
        for tier_strategy in tiers() {
            let options = ConversionOptions {
                code_block_style: CodeBlockStyle::Tildes,
                ..options(tier_strategy)
            };
            assert_eq!(converted(html, Some(options)), expected, "{tier_strategy:?}");
        }
    }
}

#[test]
fn should_keep_the_blank_lines_of_a_block_that_follows_fence_marks_in_running_text() {
    for html in [
        "<p>```</p><pre>a\n\n\nb\n</pre>",
        "<p>~~~</p><pre>a\n\n\nb\n</pre>",
        "<ul><li>```</li></ul><pre>a\n\n\nb\n</pre>",
    ] {
        for tier_strategy in tiers() {
            let actual = converted(html, Some(options(tier_strategy)));
            assert!(
                actual.ends_with("\n\n```\na\n\n\nb\n```\n"),
                "{html} ({tier_strategy:?}): {actual:?}"
            );
        }
    }
}

#[test]
fn should_keep_the_blank_lines_of_a_block_on_the_line_of_a_long_item_number() {
    let html = "<ol start=\"100\"><li><pre>a\n\n\nb\n</pre></li></ol>";
    for tier_strategy in tiers() {
        let actual = converted(html, Some(options(tier_strategy)));
        assert!(actual.starts_with("100. ```\n"), "{tier_strategy:?}: {actual:?}");
        assert!(actual.contains("a\n\n\n"), "{tier_strategy:?}: {actual:?}");
    }
}

#[test]
fn should_keep_the_white_space_of_text_outside_a_list_item_in_strict_white_space_mode() {
    let options = ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        ..options(TierStrategy::Tier2)
    };
    let actual = converted("<div>a\n\n  b</div>", Some(options));
    assert!(actual.contains("a\n\n  b"), "{actual:?}");
}

#[cfg(feature = "visitor")]
mod visitor_in_code {
    #![allow(clippy::significant_drop_tightening)]

    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
    use html_to_markdown_rs::{ConversionOptions, convert};
    use std::sync::{Arc, Mutex};

    const HTML: &str = "<pre><a href=\"/x\">link</a> <mark>hot</mark><img src=\"i.png\" alt=\"pic\"> end</pre>\
                        <p><code>b <a href=\"/y\">two</a></code></p>";

    #[derive(Debug, Default)]
    struct Recorder {
        custom: bool,
        seen: Vec<String>,
    }

    impl Recorder {
        fn answer(&self, custom: String) -> VisitResult {
            if self.custom {
                VisitResult::Custom(custom)
            } else {
                VisitResult::Continue
            }
        }
    }

    impl HtmlVisitor for Recorder {
        fn visit_link(&mut self, _ctx: &NodeContext<'_>, href: &str, text: &str, _title: Option<&str>) -> VisitResult {
            self.seen.push(format!("link {href} {text}"));
            self.answer(format!("<{text}@{href}>"))
        }

        fn visit_mark(&mut self, _ctx: &NodeContext<'_>, text: &str) -> VisitResult {
            self.seen.push(format!("mark {text}"));
            self.answer(format!("[{text}]"))
        }

        fn visit_image(&mut self, _ctx: &NodeContext<'_>, src: &str, alt: &str, _title: Option<&str>) -> VisitResult {
            self.seen.push(format!("image {src} {alt}"));
            self.answer(format!("{{{alt}}}"))
        }
    }

    fn converted_with(custom: bool) -> (String, Vec<String>) {
        let recorder = Arc::new(Mutex::new(Recorder {
            custom,
            seen: Vec::new(),
        }));
        let handle: VisitorHandle = recorder.clone();
        let options = ConversionOptions {
            extract_metadata: false,
            visitor: Some(handle),
            ..ConversionOptions::default()
        };
        let content = convert(HTML, Some(options))
            .expect("conversion must succeed")
            .content
            .unwrap_or_default();
        let seen = recorder.lock().expect("the visitor lock").seen.clone();
        (content, seen)
    }

    #[test]
    fn should_ask_the_visitor_about_a_link_a_highlight_and_an_image_in_code() {
        let (content, seen) = converted_with(false);
        assert_eq!(
            seen,
            ["link /x link", "mark hot", "image i.png pic", "link /y two"].map(String::from)
        );
        assert_eq!(content, "```\nlink hot end\n```\n\n`b two`\n");
    }

    #[test]
    fn should_write_in_code_what_the_visitor_returns() {
        let (content, seen) = converted_with(true);
        assert_eq!(content, "```\n<link@/x> [hot]{pic} end\n```\n\n`b <two@/y>`\n");
        assert_eq!(seen.len(), 4);
    }
}

#[cfg(feature = "metadata")]
#[test]
fn should_keep_a_link_of_a_code_block_in_the_metadata() {
    let html = "<pre>author: Guido &lt;<a href=\"/mail\">guido</a>&gt;</pre>";
    let result = convert(html, None).expect("conversion must succeed");
    assert_eq!(result.content.as_deref(), Some("```\nauthor: Guido <guido>\n```\n"));
    let links: Vec<(String, String)> = result
        .metadata
        .links
        .iter()
        .map(|link| (link.href.clone(), link.text.clone()))
        .collect();
    assert_eq!(links, [("/mail".to_string(), "guido".to_string())]);
}

#[test]
fn should_keep_the_break_between_line_elements_of_a_code_block_in_a_table_cell() {
    let html = "<table><tr><th>h</th></tr><tr><td><pre><div>a</div><div>b</div></pre></td></tr></table>";
    let mut outputs = Vec::new();
    for tier_strategy in tiers() {
        let options = ConversionOptions {
            br_in_tables: true,
            ..options(tier_strategy)
        };
        outputs.push(converted(html, Some(options)));
    }
    for output in &outputs {
        assert_eq!(output, "| h          |\n| ---------- |\n| `a`<br>`b` |\n");
    }
}

#[cfg(feature = "testkit")]
#[test]
fn should_write_the_same_code_block_on_both_tiers() {
    use html_to_markdown_rs::prescan::PrescanReport;
    use html_to_markdown_rs::tier1;

    let mut failures = Vec::new();
    let mut checked = 0;
    let mut rendered_by_tier1 = 0;
    for case in CASES {
        checked += 1;
        let tier1_output = converted(case.html, Some(options(TierStrategy::Tier1)));
        let tier2_output = converted(case.html, Some(options(TierStrategy::Tier2)));
        if tier1_output != tier2_output {
            failures.push(format!(
                "{}:\n  tier 1 {tier1_output:?}\n  tier 2 {tier2_output:?}",
                case.name
            ));
        }
        let direct = tier1::run(case.html, &PrescanReport::default(), &options(TierStrategy::Tier1));
        match (case.tier1_renders, direct) {
            (true, Ok(markdown)) => {
                rendered_by_tier1 += 1;
                if markdown != case.expected {
                    failures.push(format!(
                        "{} (tier 1 with no fallback):\n  expected {:?}\n  actual   {markdown:?}",
                        case.name, case.expected
                    ));
                }
            }
            (true, Err(reason)) => failures.push(format!("{}: tier 1 fell back to tier 2: {reason}", case.name)),
            (false, Ok(markdown)) => {
                if markdown != case.expected {
                    failures.push(format!(
                        "{} (tier 1 with no fallback, not expected to render):\n  expected {:?}\n  actual   {markdown:?}",
                        case.name, case.expected
                    ));
                }
            }
            (false, Err(_)) => {}
        }
    }
    assert_eq!(checked, CASES.len());
    assert_none(&failures, checked);
    assert_eq!(
        rendered_by_tier1,
        CASES.iter().filter(|case| case.tier1_renders).count()
    );
    assert!(
        rendered_by_tier1 > 20,
        "tier 1 rendered only {rendered_by_tier1} inputs"
    );
}

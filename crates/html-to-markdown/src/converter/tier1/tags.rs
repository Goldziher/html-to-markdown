//! Compile-time table mapping tag-name bytes → semantic spec consulted by the Tier-1 byte scanner (M3c).
//!
//! All lookups go through `lookup(tag_name_bytes)`; callers are expected to lowercase their input first.

use phf::phf_map;

/// Coarse semantic category of an HTML element.
///
/// `Block` and `Inline` are the catch-all defaults; specific kinds get their
/// own variant so the scanner / emit dispatch can branch cleanly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
    /// Generic block container (`<div>`, `<section>`, etc.).
    Block,
    /// Paragraph (`<p>`).
    Paragraph,
    /// Heading level 1–6.
    Heading(u8),
    /// Block quotation (`<blockquote>`).
    Blockquote,
    /// Horizontal rule (`<hr>`).
    Hr,
    /// Code-block container.
    Pre,

    /// Generic inline container (`<span>`, etc.).
    Inline,
    /// Bold/strong emphasis (`<strong>`, `<b>`).
    Strong,
    /// Italic/emphasis (`<em>`, `<i>`).
    Emphasis,
    /// Strikethrough (`<del>`, `<s>`, `<strike>`).  Emits `~~content~~` in
    /// GFM/CommonMark output.  Mirrors Tier-2's
    /// `inline/semantic/marks.rs::handle_strikethrough`.
    Strikethrough,
    /// Inserted/underlined text (`<ins>`).  Emits `==content==` in
    /// GFM/CommonMark output.  Mirrors Tier-2's
    /// `inline/semantic/marks.rs::handle_inserted`.
    Inserted,
    /// Inline `code`.
    Code,
    /// `<br>` — forces a line break.
    LineBreak,
    /// `<a>` — hyperlink.
    Link,
    /// `<img>` — embedded image.
    Image,

    /// Ordered or unordered list container.
    List(ListKind),
    /// List item (`<li>`).
    ListItem,
    /// Definition term (`<dt>`).
    DefinitionTerm,
    /// Definition description (`<dd>`).
    DefinitionDescription,

    /// Table root (`<table>`).
    Table,
    /// Table header section (`<thead>`).
    TableHead,
    /// Table body section (`<tbody>`).
    TableBody,
    /// Table footer section (`<tfoot>`).
    TableFoot,
    /// Table row (`<tr>`).
    TableRow,
    /// Table cell; `is_header` distinguishes `<th>` from `<td>`.
    TableCell {
        /// True for `<th>` header cells, false for `<td>` data cells.
        is_header: bool,
    },
    /// Table caption (`<caption>`).
    TableCaption,

    /// `<summary>` — disclosure summary; content collected into a buffer and
    /// emitted as `**…**\n\n` (strong-wrap).
    Summary,
    /// `<figcaption>` — figure caption; content collected into a buffer and
    /// emitted as `*…*\n\n` (italic-wrap). Mirrors Tier-2's
    /// `semantic/figure.rs::handle_figcaption`.
    Figcaption,
    /// `<button>` — clickable button. Tier-2 (`form/elements.rs`'s `handle_button`)
    /// emits `\n\n` after the button's content in block mode and no block separator before
    /// it; mirror with close-only block-separator semantics.
    Button,

    /// Raw-text container whose content the scanner skips until the matching close tag.
    RawText(RawKind),

    /// Explicitly drop (e.g. `<script>`, `<style>`).
    Ignored,
}

/// List flavour for `TagKind::List`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    /// Bullet list (`<ul>`).
    Unordered,
    /// Numbered list (`<ol>`).
    Ordered,
    /// Definition list (`<dl>`).
    Definition,
}

/// Raw-text sub-type for `TagKind::RawText`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawKind {
    /// `<script>` element.
    Script,
    /// `<style>` element.
    Style,
    /// `<textarea>` element.
    Textarea,
    /// `<title>` element.
    Title,
    /// `<xmp>` element (obsolete preformatted text).
    Xmp,
    /// `<iframe>` element.
    Iframe,
    /// `<noscript>` element.
    Noscript,
    /// `<noembed>` element.
    NoEmbed,
    /// `<noframes>` element.
    NoFrames,
}

/// One row of the static tag spec table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagSpec {
    /// Coarse semantic category.
    pub kind: TagKind,
    /// HTML void element — no closing tag, no children.
    pub is_void: bool,
    /// True for block-level elements (HTML5 flow content that establishes a
    /// block formatting context); false for inline/text-level elements.
    pub is_block: bool,
    /// HTML5 optional-close-tag rule, if any.
    pub optional_close: Option<OptionalCloseRule>,
    /// If true, the scanner enters rawtext mode and skips contents until a
    /// matching close tag.  Set for `script`, `style`, `textarea`, `title`,
    /// `xmp`.
    pub is_rawtext: bool,
}

/// HTML5 optional-close-tag rules.
///
/// The scanner consults this when it sees a new open tag — if the new tag's
/// rule matches the currently-open tag, the open tag is implicitly closed.
/// See spec: <https://html.spec.whatwg.org/multipage/syntax.html#optional-tags>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalCloseRule {
    /// `<li>` closes any open `<li>`.
    CloseSameKind,
    /// `<p>` closes when a block-level child opens.
    CloseOnBlockChild,
    /// `<dt>`/`<dd>` close any open `<dt>`/`<dd>`.
    CloseSiblingDtDd,
    /// `<tr>` closes any open `<tr>` (after closing any open `<td>`/`<th>`).
    CloseTableRow,
    /// `<td>`/`<th>` close any open cell.
    CloseTableCell,
    /// `<option>` closes any open `<option>`.
    CloseOption,
    /// First `<tr>` inside `<table>` without `<tbody>`/`<thead>`/`<tfoot>` →
    /// the scanner emits an implicit `<tbody>` open.
    ImplicitTbody,
}

/// Look up a tag by its lowercased name bytes.
///
/// Returns `None` for unknown tags.  Callers **must** lowercase the tag name
/// before calling; the table keys are all ASCII-lowercase.
#[inline]
#[must_use]
pub fn lookup(tag_name_lower: &[u8]) -> Option<&'static TagSpec> {
    TAGS.get(tag_name_lower)
}

const fn block(kind: TagKind) -> TagSpec {
    TagSpec {
        kind,
        is_void: false,
        is_block: true,
        optional_close: None,
        is_rawtext: false,
    }
}

const fn block_opt(kind: TagKind, rule: OptionalCloseRule) -> TagSpec {
    TagSpec {
        kind,
        is_void: false,
        is_block: true,
        optional_close: Some(rule),
        is_rawtext: false,
    }
}

const fn void_block(kind: TagKind) -> TagSpec {
    TagSpec {
        kind,
        is_void: true,
        is_block: true,
        optional_close: None,
        is_rawtext: false,
    }
}

const fn void_inline(kind: TagKind) -> TagSpec {
    TagSpec {
        kind,
        is_void: true,
        is_block: false,
        optional_close: None,
        is_rawtext: false,
    }
}

const fn inline(kind: TagKind) -> TagSpec {
    TagSpec {
        kind,
        is_void: false,
        is_block: false,
        optional_close: None,
        is_rawtext: false,
    }
}

const fn rawtext_ignored() -> TagSpec {
    TagSpec {
        kind: TagKind::Ignored,
        is_void: false,
        is_block: true,
        optional_close: None,
        is_rawtext: true,
    }
}

const fn rawtext_block(kind: TagKind) -> TagSpec {
    TagSpec {
        kind,
        is_void: false,
        is_block: true,
        optional_close: None,
        is_rawtext: true,
    }
}

const fn ignored_block() -> TagSpec {
    TagSpec {
        kind: TagKind::Ignored,
        is_void: false,
        is_block: true,
        optional_close: None,
        is_rawtext: false,
    }
}

const fn ignored_void() -> TagSpec {
    TagSpec {
        kind: TagKind::Ignored,
        is_void: true,
        is_block: false,
        optional_close: None,
        is_rawtext: false,
    }
}

static TAGS: phf::Map<&'static [u8], TagSpec> = phf_map! {
    b"html"    => block(TagKind::Block),
    b"head"    => ignored_block(),
    b"body"    => block(TagKind::Block),
    b"meta"    => ignored_void(),
    b"link"    => ignored_void(),
    b"base"    => void_block(TagKind::Block),

    b"div"     => block(TagKind::Block),
    b"section" => block(TagKind::Block),
    b"article" => block(TagKind::Block),
    b"header"  => block(TagKind::Block),
    b"footer"  => block(TagKind::Block),
    b"aside"   => block(TagKind::Block),
    b"main"    => block(TagKind::Block),
    b"nav"     => block(TagKind::Block),
    b"address" => block(TagKind::Block),
    b"details" => block(TagKind::Block),
    b"summary" => block(TagKind::Summary),
    b"dialog"  => block(TagKind::Block),
    b"figure"  => block(TagKind::Block),
    b"figcaption" => block(TagKind::Figcaption),

    b"p" => block_opt(TagKind::Paragraph, OptionalCloseRule::CloseOnBlockChild),

    b"h1" => block(TagKind::Heading(1)),
    b"h2" => block(TagKind::Heading(2)),
    b"h3" => block(TagKind::Heading(3)),
    b"h4" => block(TagKind::Heading(4)),
    b"h5" => block(TagKind::Heading(5)),
    b"h6" => block(TagKind::Heading(6)),

    b"blockquote" => block(TagKind::Blockquote),

    b"hr" => void_block(TagKind::Hr),

    b"pre" => block(TagKind::Pre),

    b"br" => TagSpec {
        kind: TagKind::LineBreak,
        is_void: true,
        is_block: false,
        optional_close: None,
        is_rawtext: false,
    },

    b"strong" => inline(TagKind::Strong),
    b"b"      => inline(TagKind::Strong),
    b"em"     => inline(TagKind::Emphasis),
    b"i"      => inline(TagKind::Emphasis),
    b"s"      => inline(TagKind::Strikethrough),
    b"del"    => inline(TagKind::Strikethrough),
    b"strike" => inline(TagKind::Strikethrough),
    b"u"      => inline(TagKind::Inline),
    b"ins"    => inline(TagKind::Inserted),

    b"mark"   => inline(TagKind::Inline),
    b"small"  => inline(TagKind::Inline),
    b"sub"    => inline(TagKind::Inline),
    b"sup"    => inline(TagKind::Inline),
    b"kbd"    => inline(TagKind::Code),
    b"code"   => inline(TagKind::Code),
    b"samp"   => inline(TagKind::Code),
    // ~keep Tier-2 wraps <var>/<dfn> in the same single `strong_em_symbol` pair <em>/<i> use
    // ~keep (`typography::handle_variable`), and the router only picks Tier-1 when that symbol
    // ~keep is the default `*`, so Emphasis reproduces them exactly. Classifying them as plain
    // ~keep Inline silently dropped both markers.
    b"var"    => inline(TagKind::Emphasis),
    b"dfn"    => inline(TagKind::Emphasis),
    b"abbr"   => inline(TagKind::Inline),
    b"span"   => inline(TagKind::Inline),
    b"bdi"    => inline(TagKind::Inline),
    b"bdo"    => inline(TagKind::Inline),
    b"cite"   => inline(TagKind::Inline),
    b"q"      => inline(TagKind::Inline),
    b"time"   => inline(TagKind::Inline),
    b"data"   => inline(TagKind::Inline),

    b"a"   => inline(TagKind::Link),
    b"img" => void_inline(TagKind::Image),

    b"ul" => block(TagKind::List(ListKind::Unordered)),
    b"ol" => block(TagKind::List(ListKind::Ordered)),
    b"dl" => block(TagKind::List(ListKind::Definition)),
    b"li" => block_opt(TagKind::ListItem, OptionalCloseRule::CloseSameKind),
    b"dt" => block_opt(TagKind::DefinitionTerm, OptionalCloseRule::CloseSiblingDtDd),
    b"dd" => block_opt(TagKind::DefinitionDescription, OptionalCloseRule::CloseSiblingDtDd),

    b"table"    => block(TagKind::Table),
    b"thead"    => block(TagKind::TableHead),
    b"tbody"    => block(TagKind::TableBody),
    b"tfoot"    => block(TagKind::TableFoot),
    b"tr"       => block_opt(TagKind::TableRow, OptionalCloseRule::CloseTableRow),
    b"th"       => block_opt(TagKind::TableCell { is_header: true  }, OptionalCloseRule::CloseTableCell),
    b"td"       => block_opt(TagKind::TableCell { is_header: false }, OptionalCloseRule::CloseTableCell),
    b"caption"  => block(TagKind::TableCaption),
    b"colgroup" => block(TagKind::Block),
    b"col"      => void_block(TagKind::Block),

    b"script"   => rawtext_ignored(),
    b"style"    => rawtext_ignored(),
    b"title"    => rawtext_block(TagKind::RawText(RawKind::Title)),
    b"xmp"      => rawtext_block(TagKind::RawText(RawKind::Xmp)),
    b"textarea" => rawtext_block(TagKind::RawText(RawKind::Textarea)),
    b"iframe"   => rawtext_block(TagKind::RawText(RawKind::Iframe)),
    b"noscript" => rawtext_block(TagKind::RawText(RawKind::Noscript)),
    b"noembed"  => rawtext_block(TagKind::RawText(RawKind::NoEmbed)),
    b"noframes" => rawtext_block(TagKind::RawText(RawKind::NoFrames)),

    b"form"      => block(TagKind::Block),
    b"fieldset"  => block(TagKind::Block),
    b"legend"    => block(TagKind::Summary),
    b"label"     => inline(TagKind::Inline),
    b"input"     => void_inline(TagKind::Inline),
    b"select"    => inline(TagKind::Inline),
    b"option"    => TagSpec {
        kind: TagKind::Inline,
        is_void: false,
        is_block: false,
        optional_close: Some(OptionalCloseRule::CloseOption),
        is_rawtext: false,
    },
    b"optgroup"  => inline(TagKind::Inline),
    // ~keep <button>: Tier-2 form/elements.rs's handle_button emits `\n\n` after content in
    // ~keep block mode but no leading separator.  TagKind::Button gives us
    // ~keep close-only block-separator semantics — distinct from Block which also
    // ~keep emits a leading `\n\n` on open.
    b"button"    => block(TagKind::Button),
    // ~keep Tier-2 writes these three with the function that writes a button's content.
    b"progress"  => block(TagKind::Button),
    b"meter"     => block(TagKind::Button),
    b"output"    => block(TagKind::Button),
    b"datalist"  => inline(TagKind::Inline),

    b"audio"   => block(TagKind::Block),
    b"video"   => block(TagKind::Block),
    b"picture" => inline(TagKind::Inline),
    b"canvas"  => inline(TagKind::Inline),
    b"map"     => inline(TagKind::Inline),

    b"area"   => void_inline(TagKind::Inline),
    b"embed"  => void_inline(TagKind::Inline),
    b"source" => void_inline(TagKind::Inline),
    b"track"  => void_inline(TagKind::Inline),
    b"wbr"    => void_inline(TagKind::Inline),
    b"keygen" => void_inline(TagKind::Inline),
    b"param"  => void_inline(TagKind::Inline),

    b"hgroup"    => block(TagKind::Block),
    b"menu"      => block(TagKind::Block),
    b"search"    => block(TagKind::Block),
    // ~keep `<center>` is a legacy HTML element that acts as a block container.
    // ~keep Tier-2 emits its inner content as plain block text (no centering markup
    // ~keep in Markdown), so a generic `Block` spec matches Tier-2 byte-for-byte.
    b"center"    => block(TagKind::Block),
};

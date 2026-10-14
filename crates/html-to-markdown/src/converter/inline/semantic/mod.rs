//! Handler for semantic inline elements (mark, del, s, ins, u, small, sub, sup, var, dfn, abbr, span).
//!
//! Converts HTML semantic tags to Markdown formatting with support for:
//! - Highlight/mark element with configurable styles (==, ::, ^^, <mark>)
//! - Strikethrough (del, s tags) with ~~ syntax
//! - Underline/inserted text (ins, u tags) with == syntax
//! - Small text (passes through without formatting)
//! - Subscript and superscript with configurable symbols
//! - Variable (var) and definition (dfn) text with italic formatting
//! - Abbreviation (abbr) text with optional title attribute
//! - Span element with special handling for OCR words and whitespace
//! - Visitor callbacks for custom processing (feature-gated)

mod marks;
mod typography;

#[cfg(feature = "visitor")]
pub use marks::visit_underline;
pub use marks::{close_inserted, close_mark, close_strikethrough};
pub use typography::{ScriptEnd, close_abbreviation, close_italic_semantic, close_script, open_span};

use crate::converter::inline::HandlerContext;
use crate::converter::walk::Open;

/// Open a semantic inline element: mark, del, s, strike, ins, sub, sup, var, dfn, abbr.
///
/// - Mark: configurable highlight style (==, <mark>, **bold, none), visitor callback first
/// - Del, s, strike: strikethrough with ~~ and visitor callback support
/// - Ins: inserted text with == and visitor callback support
/// - Sub, sup: wrap with configurable symbols
/// - Var, dfn: wrap with italic symbol (`strong_em_symbol`)
/// - Abbr: pass through content with optional title in parentheses
///
/// Lists the children in `children`. Returns `None` for an element that is complete.
pub fn open(tag_name: &str, context: HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>) -> Option<Open> {
    match tag_name {
        "mark" => marks::open_mark(context, children),
        "del" | "s" | "strike" => marks::open_strikethrough(tag_name, &context, children),
        "ins" => marks::open_inserted(context, children),
        "sub" | "sup" => typography::open_script(tag_name, &context, children),
        "var" | "dfn" => typography::open_italic_semantic(&context, children),
        "abbr" => typography::open_abbreviation(&context, children),
        _ => None,
    }
}

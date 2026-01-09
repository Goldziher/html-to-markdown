#![allow(clippy::all, clippy::pedantic, clippy::nursery, missing_docs)]
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions, ConversionOptionsUpdate, DEFAULT_INLINE_IMAGE_LIMIT, HeadingStyle,
    HighlightStyle, HtmlExtraction, InlineImage, InlineImageConfig, InlineImageConfigUpdate, InlineImageWarning,
    ListIndentType, NewlineStyle, PreprocessingOptionsUpdate, PreprocessingPreset, WhitespaceMode,
    convert as convert_inner, convert_with_inline_images as convert_with_inline_images_inner, error::ConversionError,
    safety::guard_panic,
};

#[cfg(feature = "visitor")]
use html_to_markdown_rs::{
    convert_with_visitor as convert_with_visitor_inner,
    visitor::{HtmlVisitor, NodeContext, NodeType, VisitResult},
};

#[cfg(feature = "metadata")]
use html_to_markdown_rs::convert_with_metadata as convert_with_metadata_inner;
mod profiling;
#[cfg(feature = "metadata")]
use html_to_markdown_rs::metadata::{
    DocumentMetadata as RustDocumentMetadata, ExtendedMetadata as RustExtendedMetadata,
    HeaderMetadata as RustHeaderMetadata, ImageMetadata as RustImageMetadata, LinkMetadata as RustLinkMetadata,
    MetadataConfig as RustMetadataConfig, StructuredData as RustStructuredData, TextDirection as RustTextDirection,
};
use magnus::prelude::*;
use magnus::r_hash::ForEach;
use magnus::value::ReprValue;
use magnus::{Error, RArray, RHash, Ruby, Symbol, TryConvert, Value, function, scan_args::scan_args};
#[cfg(feature = "visitor")]
use std::panic::AssertUnwindSafe;
#[cfg(feature = "profiling")]
use std::path::PathBuf;

#[derive(Clone)]
#[magnus::wrap(class = "HtmlToMarkdown::Options", free_immediately)]
struct OptionsHandle(ConversionOptions);

#[cfg(feature = "visitor")]
#[derive(Clone)]
struct RubyVisitorWrapper {
    ruby_visitor: Value,
    last_error: std::rc::Rc<std::cell::RefCell<Option<String>>>,
}

#[cfg(feature = "visitor")]
impl RubyVisitorWrapper {
    fn new(ruby_visitor: Value) -> Self {
        Self {
            ruby_visitor,
            last_error: std::rc::Rc::new(std::cell::RefCell::new(None)),
        }
    }

    fn utf8_str(&self, ruby: &Ruby, s: &str) -> Value {
        if let Ok(val) = ruby.eval::<Value>(&format!("String.new({s:?}, encoding: 'UTF-8')")) {
            val
        } else {
            let str_val = ruby.str_from_slice(s.as_bytes());
            str_val.as_value()
        }
    }

    fn call_visitor_method(&self, method_name: &str, args: &[Value]) -> Result<VisitResult, Error> {
        let ruby = Ruby::get().expect("Ruby not initialized");

        let result: Value = match args.len() {
            0 => match self.ruby_visitor.funcall::<&str, (), Value>(method_name, ()) {
                Ok(val) => val,
                Err(e) => {
                    *self.last_error.borrow_mut() = Some(format!("Visitor error in {method_name}: {e}"));
                    return Err(e);
                }
            },
            1 => match self
                .ruby_visitor
                .funcall::<&str, (Value,), Value>(method_name, (args[0],))
            {
                Ok(val) => val,
                Err(e) => {
                    *self.last_error.borrow_mut() = Some(format!("Visitor error in {method_name}: {e}"));
                    return Err(e);
                }
            },
            2 => match self
                .ruby_visitor
                .funcall::<&str, (Value, Value), Value>(method_name, (args[0], args[1]))
            {
                Ok(val) => val,
                Err(e) => {
                    *self.last_error.borrow_mut() = Some(format!("Visitor error in {method_name}: {e}"));
                    return Err(e);
                }
            },
            3 => match self
                .ruby_visitor
                .funcall::<&str, (Value, Value, Value), Value>(method_name, (args[0], args[1], args[2]))
            {
                Ok(val) => val,
                Err(e) => {
                    *self.last_error.borrow_mut() = Some(format!("Visitor error in {method_name}: {e}"));
                    return Err(e);
                }
            },
            4 => match self
                .ruby_visitor
                .funcall::<&str, (Value, Value, Value, Value), Value>(method_name, (args[0], args[1], args[2], args[3]))
            {
                Ok(val) => val,
                Err(e) => {
                    *self.last_error.borrow_mut() = Some(format!("Visitor error in {method_name}: {e}"));
                    return Err(e);
                }
            },
            _ => {
                return Err(arg_error(format!(
                    "Unsupported number of visitor method arguments: {}",
                    args.len()
                )));
            }
        };

        let hash = RHash::from_value(result)
            .ok_or_else(|| arg_error(format!("visitor method {method_name} must return a Hash")))?;

        let type_value: Value = hash
            .get(ruby.intern("type"))
            .ok_or_else(|| arg_error(format!("visitor method {method_name} result Hash must have :type key")))?;

        let type_str = symbol_to_string(type_value)?;

        match type_str.as_str() {
            "continue" => Ok(VisitResult::Continue),
            "custom" => {
                let output_value: Value = hash.get(ruby.intern("output")).ok_or_else(|| {
                    arg_error(format!(
                        "visitor method {method_name} with type :custom must provide :output string"
                    ))
                })?;
                let output = String::try_convert(output_value)?;
                Ok(VisitResult::Custom(output))
            }
            "skip" => Ok(VisitResult::Skip),
            "preserve_html" => Ok(VisitResult::PreserveHtml),
            "error" => {
                let message_value: Value = hash.get(ruby.intern("message")).ok_or_else(|| {
                    arg_error(format!(
                        "visitor method {method_name} with type :error must provide :message string"
                    ))
                })?;
                let message = String::try_convert(message_value)?;
                Ok(VisitResult::Error(message))
            }
            other => Err(arg_error(format!(
                "visitor method {method_name} returned invalid type: {other}"
            ))),
        }
    }

    fn ruby_to_node_context(&self, ctx: &NodeContext, ruby: &Ruby) -> Result<Value, Error> {
        let hash = ruby.hash_new();

        let node_type_str = match ctx.node_type {
            NodeType::Text => "text",
            NodeType::Element => "element",
            NodeType::Heading => "heading",
            NodeType::Paragraph => "paragraph",
            NodeType::Div => "div",
            NodeType::Blockquote => "blockquote",
            NodeType::Pre => "pre",
            NodeType::Hr => "hr",
            NodeType::List => "list",
            NodeType::ListItem => "list_item",
            NodeType::DefinitionList => "definition_list",
            NodeType::DefinitionTerm => "definition_term",
            NodeType::DefinitionDescription => "definition_description",
            NodeType::Table => "table",
            NodeType::TableRow => "table_row",
            NodeType::TableCell => "table_cell",
            NodeType::TableHeader => "table_header",
            NodeType::TableBody => "table_body",
            NodeType::TableHead => "table_head",
            NodeType::TableFoot => "table_foot",
            NodeType::Link => "link",
            NodeType::Image => "image",
            NodeType::Strong => "strong",
            NodeType::Em => "em",
            NodeType::Code => "code",
            NodeType::Strikethrough => "strikethrough",
            NodeType::Underline => "underline",
            NodeType::Subscript => "subscript",
            NodeType::Superscript => "superscript",
            NodeType::Mark => "mark",
            NodeType::Small => "small",
            NodeType::Br => "br",
            NodeType::Span => "span",
            NodeType::Article => "article",
            NodeType::Section => "section",
            NodeType::Nav => "nav",
            NodeType::Aside => "aside",
            NodeType::Header => "header",
            NodeType::Footer => "footer",
            NodeType::Main => "main",
            NodeType::Figure => "figure",
            NodeType::Figcaption => "figcaption",
            NodeType::Time => "time",
            NodeType::Details => "details",
            NodeType::Summary => "summary",
            NodeType::Form => "form",
            NodeType::Input => "input",
            NodeType::Select => "select",
            NodeType::Option => "option",
            NodeType::Button => "button",
            NodeType::Textarea => "textarea",
            NodeType::Label => "label",
            NodeType::Fieldset => "fieldset",
            NodeType::Legend => "legend",
            NodeType::Audio => "audio",
            NodeType::Video => "video",
            NodeType::Picture => "picture",
            NodeType::Source => "source",
            NodeType::Iframe => "iframe",
            NodeType::Svg => "svg",
            NodeType::Canvas => "canvas",
            NodeType::Ruby => "ruby",
            NodeType::Rt => "rt",
            NodeType::Rp => "rp",
            NodeType::Abbr => "abbr",
            NodeType::Kbd => "kbd",
            NodeType::Samp => "samp",
            NodeType::Var => "var",
            NodeType::Cite => "cite",
            NodeType::Q => "q",
            NodeType::Del => "del",
            NodeType::Ins => "ins",
            NodeType::Data => "data",
            NodeType::Meter => "meter",
            NodeType::Progress => "progress",
            NodeType::Output => "output",
            NodeType::Template => "template",
            NodeType::Slot => "slot",
            NodeType::Html => "html",
            NodeType::Head => "head",
            NodeType::Body => "body",
            NodeType::Title => "title",
            NodeType::Meta => "meta",
            NodeType::LinkTag => "link_tag",
            NodeType::Style => "style",
            NodeType::Script => "script",
            NodeType::Base => "base",
            NodeType::Custom => "custom",
        };
        hash.aset(ruby.intern("node_type"), ruby.intern(node_type_str))?;

        hash.aset(ruby.intern("tag_name"), ctx.tag_name.as_str())?;

        let attrs_hash = ruby.hash_new();
        for (key, value) in &ctx.attributes {
            attrs_hash.aset(key.as_str(), value.as_str())?;
        }
        hash.aset(ruby.intern("attributes"), attrs_hash)?;

        hash.aset(ruby.intern("depth"), ctx.depth as i64)?;

        hash.aset(ruby.intern("index_in_parent"), ctx.index_in_parent as i64)?;

        match &ctx.parent_tag {
            Some(tag) => hash.aset(ruby.intern("parent_tag"), tag.as_str())?,
            None => hash.aset(ruby.intern("parent_tag"), ruby.qnil())?,
        }

        hash.aset(ruby.intern("is_inline"), ctx.is_inline)?;

        Ok(hash.as_value())
    }
}

#[cfg(feature = "visitor")]
impl std::fmt::Debug for RubyVisitorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RubyVisitorWrapper")
            .field("ruby_visitor", &self.ruby_visitor)
            .finish()
    }
}

#[cfg(feature = "visitor")]
impl HtmlVisitor for RubyVisitorWrapper {
    fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method("visit_element_start", &[node_ctx]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_element_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_element_end",
                    &[node_ctx, ruby.str_from_slice(output.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_text(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method("visit_text", &[node_ctx, self.utf8_str(&ruby, text)]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_link(&mut self, ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let title_val = match title {
                    Some(t) => ruby.str_from_slice(t.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method(
                    "visit_link",
                    &[
                        node_ctx,
                        ruby.str_from_slice(href.as_bytes()).as_value(),
                        ruby.str_from_slice(text.as_bytes()).as_value(),
                        title_val,
                    ],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_image(&mut self, ctx: &NodeContext, src: &str, alt: &str, title: Option<&str>) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let title_val = match title {
                    Some(t) => ruby.str_from_slice(t.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method(
                    "visit_image",
                    &[
                        node_ctx,
                        ruby.str_from_slice(src.as_bytes()).as_value(),
                        ruby.str_from_slice(alt.as_bytes()).as_value(),
                        title_val,
                    ],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_heading(&mut self, ctx: &NodeContext, level: u32, text: &str, id: Option<&str>) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let id_val = match id {
                    Some(i) => ruby.str_from_slice(i.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method(
                    "visit_heading",
                    &[
                        node_ctx,
                        ruby.integer_from_i64(i64::from(level)).as_value(),
                        ruby.str_from_slice(text.as_bytes()).as_value(),
                        id_val,
                    ],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_code_block(&mut self, ctx: &NodeContext, lang: Option<&str>, code: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let lang_val = match lang {
                    Some(l) => ruby.str_from_slice(l.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method(
                    "visit_code_block",
                    &[node_ctx, lang_val, ruby.str_from_slice(code.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_code_inline(&mut self, ctx: &NodeContext, code: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_code_inline",
                    &[node_ctx, ruby.str_from_slice(code.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_list_item(&mut self, ctx: &NodeContext, ordered: bool, marker: &str, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let ordered_val = if ordered {
                    ruby.qtrue().as_value()
                } else {
                    ruby.qfalse().as_value()
                };
                if let Ok(result) = self.call_visitor_method(
                    "visit_list_item",
                    &[
                        node_ctx,
                        ordered_val,
                        ruby.str_from_slice(marker.as_bytes()).as_value(),
                        ruby.str_from_slice(text.as_bytes()).as_value(),
                    ],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_list_start(&mut self, ctx: &NodeContext, ordered: bool) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let ordered_val = if ordered {
                    ruby.qtrue().as_value()
                } else {
                    ruby.qfalse().as_value()
                };
                if let Ok(result) = self.call_visitor_method("visit_list_start", &[node_ctx, ordered_val]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_list_end(&mut self, ctx: &NodeContext, ordered: bool, output: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let ordered_val = if ordered {
                    ruby.qtrue().as_value()
                } else {
                    ruby.qfalse().as_value()
                };
                if let Ok(result) = self.call_visitor_method(
                    "visit_list_end",
                    &[node_ctx, ordered_val, ruby.str_from_slice(output.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_table_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method("visit_table_start", &[node_ctx]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_table_row(&mut self, ctx: &NodeContext, cells: &[String], is_header: bool) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let cells_array = ruby.ary_new();
                for cell in cells {
                    let _ = cells_array.push(ruby.str_from_slice(cell.as_bytes()).as_value());
                }
                let is_header_val = if is_header {
                    ruby.qtrue().as_value()
                } else {
                    ruby.qfalse().as_value()
                };
                if let Ok(result) =
                    self.call_visitor_method("visit_table_row", &[node_ctx, cells_array.as_value(), is_header_val])
                {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_table_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_table_end",
                    &[node_ctx, ruby.str_from_slice(output.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_blockquote(&mut self, ctx: &NodeContext, content: &str, depth: usize) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_blockquote",
                    &[
                        node_ctx,
                        ruby.str_from_slice(content.as_bytes()).as_value(),
                        ruby.integer_from_i64(depth as i64).as_value(),
                    ],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_strong(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_strong",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_emphasis(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_emphasis",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_strikethrough(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_strikethrough",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_underline(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_underline",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_subscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_subscript",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_superscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_superscript",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_mark(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_mark",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_line_break(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method("visit_line_break", &[node_ctx]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_horizontal_rule(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method("visit_horizontal_rule", &[node_ctx]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_custom_element(&mut self, ctx: &NodeContext, tag_name: &str, html: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_custom_element",
                    &[
                        node_ctx,
                        ruby.str_from_slice(tag_name.as_bytes()).as_value(),
                        ruby.str_from_slice(html.as_bytes()).as_value(),
                    ],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_definition_list_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method("visit_definition_list_start", &[node_ctx]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_definition_term(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_definition_term",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_definition_description(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_definition_description",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_definition_list_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_definition_list_end",
                    &[node_ctx, ruby.str_from_slice(output.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_form(&mut self, ctx: &NodeContext, action: Option<&str>, method: Option<&str>) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let action_val = match action {
                    Some(a) => ruby.str_from_slice(a.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                let method_val = match method {
                    Some(m) => ruby.str_from_slice(m.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method("visit_form", &[node_ctx, action_val, method_val]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_input(
        &mut self,
        ctx: &NodeContext,
        input_type: &str,
        name: Option<&str>,
        value: Option<&str>,
    ) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let name_val = match name {
                    Some(n) => ruby.str_from_slice(n.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                let value_val = match value {
                    Some(v) => ruby.str_from_slice(v.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method(
                    "visit_input",
                    &[
                        node_ctx,
                        ruby.str_from_slice(input_type.as_bytes()).as_value(),
                        name_val,
                        value_val,
                    ],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_button(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_button",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_audio(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let src_val = match src {
                    Some(s) => ruby.str_from_slice(s.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method("visit_audio", &[node_ctx, src_val]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_video(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let src_val = match src {
                    Some(s) => ruby.str_from_slice(s.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method("visit_video", &[node_ctx, src_val]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_iframe(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let src_val = match src {
                    Some(s) => ruby.str_from_slice(s.as_bytes()).as_value(),
                    None => ruby.qnil().as_value(),
                };
                if let Ok(result) = self.call_visitor_method("visit_iframe", &[node_ctx, src_val]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_details(&mut self, ctx: &NodeContext, open: bool) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                let open_val = if open {
                    ruby.qtrue().as_value()
                } else {
                    ruby.qfalse().as_value()
                };
                if let Ok(result) = self.call_visitor_method("visit_details", &[node_ctx, open_val]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_summary(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_summary",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_figure_start(&mut self, ctx: &NodeContext) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method("visit_figure_start", &[node_ctx]) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_figcaption(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_figcaption",
                    &[node_ctx, ruby.str_from_slice(text.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }

    fn visit_figure_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        if let Ok(ruby) = Ruby::get() {
            if let Ok(node_ctx) = self.ruby_to_node_context(ctx, &ruby) {
                if let Ok(result) = self.call_visitor_method(
                    "visit_figure_end",
                    &[node_ctx, ruby.str_from_slice(output.as_bytes()).as_value()],
                ) {
                    return result;
                }
            }
        }
        VisitResult::Continue
    }
}

fn conversion_error(err: ConversionError) -> Error {
    match err {
        ConversionError::ConfigError(msg) => arg_error(msg),
        ConversionError::Panic(message) => {
            runtime_error(format!("html-to-markdown panic during conversion: {message}"))
        }
        other => runtime_error(other.to_string()),
    }
}

fn arg_error(message: impl Into<String>) -> Error {
    let ruby = Ruby::get().expect("Ruby not initialised");
    Error::new(ruby.exception_arg_error(), message.into())
}

fn runtime_error(message: impl Into<String>) -> Error {
    let ruby = Ruby::get().expect("Ruby not initialised");
    Error::new(ruby.exception_runtime_error(), message.into())
}

fn symbol_to_string(value: Value) -> Result<String, Error> {
    if let Some(symbol) = Symbol::from_value(value) {
        Ok(symbol.name()?.to_string())
    } else {
        String::try_convert(value)
    }
}

fn parse_heading_style(value: Value) -> Result<HeadingStyle, Error> {
    match symbol_to_string(value)?.as_str() {
        "underlined" => Ok(HeadingStyle::Underlined),
        "atx" => Ok(HeadingStyle::Atx),
        "atx_closed" => Ok(HeadingStyle::AtxClosed),
        other => Err(arg_error(format!("invalid heading_style: {other}"))),
    }
}

fn parse_list_indent_type(value: Value) -> Result<ListIndentType, Error> {
    match symbol_to_string(value)?.as_str() {
        "spaces" => Ok(ListIndentType::Spaces),
        "tabs" => Ok(ListIndentType::Tabs),
        other => Err(arg_error(format!("invalid list_indent_type: {other}"))),
    }
}

fn parse_highlight_style(value: Value) -> Result<HighlightStyle, Error> {
    match symbol_to_string(value)?.as_str() {
        "double_equal" => Ok(HighlightStyle::DoubleEqual),
        "html" => Ok(HighlightStyle::Html),
        "bold" => Ok(HighlightStyle::Bold),
        "none" => Ok(HighlightStyle::None),
        other => Err(arg_error(format!("invalid highlight_style: {other}"))),
    }
}

fn parse_whitespace_mode(value: Value) -> Result<WhitespaceMode, Error> {
    match symbol_to_string(value)?.as_str() {
        "normalized" => Ok(WhitespaceMode::Normalized),
        "strict" => Ok(WhitespaceMode::Strict),
        other => Err(arg_error(format!("invalid whitespace_mode: {other}"))),
    }
}

fn parse_newline_style(value: Value) -> Result<NewlineStyle, Error> {
    match symbol_to_string(value)?.as_str() {
        "spaces" => Ok(NewlineStyle::Spaces),
        "backslash" => Ok(NewlineStyle::Backslash),
        other => Err(arg_error(format!("invalid newline_style: {other}"))),
    }
}

fn parse_code_block_style(value: Value) -> Result<CodeBlockStyle, Error> {
    match symbol_to_string(value)?.as_str() {
        "indented" => Ok(CodeBlockStyle::Indented),
        "backticks" => Ok(CodeBlockStyle::Backticks),
        "tildes" => Ok(CodeBlockStyle::Tildes),
        other => Err(arg_error(format!("invalid code_block_style: {other}"))),
    }
}

fn parse_preset(value: Value) -> Result<PreprocessingPreset, Error> {
    match symbol_to_string(value)?.as_str() {
        "minimal" => Ok(PreprocessingPreset::Minimal),
        "standard" => Ok(PreprocessingPreset::Standard),
        "aggressive" => Ok(PreprocessingPreset::Aggressive),
        other => Err(arg_error(format!("invalid preprocessing preset: {other}"))),
    }
}

fn parse_vec_of_strings(value: Value) -> Result<Vec<String>, Error> {
    let array = RArray::from_value(value).ok_or_else(|| arg_error("expected an Array of strings"))?;

    array.to_vec::<String>()
}

fn parse_preprocessing_options(_ruby: &Ruby, value: Value) -> Result<PreprocessingOptionsUpdate, Error> {
    let hash = RHash::from_value(value).ok_or_else(|| arg_error("expected preprocessing to be a Hash"))?;

    let mut update = PreprocessingOptionsUpdate::default();

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "enabled" => {
                update.enabled = Some(bool::try_convert(val)?);
            }
            "preset" => {
                update.preset = Some(parse_preset(val)?);
            }
            "remove_navigation" => {
                update.remove_navigation = Some(bool::try_convert(val)?);
            }
            "remove_forms" => {
                update.remove_forms = Some(bool::try_convert(val)?);
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(update)
}

fn build_conversion_options(ruby: &Ruby, options: Option<Value>) -> Result<ConversionOptions, Error> {
    let mut update = ConversionOptionsUpdate::default();

    let Some(options) = options else {
        return Ok(ConversionOptions::default());
    };

    if options.is_nil() {
        return Ok(ConversionOptions::default());
    }

    let hash = RHash::from_value(options).ok_or_else(|| arg_error("options must be provided as a Hash"))?;

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "heading_style" => {
                update.heading_style = Some(parse_heading_style(val)?);
            }
            "list_indent_type" => {
                update.list_indent_type = Some(parse_list_indent_type(val)?);
            }
            "list_indent_width" => {
                update.list_indent_width = Some(usize::try_convert(val)?);
            }
            "bullets" => {
                update.bullets = Some(String::try_convert(val)?);
            }
            "strong_em_symbol" => {
                let value = String::try_convert(val)?;
                let mut chars = value.chars();
                let ch = chars
                    .next()
                    .ok_or_else(|| arg_error("strong_em_symbol must not be empty"))?;
                if chars.next().is_some() {
                    return Err(arg_error("strong_em_symbol must be a single character"));
                }
                update.strong_em_symbol = Some(ch);
            }
            "escape_asterisks" => {
                update.escape_asterisks = Some(bool::try_convert(val)?);
            }
            "escape_underscores" => {
                update.escape_underscores = Some(bool::try_convert(val)?);
            }
            "escape_misc" => {
                update.escape_misc = Some(bool::try_convert(val)?);
            }
            "escape_ascii" => {
                update.escape_ascii = Some(bool::try_convert(val)?);
            }
            "code_language" => {
                update.code_language = Some(String::try_convert(val)?);
            }
            "autolinks" => {
                update.autolinks = Some(bool::try_convert(val)?);
            }
            "default_title" => {
                update.default_title = Some(bool::try_convert(val)?);
            }
            "br_in_tables" => {
                update.br_in_tables = Some(bool::try_convert(val)?);
            }
            "hocr_spatial_tables" => {
                update.hocr_spatial_tables = Some(bool::try_convert(val)?);
            }
            "highlight_style" => {
                update.highlight_style = Some(parse_highlight_style(val)?);
            }
            "extract_metadata" => {
                update.extract_metadata = Some(bool::try_convert(val)?);
            }
            "whitespace_mode" => {
                update.whitespace_mode = Some(parse_whitespace_mode(val)?);
            }
            "strip_newlines" => {
                update.strip_newlines = Some(bool::try_convert(val)?);
            }
            "wrap" => {
                update.wrap = Some(bool::try_convert(val)?);
            }
            "wrap_width" => {
                update.wrap_width = Some(usize::try_convert(val)?);
            }
            "convert_as_inline" => {
                update.convert_as_inline = Some(bool::try_convert(val)?);
            }
            "sub_symbol" => {
                update.sub_symbol = Some(String::try_convert(val)?);
            }
            "sup_symbol" => {
                update.sup_symbol = Some(String::try_convert(val)?);
            }
            "newline_style" => {
                update.newline_style = Some(parse_newline_style(val)?);
            }
            "code_block_style" => {
                update.code_block_style = Some(parse_code_block_style(val)?);
            }
            "keep_inline_images_in" => {
                update.keep_inline_images_in = Some(parse_vec_of_strings(val)?);
            }
            "preprocessing" => {
                update.preprocessing = Some(parse_preprocessing_options(ruby, val)?);
            }
            "encoding" => {
                update.encoding = Some(String::try_convert(val)?);
            }
            "debug" => {
                update.debug = Some(bool::try_convert(val)?);
            }
            "strip_tags" => {
                update.strip_tags = Some(parse_vec_of_strings(val)?);
            }
            "preserve_tags" => {
                update.preserve_tags = Some(parse_vec_of_strings(val)?);
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(ConversionOptions::from(update))
}

fn build_inline_image_config(_ruby: &Ruby, config: Option<Value>) -> Result<InlineImageConfig, Error> {
    let mut update = InlineImageConfigUpdate::default();

    let Some(config) = config else {
        return Ok(InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    };

    if config.is_nil() {
        return Ok(InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    }

    let hash = RHash::from_value(config).ok_or_else(|| arg_error("inline image config must be provided as a Hash"))?;

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "max_decoded_size_bytes" => {
                update.max_decoded_size_bytes = Some(u64::try_convert(val)?);
            }
            "filename_prefix" => {
                update.filename_prefix = if val.is_nil() {
                    None
                } else {
                    Some(String::try_convert(val)?)
                };
            }
            "capture_svg" => {
                update.capture_svg = Some(bool::try_convert(val)?);
            }
            "infer_dimensions" => {
                update.infer_dimensions = Some(bool::try_convert(val)?);
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(InlineImageConfig::from_update(update))
}

fn inline_image_to_value(ruby: &Ruby, image: InlineImage) -> Result<Value, Error> {
    let InlineImage {
        data,
        format,
        filename,
        description,
        dimensions,
        source,
        attributes,
    } = image;

    let hash = ruby.hash_new();
    let data_value = ruby.str_from_slice(&data);
    hash.aset(ruby.intern("data"), data_value)?;

    let format_value = format.to_string();
    hash.aset(ruby.intern("format"), format_value)?;

    match filename {
        Some(name) => hash.aset(ruby.intern("filename"), name)?,
        None => hash.aset(ruby.intern("filename"), ruby.qnil())?,
    }

    match description {
        Some(desc) => hash.aset(ruby.intern("description"), desc)?,
        None => hash.aset(ruby.intern("description"), ruby.qnil())?,
    }

    if let Some((width, height)) = dimensions {
        let dims = ruby.ary_new();
        dims.push(i64::from(width))?;
        dims.push(i64::from(height))?;
        hash.aset(ruby.intern("dimensions"), dims)?;
    } else {
        hash.aset(ruby.intern("dimensions"), ruby.qnil())?;
    }

    let source_value = source.to_string();
    hash.aset(ruby.intern("source"), source_value)?;

    let attrs = ruby.hash_new();
    for (key, value) in attributes {
        attrs.aset(key, value)?;
    }
    hash.aset(ruby.intern("attributes"), attrs)?;

    Ok(hash.as_value())
}

fn warning_to_value(ruby: &Ruby, warning: InlineImageWarning) -> Result<Value, Error> {
    let hash = ruby.hash_new();
    hash.aset(ruby.intern("index"), warning.index as i64)?;
    hash.aset(ruby.intern("message"), warning.message)?;
    Ok(hash.as_value())
}

fn extraction_to_value(ruby: &Ruby, extraction: HtmlExtraction) -> Result<Value, Error> {
    let hash = ruby.hash_new();
    hash.aset(ruby.intern("markdown"), extraction.markdown)?;

    let inline_images = ruby.ary_new();
    for image in extraction.inline_images {
        inline_images.push(inline_image_to_value(ruby, image)?)?;
    }
    hash.aset(ruby.intern("inline_images"), inline_images)?;

    let warnings = ruby.ary_new();
    for warning in extraction.warnings {
        warnings.push(warning_to_value(ruby, warning)?)?;
    }
    hash.aset(ruby.intern("warnings"), warnings)?;

    Ok(hash.as_value())
}

fn convert_fn(ruby: &Ruby, args: &[Value]) -> Result<String, Error> {
    let parsed = scan_args::<(String,), (Option<Value>,), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let options = build_conversion_options(ruby, parsed.optional.0)?;

    guard_panic(|| profiling::maybe_profile(|| convert_inner(&html, Some(options)))).map_err(conversion_error)
}

fn options_handle_fn(ruby: &Ruby, args: &[Value]) -> Result<OptionsHandle, Error> {
    let parsed = scan_args::<(), (Option<Value>,), (), (), (), ()>(args)?;
    let options = build_conversion_options(ruby, parsed.optional.0)?;
    Ok(OptionsHandle(options))
}

fn convert_with_options_handle_fn(_ruby: &Ruby, args: &[Value]) -> Result<String, Error> {
    let parsed = scan_args::<(String, &OptionsHandle), (), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let handle = parsed.required.1;
    let options = handle.0.clone();

    guard_panic(|| profiling::maybe_profile(|| convert_inner(&html, Some(options)))).map_err(conversion_error)
}

#[cfg(feature = "inline-images")]
fn convert_with_inline_images_fn(ruby: &Ruby, args: &[Value]) -> Result<Value, Error> {
    let parsed = scan_args::<(String,), (Option<Value>, Option<Value>), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let options = build_conversion_options(ruby, parsed.optional.0)?;
    let config = build_inline_image_config(ruby, parsed.optional.1)?;

    let extraction =
        guard_panic(|| convert_with_inline_images_inner(&html, Some(options), config)).map_err(conversion_error)?;

    extraction_to_value(ruby, extraction)
}

#[cfg(feature = "inline-images")]
fn convert_with_inline_images_handle_fn(ruby: &Ruby, args: &[Value]) -> Result<Value, Error> {
    let parsed = scan_args::<(String, &OptionsHandle), (Option<Value>,), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let handle = parsed.required.1;
    let options = handle.0.clone();
    let config = build_inline_image_config(ruby, parsed.optional.0)?;

    let extraction =
        guard_panic(|| convert_with_inline_images_inner(&html, Some(options), config)).map_err(conversion_error)?;

    extraction_to_value(ruby, extraction)
}

#[cfg(feature = "metadata")]
fn build_metadata_config(_ruby: &Ruby, config: Option<Value>) -> Result<RustMetadataConfig, Error> {
    let mut cfg = RustMetadataConfig::default();

    let Some(config) = config else {
        return Ok(cfg);
    };

    if config.is_nil() {
        return Ok(cfg);
    }

    let hash = RHash::from_value(config).ok_or_else(|| arg_error("metadata_config must be provided as a Hash"))?;

    hash.foreach(|key: Value, val: Value| {
        let key_name = symbol_to_string(key)?;
        match key_name.as_str() {
            "extract_document" => {
                cfg.extract_document = bool::try_convert(val)?;
            }
            "extract_headers" => {
                cfg.extract_headers = bool::try_convert(val)?;
            }
            "extract_links" => {
                cfg.extract_links = bool::try_convert(val)?;
            }
            "extract_images" => {
                cfg.extract_images = bool::try_convert(val)?;
            }
            "extract_structured_data" => {
                cfg.extract_structured_data = bool::try_convert(val)?;
            }
            "max_structured_data_size" => {
                cfg.max_structured_data_size = usize::try_convert(val)?;
            }
            _ => {}
        }
        Ok(ForEach::Continue)
    })?;

    Ok(cfg)
}

#[cfg(feature = "metadata")]
fn opt_string_to_ruby(ruby: &Ruby, opt: Option<String>) -> Result<Value, Error> {
    match opt {
        Some(val) => Ok(ruby.str_from_slice(val.as_bytes()).as_value()),
        None => Ok(ruby.qnil().as_value()),
    }
}

#[cfg(feature = "metadata")]
fn btreemap_to_ruby_hash(ruby: &Ruby, map: std::collections::BTreeMap<String, String>) -> Result<Value, Error> {
    let hash = ruby.hash_new();
    for (k, v) in map {
        hash.aset(k, v)?;
    }
    Ok(hash.as_value())
}

#[cfg(feature = "metadata")]
fn text_direction_to_string(text_direction: Option<RustTextDirection>) -> Option<String> {
    text_direction.map(|direction| direction.to_string())
}

#[cfg(feature = "metadata")]
fn document_metadata_to_ruby(ruby: &Ruby, doc: RustDocumentMetadata) -> Result<Value, Error> {
    let hash = ruby.hash_new();

    hash.aset(ruby.intern("title"), opt_string_to_ruby(ruby, doc.title)?)?;
    hash.aset(ruby.intern("description"), opt_string_to_ruby(ruby, doc.description)?)?;

    let keywords = ruby.ary_new();
    for keyword in doc.keywords {
        keywords.push(keyword)?;
    }
    hash.aset(ruby.intern("keywords"), keywords)?;

    hash.aset(ruby.intern("author"), opt_string_to_ruby(ruby, doc.author)?)?;
    hash.aset(
        ruby.intern("canonical_url"),
        opt_string_to_ruby(ruby, doc.canonical_url)?,
    )?;
    hash.aset(ruby.intern("base_href"), opt_string_to_ruby(ruby, doc.base_href)?)?;
    hash.aset(ruby.intern("language"), opt_string_to_ruby(ruby, doc.language)?)?;

    match text_direction_to_string(doc.text_direction) {
        Some(dir) => hash.aset(ruby.intern("text_direction"), dir)?,
        None => hash.aset(ruby.intern("text_direction"), ruby.qnil())?,
    }

    hash.aset(ruby.intern("open_graph"), btreemap_to_ruby_hash(ruby, doc.open_graph)?)?;
    hash.aset(
        ruby.intern("twitter_card"),
        btreemap_to_ruby_hash(ruby, doc.twitter_card)?,
    )?;
    hash.aset(ruby.intern("meta_tags"), btreemap_to_ruby_hash(ruby, doc.meta_tags)?)?;

    Ok(hash.as_value())
}

#[cfg(feature = "metadata")]
fn headers_to_ruby(ruby: &Ruby, headers: Vec<RustHeaderMetadata>) -> Result<Value, Error> {
    let array = ruby.ary_new();
    for header in headers {
        let hash = ruby.hash_new();
        hash.aset(ruby.intern("level"), header.level)?;
        hash.aset(ruby.intern("text"), header.text)?;
        hash.aset(ruby.intern("id"), opt_string_to_ruby(ruby, header.id)?)?;
        hash.aset(ruby.intern("depth"), header.depth as i64)?;
        hash.aset(ruby.intern("html_offset"), header.html_offset as i64)?;
        array.push(hash)?;
    }
    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn links_to_ruby(ruby: &Ruby, links: Vec<RustLinkMetadata>) -> Result<Value, Error> {
    let array = ruby.ary_new();
    for link in links {
        let hash = ruby.hash_new();
        hash.aset(ruby.intern("href"), link.href)?;
        hash.aset(ruby.intern("text"), link.text)?;
        hash.aset(ruby.intern("title"), opt_string_to_ruby(ruby, link.title)?)?;
        hash.aset(ruby.intern("link_type"), link.link_type.to_string())?;

        let rel_array = ruby.ary_new();
        for r in link.rel {
            rel_array.push(r)?;
        }
        hash.aset(ruby.intern("rel"), rel_array)?;

        hash.aset(ruby.intern("attributes"), btreemap_to_ruby_hash(ruby, link.attributes)?)?;
        array.push(hash)?;
    }
    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn images_to_ruby(ruby: &Ruby, images: Vec<RustImageMetadata>) -> Result<Value, Error> {
    let array = ruby.ary_new();
    for image in images {
        let hash = ruby.hash_new();
        hash.aset(ruby.intern("src"), image.src)?;
        hash.aset(ruby.intern("alt"), opt_string_to_ruby(ruby, image.alt)?)?;
        hash.aset(ruby.intern("title"), opt_string_to_ruby(ruby, image.title)?)?;

        match image.dimensions {
            Some((width, height)) => {
                let dims = ruby.ary_new();
                dims.push(i64::from(width))?;
                dims.push(i64::from(height))?;
                hash.aset(ruby.intern("dimensions"), dims)?;
            }
            None => {
                hash.aset(ruby.intern("dimensions"), ruby.qnil())?;
            }
        }

        hash.aset(ruby.intern("image_type"), image.image_type.to_string())?;
        hash.aset(
            ruby.intern("attributes"),
            btreemap_to_ruby_hash(ruby, image.attributes)?,
        )?;
        array.push(hash)?;
    }
    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn structured_data_to_ruby(ruby: &Ruby, data: Vec<RustStructuredData>) -> Result<Value, Error> {
    let array = ruby.ary_new();
    for item in data {
        let hash = ruby.hash_new();
        hash.aset(ruby.intern("data_type"), item.data_type.to_string())?;
        hash.aset(ruby.intern("raw_json"), item.raw_json)?;
        hash.aset(ruby.intern("schema_type"), opt_string_to_ruby(ruby, item.schema_type)?)?;
        array.push(hash)?;
    }
    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn extended_metadata_to_ruby(ruby: &Ruby, metadata: RustExtendedMetadata) -> Result<Value, Error> {
    let hash = ruby.hash_new();

    hash.aset(
        ruby.intern("document"),
        document_metadata_to_ruby(ruby, metadata.document)?,
    )?;
    hash.aset(ruby.intern("headers"), headers_to_ruby(ruby, metadata.headers)?)?;
    hash.aset(ruby.intern("links"), links_to_ruby(ruby, metadata.links)?)?;
    hash.aset(ruby.intern("images"), images_to_ruby(ruby, metadata.images)?)?;
    hash.aset(
        ruby.intern("structured_data"),
        structured_data_to_ruby(ruby, metadata.structured_data)?,
    )?;

    Ok(hash.as_value())
}

#[cfg(feature = "metadata")]
fn convert_with_metadata_fn(ruby: &Ruby, args: &[Value]) -> Result<Value, Error> {
    let parsed = scan_args::<(String,), (Option<Value>, Option<Value>), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let options = build_conversion_options(ruby, parsed.optional.0)?;
    let metadata_config = build_metadata_config(ruby, parsed.optional.1)?;

    let (markdown, metadata) =
        guard_panic(|| convert_with_metadata_inner(&html, Some(options), metadata_config)).map_err(conversion_error)?;

    let array = ruby.ary_new();
    array.push(markdown)?;
    array.push(extended_metadata_to_ruby(ruby, metadata)?)?;

    Ok(array.as_value())
}

#[cfg(feature = "metadata")]
fn convert_with_metadata_handle_fn(ruby: &Ruby, args: &[Value]) -> Result<Value, Error> {
    let parsed = scan_args::<(String, &OptionsHandle), (Option<Value>,), (), (), (), ()>(args)?;
    let html = parsed.required.0;
    let handle = parsed.required.1;
    let options = handle.0.clone();
    let metadata_config = build_metadata_config(ruby, parsed.optional.0)?;

    let (markdown, metadata) =
        guard_panic(|| convert_with_metadata_inner(&html, Some(options), metadata_config)).map_err(conversion_error)?;

    let array = ruby.ary_new();
    array.push(markdown)?;
    array.push(extended_metadata_to_ruby(ruby, metadata)?)?;

    Ok(array.as_value())
}

#[cfg(feature = "visitor")]
fn convert_with_visitor_fn(ruby: &Ruby, args: &[Value]) -> Result<String, Error> {
    let parsed = scan_args::<(String,), (Option<Value>, Option<Value>), (), (), (), ()>(args)?;
    let html = parsed.required.0;

    let options = match parsed.optional.0 {
        Some(opt_val) => match <&OptionsHandle>::try_convert(opt_val) {
            Ok(handle) => handle.0.clone(),
            Err(_) => build_conversion_options(ruby, Some(opt_val))?,
        },
        None => ConversionOptions::default(),
    };

    let visitor_value = match parsed.optional.1 {
        Some(val) => {
            if val.is_nil() {
                return guard_panic(AssertUnwindSafe(|| {
                    profiling::maybe_profile(|| convert_inner(&html, Some(options)))
                }))
                .map_err(conversion_error);
            }
            val
        }
        None => return Err(arg_error("visitor argument is required")),
    };

    let visitor_wrapper = RubyVisitorWrapper::new(visitor_value);
    let visitor_handle = std::rc::Rc::new(std::cell::RefCell::new(visitor_wrapper.clone()));

    let result = guard_panic(AssertUnwindSafe(|| {
        profiling::maybe_profile(|| convert_with_visitor_inner(&html, Some(options), Some(visitor_handle)))
    }))
    .map_err(conversion_error)?;

    if let Some(error_msg) = visitor_wrapper.last_error.borrow().as_ref() {
        return Err(runtime_error(error_msg.clone()));
    }

    Ok(result)
}

#[cfg(feature = "profiling")]
fn start_profiling_fn(_ruby: &Ruby, args: &[Value]) -> Result<bool, Error> {
    let output = args.first().ok_or_else(|| arg_error("output_path required"))?;
    let output: String = String::try_convert(*output)?;
    let freq = if let Some(value) = args.get(1) {
        i32::try_convert(*value)?
    } else {
        1000
    };
    profiling::start(PathBuf::from(output), freq).map_err(conversion_error)?;
    Ok(true)
}

#[cfg(feature = "profiling")]
fn stop_profiling_fn(_ruby: &Ruby, _args: &[Value]) -> Result<bool, Error> {
    profiling::stop().map_err(conversion_error)?;
    Ok(true)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("HtmlToMarkdown")?;
    module.define_singleton_method("convert", function!(convert_fn, -1))?;
    module.define_singleton_method("options", function!(options_handle_fn, -1))?;
    module.define_singleton_method("convert_with_options", function!(convert_with_options_handle_fn, -1))?;
    module.define_singleton_method(
        "convert_with_inline_images",
        function!(convert_with_inline_images_fn, -1),
    )?;
    module.define_singleton_method(
        "convert_with_inline_images_handle",
        function!(convert_with_inline_images_handle_fn, -1),
    )?;

    #[cfg(feature = "metadata")]
    module.define_singleton_method("convert_with_metadata", function!(convert_with_metadata_fn, -1))?;
    #[cfg(feature = "metadata")]
    module.define_singleton_method(
        "convert_with_metadata_handle",
        function!(convert_with_metadata_handle_fn, -1),
    )?;

    #[cfg(feature = "visitor")]
    module.define_singleton_method("convert_with_visitor", function!(convert_with_visitor_fn, -1))?;

    #[cfg(feature = "profiling")]
    module.define_singleton_method("start_profiling", function!(start_profiling_fn, -1))?;
    #[cfg(feature = "profiling")]
    module.define_singleton_method("stop_profiling", function!(stop_profiling_fn, -1))?;

    Ok(())
}

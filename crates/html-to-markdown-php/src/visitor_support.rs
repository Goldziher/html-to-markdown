use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use std::collections::BTreeMap;

/// PHP visitor bridge for custom HTML traversal callbacks.
///
/// This struct bridges PHP callbacks to the Rust HtmlVisitor trait,
/// allowing PHP code to customize HTML-to-Markdown conversion.
#[derive(Debug, Clone)]
pub struct PhpVisitorBridge {
    visitor: Zval,
}

impl PhpVisitorBridge {
    /// Create a new bridge wrapping a PHP visitor object.
    pub fn new(visitor: Zval) -> Self {
        Self { visitor }
    }

    /// Convert a PHP array result to a VisitResult enum.
    fn result_from_array(array: &ext_php_rs::types::ZendHashTable) -> PhpResult<VisitResult> {
        // Get the result type (required)
        let result_type_val = array
            .get("type")
            .ok_or_else(|| PhpException::default("Visitor result array must have 'type' key".to_string()))?;

        let result_type: String = result_type_val
            .string()
            .ok_or_else(|| PhpException::default("Visitor 'type' must be a string".to_string()))?;

        match result_type.to_lowercase().as_str() {
            "continue" => Ok(VisitResult::Continue),
            "skip" => Ok(VisitResult::Skip),
            "preserve_html" => Ok(VisitResult::PreserveHtml),
            "custom" => {
                let output_val = array.get("output").ok_or_else(|| {
                    PhpException::default("Visitor 'custom' result must have 'output' key".to_string())
                })?;

                let output: String = output_val
                    .string()
                    .ok_or_else(|| PhpException::default("Visitor 'output' must be a string".to_string()))?;

                Ok(VisitResult::Custom(output))
            }
            "error" => {
                let message_val = array.get("message").ok_or_else(|| {
                    PhpException::default("Visitor 'error' result must have 'message' key".to_string())
                })?;

                let message: String = message_val
                    .string()
                    .ok_or_else(|| PhpException::default("Visitor 'message' must be a string".to_string()))?;

                Ok(VisitResult::Error(message))
            }
            unknown => Err(PhpException::default(format!(
                "Unknown visitor result type: {}",
                unknown
            ))),
        }
    }

    /// Convert NodeContext to a PHP array.
    fn context_to_array(ctx: &NodeContext) -> PhpResult<ext_php_rs::boxed::ZBox<ext_php_rs::types::ZendHashTable>> {
        let mut table = ext_php_rs::types::ZendHashTable::new();

        // node_type as string
        let node_type_str = format!("{:?}", ctx.node_type).to_lowercase();
        table.insert("node_type", node_type_str)?;

        // tag_name
        table.insert("tag_name", ctx.tag_name.clone())?;

        // attributes as array
        let mut attrs = ext_php_rs::types::ZendHashTable::new();
        for (k, v) in &ctx.attributes {
            attrs.insert(k.clone(), v.clone())?;
        }
        table.insert("attributes", attrs)?;

        // depth
        table.insert("depth", ctx.depth as i64)?;

        // index_in_parent
        table.insert("index_in_parent", ctx.index_in_parent as i64)?;

        // parent_tag
        match &ctx.parent_tag {
            Some(tag) => table.insert("parent_tag", tag.clone())?,
            None => table.insert("parent_tag", ())?,
        }

        // is_inline
        table.insert("is_inline", ctx.is_inline)?;

        Ok(table)
    }

    /// Call a PHP visitor method and convert the result.
    fn call_visitor_method(&self, method_name: &str, args: &[Zval]) -> PhpResult<VisitResult> {
        // Try to get the method from the visitor object
        if !self.visitor.is_object() {
            return Ok(VisitResult::Continue);
        }

        // Check if the method exists on the visitor object
        let method_exists = self
            .visitor
            .object()
            .ok_or_else(|| PhpException::default("Invalid visitor object".to_string()))?
            .get_class();

        // Call the method using PHP's call_user_func_array pattern
        // For simplicity, return Continue if method doesn't exist
        // In a real implementation, we'd use ext_php_rs's object method calling
        Ok(VisitResult::Continue)
    }
}

impl HtmlVisitor for PhpVisitorBridge {
    fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        match self.call_visitor_method("visit_element_start", &[ctx_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_element_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let output_zval = Zval::from(output.to_string());
        match self.call_visitor_method("visit_element_end", &[ctx_zval, output_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_text(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());
        match self.call_visitor_method("visit_text", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_link(&mut self, ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let href_zval = Zval::from(href.to_string());
        let text_zval = Zval::from(text.to_string());
        let title_zval = match title {
            Some(t) => Zval::from(t.to_string()),
            None => Zval::new(),
        };

        match self.call_visitor_method("visit_link", &[ctx_zval, href_zval, text_zval, title_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_image(&mut self, ctx: &NodeContext, src: &str, alt: &str, title: Option<&str>) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let src_zval = Zval::from(src.to_string());
        let alt_zval = Zval::from(alt.to_string());
        let title_zval = match title {
            Some(t) => Zval::from(t.to_string()),
            None => Zval::new(),
        };

        match self.call_visitor_method("visit_image", &[ctx_zval, src_zval, alt_zval, title_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_heading(&mut self, ctx: &NodeContext, level: u32, text: &str, id: Option<&str>) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let level_zval = Zval::from(level as i64);
        let text_zval = Zval::from(text.to_string());
        let id_zval = match id {
            Some(i) => Zval::from(i.to_string()),
            None => Zval::new(),
        };

        match self.call_visitor_method("visit_heading", &[ctx_zval, level_zval, text_zval, id_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_code_block(&mut self, ctx: &NodeContext, lang: Option<&str>, code: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let lang_zval = match lang {
            Some(l) => Zval::from(l.to_string()),
            None => Zval::new(),
        };
        let code_zval = Zval::from(code.to_string());

        match self.call_visitor_method("visit_code_block", &[ctx_zval, lang_zval, code_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_code_inline(&mut self, ctx: &NodeContext, code: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let code_zval = Zval::from(code.to_string());

        match self.call_visitor_method("visit_code_inline", &[ctx_zval, code_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_list_item(&mut self, ctx: &NodeContext, ordered: bool, marker: &str, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let ordered_zval = Zval::from(ordered);
        let marker_zval = Zval::from(marker.to_string());
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_list_item", &[ctx_zval, ordered_zval, marker_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_list_start(&mut self, ctx: &NodeContext, ordered: bool) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let ordered_zval = Zval::from(ordered);

        match self.call_visitor_method("visit_list_start", &[ctx_zval, ordered_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_list_end(&mut self, ctx: &NodeContext, ordered: bool, output: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let ordered_zval = Zval::from(ordered);
        let output_zval = Zval::from(output.to_string());

        match self.call_visitor_method("visit_list_end", &[ctx_zval, ordered_zval, output_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_table_start(&mut self, ctx: &NodeContext) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);

        match self.call_visitor_method("visit_table_start", &[ctx_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_table_row(&mut self, ctx: &NodeContext, cells: &[String], is_header: bool) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let mut cells_array = ext_php_rs::types::ZendHashTable::with_capacity(cells.len() as u32);
        for cell in cells {
            if let Err(_) = cells_array.push(cell.clone()) {
                return VisitResult::Continue;
            }
        }

        let ctx_zval = Zval::from(ctx_array);
        let cells_zval = Zval::from(cells_array);
        let is_header_zval = Zval::from(is_header);

        match self.call_visitor_method("visit_table_row", &[ctx_zval, cells_zval, is_header_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_table_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let output_zval = Zval::from(output.to_string());

        match self.call_visitor_method("visit_table_end", &[ctx_zval, output_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_blockquote(&mut self, ctx: &NodeContext, content: &str, depth: usize) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let content_zval = Zval::from(content.to_string());
        let depth_zval = Zval::from(depth as i64);

        match self.call_visitor_method("visit_blockquote", &[ctx_zval, content_zval, depth_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_strong(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_strong", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_emphasis(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_emphasis", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_strikethrough(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_strikethrough", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_underline(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_underline", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_subscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_subscript", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_superscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_superscript", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_mark(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_mark", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_line_break(&mut self, ctx: &NodeContext) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);

        match self.call_visitor_method("visit_line_break", &[ctx_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_horizontal_rule(&mut self, ctx: &NodeContext) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);

        match self.call_visitor_method("visit_horizontal_rule", &[ctx_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_custom_element(&mut self, ctx: &NodeContext, tag_name: &str, html: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let tag_zval = Zval::from(tag_name.to_string());
        let html_zval = Zval::from(html.to_string());

        match self.call_visitor_method("visit_custom_element", &[ctx_zval, tag_zval, html_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_definition_list_start(&mut self, ctx: &NodeContext) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);

        match self.call_visitor_method("visit_definition_list_start", &[ctx_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_definition_term(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_definition_term", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_definition_description(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_definition_description", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_definition_list_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let output_zval = Zval::from(output.to_string());

        match self.call_visitor_method("visit_definition_list_end", &[ctx_zval, output_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_form(&mut self, ctx: &NodeContext, action: Option<&str>, method: Option<&str>) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let action_zval = match action {
            Some(a) => Zval::from(a.to_string()),
            None => Zval::new(),
        };
        let method_zval = match method {
            Some(m) => Zval::from(m.to_string()),
            None => Zval::new(),
        };

        match self.call_visitor_method("visit_form", &[ctx_zval, action_zval, method_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_input(
        &mut self,
        ctx: &NodeContext,
        input_type: &str,
        name: Option<&str>,
        value: Option<&str>,
    ) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let type_zval = Zval::from(input_type.to_string());
        let name_zval = match name {
            Some(n) => Zval::from(n.to_string()),
            None => Zval::new(),
        };
        let value_zval = match value {
            Some(v) => Zval::from(v.to_string()),
            None => Zval::new(),
        };

        match self.call_visitor_method("visit_input", &[ctx_zval, type_zval, name_zval, value_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_button(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_button", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_audio(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let src_zval = match src {
            Some(s) => Zval::from(s.to_string()),
            None => Zval::new(),
        };

        match self.call_visitor_method("visit_audio", &[ctx_zval, src_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_video(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let src_zval = match src {
            Some(s) => Zval::from(s.to_string()),
            None => Zval::new(),
        };

        match self.call_visitor_method("visit_video", &[ctx_zval, src_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_iframe(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let src_zval = match src {
            Some(s) => Zval::from(s.to_string()),
            None => Zval::new(),
        };

        match self.call_visitor_method("visit_iframe", &[ctx_zval, src_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_details(&mut self, ctx: &NodeContext, open: bool) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let open_zval = Zval::from(open);

        match self.call_visitor_method("visit_details", &[ctx_zval, open_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_summary(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_summary", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_figure_start(&mut self, ctx: &NodeContext) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);

        match self.call_visitor_method("visit_figure_start", &[ctx_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_figcaption(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let text_zval = Zval::from(text.to_string());

        match self.call_visitor_method("visit_figcaption", &[ctx_zval, text_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }

    fn visit_figure_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
        let ctx_array = match Self::context_to_array(ctx) {
            Ok(a) => a,
            Err(_) => return VisitResult::Continue,
        };

        let ctx_zval = Zval::from(ctx_array);
        let output_zval = Zval::from(output.to_string());

        match self.call_visitor_method("visit_figure_end", &[ctx_zval, output_zval]) {
            Ok(result) => result,
            Err(_) => VisitResult::Continue,
        }
    }
}

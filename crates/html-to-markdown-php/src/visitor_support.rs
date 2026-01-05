use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};

/// PHP visitor bridge for custom HTML traversal callbacks.
///
/// This struct bridges PHP callbacks to the Rust HtmlVisitor trait,
/// allowing PHP code to customize HTML-to-Markdown conversion.
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhpVisitorBridge {
    visitor: Zval,
}

impl PhpVisitorBridge {
    /// Create a new bridge wrapping a PHP visitor object.
    #[allow(dead_code)]
    pub fn new(visitor: Zval) -> Self {
        Self { visitor }
    }

    /// Convert a PHP array result to a VisitResult enum.
    #[allow(dead_code)]
    fn result_from_array(array: &ext_php_rs::types::ZendHashTable) -> PhpResult<VisitResult> {
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
    #[allow(dead_code)]
    fn context_to_array(ctx: &NodeContext) -> PhpResult<ext_php_rs::boxed::ZBox<ext_php_rs::types::ZendHashTable>> {
        let mut table = ext_php_rs::types::ZendHashTable::new();

        let node_type_str = format!("{:?}", ctx.node_type).to_lowercase();
        table.insert("node_type", node_type_str)?;

        table.insert("tag_name", ctx.tag_name.clone())?;

        let mut attrs = ext_php_rs::types::ZendHashTable::new();
        for (k, v) in &ctx.attributes {
            attrs.insert(k.clone(), v.clone())?;
        }
        table.insert("attributes", attrs)?;

        table.insert("depth", ctx.depth as i64)?;

        table.insert("index_in_parent", ctx.index_in_parent as i64)?;

        match &ctx.parent_tag {
            Some(tag) => table.insert("parent_tag", tag.clone())?,
            None => table.insert("parent_tag", ())?,
        }

        table.insert("is_inline", ctx.is_inline)?;

        Ok(table)
    }

    /// Call a PHP visitor method and convert the result.
    #[allow(dead_code)]
    fn call_visitor_method(&self, method_name: &str, args: Vec<Zval>) -> PhpResult<VisitResult> {
        if !self.visitor.is_object() {
            return Ok(VisitResult::Continue);
        }

        let borrowed_args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> = args
            .iter()
            .map(|arg| arg as &dyn ext_php_rs::convert::IntoZvalDyn)
            .collect();

        match self.visitor.try_call_method(method_name, borrowed_args) {
            Ok(php_result) => {
                if let Some(array) = php_result.array() {
                    match Self::result_from_array(array) {
                        Ok(result) => Ok(result),
                        Err(_) => Ok(VisitResult::Continue),
                    }
                } else {
                    Ok(VisitResult::Continue)
                }
            }
            Err(_) => Ok(VisitResult::Continue),
        }
    }
}

impl HtmlVisitor for PhpVisitorBridge {
    fn visit_element_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        // TODO: Implement PHP visitor callback for element start
        VisitResult::Continue
    }

    fn visit_element_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        // TODO: Implement PHP visitor callback for element end
        VisitResult::Continue
    }

    fn visit_text(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_link(&mut self, _ctx: &NodeContext, _href: &str, _text: &str, _title: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_image(&mut self, _ctx: &NodeContext, _src: &str, _alt: &str, _title: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_heading(&mut self, _ctx: &NodeContext, _level: u32, _text: &str, _id: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_code_block(&mut self, _ctx: &NodeContext, _lang: Option<&str>, _code: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_code_inline(&mut self, _ctx: &NodeContext, _code: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_list_item(&mut self, _ctx: &NodeContext, _ordered: bool, _marker: &str, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_list_start(&mut self, _ctx: &NodeContext, _ordered: bool) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_list_end(&mut self, _ctx: &NodeContext, _ordered: bool, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_table_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_table_row(&mut self, _ctx: &NodeContext, _cells: &[String], _is_header: bool) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_table_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_blockquote(&mut self, _ctx: &NodeContext, _content: &str, _depth: usize) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_strong(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_emphasis(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_strikethrough(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_underline(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_subscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_superscript(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_mark(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_line_break(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_horizontal_rule(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_custom_element(&mut self, _ctx: &NodeContext, _tag_name: &str, _html: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_definition_list_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_definition_term(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_definition_description(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_definition_list_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_form(&mut self, _ctx: &NodeContext, _action: Option<&str>, _method: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_input(
        &mut self,
        _ctx: &NodeContext,
        _input_type: &str,
        _name: Option<&str>,
        _value: Option<&str>,
    ) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_button(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_audio(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_video(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_iframe(&mut self, _ctx: &NodeContext, _src: Option<&str>) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_details(&mut self, _ctx: &NodeContext, _open: bool) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_summary(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_figure_start(&mut self, _ctx: &NodeContext) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_figcaption(&mut self, _ctx: &NodeContext, _text: &str) -> VisitResult {
        VisitResult::Continue
    }

    fn visit_figure_end(&mut self, _ctx: &NodeContext, _output: &str) -> VisitResult {
        VisitResult::Continue
    }
}

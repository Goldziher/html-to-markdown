//! HTML-to-Markdown MCP server implementation.

use crate::options::ConversionOptions;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, CompleteRequestParams, CompleteResult, ContentBlock, GetPromptRequestParams, GetPromptResponse,
        Implementation, InitializeResult, JsonObject, ListPromptsResult, ListResourcesResult, PaginatedRequestParams,
        PromptsCapability, ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse, ResourcesCapability,
        ServerCapabilities, ServerConfig, ToolsCapability,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
};

#[cfg(feature = "mcp-http")]
mod http_hardening;
#[cfg(feature = "mcp-http")]
use http_hardening::build_http_router;

/// HTML-to-Markdown MCP server.
///
/// Exposes two tools:
/// - `convert_html` — convert HTML to Markdown (or full JSON output) with typed
///   `ConvertConfig` options.
/// - `extract_metadata` — extract structured `<head>`/`<meta>` metadata as JSON.
#[cfg_attr(alef, alef(skip))]
#[derive(Clone)]
pub struct HtmlToMarkdownMcp {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl HtmlToMarkdownMcp {
    /// Create a new server instance.
    pub(crate) fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Convert HTML to Markdown.
    ///
    /// Converts the provided HTML string to Markdown using the html-to-markdown engine.
    /// Pass `json: true` to receive the full `ConversionResult` as a JSON object
    /// (including content, tables, document structure, metadata, and warnings).
    /// Pass `config` to customise conversion behaviour with typed options.
    #[tool(
        description = "Convert HTML to Markdown (or Djot/plain via config.output_format). Pass json:true for the full ConversionResult (content, tables, document structure, metadata, warnings). Pass config to control heading style, list formatting, escaping, preprocessing, image extraction, and more — see the input schema for every option.",
        annotations(
            title = "Convert HTML",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    #[tracing::instrument(
        level = "debug",
        name = "html_to_markdown::mcp::convert_html",
        skip_all,
        fields(
            input_len = tracing::field::Empty,
            json = tracing::field::Empty,
            config_present = tracing::field::Empty,
            output_len = tracing::field::Empty,
        )
    )]
    async fn convert_html(
        &self,
        Parameters(params): Parameters<super::params::ConvertHtmlParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        use super::errors::{map_conversion_error_to_mcp, map_invalid_enum_to_mcp};
        use super::format::{conversion_result_value, format_conversion_result};

        let span = tracing::Span::current();
        span.record("input_len", params.html.len());
        span.record("json", params.json);
        span.record("config_present", params.config.is_some());

        let opts: ConversionOptions = match params.config {
            Some(config) => config.try_into().map_err(|error: super::params::InvalidEnumValue| {
                tracing::warn!(
                    target: "html_to_markdown::mcp",
                    field = error.field,
                    value = %error.value,
                    "convert_html rejected an unrecognized enum value"
                );
                map_invalid_enum_to_mcp(error)
            })?,
            None => ConversionOptions::default(),
        };

        let html = params.html;
        let want_json = params.json;

        // ~keep spawn_blocking runs on a separate blocking-pool thread that does not inherit
        // ~keep the calling task's tracing span automatically; entering the captured span
        // ~keep inside the (fully synchronous) closure restores the parent/child relationship
        // ~keep with crate::convert's own `html_to_markdown::convert` span.
        let mcp_span = tracing::Span::current();
        let result = tokio::task::spawn_blocking(move || {
            let _guard = mcp_span.enter();
            crate::convert(&html, opts)
        })
        .await
        .map_err(|e| {
            tracing::error!(target: "html_to_markdown::mcp", error = %e, "convert_html task panicked");
            rmcp::ErrorData::internal_error(format!("Conversion task panicked: {e}"), None)
        })?
        .map_err(|error| {
            tracing::warn!(target: "html_to_markdown::mcp", error = %error, "convert_html conversion failed");
            map_conversion_error_to_mcp(error)
        })?;

        if want_json {
            // SEP-2106: return the full ConversionResult as structuredContent (a JSON value),
            // with the pretty-printed JSON mirrored into text content for clients that ignore it.
            let text = format_conversion_result(&result);
            span.record("output_len", text.len());
            let mut tool_result = CallToolResult::success(vec![ContentBlock::text(text)]);
            tool_result.structured_content = Some(conversion_result_value(&result));
            Ok(tool_result)
        } else {
            let content = result.content.unwrap_or_default();
            span.record("output_len", content.len());
            Ok(CallToolResult::success(vec![ContentBlock::text(content)]))
        }
    }

    /// Extract structured metadata from HTML.
    ///
    /// Runs the metadata extraction pass and returns only the `HtmlMetadata`
    /// (document title/description, Open Graph, Twitter Card, JSON-LD/microdata,
    /// headers, links, images) serialised as JSON.
    #[tool(
        description = "Extract structured metadata from HTML as JSON: document title/description/keywords/author, Open Graph and Twitter Card tags, JSON-LD and microdata, plus header, link, and image inventories. Convenience over convert_html for metadata-only use.",
        annotations(
            title = "Extract HTML Metadata",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    #[tracing::instrument(
        level = "debug",
        name = "html_to_markdown::mcp::extract_metadata",
        skip_all,
        fields(input_len = tracing::field::Empty, output_len = tracing::field::Empty)
    )]
    async fn extract_metadata(
        &self,
        Parameters(params): Parameters<super::params::ExtractMetadataParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        use super::errors::map_conversion_error_to_mcp;
        use super::format::{format_metadata, metadata_value};

        let span = tracing::Span::current();
        span.record("input_len", params.html.len());

        let opts = ConversionOptions {
            extract_metadata: true,
            ..ConversionOptions::default()
        };
        let html = params.html;

        // ~keep see convert_html: entering the captured span inside the blocking closure
        // ~keep restores parent/child span linkage across the spawn_blocking boundary.
        let mcp_span = tracing::Span::current();
        let result = tokio::task::spawn_blocking(move || {
            let _guard = mcp_span.enter();
            crate::convert(&html, opts)
        })
        .await
        .map_err(|e| {
            tracing::error!(target: "html_to_markdown::mcp", error = %e, "extract_metadata task panicked");
            rmcp::ErrorData::internal_error(format!("Conversion task panicked: {e}"), None)
        })?
        .map_err(|error| {
            tracing::warn!(target: "html_to_markdown::mcp", error = %error, "extract_metadata conversion failed");
            map_conversion_error_to_mcp(error)
        })?;

        // SEP-2106: metadata is already structured JSON — expose it as structuredContent too.
        let text = format_metadata(&result.metadata);
        span.record("output_len", text.len());
        let mut tool_result = CallToolResult::success(vec![ContentBlock::text(text)]);
        tool_result.structured_content = Some(metadata_value(&result.metadata));
        Ok(tool_result)
    }
}

#[tool_handler]
impl ServerHandler for HtmlToMarkdownMcp {
    fn get_info(&self) -> ServerConfig {
        let mut capabilities = ServerCapabilities::default();
        capabilities.tools = Some(ToolsCapability::default());
        capabilities.prompts = Some(PromptsCapability::default());
        capabilities.resources = Some(ResourcesCapability::default());
        capabilities.completions = Some(JsonObject::default());

        let server_info = Implementation::new("html-to-markdown-mcp", env!("CARGO_PKG_VERSION"))
            .with_title("HTML-to-Markdown MCP Server")
            .with_description(
                "Fast, lossless HTML to Markdown conversion. \
                 Supports optional ConversionOptions for heading style, list formatting, \
                 escaping, metadata extraction, and more.",
            )
            .with_website_url("https://github.com/xberg-io/html-to-markdown");

        InitializeResult::new(capabilities)
            // ~keep Pin the advertised revision so an SDK upgrade cannot change protocol compatibility implicitly.
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(server_info)
            .with_instructions(
                "Two tools are available. convert_html converts an HTML string to Markdown \
                 (or Djot/plain via config.output_format); pass json:true for the full \
                 ConversionResult (content, tables, document structure, metadata, warnings), \
                 and pass config for typed options (heading_style, escape_asterisks, \
                 preprocessing, extract_images, …) — every option is described in the tool's \
                 input schema. extract_metadata returns only the structured metadata \
                 (title, Open Graph, Twitter Card, JSON-LD, headers, links, images) as JSON. \
                 Prompts (convert_to_markdown, extract_main_content, inspect_metadata) provide \
                 ready-made workflows, and the htmltomarkdown:// resources document every option.",
            )
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(super::catalog::list_prompts())
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, McpError> {
        super::catalog::get_prompt(&request.name, request.arguments.as_ref()).map(Into::into)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(super::catalog::list_resources())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        super::catalog::read_resource(&request.uri).map(Into::into)
    }

    async fn complete(
        &self,
        request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, McpError> {
        Ok(super::catalog::complete(&request.r#ref, &request.argument))
    }
}

/// Start the HTML-to-Markdown MCP server using stdio transport.
///
/// Blocks until the server shuts down.
///
/// # Errors
///
/// Returns an error if the server fails to start or encounters a fatal error.
///
/// # Example
///
/// ```rust,no_run
/// use html_to_markdown_rs::mcp::start_mcp_server;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     start_mcp_server().await?;
///     Ok(())
/// }
/// ```
#[cfg_attr(alef, alef(skip))]
pub async fn start_mcp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = HtmlToMarkdownMcp::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Start the HTML-to-Markdown MCP server with HTTP Stream transport.
///
/// Applies production hardening (see [`http_hardening`]) before binding:
/// a request body size limit, a request timeout, a global concurrency limit,
/// and `Origin`/`Host` validation that defaults to rejecting anything but the
/// bound `host:port` (DNS-rebinding protection).
///
/// # Arguments
///
/// * `host` - Host to bind to (e.g., `"127.0.0.1"` or `"0.0.0.0"`)
/// * `port` - Port number (e.g., `8001`)
///
/// # Example
///
/// ```no_run
/// use html_to_markdown_rs::mcp::start_mcp_server_http;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     start_mcp_server_http("127.0.0.1", 8001).await?;
///     Ok(())
/// }
/// ```
#[cfg(feature = "mcp-http")]
#[cfg_attr(alef, alef(skip))]
pub async fn start_mcp_server_http(
    host: impl AsRef<str>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::net::SocketAddr;

    let router = build_http_router(host.as_ref(), port);

    let addr: SocketAddr = format!("{}:{}", host.as_ref(), port)
        .parse()
        .map_err(|e| format!("Invalid address: {e}"))?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::params::{ConvertConfig, ConvertHtmlParams, ExtractMetadataParams};
    use super::*;
    use rmcp::model::ProtocolVersion;

    fn text_of(result: &CallToolResult) -> String {
        match &result.content[0] {
            rmcp::model::ContentBlock::Text(t) => t.text.clone(),
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_tool_router_has_both_tools() {
        let router = HtmlToMarkdownMcp::tool_router();
        assert!(router.has_route("convert_html"), "convert_html tool must be registered");
        assert!(
            router.has_route("extract_metadata"),
            "extract_metadata tool must be registered"
        );
    }

    #[test]
    fn test_tools_carry_read_only_annotations() {
        let router = HtmlToMarkdownMcp::tool_router();
        for name in ["convert_html", "extract_metadata"] {
            let tool = router.get(name).unwrap_or_else(|| panic!("{name} must exist"));
            let ann = tool
                .annotations
                .as_ref()
                .unwrap_or_else(|| panic!("{name} must carry annotations"));
            assert_eq!(ann.read_only_hint, Some(true), "{name} read_only_hint");
            assert_eq!(ann.idempotent_hint, Some(true), "{name} idempotent_hint");
            assert_eq!(ann.destructive_hint, Some(false), "{name} destructive_hint");
            assert_eq!(ann.open_world_hint, Some(false), "{name} open_world_hint");
            assert!(ann.title.is_some(), "{name} title");
        }
    }

    #[test]
    fn test_convert_html_input_schema_exposes_typed_config() {
        let router = HtmlToMarkdownMcp::tool_router();
        let tool = router.get("convert_html").expect("convert_html must exist");
        let schema = serde_json::to_string(&tool.input_schema).expect("schema serialises");
        assert!(
            schema.contains("heading_style"),
            "input schema must expose heading_style"
        );
        assert!(
            schema.contains("output_format"),
            "input schema must expose output_format"
        );
        assert!(
            schema.contains("preprocessing"),
            "input schema must expose preprocessing"
        );
    }

    #[test]
    fn test_server_info_fields() {
        let server = HtmlToMarkdownMcp::new();
        let info = server.get_info();

        assert_eq!(info.server_info.name, "html-to-markdown-mcp");
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.capabilities.tools.is_some(), "tools capability");
        assert!(info.capabilities.prompts.is_some(), "prompts capability");
        assert!(info.capabilities.resources.is_some(), "resources capability");
        assert!(info.capabilities.completions.is_some(), "completions capability");
        assert!(info.instructions.is_some());
    }

    #[test]
    fn test_server_info_has_description() {
        let server = HtmlToMarkdownMcp::new();
        let info = server.get_info();
        assert!(info.server_info.title.is_some());
        assert!(info.server_info.website_url.is_some());
    }

    #[test]
    fn test_server_info_protocol_version() {
        let server = HtmlToMarkdownMcp::new();
        let info = server.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::V_2026_07_28);
    }

    #[tokio::test]
    async fn test_convert_html_basic() {
        let server = HtmlToMarkdownMcp::new();
        let params = ConvertHtmlParams {
            html: "<h1>Hello</h1>".into(),
            config: None,
            json: false,
        };
        let result = server
            .convert_html(Parameters(params))
            .await
            .expect("conversion must succeed");

        assert!(!result.content.is_empty(), "result must have content");
        let text = text_of(&result);
        assert!(text.contains("# Hello"), "markdown must contain heading; got: {text}");
    }

    #[tokio::test]
    async fn test_convert_html_json_output() {
        let server = HtmlToMarkdownMcp::new();
        let params = ConvertHtmlParams {
            html: "<h1>World</h1>".into(),
            config: None,
            json: true,
        };
        let result = server
            .convert_html(Parameters(params))
            .await
            .expect("conversion must succeed");

        let text = text_of(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("json output must be valid JSON");
        assert!(parsed.get("content").is_some(), "JSON must have content field");

        // SEP-2106: the json path must also carry structuredContent matching the text.
        let structured = result
            .structured_content
            .as_ref()
            .expect("json output must populate structured_content");
        assert_eq!(
            structured.get("content"),
            parsed.get("content"),
            "structured_content must mirror the text JSON"
        );
    }

    #[tokio::test]
    async fn test_convert_html_with_typed_config() {
        let server = HtmlToMarkdownMcp::new();
        let params = ConvertHtmlParams {
            html: "<p>*bold*</p>".into(),
            config: Some(ConvertConfig {
                escape_asterisks: Some(true),
                ..ConvertConfig::default()
            }),
            json: false,
        };
        let result = server
            .convert_html(Parameters(params))
            .await
            .expect("conversion must succeed");

        let text = text_of(&result);
        assert_eq!(text.trim(), r"\*bold\*", "escape_asterisks must escape both asterisks");
    }

    #[tokio::test]
    async fn should_reject_convert_html_call_with_invalid_enum_value() {
        let server = HtmlToMarkdownMcp::new();
        let params = ConvertHtmlParams {
            html: "<h1>Hi</h1>".into(),
            config: Some(ConvertConfig {
                heading_style: Some("not-a-real-style".into()),
                ..ConvertConfig::default()
            }),
            json: false,
        };
        let error = server
            .convert_html(Parameters(params))
            .await
            .expect_err("unrecognized heading_style must be rejected");

        assert_eq!(
            error.code,
            rmcp::model::ErrorCode::INVALID_PARAMS,
            "must surface as invalid_params, not a silent default substitution"
        );
        assert!(
            error.message.contains("heading_style"),
            "message must name the offending field: {}",
            error.message
        );
        assert!(
            error.message.contains("not-a-real-style"),
            "message must echo the offending value: {}",
            error.message
        );
    }

    #[tokio::test]
    async fn test_convert_html_output_format_djot() {
        let server = HtmlToMarkdownMcp::new();
        let params = ConvertHtmlParams {
            html: "<h1>Hi</h1>".into(),
            config: Some(ConvertConfig {
                output_format: Some("djot".into()),
                ..ConvertConfig::default()
            }),
            json: false,
        };
        let result = server
            .convert_html(Parameters(params))
            .await
            .expect("conversion must succeed");
        assert!(
            text_of(&result).contains("Hi"),
            "djot output must carry the heading text"
        );
    }

    #[tokio::test]
    async fn test_extract_metadata_returns_metadata_json() {
        let server = HtmlToMarkdownMcp::new();
        let html = r#"<html><head><title>My Page</title>
            <meta property="og:title" content="OG Title"></head><body><p>hi</p></body></html>"#;
        let params = ExtractMetadataParams { html: html.into() };
        let result = server
            .extract_metadata(Parameters(params))
            .await
            .expect("metadata extraction must succeed");

        let parsed: serde_json::Value = serde_json::from_str(&text_of(&result)).expect("valid JSON");
        assert_eq!(parsed["document"]["title"], "My Page", "title must be extracted");
        assert!(parsed.get("structured_data").is_some(), "metadata JSON shape present");
    }
}

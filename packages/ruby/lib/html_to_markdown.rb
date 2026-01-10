# frozen_string_literal: true

require_relative 'html_to_markdown/version'
require 'html_to_markdown_rb'

module HtmlToMarkdown
  autoload :CLI, 'html_to_markdown/cli'
  autoload :CLIProxy, 'html_to_markdown/cli_proxy'

  class Options; end # rubocop:disable Lint/EmptyClass

  class << self
    alias native_convert convert
    alias native_convert_with_inline_images convert_with_inline_images
    alias native_convert_with_inline_images_handle convert_with_inline_images_handle
    alias native_options options
    alias native_convert_with_options convert_with_options
    alias native_convert_with_metadata convert_with_metadata
    alias native_convert_with_metadata_handle convert_with_metadata_handle
  end

  module_function

  def convert(html, options = nil, visitor = nil)
    native_convert(html.to_s, options, visitor)
  end

  def convert_with_options(html, options_handle)
    native_convert_with_options(html.to_s, options_handle)
  end

  def convert_with_inline_images(html, options = nil, image_config = nil, visitor = nil)
    native_convert_with_inline_images(html.to_s, options, image_config, visitor)
  end

  def convert_with_inline_images_handle(html, options_handle, image_config = nil)
    native_convert_with_inline_images_handle(html.to_s, options_handle, image_config)
  end

  def options(options_hash = nil)
    native_options(options_hash)
  end

  # Convert HTML to Markdown with comprehensive metadata extraction.
  #
  # Performs HTML-to-Markdown conversion while extracting document metadata, headers,
  # links, images, and structured data in a single pass. Ideal for content analysis,
  # SEO workflows, and document indexing.
  #
  # @param html [String] HTML string to convert. Line endings are normalized (CRLF -> LF).
  # @param options [ConversionOptions, Hash, nil] Optional conversion configuration.
  #   When a Hash, keys should match ConversionOptions field names (as symbols or strings).
  #   Common options:
  #   - :heading_style [String] "atx", "atx_closed", or "underlined" (default: "underlined")
  #   - :list_indent_type [String] "spaces" or "tabs" (default: "spaces")
  #   - :list_indent_width [Integer] Spaces per indent level (default: 4)
  #   - :wrap [true, false] Enable text wrapping (default: false)
  #   - :wrap_width [Integer] Wrap at this column width (default: 80)
  #   See ConversionOptions documentation for complete list.
  #
  # @param metadata_config [Hash, nil] Optional metadata extraction configuration.
  #   Keys should be symbols or strings. Supported keys:
  #   - :extract_headers [true, false] Extract h1-h6 heading elements (default: true)
  #   - :extract_links [true, false] Extract hyperlinks with type classification (default: true)
  #   - :extract_images [true, false] Extract image elements (default: true)
  #   - :extract_structured_data [true, false] Extract JSON-LD/Microdata/RDFa (default: true)
  #   - :max_structured_data_size [Integer] Size limit for structured data in bytes (default: 1_000_000)
  #
  # @return [Array<String, Hash>] Tuple of [markdown_string, metadata_hash]
  #   markdown_string: String - The converted Markdown output
  #
  #   metadata_hash: Hash with keys:
  #   - :document [Hash] Document-level metadata:
  #     - :title [String, nil] From <title> tag
  #     - :description [String, nil] From <meta name="description">
  #     - :keywords [Array<String>] From <meta name="keywords">
  #     - :author [String, nil] From <meta name="author">
  #     - :language [String, nil] From lang attribute (e.g., "en")
  #     - :text_direction [String, nil] "ltr", "rtl", or "auto"
  #     - :canonical_url [String, nil] From <link rel="canonical">
  #     - :base_href [String, nil] From <base href="">
  #     - :open_graph [Hash<String, String>] Open Graph properties (og:* meta tags)
  #     - :twitter_card [Hash<String, String>] Twitter Card properties (twitter:* meta tags)
  #     - :meta_tags [Hash<String, String>] Other meta tags
  #
  #   - :headers [Array<Hash>] Heading elements:
  #     - :level [Integer] 1-6
  #     - :text [String] Header text content
  #     - :id [String, nil] HTML id attribute
  #     - :depth [Integer] Tree nesting depth
  #     - :html_offset [Integer] Byte offset in original HTML
  #
  #   - :links [Array<Hash>] Hyperlinks:
  #     - :href [String] Link URL
  #     - :text [String] Link text content
  #     - :title [String, nil] Title attribute
  #     - :link_type [String] "anchor", "internal", "external", "email", "phone", or "other"
  #     - :rel [Array<String>] Rel attribute values
  #     - :attributes [Hash<String, String>] Additional HTML attributes
  #
  #   - :images [Array<Hash>] Image elements:
  #     - :src [String] Image source URL or data URI
  #     - :alt [String, nil] Alt text for accessibility
  #     - :title [String, nil] Title attribute
  #     - :dimensions [Array<Integer>, nil] [width, height] if available
  #     - :image_type [String] "data_uri", "external", "relative", or "inline_svg"
  #     - :attributes [Hash<String, String>] Additional HTML attributes
  #
  #   - :structured_data [Array<Hash>] Structured data blocks:
  #     - :data_type [String] "json_ld", "microdata", or "rdfa"
  #     - :raw_json [String] Raw JSON content
  #     - :schema_type [String, nil] Schema type (e.g., "Article", "Event")
  #
  # @raise [StandardError] If conversion fails or invalid configuration
  #
  # @example Basic usage
  #   html = <<~HTML
  #     <html lang="en">
  #       <head>
  #         <title>My Article</title>
  #         <meta name="description" content="A great read">
  #       </head>
  #       <body>
  #         <h1 id="intro">Introduction</h1>
  #         <p>Visit <a href="https://example.com">our site</a></p>
  #         <img src="photo.jpg" alt="Beautiful landscape">
  #       </body>
  #     </html>
  #   HTML
  #
  #   markdown, metadata = HtmlToMarkdown.convert_with_metadata(html)
  #
  #   puts metadata[:document][:title]  # => "My Article"
  #   puts metadata[:document][:language]  # => "en"
  #   puts metadata[:headers].length  # => 1
  #   puts metadata[:headers][0][:text]  # => "Introduction"
  #   puts metadata[:links].length  # => 1
  #   puts metadata[:images].length  # => 1
  #
  # @example With selective metadata extraction
  #   config = {
  #     extract_headers: true,
  #     extract_links: true,
  #     extract_images: false,      # Skip images
  #     extract_structured_data: false  # Skip structured data
  #   }
  #
  #   markdown, metadata = HtmlToMarkdown.convert_with_metadata(html, nil, config)
  #   puts metadata[:images].empty?  # => true (not extracted)
  #
  # @example With conversion options
  #   options = {
  #     heading_style: "atx",     # Use # H1, ## H2 style
  #     wrap: true,
  #     wrap_width: 80
  #   }
  #
  #   config = { extract_headers: true }
  #
  #   markdown, metadata = HtmlToMarkdown.convert_with_metadata(html, options, config)
  #   # Markdown uses ATX-style headings and wraps at 80 characters
  #
  # @see #convert Simple conversion without metadata
  # @see #convert_with_inline_images Extract inline images during conversion
  # @see ConversionOptions Detailed conversion configuration
  def convert_with_metadata(html, options = nil, metadata_config = nil, visitor = nil)
    native_convert_with_metadata(html.to_s, options, metadata_config, visitor)
  end

  def convert_with_metadata_handle(html, options_handle, metadata_config = nil)
    native_convert_with_metadata_handle(html.to_s, options_handle, metadata_config)
  end
end

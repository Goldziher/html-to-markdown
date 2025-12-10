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
    alias native_options options
    alias native_convert_with_options convert_with_options
    alias native_convert_with_metadata convert_with_metadata
  end

  module_function

  def convert(html, options = nil)
    native_convert(html.to_s, options)
  end

  def convert_with_options(html, options_handle)
    native_convert_with_options(html.to_s, options_handle)
  end

  def convert_with_inline_images(html, options = nil, image_config = nil)
    native_convert_with_inline_images(html.to_s, options, image_config)
  end

  def options(options_hash = nil)
    native_options(options_hash)
  end

  # Convert HTML to Markdown with comprehensive metadata extraction
  #
  # @param html [String] HTML string to convert
  # @param options [Hash, nil] Optional conversion configuration
  # @param metadata_config [Hash, nil] Optional metadata extraction configuration
  # @return [Array<String, Hash>] Array containing [markdown_string, metadata_hash]
  def convert_with_metadata(html, options = nil, metadata_config = nil)
    native_convert_with_metadata(html.to_s, options, metadata_config)
  end
end

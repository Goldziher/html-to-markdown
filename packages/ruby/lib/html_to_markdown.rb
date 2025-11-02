# frozen_string_literal: true

require_relative 'html_to_markdown/version'
require 'html_to_markdown_rb'

module HtmlToMarkdown
  autoload :CLI, 'html_to_markdown/cli'
  autoload :CLIProxy, 'html_to_markdown/cli_proxy'

  class << self
    alias native_convert convert
    alias native_convert_with_inline_images convert_with_inline_images
  end

  module_function

  def convert(html, options = nil)
    native_convert(html.to_s, options)
  end

  def convert_with_inline_images(html, options = nil, image_config = nil)
    native_convert_with_inline_images(html.to_s, options, image_config)
  end
end

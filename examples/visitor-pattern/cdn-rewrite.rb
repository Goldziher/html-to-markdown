#!/usr/bin/env ruby
# frozen_string_literal: true

##
# CDN URL Rewriting Example (Ruby)
#
# Demonstrates how to rewrite image and link URLs to use a new CDN domain.
# Useful for content migration, multi-CDN strategies, or URL standardization.

require 'html_to_markdown'

class CdnRewriter
  attr_reader :rewrites

  def initialize(old_cdn, new_cdn)
    @old_cdn = old_cdn
    @new_cdn = new_cdn
    @rewrites = 0
  end

  def visit_image(ctx, src, alt = nil, title = nil)
    if src.start_with?(@old_cdn)
      src = src.sub(@old_cdn, @new_cdn)
      @rewrites += 1
      { type: :custom, output: "![#{alt || ''}](#{src})" }
    else
      { type: :continue }
    end
  end

  def visit_link(ctx, href, text, title = nil)
    if href.start_with?(@old_cdn)
      href = href.sub(@old_cdn, @new_cdn)
      @rewrites += 1
      { type: :custom, output: "[#{text}](#{href})" }
    else
      { type: :continue }
    end
  end
end

def main
  html = <<~HTML
    <h1>Content Migration Example</h1>
    <p>We're migrating from our old CDN to a new one.</p>
    <img src="https://old-cdn.example.com/images/hero.jpg" alt="Hero image" width="800">
    <p>Download our <a href="https://old-cdn.example.com/files/guide.pdf">guide</a>.</p>
    <p>External link: <a href="https://other.com/page">Other site</a></p>
    <img src="https://other-cdn.com/image.png" alt="Other CDN">
  HTML

  visitor = CdnRewriter.new(
    'https://old-cdn.example.com',
    'https://new-cdn.example.com'
  )

  markdown = HtmlToMarkdown.convert_with_visitor(html, visitor: visitor)

  puts '=' * 70
  puts 'CDN URL Rewriting Example (Ruby)'
  puts '=' * 70
  puts
  puts 'Original HTML:'
  puts '-' * 70
  puts html.strip
  puts
  puts 'Converted Markdown:'
  puts '-' * 70
  puts markdown
  puts
  puts "URLs rewritten: #{visitor.rewrites}"
  puts
end

main if __FILE__ == $PROGRAM_NAME

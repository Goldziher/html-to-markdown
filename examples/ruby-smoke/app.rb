# frozen_string_literal: true

require 'html_to_markdown'

html = <<~HTML
  <h1>Ruby Smoke Test</h1>
  <p>Exercises the packaged Magnus bindings.</p>
HTML

markdown = HtmlToMarkdown.convert(html, heading_style: 'atx')

abort('html-to-markdown did not return the expected heading') unless markdown.include?('# Ruby Smoke Test')

puts '✓ html-to-markdown (Ruby) produced markdown'
puts '---'
puts markdown

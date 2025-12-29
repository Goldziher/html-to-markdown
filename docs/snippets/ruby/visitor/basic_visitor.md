```ruby
require 'html_to_markdown'

class MyVisitor
  def visit_link(ctx, href, text, title = nil)
    { type: :custom, output: "[#{text}](#{href})" }
  end

  def visit_image(ctx, src, alt, title = nil)
    { type: :skip }  # Remove images
  end
end

html = "<p><a href='https://example.com'>Link</a></p>"
result = HtmlToMarkdown.convert_with_visitor(html, visitor: MyVisitor.new)
```

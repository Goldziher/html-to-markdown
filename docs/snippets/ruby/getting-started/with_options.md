```ruby
require 'html_to_markdown'

html = "<h1>Hello</h1><p>This is <strong>fast</strong>!</p>"
markdown = HtmlToMarkdown.convert(html, heading_style: :atx, code_block_style: :fenced)
```

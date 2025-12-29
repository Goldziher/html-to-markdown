```ruby
require 'html_to_markdown'

html = '<html lang="en"><head><title>Test</title></head><body><h1>Hello</h1></body></html>'
markdown, metadata = HtmlToMarkdown.convert_with_metadata(html)

puts metadata[:document][:title]     # "Test"
puts metadata[:headers].first[:text] # "Hello"
```

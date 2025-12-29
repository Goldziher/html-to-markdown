```python
from html_to_markdown import ConversionOptions, convert

options = ConversionOptions(
    heading_style="atx",
    list_indent_width=2,
)
markdown = convert(html, options)
```

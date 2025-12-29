```python
from html_to_markdown import convert_with_visitor

class CustomVisitor:
    def visit_link(self, ctx, href, text, title):
        # Custom link handling
        return {"type": "continue"}

    def visit_image(self, ctx, src, alt, title):
        # Custom image handling
        return {"type": "continue"}

markdown = convert_with_visitor(html, visitor=CustomVisitor())
```

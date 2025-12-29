```typescript
import { convertWithVisitor } from '@kreuzberg/html-to-markdown';
import { Visitor, NodeContext, VisitResult } from '@kreuzberg/html-to-markdown';

const visitor: Visitor = {
  visitLink(ctx: NodeContext, href: string, text: string): VisitResult {
    // Custom handling for links
    return {
      type: 'custom',
      output: `[${text}](${href})`,
    };
  },
  visitHeading(ctx: NodeContext, level: number, text: string): VisitResult {
    // Custom handling for headings
    return {
      type: 'continue',
    };
  },
};

const markdown = convertWithVisitor('<h1>Title</h1><a href="url">Link</a>', {
  visitor,
});
```

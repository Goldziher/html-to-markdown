```typescript
import { convert, ConversionOptions } from '@kreuzberg/html-to-markdown';

const options: ConversionOptions = {
  headingStyle: 'atx',
  listIndentWidth: 2,
  wrap: true,
};

const markdown = convert('<h1>Title</h1><p>Content</p>', options);
```

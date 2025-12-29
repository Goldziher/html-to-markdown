```typescript
import { convertWithMetadata } from '@kreuzberg/html-to-markdown';

const result = convertWithMetadata('<h1>Title</h1><p>Content</p>');
const { markdown, metadata } = result;

console.log(markdown);           // Converted markdown
console.log(metadata.document);  // Document metadata (title, description, etc.)
console.log(metadata.headers);   // Header elements (h1-h6)
console.log(metadata.links);     // Extracted links
console.log(metadata.images);    // Extracted images
```

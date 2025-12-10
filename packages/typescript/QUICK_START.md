# TypeScript Metadata Extraction - Quick Start Guide

## Installation

The metadata extraction feature is included in the `html-to-markdown` package:

```bash
npm install html-to-markdown
# or
pnpm add html-to-markdown
```

## Basic Usage

```typescript
import { convertWithMetadata } from 'html-to-markdown';

const html = `
  <html lang="en">
    <head>
      <title>Hello World</title>
    </head>
    <body>
      <h1>Welcome</h1>
      <p>Check out <a href="https://example.com">this link</a></p>
    </body>
  </html>
`;

const { markdown, metadata } = convertWithMetadata(html);

// Access extracted metadata
console.log(metadata.document.title);     // "Hello World"
console.log(metadata.document.language);  // "en"
console.log(metadata.headers[0].text);    // "Welcome"
console.log(metadata.links[0].href);      // "https://example.com"
```

## API Overview

### Functions

#### `convertWithMetadata(html, options?, metadataConfig?)`
Convert HTML string to Markdown with metadata extraction.

```typescript
const { markdown, metadata } = convertWithMetadata(html);
```

#### `convertWithMetadataBuffer(buffer, options?, metadataConfig?)`
Convert from Buffer/Uint8Array (zero-copy).

```typescript
const buffer = Buffer.from(html);
const { markdown, metadata } = convertWithMetadataBuffer(buffer);
```

#### `convertFileWithMetadata(filePath, options?, metadataConfig?)`
Convert HTML file asynchronously.

```typescript
const { markdown, metadata } = await convertFileWithMetadata('page.html');
```

#### `convertStreamWithMetadata(stream, options?, metadataConfig?)`
Convert from Node.js stream.

```typescript
import fs from 'node:fs';
const stream = fs.createReadStream('page.html', 'utf8');
const { markdown, metadata } = await convertStreamWithMetadata(stream);
```

## Configuration

```typescript
import { convertWithMetadata, type JsMetadataConfig } from 'html-to-markdown';

const config: JsMetadataConfig = {
  extractHeaders: true,              // H1-H6 elements
  extractLinks: true,                // <a> elements
  extractImages: true,               // <img> and <svg>
  extractStructuredData: true,       // JSON-LD, Microdata, RDFa
  maxStructuredDataSize: 1_000_000,  // 1MB limit
};

const result = convertWithMetadata(html, undefined, config);
```

## Metadata Structure

### Document Metadata
```typescript
const doc = result.metadata.document;
console.log(doc.title);           // Page title
console.log(doc.description);     // Meta description
console.log(doc.keywords);        // Array of keywords
console.log(doc.author);          // Author name
console.log(doc.canonicalUrl);    // Canonical URL
console.log(doc.language);        // Language code (e.g., "en")
console.log(doc.textDirection);   // "ltr" | "rtl" | "auto"
console.log(doc.openGraph);       // og:* properties
console.log(doc.twitterCard);     // twitter:* properties
```

### Headers
```typescript
result.metadata.headers.forEach(header => {
  console.log(`H${header.level}: ${header.text}`);
  console.log(`ID: ${header.id}`);
  console.log(`Depth: ${header.depth}`);
});
```

### Links
```typescript
result.metadata.links.forEach(link => {
  console.log(`[${link.text}](${link.href})`);
  console.log(`Type: ${link.linkType}`); // "internal", "external", "email", etc.
  console.log(`Rel: ${link.rel.join(' ')}`);
});
```

### Images
```typescript
result.metadata.images.forEach(img => {
  console.log(`![${img.alt}](${img.src})`);
  console.log(`Type: ${img.imageType}`); // "external", "data_uri", "inline_svg", etc.
  if (img.dimensions) {
    console.log(`Size: ${img.dimensions[0]}x${img.dimensions[1]}`);
  }
});
```

### Structured Data
```typescript
result.metadata.structuredData.forEach(data => {
  console.log(`Type: ${data.dataType}`); // "json_ld", "microdata", "rdfa"
  const schema = JSON.parse(data.rawJson);
  console.log(`Schema: ${data.schemaType}`);
});
```

## Common Use Cases

### Extract SEO Metadata
```typescript
function getSeoMetadata(html: string) {
  const { metadata } = convertWithMetadata(html);
  const doc = metadata.document;

  return {
    title: doc.title,
    description: doc.description,
    keywords: doc.keywords,
    canonical: doc.canonicalUrl,
    ogImage: doc.openGraph.image,
    twitterCard: doc.twitterCard.card,
  };
}
```

### Build Table of Contents
```typescript
function buildTOC(html: string) {
  const { metadata } = convertWithMetadata(html);

  return metadata.headers.map(h => ({
    level: h.level,
    text: h.text,
    anchor: h.id || h.text.toLowerCase().replace(/\s+/g, '-'),
  }));
}
```

### Extract All Links
```typescript
function getLinks(html: string) {
  const { metadata } = convertWithMetadata(html);

  return {
    internal: metadata.links.filter(l => l.linkType === 'internal'),
    external: metadata.links.filter(l => l.linkType === 'external'),
    emails: metadata.links.filter(l => l.linkType === 'email'),
  };
}
```

### Extract Images
```typescript
function getImages(html: string) {
  const { metadata } = convertWithMetadata(html);

  return metadata.images.map(img => ({
    url: img.src,
    alt: img.alt || 'No description',
    title: img.title,
  }));
}
```

### Check Accessibility
```typescript
function checkA11y(html: string) {
  const { metadata } = convertWithMetadata(html);

  return {
    imagesWithoutAlt: metadata.images
      .filter(img => !img.alt)
      .map(img => img.src),
    linksWithoutText: metadata.links
      .filter(link => !link.text.trim())
      .map(link => link.href),
  };
}
```

## Performance Tips

1. **Use Buffer variant for large files**:
   ```typescript
   const buffer = await fs.promises.readFile('large.html');
   const result = convertWithMetadataBuffer(buffer);
   ```

2. **Use Stream for processing**:
   ```typescript
   const stream = fs.createReadStream('large.html', 'utf8');
   const result = await convertStreamWithMetadata(stream);
   ```

3. **Disable unused metadata extraction**:
   ```typescript
   const result = convertWithMetadata(html, undefined, {
     extractHeaders: true,
     extractLinks: false,    // Skip if not needed
     extractImages: false,   // Skip if not needed
     extractStructuredData: false,
   });
   ```

## Type Safety

All metadata types are fully typed for TypeScript:

```typescript
import {
  type JsMetadataExtraction,
  type JsDocumentMetadata,
  type JsHeaderMetadata,
  type JsLinkMetadata,
  type JsImageMetadata,
  type JsStructuredData,
  type JsExtendedMetadata,
  type JsMetadataConfig,
} from 'html-to-markdown';

// Full IDE autocomplete and type checking
const result: JsMetadataExtraction = convertWithMetadata(html);
const title: string | undefined = result.metadata.document.title;
const headers: JsHeaderMetadata[] = result.metadata.headers;
```

## Error Handling

```typescript
try {
  const { markdown, metadata } = convertWithMetadata(html);
  // Use result
} catch (error) {
  console.error('Conversion failed:', error);
}
```

## More Information

- **Complete API Documentation**: See `METADATA.md`
- **Implementation Details**: See `IMPLEMENTATION_SUMMARY.md`
- **Test Examples**: Check `tests/metadata.spec.ts`

## Support

For issues or questions:
- Check the main README.md
- See METADATA.md for detailed API documentation
- Review test cases in `tests/metadata.spec.ts`

#!/usr/bin/env tsx
/**
 * CDN URL Rewriting Example (TypeScript)
 *
 * Demonstrates how to rewrite image and link URLs to use a new CDN domain.
 * Useful for content migration, multi-CDN strategies, or URL standardization.
 */

import { convertWithVisitor, type Visitor, type NodeContext, type VisitResult } from 'html-to-markdown';

class CdnRewriter implements Visitor {
  private oldCdn: string;
  private newCdn: string;
  public rewrites: number = 0;

  constructor(oldCdn: string, newCdn: string) {
    this.oldCdn = oldCdn;
    this.newCdn = newCdn;
  }

  visitImage(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult {
    if (src.startsWith(this.oldCdn)) {
      src = src.replace(this.oldCdn, this.newCdn);
      this.rewrites++;
      return { type: 'custom', output: `![${alt ?? ''}](${src})` };
    }
    return { type: 'continue' };
  }

  visitLink(ctx: NodeContext, href: string, text: string, title?: string): VisitResult {
    if (href.startsWith(this.oldCdn)) {
      href = href.replace(this.oldCdn, this.newCdn);
      this.rewrites++;
      return { type: 'custom', output: `[${text}](${href})` };
    }
    return { type: 'continue' };
  }
}

function main() {
  const html = `
    <h1>Content Migration Example</h1>
    <p>We're migrating from our old CDN to a new one.</p>
    <img src="https://old-cdn.example.com/images/hero.jpg" alt="Hero image" width="800">
    <p>Download our <a href="https://old-cdn.example.com/files/guide.pdf">guide</a>.</p>
    <p>External link: <a href="https://other.com/page">Other site</a></p>
    <img src="https://other-cdn.com/image.png" alt="Other CDN">
  `;

  const visitor = new CdnRewriter(
    'https://old-cdn.example.com',
    'https://new-cdn.example.com'
  );

  const markdown = convertWithVisitor(html, { visitor });

  console.log('='.repeat(70));
  console.log('CDN URL Rewriting Example (TypeScript)');
  console.log('='.repeat(70));
  console.log();
  console.log('Original HTML:');
  console.log('-'.repeat(70));
  console.log(html.trim());
  console.log();
  console.log('Converted Markdown:');
  console.log('-'.repeat(70));
  console.log(markdown);
  console.log();
  console.log(`URLs rewritten: ${visitor.rewrites}`);
  console.log();
}

main();

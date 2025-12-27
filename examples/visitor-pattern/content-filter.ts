#!/usr/bin/env tsx
/**
 * Content Filtering Example (TypeScript)
 *
 * Demonstrates how to remove unwanted elements during conversion:
 * - Ads and tracking elements
 * - Scripts and styles
 * - Tracking pixels (1x1 images)
 * - Elements with specific classes
 */

import { convertWithVisitor, type Visitor, type NodeContext, type VisitResult } from 'html-to-markdown';

class ContentFilter implements Visitor {
  public skippedElements: Array<[string, string]> = [];

  visitDiv(ctx: NodeContext, content: string): VisitResult {
    const classes = ctx.attributes['class'] ?? '';
    const unwantedClasses = ['ad', 'advertisement', 'tracking', 'analytics'];

    if (unwantedClasses.some(cls => classes.includes(cls))) {
      this.skippedElements.push(['div', classes]);
      return { type: 'skip' };
    }
    return { type: 'continue' };
  }

  visitScript(ctx: NodeContext): VisitResult {
    this.skippedElements.push(['script', '']);
    return { type: 'skip' };
  }

  visitStyle(ctx: NodeContext): VisitResult {
    this.skippedElements.push(['style', '']);
    return { type: 'skip' };
  }

  visitImage(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult {
    const width = ctx.attributes['width'] ?? '';
    const height = ctx.attributes['height'] ?? '';

    // Skip 1x1 tracking pixels
    if (width === '1' && height === '1') {
      this.skippedElements.push(['img', `tracking pixel: ${src}`]);
      return { type: 'skip' };
    }

    // Skip images with "tracking" or "analytics" in the URL
    const srcLower = src.toLowerCase();
    if (srcLower.includes('tracking') || srcLower.includes('analytics')) {
      this.skippedElements.push(['img', `tracking URL: ${src}`]);
      return { type: 'skip' };
    }

    return { type: 'continue' };
  }

  visitLink(ctx: NodeContext, href: string, text: string, title?: string): VisitResult {
    // Remove links with utm_* tracking parameters
    if (href.toLowerCase().includes('utm_')) {
      // Strip tracking params but keep the link
      const cleanHref = href.includes('?') ? href.split('?')[0] : href;
      return { type: 'custom', output: `[${text}](${cleanHref})` };
    }

    return { type: 'continue' };
  }
}

function main() {
  const html = `
    <article>
        <h1>Blog Post Title</h1>
        <p>This is the main content of the article.</p>

        <div class="ad advertisement">
            <p>This is an advertisement block that should be removed.</p>
        </div>

        <p>More content here.</p>

        <img src="https://tracking.example.com/pixel.gif" width="1" height="1" alt="">

        <div class="content">
            <p>Legitimate content in a div.</p>
            <img src="https://cdn.example.com/image.jpg" alt="Article image" width="800">
        </div>

        <script>
            console.log("This script should be removed");
        </script>

        <p>Read more on <a href="https://example.com/article?utm_source=newsletter&utm_medium=email">our website</a>.</p>

        <div class="tracking analytics">
            <img src="https://analytics.example.com/track.png" alt="">
        </div>
    </article>
  `;

  const visitor = new ContentFilter();
  const markdown = convertWithVisitor(html, { visitor });

  console.log('='.repeat(70));
  console.log('Content Filtering Example (TypeScript)');
  console.log('='.repeat(70));
  console.log();
  console.log('Original HTML (with ads, tracking, scripts):');
  console.log('-'.repeat(70));
  console.log(html.trim());
  console.log();
  console.log('Filtered Markdown:');
  console.log('-'.repeat(70));
  console.log(markdown);
  console.log();
  console.log('Skipped elements:');
  console.log('-'.repeat(70));
  for (const [tag, info] of visitor.skippedElements) {
    console.log(`  - <${tag}>: ${info}`);
  }
  console.log();
}

main();

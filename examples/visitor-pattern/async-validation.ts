#!/usr/bin/env tsx
/**
 * Asynchronous URL Validation Example (TypeScript)
 *
 * Demonstrates how to validate URLs asynchronously during conversion.
 * Uses fetch to check HTTP status codes of all links and images.
 * Fails conversion if any URLs are broken (404, 500, etc.).
 */

import { convertWithAsyncVisitor, type AsyncVisitor, type NodeContext, type VisitResult } from 'html-to-markdown';

class AsyncUrlValidator implements AsyncVisitor {
  private timeout: number;
  public validatedUrls: Array<[string, number | string]> = [];

  constructor(timeout: number = 5000) {
    this.timeout = timeout;
  }

  async visitLink(ctx: NodeContext, href: string, text: string, title?: string): Promise<VisitResult> {
    // Skip anchor links
    if (href.startsWith('#')) {
      return { type: 'continue' };
    }

    // Skip mailto and tel links
    if (href.startsWith('mailto:') || href.startsWith('tel:')) {
      return { type: 'continue' };
    }

    // Validate HTTP(S) URLs
    if (href.startsWith('http://') || href.startsWith('https://')) {
      const [isValid, status] = await this.validateUrl(href);
      this.validatedUrls.push([href, status]);

      if (!isValid) {
        return {
          type: 'error',
          message: `Broken link (${status}): ${href}`,
        };
      }
    }

    return { type: 'continue' };
  }

  async visitImage(ctx: NodeContext, src: string, alt?: string, title?: string): Promise<VisitResult> {
    // Skip data URIs and relative paths
    if (!src.startsWith('http://') && !src.startsWith('https://')) {
      return { type: 'continue' };
    }

    const [isValid, status] = await this.validateUrl(src);
    this.validatedUrls.push([src, status]);

    if (!isValid) {
      return {
        type: 'error',
        message: `Broken image (${status}): ${src}`,
      };
    }

    return { type: 'continue' };
  }

  private async validateUrl(url: string): Promise<[boolean, number | string]> {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.timeout);

      const response = await fetch(url, {
        method: 'HEAD',
        signal: controller.signal,
        redirect: 'follow',
      });

      clearTimeout(timeoutId);

      // Consider 2xx and 3xx as valid
      const isValid = response.status < 400;
      return [isValid, response.status];
    } catch (error) {
      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          return [false, 'timeout'];
        }
        return [false, `error: ${error.message}`];
      }
      return [false, 'error: unknown'];
    }
  }
}

async function testValidUrls(): Promise<boolean> {
  console.log('Test: Valid URLs');
  console.log('-'.repeat(70));

  const html = `
    <h1>Article with Valid Links</h1>
    <p>Check out <a href="https://www.example.com">Example</a>.</p>
    <img src="https://via.placeholder.com/150" alt="Placeholder image">
    <p>Internal anchor: <a href="#section1">Section 1</a></p>
  `;

  const visitor = new AsyncUrlValidator(10000);
  try {
    const markdown = await convertWithAsyncVisitor(html, { visitor });
    console.log('✓ All URLs are valid');
    console.log();
    console.log('Validated URLs:');
    for (const [url, status] of visitor.validatedUrls) {
      console.log(`  [${status}] ${url}`);
    }
    console.log();
    console.log('Markdown output:');
    console.log('-'.repeat(70));
    console.log(markdown);
    return true;
  } catch (error) {
    console.log(`✗ Unexpected error: ${error}`);
    return false;
  }
}

async function testBrokenLink(): Promise<boolean> {
  console.log('Test: Broken Link (404)');
  console.log('-'.repeat(70));

  const html = `
    <h1>Article with Broken Link</h1>
    <p>This link is broken: <a href="https://httpstat.us/404">404 page</a>.</p>
  `;

  const visitor = new AsyncUrlValidator(10000);
  try {
    const markdown = await convertWithAsyncVisitor(html, { visitor });
    console.log('✗ Should have failed on broken link');
    return false;
  } catch (error) {
    console.log(`✓ Correctly caught broken link: ${error}`);
    console.log();
    console.log('Validated URLs:');
    for (const [url, status] of visitor.validatedUrls) {
      console.log(`  [${status}] ${url}`);
    }
    return true;
  }
}

async function main() {
  console.log('='.repeat(70));
  console.log('Asynchronous URL Validation Example (TypeScript)');
  console.log('='.repeat(70));
  console.log();
  console.log('Note: This example makes real HTTP requests to validate URLs.');
  console.log('Ensure you have an internet connection for the tests to work.');
  console.log();

  const tests: Array<[string, () => Promise<boolean>]> = [
    ['Valid URLs', testValidUrls],
    ['Broken Link (404)', testBrokenLink],
  ];

  const results: Array<[string, boolean]> = [];
  for (const [name, testFunc] of tests) {
    const passed = await testFunc();
    results.push([name, passed]);
    console.log();
  }

  console.log('='.repeat(70));
  console.log('Test Summary:');
  console.log('-'.repeat(70));
  for (const [name, passed] of results) {
    const status = passed ? 'PASS' : 'FAIL';
    console.log(`  ${status}: ${name}`);
  }
  console.log();
}

main();

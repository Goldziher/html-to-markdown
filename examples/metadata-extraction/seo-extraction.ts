#!/usr/bin/env node
/**
 * SEO Metadata Extraction Example (TypeScript)
 *
 * Demonstrates how to extract document metadata including title, description,
 * author, canonical URL, Open Graph tags, and Twitter cards for SEO analysis
 * and social media optimization.
 */

import { convertWithMetadata } from 'html-to-markdown';

interface SeoMetadata {
  title: string | undefined;
  description: string | undefined;
  keywords: string[];
  author: string | undefined;
  language: string | undefined;
  canonicalUrl: string | undefined;
  textDirection: string | undefined;
  openGraph: Record<string, string>;
  twitterCard: Record<string, string>;
  markdown: string;
  headerCount: number;
  linkCount: number;
  imageCount: number;
}

function extractSeoMetadata(html: string): SeoMetadata {
  const { markdown, metadata } = convertWithMetadata(html);
  const doc = metadata.document;

  return {
    title: doc.title,
    description: doc.description,
    keywords: doc.keywords || [],
    author: doc.author,
    language: doc.language,
    canonicalUrl: doc.canonicalUrl,
    textDirection: doc.textDirection,
    openGraph: doc.openGraph || {},
    twitterCard: doc.twitterCard || {},
    markdown,
    headerCount: metadata.headers?.length || 0,
    linkCount: metadata.links?.length || 0,
    imageCount: metadata.images?.length || 0,
  };
}

function main() {
  const html = `
    <html lang="en">
      <head>
        <title>10 Rust Performance Optimization Tips</title>
        <meta name="description" content="Learn practical techniques to optimize Rust code for production.">
        <meta name="keywords" content="Rust, performance, optimization, systems programming">
        <meta name="author" content="Alice Johnson">
        <link rel="canonical" href="https://example.com/rust-performance-tips">
        <meta property="og:title" content="10 Rust Performance Optimization Tips">
        <meta property="og:description" content="Expert tips for making your Rust code faster.">
        <meta property="og:image" content="https://example.com/images/rust-performance.jpg">
        <meta property="og:url" content="https://example.com/rust-performance-tips">
        <meta property="og:type" content="article">
        <meta name="twitter:card" content="summary_large_image">
        <meta name="twitter:creator" content="@alicedeveloper">
        <meta name="twitter:title" content="10 Rust Performance Optimization Tips">
        <meta name="twitter:image" content="https://example.com/images/rust-performance.jpg">
      </head>
      <body>
        <h1>10 Rust Performance Optimization Tips</h1>
        <p>Written by Alice Johnson • Published 2025-01-15</p>

        <h2>Introduction</h2>
        <p>Rust is already fast, but there are techniques to make it even faster. In this guide, we'll explore 10 practical tips for optimizing Rust code in production environments.</p>

        <h2>1. Use Release Mode for Benchmarks</h2>
        <p>Always compile with <code>--release</code> when measuring performance. Debug builds are much slower due to lack of optimizations.</p>

        <h2>2. Profile Your Code</h2>
        <p>Use tools like <code>cargo-flamegraph</code> and <code>perf</code> to identify bottlenecks. Don't guess where time is spent.</p>

        <h2>3. Reduce Allocations</h2>
        <p>Heap allocations are expensive. Use stack-allocated types (<code>Vec::with_capacity</code>, <code>String::with_capacity</code>) when you know the size upfront.</p>

        <h2>External Resources</h2>
        <p>Learn more at <a href="https://docs.rust-embedded.org/book/">The Embedded Rust Book</a> and <a href="https://doc.rust-lang.org/book/">The Rust Book</a>.</p>

        <h2>Author Links</h2>
        <p>Find me on <a href="https://twitter.com/alicedeveloper">Twitter</a>, <a href="https://github.com/alicedeveloper">GitHub</a>, or <a href="mailto:alice@example.com">email me</a>.</p>

        <img src="https://example.com/images/rust-logo.png" alt="Rust programming language logo" width="200" height="200">
      </body>
    </html>
  `;

  const seo = extractSeoMetadata(html);

  console.log('='.repeat(80));
  console.log('SEO METADATA EXTRACTION EXAMPLE');
  console.log('='.repeat(80));
  console.log();

  // Document metadata
  console.log('DOCUMENT METADATA');
  console.log('-'.repeat(80));
  console.log(`Title:           ${seo.title}`);
  console.log(`Description:     ${seo.description}`);
  console.log(`Keywords:        ${seo.keywords.length > 0 ? seo.keywords.join(', ') : 'None'}`);
  console.log(`Author:          ${seo.author}`);
  console.log(`Language:        ${seo.language}`);
  console.log(`Canonical URL:   ${seo.canonicalUrl}`);
  console.log(`Text Direction:  ${seo.textDirection || 'None'}`);
  console.log();

  // Open Graph metadata
  console.log('OPEN GRAPH METADATA (Social Media)');
  console.log('-'.repeat(80));
  if (Object.keys(seo.openGraph).length > 0) {
    for (const [key, value] of Object.entries(seo.openGraph)) {
      console.log(`${key.padEnd(20)} ${value}`);
    }
  } else {
    console.log('No Open Graph metadata found');
  }
  console.log();

  // Twitter Card metadata
  console.log('TWITTER CARD METADATA');
  console.log('-'.repeat(80));
  if (Object.keys(seo.twitterCard).length > 0) {
    for (const [key, value] of Object.entries(seo.twitterCard)) {
      console.log(`${key.padEnd(20)} ${value}`);
    }
  } else {
    console.log('No Twitter Card metadata found');
  }
  console.log();

  // Content analysis
  console.log('CONTENT ANALYSIS');
  console.log('-'.repeat(80));
  console.log(`Headers found:   ${seo.headerCount}`);
  console.log(`Links found:     ${seo.linkCount}`);
  console.log(`Images found:    ${seo.imageCount}`);
  console.log();

  // Preview of converted markdown
  console.log('MARKDOWN OUTPUT PREVIEW');
  console.log('-'.repeat(80));
  const lines = seo.markdown.split('\n');
  const preview = lines.slice(0, 15).join('\n');
  const hasMore = lines.length > 15;
  console.log(preview);
  if (hasMore) {
    console.log(`... (${lines.length - 15} more lines)`);
  }
  console.log();
}

main();

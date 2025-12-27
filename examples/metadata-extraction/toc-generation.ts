#!/usr/bin/env node
/**
 * Table of Contents Generation Example (TypeScript)
 *
 * Demonstrates how to build a hierarchical table of contents from HTML headers.
 * Generates anchor-based navigation with proper heading hierarchy detection.
 */

import { convertWithMetadata } from 'html-to-markdown';

interface HeaderAnalysis {
  totalHeaders: number;
  byLevel: Record<number, number>;
  hierarchyIssues: string[];
  headersWithoutIds: string[];
}

function buildTableOfContents(html: string): string {
  const { metadata } = convertWithMetadata(html);
  const headers = metadata.headers || [];

  if (headers.length === 0) {
    return '# Table of Contents\n\nNo headers found in document.';
  }

  const lines = ['# Table of Contents\n'];

  for (const header of headers) {
    const level = header.level;
    const anchor = header.id || header.text.toLowerCase().replace(/\s+/g, '-');
    const indent = '  '.repeat(level - 1);

    lines.push(`${indent}- [${header.text}](#${anchor})`);
  }

  return lines.join('\n');
}

function analyzeHeadingStructure(html: string): HeaderAnalysis {
  const { metadata } = convertWithMetadata(html);
  const headers = metadata.headers || [];

  const analysis: HeaderAnalysis = {
    totalHeaders: headers.length,
    byLevel: {},
    hierarchyIssues: [],
    headersWithoutIds: [],
  };

  // Count headers by level
  for (const header of headers) {
    const level = header.level;
    analysis.byLevel[level] = (analysis.byLevel[level] || 0) + 1;

    // Track headers without explicit IDs
    if (!header.id) {
      analysis.headersWithoutIds.push(header.text);
    }
  }

  // Check for hierarchy issues
  let prevLevel: number | null = null;
  for (const header of headers) {
    const level = header.level;

    if (prevLevel === null) {
      prevLevel = level;
      continue;
    }

    // Warn about skipped levels
    if (level > prevLevel + 1) {
      analysis.hierarchyIssues.push(
        `Jumped from H${prevLevel} to H${level}: '${header.text}'`
      );
    }

    prevLevel = level;
  }

  return analysis;
}

function main() {
  const html = `
    <html>
      <head>
        <title>Advanced Rust Programming Guide</title>
      </head>
      <body>
        <h1 id="intro">Introduction to Advanced Rust</h1>
        <p>This guide covers advanced Rust concepts and patterns.</p>

        <h2 id="memory">Memory Management</h2>
        <p>Understanding ownership and borrowing is fundamental to Rust.</p>

        <h3 id="ownership">Ownership Rules</h3>
        <p>Each value has a single owner.</p>

        <h3 id="borrowing">Borrowing and References</h3>
        <p>References allow temporary access without transfer.</p>

        <h2 id="concurrency">Concurrency Patterns</h2>
        <p>Rust provides powerful concurrency primitives.</p>

        <h3 id="threads">Thread Safety</h3>
        <p>The type system ensures thread safety at compile time.</p>

        <h3 id="async">Async/Await Programming</h3>
        <p>Non-blocking concurrent code with async/await.</p>

        <h4 id="async-futures">Futures and Tasks</h4>
        <p>Understanding the foundations of async Rust.</p>

        <h2 id="performance">Performance Optimization</h2>
        <p>Practical techniques for optimizing Rust code.</p>

        <h3 id="profiling">Profiling Tools</h3>
        <p>Use cargo-flamegraph and perf for analysis.</p>

        <h3 id="benchmarking">Benchmarking with Criterion</h3>
        <p>Accurate performance measurements using criterion.rs.</p>

        <h1 id="conclusion">Conclusion</h1>
        <p>Mastering these patterns unlocks Rust's full potential.</p>
      </body>
    </html>
  `;

  console.log('='.repeat(80));
  console.log('TABLE OF CONTENTS GENERATION EXAMPLE');
  console.log('='.repeat(80));
  console.log();

  // Generate TOC
  const toc = buildTableOfContents(html);
  console.log('GENERATED TABLE OF CONTENTS');
  console.log('-'.repeat(80));
  console.log(toc);
  console.log();

  // Analyze structure
  const analysis = analyzeHeadingStructure(html);
  console.log('HEADING STRUCTURE ANALYSIS');
  console.log('-'.repeat(80));
  console.log(`Total headers:        ${analysis.totalHeaders}`);
  console.log('Headers by level:');
  for (const level of Object.keys(analysis.byLevel).map(Number).sort()) {
    const count = analysis.byLevel[level];
    console.log(`  H${level}: ${count}`);
  }
  console.log();

  if (analysis.hierarchyIssues.length > 0) {
    console.log('HIERARCHY ISSUES (⚠️ skipped heading levels)');
    console.log('-'.repeat(80));
    for (const issue of analysis.hierarchyIssues) {
      console.log(`  - ${issue}`);
    }
    console.log();
  }

  if (analysis.headersWithoutIds.length > 0) {
    console.log('HEADERS WITHOUT EXPLICIT IDs');
    console.log('-'.repeat(80));
    console.log('The following headers do not have explicit IDs.');
    console.log('Auto-generated anchors will be created from the text:\n');
    for (const text of analysis.headersWithoutIds) {
      const autoId = text.toLowerCase().replace(/\s+/g, '-');
      console.log(`  - '${text}' → #${autoId}`);
    }
    console.log();
  }
}

main();

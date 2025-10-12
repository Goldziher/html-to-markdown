/**
 * Example: Using Node.js native bindings directly
 * This uses the fastest backend (~691k ops/sec)
 */

const { convert } = require("@html-to-markdown/node");

const html = `
<!DOCTYPE html>
<html>
<head>
  <title>Example Document</title>
</head>
<body>
  <h1>Welcome to HTML to Markdown</h1>
  <p>This is a <strong>high-performance</strong> converter built with <em>Rust</em>.</p>

  <h2>Features</h2>
  <ul>
    <li>Fast native bindings</li>
    <li>Full TypeScript support</li>
    <li>Multiple output formats</li>
  </ul>

  <h2>Code Example</h2>
  <pre><code class="language-rust">
fn main() {
    println!("Hello, world!");
}
  </code></pre>

  <blockquote>
    Performance matters when processing large documents.
  </blockquote>
</body>
</html>
`;

// Basic conversion
console.log("=== Basic Conversion ===\n");
const markdown = convert(html);
console.log(markdown);

// With custom options
console.log("\n\n=== With Custom Options ===\n");
const markdownWithOptions = convert(html, {
  headingStyle: "Atx",
  codeBlockStyle: "Backticks",
  listIndentWidth: 2,
  wrap: true,
  wrapWidth: 80,
});
console.log(markdownWithOptions);

// Benchmark
console.log("\n\n=== Performance Benchmark ===\n");
const iterations = 10000;
const start = Date.now();
for (let i = 0; i < iterations; i++) {
  convert(html);
}
const elapsed = Date.now() - start;
console.log(`Converted ${iterations} documents in ${elapsed}ms`);
console.log(`Average: ${(elapsed / iterations).toFixed(3)}ms per document`);
console.log(`Throughput: ${Math.round((iterations / elapsed) * 1000)} docs/sec`);

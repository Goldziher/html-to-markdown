/**
 * Example: Using the TypeScript package with auto-detection
 * This automatically uses native bindings when available, falls back to WASM
 */

import {
  convert,
  getBackend,
  type ConversionOptions,
  HeadingStyle,
  CodeBlockStyle,
} from "html-to-markdown";

const html = `
<!DOCTYPE html>
<html>
<head>
  <title>TypeScript Example</title>
</head>
<body>
  <h1>TypeScript Integration</h1>
  <p>Full type safety with <code>html-to-markdown</code>.</p>

  <h2>Type-Safe Options</h2>
  <ul>
    <li>Enum-based configuration</li>
    <li>IntelliSense support</li>
    <li>Compile-time checks</li>
  </ul>
</body>
</html>
`;

async function main() {
  console.log("=== TypeScript Example ===\n");

  // Basic conversion
  const markdown = await convert(html);
  console.log(markdown);

  // Check which backend is being used
  console.log("\n=== Backend Detection ===");
  console.log(`Using backend: ${getBackend()}`);

  // Type-safe options
  const options: ConversionOptions = {
    headingStyle: HeadingStyle.Atx,
    codeBlockStyle: CodeBlockStyle.Backticks,
    listIndentWidth: 2,
    wrap: true,
    wrapWidth: 100,
    preprocessing: {
      enabled: true,
      removeNavigation: true,
      removeForms: true,
    },
  };

  console.log("\n=== With Typed Options ===\n");
  const markdownWithOptions = await convert(html, options);
  console.log(markdownWithOptions);

  // Error handling
  console.log("\n=== Error Handling ===");
  try {
    const invalidHtml = "<h1>Unclosed tag";
    const result = await convert(invalidHtml);
    console.log("Conversion succeeded:", result);
  } catch (error) {
    console.error("Conversion failed:", error);
  }
}

main().catch(console.error);

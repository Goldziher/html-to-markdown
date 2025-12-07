/**
 * Simple test for Node.js bindings
 */

const { convert } = require("./index.js");

function assert(condition, message) {
	if (!condition) {
		throw new Error(message || "Assertion failed");
	}
}

console.log("Testing html-to-markdown-node...\n");

// Test 1: Basic conversion
console.log("Test 1: Basic conversion");
const html1 = "<h1>Hello World</h1>";
const md1 = convert(html1);
assert(md1.includes("Hello World"), "Should convert heading");
console.log("✓ Basic conversion works\n");

// Test 2: With options
console.log("Test 2: With options");
const html2 = "<h1>Test</h1>";
const md2 = convert(html2, { headingStyle: "Atx" });
assert(md2.includes("#"), "Should use ATX style");
console.log("✓ Options work\n");

// Test 3: Complex HTML
console.log("Test 3: Complex HTML");
const html3 = `
  <h1>Title</h1>
  <p>Paragraph with <strong>bold</strong> and <em>italic</em>.</p>
  <ul>
    <li>Item 1</li>
    <li>Item 2</li>
  </ul>
`;
const md3 = convert(html3);
assert(md3.includes("Title"), "Should have title");
assert(md3.includes("**bold**") || md3.includes("__bold__"), "Should have bold");
assert(md3.includes("*italic*") || md3.includes("_italic_"), "Should have italic");
console.log("✓ Complex HTML works\n");

// Test 4: Performance
console.log("Test 4: Performance benchmark");
const iterations = 1000;
const start = Date.now();
for (let i = 0; i < iterations; i++) {
	convert(html3);
}
const elapsed = Date.now() - start;
const opsPerSec = Math.round((iterations / elapsed) * 1000);
console.log(`${iterations} conversions in ${elapsed}ms`);
console.log(`${opsPerSec} ops/sec`);
console.log("✓ Performance test complete\n");

console.log("All tests passed! ✅");

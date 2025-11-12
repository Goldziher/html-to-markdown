import { convert, convertBytes } from 'html-to-markdown-wasm/dist-node/html_to_markdown_wasm.js';

const html = `
  <h1>WASM Smoke Test</h1>
  <p>Ensures the WebAssembly bundle works under Node.js/Deno runtimes.</p>
`;

const fromString = convert(html, { headingStyle: 'atx' });
const fromBytes = convertBytes(new TextEncoder().encode(html));

if (!fromString.includes('# WASM Smoke Test') || !fromBytes.includes('# WASM Smoke Test')) {
  throw new Error('html-to-markdown-wasm did not return the expected heading');
}

console.log('\u001b[36m✓ html-to-markdown-wasm produced markdown from string input\u001b[0m');
console.log('\u001b[36m✓ html-to-markdown-wasm produced markdown from byte input\u001b[0m');
console.log('---');
console.log(fromString.trim());

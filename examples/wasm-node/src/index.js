import { convert } from "html-to-markdown-wasm/dist-node";

const html = `
  <article>
    <h1>Node.js + html-to-markdown-wasm</h1>
    <p>This example runs in Node.js using the dist-node target.</p>
    <table>
      <tr><th>Runtime</th><th>Speed</th></tr>
      <tr><td>Node.js</td><td>🚀</td></tr>
      <tr><td>WASM</td><td>⚡</td></tr>
    </table>
  </article>
`;

const options = {
  wrap: true,
  wrapWidth: 60,
  escapeMisc: true,
};

try {
  const markdown = convert(html, options);
  console.log("Converted Markdown:\n");
  console.log(markdown);
} catch (error) {
  console.error("Conversion failed:", error);
  process.exit(1);
}

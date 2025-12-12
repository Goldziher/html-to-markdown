import init, { convert, type WasmConversionOptions } from "html-to-markdown-wasm/dist-web";

async function ensureWasmReady() {
  await init();
}

async function main() {
  await ensureWasmReady();

  const html = `
    <article>
      <h1>Rollup + html-to-markdown-wasm</h1>
      <p>This example runs entirely in the browser.</p>
      <table>
        <tr><th>Language</th><th>Speed</th></tr>
        <tr><td>Rust</td><td>🚀</td></tr>
        <tr><td>WASM</td><td>⚡</td></tr>
      </table>
    </article>
  `;

  const options: WasmConversionOptions = {
    wrap: true,
    wrapWidth: 60,
    escapeMisc: true,
  };

  const markdown = convert(html, options);

  const pre = document.createElement("pre");
  pre.textContent = markdown;
  document.body.append("Rendered Markdown:\n\n", pre);

  // eslint-disable-next-line no-console -- dev example
  console.log(markdown);
}

void main().catch((error) => {
  // eslint-disable-next-line no-console -- dev example
  console.error("WASM example failed", error);
});

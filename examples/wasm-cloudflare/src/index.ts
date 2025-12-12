import { convert, initWasm, wasmReady, type WasmConversionOptions } from "html-to-markdown-wasm";

export default {
  async fetch(request: Request): Promise<Response> {
    if (request.method === "POST") {
      try {
        await (wasmReady ?? initWasm());

        const html = await request.text();

        const options: WasmConversionOptions = {
          wrap: true,
          wrapWidth: 80,
          escapeMisc: true,
        };

        const markdown = convert(html, options);

        return new Response(markdown, {
          headers: { "Content-Type": "text/markdown; charset=utf-8" },
        });
      } catch (error) {
        return new Response(
          JSON.stringify({
            error: "Conversion failed",
            message: error instanceof Error ? error.message : String(error),
          }),
          {
            status: 500,
            headers: { "Content-Type": "application/json" },
          }
        );
      }
    }

    return new Response(
      `
<!DOCTYPE html>
<html>
  <head><title>html-to-markdown WASM on Cloudflare Workers</title></head>
  <body>
    <h1>html-to-markdown-wasm + Cloudflare Workers</h1>
    <p>Send a POST request with HTML to convert it to Markdown.</p>
    <pre>curl -X POST http://localhost:8787/ -H "Content-Type: text/html" -d "&lt;h1&gt;Hello&lt;/h1&gt;"</pre>
  </body>
</html>
      `.trim(),
      {
        headers: { "Content-Type": "text/html" },
      }
    );
  },
};

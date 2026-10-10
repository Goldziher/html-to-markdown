import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, writeFile, rm, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import plugin from "../../plugin/.opencode/plugins/html-to-markdown.js";

async function setup(directory) {
  const records = { tool: [], skill: [], command: [], mcp: [] };
  const ctx = { location: { directory }, session: {} };
  for (const kind of Object.keys(records)) {
    ctx[kind] = {
      transform: (callback) =>
        callback({
          get: () => undefined,
          add: (value) => records[kind].push(value),
          set: (name, value) => records[kind].push({ name, ...value }),
        }),
    };
  }
  await plugin.setup(ctx);
  return records;
}

test("should_register_tools_skills_commands_and_mcp_from_installed_bundle", async () => {
  assert.equal(plugin.id, "html-to-markdown");
  const records = await setup(process.cwd());
  assert.deepEqual(
    records.tool.map((tool) => tool.name),
    ["html_to_markdown_convert", "html_to_markdown_fetch_url", "html_to_markdown_extract"],
  );
  assert.equal(records.skill.length, 6);
  assert.deepEqual(
    records.command.map((command) => command.name),
    ["html-to-markdown-convert", "html-to-markdown-metadata"],
  );
  assert.equal(records.mcp.length, 1);
});

test("should_forward_html_options_stdin_and_abort_to_cli", async () => {
  const directory = await realpath(await mkdtemp(path.join(tmpdir(), "html-to-markdown-plugin-test-")));
  const previousPath = process.env.PATH;
  try {
    await writeFile(
      path.join(directory, "html-to-markdown"),
      `#!/usr/bin/env node
if (process.argv.includes('wait')) setInterval(() => {}, 1000);
else { let html = ''; process.stdin.on('data', value => { html += value; }); process.stdin.on('end', () => console.log(JSON.stringify({ args: process.argv.slice(2), html, cwd: process.cwd() }))); }
`,
      { mode: 0o755 },
    );
    process.env.PATH = `${directory}${path.delimiter}${previousPath}`;
    const records = await setup(directory);
    const result = await records.tool[0].execute(
      { html: "<h1>Hello</h1>", output_format: "djot", preprocess: true },
      {},
    );
    assert.deepEqual(JSON.parse(result.content), {
      args: ["--output-format", "djot", "--preprocess"],
      html: "<h1>Hello</h1>",
      cwd: directory,
    });
    assert.equal(result.metadata.exitCode, 0);
    const extracted = await records.tool[2].execute(
      { html: "<h1>Hello</h1>", include_structure: true, no_content: true },
      {},
    );
    assert.deepEqual(JSON.parse(extracted.content), {
      args: ["--json", "--include-structure", "--no-content"],
      html: "<h1>Hello</h1>",
      cwd: directory,
    });

    await assert.rejects(records.tool[0].execute({}, {}), /Provide either/);
    const controller = new AbortController();
    const pending = records.tool[0].execute({ path: "wait" }, { signal: controller.signal });
    controller.abort();
    await assert.rejects(pending, { name: "AbortError" });
  } finally {
    process.env.PATH = previousPath;
    await rm(directory, { recursive: true, force: true });
  }
});

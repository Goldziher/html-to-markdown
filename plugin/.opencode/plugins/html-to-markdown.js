// AI-RULEZ :: GENERATED FILE — DO NOT EDIT
// Content-Hash: blake3:2e612761528e736297ad6fea35c069568eb341e0d28abd11dec8115b1d83d010
// Source-Hash: blake3:a4e8ced2e22ff967783cc9be245a7380621eb0efd2b6a67c9d5d78bc48238b43
// Schema-Version: v1

import { registerBundledContent } from "../ai-rulez-content.js";
import { spawn } from "node:child_process";

const headingStyle = {
  type: "string",
  enum: ["atx", "underlined", "atx-closed"],
  description: "Markdown heading style. Default: atx.",
};
const codeBlockStyle = {
  type: "string",
  enum: ["backticks", "indented", "tildes"],
  description: "Markdown code block style. Default: backticks.",
};
const outputFormat = {
  type: "string",
  enum: ["markdown", "djot"],
  description: "Output markup format. Default: markdown.",
};
const preset = {
  type: "string",
  enum: ["minimal", "standard", "aggressive"],
  description: "Preprocessing aggressiveness. Requires `preprocess`. Default: standard.",
};

function hasValue(value) {
  return value !== undefined && value !== null && value !== "";
}

function pushOption(args, name, value) {
  if (hasValue(value)) {
    args.push(name, String(value));
  }
}

function pushFlag(args, name, value) {
  if (value === true) {
    args.push(name);
  }
}

function runCli(args, context, directory, stdin) {
  return new Promise((resolve, reject) => {
    const child = spawn("html-to-markdown", args, {
      cwd: directory,
      env: process.env,
      signal: context?.signal,
      stdio: [stdin === undefined ? "ignore" : "pipe", "pipe", "pipe"],
    });

    const stdout = [];
    const stderr = [];

    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", (error) => {
      if (error.code === "ENOENT") {
        resolve({
          content:
            "Install the html-to-markdown CLI with `brew install xberg-io/tap/html-to-markdown`, or run it via `npx -y @xberg-io/html-to-markdown-cli` / `uvx --from html-to-markdown-cli html-to-markdown`.",
          metadata: { exitCode: 127, command: "html-to-markdown" },
        });
        return;
      }
      reject(error);
    });
    child.on("close", (exitCode, signal) => {
      const stdoutText = Buffer.concat(stdout).toString("utf8").trim();
      const stderrText = Buffer.concat(stderr).toString("utf8").trim();
      const output = [stdoutText, stderrText && `stderr:\n${stderrText}`].filter(Boolean).join("\n\n");

      resolve({
        content: output || "(no output)",
        metadata: { exitCode, signal, command: "html-to-markdown" },
      });
    });

    if (stdin !== undefined) {
      child.stdin.write(stdin);
      child.stdin.end();
    }
  });
}

function styleArgs(args, params) {
  pushOption(args, "--heading-style", params.heading_style);
  pushOption(args, "--code-block-style", params.code_block_style);
  pushOption(args, "--output-format", params.output_format);
  pushFlag(args, "--preprocess", params.preprocess);
  pushOption(args, "--preset", params.preset);
}

export default {
  id: "html-to-markdown",
  async setup(ctx) {
    await registerBundledContent(ctx);
    await ctx.tool.transform((editor) => {
      editor.add({
        name: "html_to_markdown_convert",
        description:
          "Convert an HTML file or HTML string to Markdown (or Djot) with the html-to-markdown CLI. Provide either `path` or `html`.",
        input: {
          type: "object",
          properties: {
            path: { type: "string", minLength: 1, description: "Path to a local HTML file." },
            html: { type: "string", minLength: 1, description: "Inline HTML to convert when path is omitted." },
            heading_style: headingStyle,
            code_block_style: codeBlockStyle,
            output_format: outputFormat,
            preprocess: { type: "boolean", description: "Strip navigation, ads, and forms before converting." },
            preset,
          },
          additionalProperties: false,
        },
        async execute(args, context) {
          const cliArgs = [];
          styleArgs(cliArgs, args);

          if (hasValue(args.path)) {
            cliArgs.push(args.path);
            return runCli(cliArgs, context, ctx.location.directory);
          }
          if (hasValue(args.html)) {
            return runCli(cliArgs, context, ctx.location.directory, args.html);
          }
          throw new Error("Provide either `path` or `html`.");
        },
      });
      editor.add({
        name: "html_to_markdown_fetch_url",
        description: "Fetch a URL and convert its HTML to Markdown (or Djot) with the html-to-markdown CLI.",
        input: {
          type: "object",
          properties: {
            url: { type: "string", minLength: 1, description: "URL to fetch and convert." },
            heading_style: headingStyle,
            code_block_style: codeBlockStyle,
            output_format: outputFormat,
            preprocess: { type: "boolean", description: "Strip navigation, ads, and forms before converting." },
            preset,
            user_agent: { type: "string", minLength: 1, description: "Custom User-Agent header for the fetch." },
          },
          required: ["url"],
          additionalProperties: false,
        },
        async execute(args, context) {
          const cliArgs = ["--url", args.url];
          pushOption(cliArgs, "--user-agent", args.user_agent);
          styleArgs(cliArgs, args);
          return runCli(cliArgs, context, ctx.location.directory);
        },
      });
      editor.add({
        name: "html_to_markdown_extract",
        description:
          "Extract structured metadata, tables, and (optionally) document structure from HTML as JSON. Returns the full ConversionResult. Provide `path`, `html`, or `url`.",
        input: {
          type: "object",
          properties: {
            path: { type: "string", minLength: 1, description: "Path to a local HTML file." },
            html: {
              type: "string",
              minLength: 1,
              description: "Inline HTML to analyze when path and url are omitted.",
            },
            url: { type: "string", minLength: 1, description: "URL to fetch and analyze." },
            include_structure: {
              type: "boolean",
              description: "Include the document structure tree in the JSON output.",
            },
            no_content: {
              type: "boolean",
              description: "Suppress Markdown content and return metadata, tables, and images only.",
            },
          },
          additionalProperties: false,
        },
        async execute(args, context) {
          const cliArgs = ["--json"];
          pushFlag(cliArgs, "--include-structure", args.include_structure);
          pushFlag(cliArgs, "--no-content", args.no_content);

          if (hasValue(args.url)) {
            cliArgs.push("--url", args.url);
            return runCli(cliArgs, context, ctx.location.directory);
          }
          if (hasValue(args.path)) {
            cliArgs.push(args.path);
            return runCli(cliArgs, context, ctx.location.directory);
          }
          if (hasValue(args.html)) {
            return runCli(cliArgs, context, ctx.location.directory, args.html);
          }
          throw new Error("Provide one of `path`, `html`, or `url`.");
        },
      });
    });
  },
};

#!/usr/bin/env node
import { promises as fs } from "node:fs";
import { stderr, stdin, stdout } from "node:process";

import {
  convert as convertHtml,
  convertWithInlineImages,
  type JsConversionOptions,
  type JsInlineImageConfig,
} from "html-to-markdown-node";

import { convertStream, convertStreamWithInlineImages } from "./index";

interface CliOptions {
  input?: string;
  output?: string;
  options?: JsConversionOptions;
  inlineImages?: boolean;
  inlineImageConfig?: JsInlineImageConfig;
}

function printUsage(): void {
  stdout.write(`html-to-markdown CLI\n\n`);
  stdout.write(`Usage:\n`);
  stdout.write(`  html-to-markdown [--input <file>] [--output <file>] [--options '{...}'] [--inline-images]\n\n`);
  stdout.write(`Options:\n`);
  stdout.write(`  --input <file>           Read HTML from a file instead of stdin\n`);
  stdout.write(`  --output <file>          Write Markdown to a file instead of stdout\n`);
  stdout.write(`  --options <json>         JSON encoded conversion options\n`);
  stdout.write(`  --inline-images          Collect inline images (writes JSON to <output>.images.json)\n`);
  stdout.write(`  --inline-image-config    JSON encoded inline image extraction options\n`);
  stdout.write(`  -h, --help               Show this help message\n`);
  stdout.write(`  -v, --version            Print the package version\n`);
}

async function loadJson<T>(value: string, label: string): Promise<T> {
  try {
    return JSON.parse(value) as T;
  } catch (error) {
    throw new Error(`Failed to parse ${label} JSON: ${(error as Error).message}`);
  }
}

async function parseArgs(): Promise<CliOptions | "help" | "version"> {
  const args = process.argv.slice(2);
  const opts: CliOptions = {};

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    switch (arg) {
      case "-h":
      case "--help":
        return "help";
      case "-v":
      case "--version":
        return "version";
      case "--input":
        opts.input = args[++index];
        break;
      case "--output":
        opts.output = args[++index];
        break;
      case "--options":
        opts.options = await loadJson<JsConversionOptions>(args[++index], "--options");
        break;
      case "--inline-images":
        opts.inlineImages = true;
        break;
      case "--inline-image-config":
        opts.inlineImageConfig = await loadJson<JsInlineImageConfig>(args[++index], "--inline-image-config");
        break;
      default:
        throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return opts;
}

async function readInput(path?: string): Promise<string | AsyncIterable<string | Buffer>> {
  if (path) {
    return fs.readFile(path, "utf8");
  }
  if (!stdin.isTTY) {
    return stdin;
  }
  throw new Error("No input provided. Pass --input <file> or pipe HTML via stdin.");
}

async function writeOutput(content: string, path?: string): Promise<void> {
  if (!path) {
    stdout.write(content);
    if (!content.endsWith("\n")) {
      stdout.write("\n");
    }
    return;
  }

  await fs.writeFile(path, content, "utf8");
}

async function writeInlineImages(
  extractionPath: string,
  inlineData: Awaited<ReturnType<typeof convertWithInlineImages>>,
): Promise<void> {
  const payload = {
    markdown: inlineData.markdown,
    inlineImages: inlineData.inlineImages,
    warnings: inlineData.warnings,
  };

  await fs.writeFile(extractionPath, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
}

async function main(): Promise<void> {
  try {
    const parsed = await parseArgs();

    if (parsed === "help") {
      printUsage();
      return;
    }

    if (parsed === "version") {
      const { version } = require("../package.json") as { version: string };
      stdout.write(`${version}\n`);
      return;
    }

    const { input, output, options, inlineImages, inlineImageConfig } = parsed;
    const inputContent = await readInput(input);

    if (inlineImages) {
      const inlineResult =
        typeof inputContent === "string"
          ? convertWithInlineImages(inputContent, options, inlineImageConfig)
          : await convertStreamWithInlineImages(inputContent, options, inlineImageConfig);

      if (output) {
        await writeOutput(inlineResult.markdown, output);
        const imagePath = `${output}.images.json`;
        await writeInlineImages(imagePath, inlineResult);
        stdout.write(`Inline images written to ${imagePath}\n`);
      } else {
        stdout.write(inlineResult.markdown);
        if (!inlineResult.markdown.endsWith("\n")) {
          stdout.write("\n");
        }
        stdout.write(
          `${JSON.stringify({ inlineImages: inlineResult.inlineImages, warnings: inlineResult.warnings }, null, 2)}\n`,
        );
      }
    } else if (typeof inputContent === "string") {
      const markdown = convertHtml(inputContent, options);
      await writeOutput(markdown, output);
    } else {
      const markdown = await convertStream(inputContent, options);
      await writeOutput(markdown, output);
    }
  } catch (error) {
    stderr.write(`Error: ${(error as Error).message}\n`);
    process.exitCode = 1;
  }
}

void main();

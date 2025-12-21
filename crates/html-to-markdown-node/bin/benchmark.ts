#!/usr/bin/env tsx
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  convertBuffer,
  convertInlineImagesBuffer,
  convertWithMetadataBuffer,
  startProfiling,
  stopProfiling,
} = require("../index.js") as {
  convertBuffer: (html: Buffer, options?: Record<string, unknown>) => string;
  convertInlineImagesBuffer: (
    html: Buffer,
    options?: Record<string, unknown>,
    imageConfig?: Record<string, unknown>,
  ) => { markdown: string };
  convertWithMetadataBuffer: (
    html: Buffer,
    options?: Record<string, unknown>,
    metadataConfig?: Record<string, unknown>,
  ) => { markdown: string };
  startProfiling?: (outputPath: string, frequency?: number) => void;
  stopProfiling?: () => void;
};

type Format = "html" | "hocr";
type Scenario =
  | "convert-default"
  | "convert-options"
  | "inline-images-default"
  | "inline-images-options"
  | "metadata-default"
  | "metadata-options";

interface Args {
  file: string;
  iterations: number;
  format: Format;
  scenario: Scenario;
}

function parseArgs(): Args {
  const args = process.argv.slice(2);
  const parsed: Partial<Args> = {
    iterations: 50,
    format: "html",
    scenario: "convert-default",
  };

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === "--file") {
      parsed.file = args[++i];
    } else if (arg === "--iterations") {
      parsed.iterations = Math.max(1, Number.parseInt(args[++i] ?? "1", 10));
    } else if (arg === "--scenario") {
      parsed.scenario = (args[++i] ?? "convert-default") as Scenario;
    } else if (arg === "--format") {
      parsed.format = (args[++i] ?? "html").toLowerCase() as Format;
    }
  }

  if (!parsed.file) {
    throw new Error("Missing --file parameter");
  }
  if (!["html", "hocr"].includes(parsed.format ?? "")) {
    throw new Error(`Unsupported format: ${parsed.format}`);
  }
  if (
    ![
      "convert-default",
      "convert-options",
      "inline-images-default",
      "inline-images-options",
      "metadata-default",
      "metadata-options",
    ].includes(parsed.scenario ?? "")
  ) {
    throw new Error(`Unsupported scenario: ${parsed.scenario}`);
  }

  return parsed as Args;
}

function buildOptions(format: Format) {
  if (format === "hocr") {
    return { hocrSpatialTables: false };
  }
  return {};
}

function buildMetadataConfig() {
  return {
    extract_document: true,
    extract_headers: true,
    extract_links: true,
    extract_images: true,
    extract_structured_data: true,
  };
}

function runScenario(html: Buffer, scenario: Scenario, options: Record<string, unknown>) {
  switch (scenario) {
    case "convert-default":
      convertBuffer(html);
      break;
    case "convert-options":
      convertBuffer(html, options);
      break;
    case "inline-images-default":
      convertInlineImagesBuffer(html);
      break;
    case "inline-images-options":
      convertInlineImagesBuffer(html, options);
      break;
    case "metadata-default":
      convertWithMetadataBuffer(html, undefined, buildMetadataConfig());
      break;
    case "metadata-options":
      convertWithMetadataBuffer(html, options, buildMetadataConfig());
      break;
  }
}

function main() {
  const args = parseArgs();
  const fixturePath = path.resolve(process.cwd(), args.file);

  if (!fs.existsSync(fixturePath)) {
    throw new Error(`Fixture not found: ${fixturePath}`);
  }

  const html = fs.readFileSync(fixturePath);
  const options = buildOptions(args.format);

  runScenario(html, args.scenario, options);

  const profileOutput = process.env.HTML_TO_MARKDOWN_PROFILE_OUTPUT;
  if (profileOutput && startProfiling) {
    const freqEnv = process.env.HTML_TO_MARKDOWN_PROFILE_FREQUENCY;
    const frequency = freqEnv ? Number.parseInt(freqEnv, 10) : 1000;
    startProfiling(profileOutput, Number.isFinite(frequency) ? frequency : 1000);
  }

  const start = process.hrtime.bigint();
  for (let i = 0; i < args.iterations; i += 1) {
    runScenario(html, args.scenario, options);
  }
  const elapsedSeconds = Number(process.hrtime.bigint() - start) / 1e9;

  if (profileOutput && stopProfiling) {
    stopProfiling();
  }

  const bytesProcessed = html.byteLength * args.iterations;
  const opsPerSec = args.iterations / elapsedSeconds;
  const mbPerSec = (bytesProcessed / (1024 * 1024)) / elapsedSeconds;

  const result = {
    language: "node",
    fixture: path.basename(fixturePath),
    fixture_path: fixturePath,
    scenario: args.scenario,
    iterations: args.iterations,
    elapsed_seconds: elapsedSeconds,
    ops_per_sec: opsPerSec,
    mb_per_sec: mbPerSec,
    bytes_processed: bytesProcessed,
  };

  console.log(JSON.stringify(result));
}

main();

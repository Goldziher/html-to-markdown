#!/usr/bin/env tsx
import fs from "node:fs";
import path from "node:path";
import {
  WasmMetadataConfig,
  convertBytes,
  convertBytesWithInlineImages,
  convertBytesWithMetadata,
} from "html-to-markdown-wasm/dist-node";

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
  warmup: number;
  format: Format;
  scenario: Scenario;
}

function parseArgs(): Args {
  const args = process.argv.slice(2);
  const parsed: Partial<Args> = {
    iterations: 50,
    warmup: 1,
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

  const warmupEnv = process.env.HTML_TO_MARKDOWN_BENCH_WARMUP;
  if (warmupEnv) {
    const parsedWarmup = Number.parseInt(warmupEnv, 10);
    if (Number.isFinite(parsedWarmup)) {
      parsed.warmup = Math.max(0, parsedWarmup);
    }
  }

  if (!parsed.file) {
    throw new Error("Missing --file parameter");
  }
  if (!parsed.format || !["html", "hocr"].includes(parsed.format)) {
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

function runScenario(
  htmlBytes: Uint8Array,
  scenario: Scenario,
  options: Record<string, unknown>,
  metadataConfig: WasmMetadataConfig,
) {
  switch (scenario) {
    case "convert-default":
      convertBytes(htmlBytes, undefined);
      break;
    case "convert-options":
      convertBytes(htmlBytes, options);
      break;
    case "inline-images-default":
      convertBytesWithInlineImages(htmlBytes, undefined, undefined);
      break;
    case "inline-images-options":
      convertBytesWithInlineImages(htmlBytes, options, undefined);
      break;
    case "metadata-default":
      convertBytesWithMetadata(htmlBytes, undefined, metadataConfig);
      break;
    case "metadata-options":
      convertBytesWithMetadata(htmlBytes, options, metadataConfig);
      break;
  }
}

function main() {
  const args = parseArgs();
  const fixturePath = path.resolve(process.cwd(), args.file);

  if (!fs.existsSync(fixturePath)) {
    throw new Error(`Fixture not found: ${fixturePath}`);
  }

  const html = fs.readFileSync(fixturePath, "utf8");
  const htmlBytes = new TextEncoder().encode(html);
  const options = buildOptions(args.format);
  const metadataConfig = new WasmMetadataConfig();

  for (let i = 0; i < (args.warmup ?? 1); i += 1) {
  runScenario(htmlBytes, args.scenario, options, metadataConfig);
  }

  const start = process.hrtime.bigint();
  for (let i = 0; i < args.iterations; i += 1) {
    runScenario(htmlBytes, args.scenario, options, metadataConfig);
  }
  const elapsedSeconds = Number(process.hrtime.bigint() - start) / 1e9;

  const bytesProcessed = Buffer.byteLength(html) * args.iterations;
  const opsPerSec = args.iterations / elapsedSeconds;
  const mbPerSec = (bytesProcessed / (1024 * 1024)) / elapsedSeconds;

  const result = {
    language: "wasm",
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

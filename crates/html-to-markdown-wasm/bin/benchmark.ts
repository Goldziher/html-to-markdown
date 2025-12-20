#!/usr/bin/env tsx
import fs from "node:fs";
import path from "node:path";
import { convert } from "html-to-markdown-wasm/dist-node";

type Format = "html" | "hocr";

interface Args {
  file: string;
  iterations: number;
  format: Format;
}

function parseArgs(): Args {
  const args = process.argv.slice(2);
  const parsed: Partial<Args> = {
    iterations: 50,
    format: "html",
  };

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === "--file") {
      parsed.file = args[++i];
    } else if (arg === "--iterations") {
      parsed.iterations = Math.max(1, Number.parseInt(args[++i] ?? "1", 10));
    } else if (arg === "--format") {
      parsed.format = (args[++i] ?? "html").toLowerCase() as Format;
    }
  }

  if (!parsed.file) {
    throw new Error("Missing --file parameter");
  }
  if (!parsed.format || !["html", "hocr"].includes(parsed.format)) {
    throw new Error(`Unsupported format: ${parsed.format}`);
  }

  return parsed as Args;
}

function buildOptions(format: Format) {
  if (format === "hocr") {
    return { hocrSpatialTables: false };
  }
  return undefined;
}

function main() {
  const args = parseArgs();
  const fixturePath = path.resolve(process.cwd(), args.file);

  if (!fs.existsSync(fixturePath)) {
    throw new Error(`Fixture not found: ${fixturePath}`);
  }

  const html = fs.readFileSync(fixturePath, "utf8");
  const options = buildOptions(args.format);

  convert(html, options);

  const start = process.hrtime.bigint();
  for (let i = 0; i < args.iterations; i += 1) {
    convert(html, options);
  }
  const elapsedSeconds = Number(process.hrtime.bigint() - start) / 1e9;

  const bytesProcessed = Buffer.byteLength(html) * args.iterations;
  const opsPerSec = args.iterations / elapsedSeconds;
  const mbPerSec = (bytesProcessed / (1024 * 1024)) / elapsedSeconds;

  const result = {
    language: "wasm",
    fixture: path.basename(fixturePath),
    fixture_path: fixturePath,
    iterations: args.iterations,
    elapsed_seconds: elapsedSeconds,
    ops_per_sec: opsPerSec,
    mb_per_sec: mbPerSec,
    bytes_processed: bytesProcessed,
  };

  console.log(JSON.stringify(result));
}

main();

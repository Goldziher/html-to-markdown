#!/usr/bin/env node

// Only run in Node.js/Bun environments
if (typeof process === "undefined" || !process.versions?.node) {
  console.error("This CLI is only available in Node.js or Bun environments");
  process.exit(1);
}

import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Find the Rust CLI binary
const binaryName = process.platform === "win32" ? "html-to-markdown.exe" : "html-to-markdown";
const binaryPath = join(__dirname, "..", "bin", binaryName);

// Proxy all arguments to the Rust CLI
const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  shell: false,
});

process.exit(result.status ?? 1);

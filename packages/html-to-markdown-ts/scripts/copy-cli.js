#!/usr/bin/env node

import { copyFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const rootDir = join(__dirname, "../../..");
const targetDir = join(rootDir, "target/release");
const binDir = join(__dirname, "../bin");

// Create bin directory
mkdirSync(binDir, { recursive: true });

// Determine binary name based on platform
const binaryName = process.platform === "win32" ? "html-to-markdown.exe" : "html-to-markdown";

// Copy the binary
const sourcePath = join(targetDir, binaryName);
const destPath = join(binDir, binaryName);

console.log(`Copying ${sourcePath} -> ${destPath}`);
copyFileSync(sourcePath, destPath);

// Make it executable on Unix-like systems
if (process.platform !== "win32") {
  import("node:fs").then(({ chmodSync }) => {
    chmodSync(destPath, 0o755);
    console.log("Made binary executable");
  });
}

console.log("CLI binary copied successfully");

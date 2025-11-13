#!/usr/bin/env node

/**
 * Post-process the bundler entry emitted by wasm-pack so we can support runtimes
 * that instantiate WebAssembly modules asynchronously (Cloudflare Workers, esbuild, etc).
 */

const fs = require('node:fs');
const path = require('node:path');

const rootDir = path.resolve(__dirname, '..');
const distDir = path.resolve(rootDir, process.argv[2] || 'dist');
const entryPath = path.join(distDir, 'html_to_markdown_wasm.js');
const dtsPath = path.join(distDir, 'html_to_markdown_wasm.d.ts');

if (!fs.existsSync(entryPath)) {
  console.error(`[patch-bundler-entry] Missing entry file at ${entryPath}`);
  process.exit(1);
}

const wrapper = `import * as wasmModule from "./html_to_markdown_wasm_bg.wasm";
export * from "./html_to_markdown_wasm_bg.js";
import { __wbg_set_wasm } from "./html_to_markdown_wasm_bg.js";

const notReadyError = () =>
  new Error("html-to-markdown-wasm: WebAssembly bundle is still initializing. Await initWasm() before calling convert() in runtimes that load WASM asynchronously (e.g., Cloudflare Workers).");

const notReadyProxy = new Proxy({}, {
  get(_target, prop) {
    if (prop === "__esModule") {
      return true;
    }
    throw notReadyError();
  }
});

let wasmExports;
let initialized = false;
let initPromise;

__wbg_set_wasm(notReadyProxy);

function asExports(value) {
  if (!value) {
    return null;
  }
  if (typeof value.__wbindgen_start === "function") {
    return value;
  }
  if (value instanceof WebAssembly.Instance) {
    return value.exports;
  }
  if (typeof value === "object") {
    if (value.instance instanceof WebAssembly.Instance) {
      return value.instance.exports;
    }
    if (value.default instanceof WebAssembly.Instance) {
      return value.default.exports;
    }
    if (value.default && value.default.instance instanceof WebAssembly.Instance) {
      return value.default.instance.exports;
    }
  }
  return null;
}

function finalize(exports) {
  wasmExports = exports;
  __wbg_set_wasm(exports);
  if (typeof exports.__wbindgen_start === "function") {
    exports.__wbindgen_start();
  }
  initialized = true;
  return exports;
}

function trySyncInit() {
  try {
    const exports = asExports(wasmModule);
    if (exports) {
      finalize(exports);
    }
  } catch {
    // ignore and fall back to async init
  }
}

trySyncInit();

function ensureInitPromise() {
  if (initialized) {
    return Promise.resolve(wasmExports);
  }
  if (!initPromise) {
    initPromise = (async () => {
      let module = wasmModule;
      if (module && typeof module.then === "function") {
        module = await module;
      }
      const exports = asExports(module);
      if (!exports) {
        throw new Error("html-to-markdown-wasm: failed to initialize WebAssembly bundle. Call initWasm() with a supported bundler configuration.");
      }
      return finalize(exports);
    })();
  }
  return initPromise;
}

export const wasmReady = ensureInitPromise();

export async function initWasm() {
  return ensureInitPromise();
}
`;

fs.writeFileSync(entryPath, wrapper, 'utf8');

if (fs.existsSync(dtsPath)) {
  let content = fs.readFileSync(dtsPath, 'utf8');
  const additions = `\nexport declare function initWasm(): Promise<void>;\nexport declare const wasmReady: Promise<void>;\n`;
  if (!content.includes('initWasm():')) {
    content += additions;
    fs.writeFileSync(dtsPath, content, 'utf8');
  }
}

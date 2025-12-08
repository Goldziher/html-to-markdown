#!/usr/bin/env node

/**
 * Post-process the bundler entry emitted by wasm-pack so we can support runtimes
 * that instantiate WebAssembly modules asynchronously (Cloudflare Workers, esbuild, etc).
 */

const fs = require('node:fs');
const path = require('node:path');

const rootDir = path.resolve(__dirname, '..');
const args = process.argv.slice(2);
let distArg = args.find((arg) => !arg.startsWith('--'));
distArg = distArg || 'dist';
const flags = new Set(args.filter((arg) => arg.startsWith('--')));
const typesOnly = flags.has('--types-only');

const distDir = path.resolve(rootDir, distArg);
const entryPath = path.join(distDir, 'html_to_markdown_wasm.js');
const dtsPath = path.join(distDir, 'html_to_markdown_wasm.d.ts');
const bgPath = path.join(distDir, 'html_to_markdown_wasm_bg.js');

const typeDefinitions = `
export type WasmHeadingStyle = "underlined" | "atx" | "atxClosed";
export type WasmListIndentType = "spaces" | "tabs";
export type WasmWhitespaceMode = "normalized" | "strict";
export type WasmNewlineStyle = "spaces" | "backslash";
export type WasmCodeBlockStyle = "indented" | "backticks" | "tildes";
export type WasmHighlightStyle = "doubleEqual" | "html" | "bold" | "none";
export type WasmPreprocessingPreset = "minimal" | "standard" | "aggressive";

export interface WasmPreprocessingOptions {
  enabled?: boolean;
  preset?: WasmPreprocessingPreset;
  removeNavigation?: boolean;
  removeForms?: boolean;
}

export interface WasmConversionOptions {
  headingStyle?: WasmHeadingStyle;
  listIndentType?: WasmListIndentType;
  listIndentWidth?: number;
  bullets?: string;
  strongEmSymbol?: string;
  escapeAsterisks?: boolean;
  escapeUnderscores?: boolean;
  escapeMisc?: boolean;
  escapeAscii?: boolean;
  codeLanguage?: string;
  autolinks?: boolean;
  defaultTitle?: boolean;
  brInTables?: boolean;
  hocrSpatialTables?: boolean;
  highlightStyle?: WasmHighlightStyle;
  extractMetadata?: boolean;
  whitespaceMode?: WasmWhitespaceMode;
  stripNewlines?: boolean;
  wrap?: boolean;
  wrapWidth?: number;
  convertAsInline?: boolean;
  subSymbol?: string;
  supSymbol?: string;
  newlineStyle?: WasmNewlineStyle;
  codeBlockStyle?: WasmCodeBlockStyle;
  keepInlineImagesIn?: string[];
  preprocessing?: WasmPreprocessingOptions | null;
  encoding?: string;
  debug?: boolean;
  stripTags?: string[];
  preserveTags?: string[];
}
`;

function injectTypedef(content, specifier) {
  const typedefBlock = `\n/**\n * @typedef {import("${specifier}").WasmConversionOptions} WasmConversionOptions\n */\n`;
  if (content.includes('WasmConversionOptions} WasmConversionOptions')) {
    return content;
  }
  if (content.includes('let wasm;')) {
    return content.replace('let wasm;', `let wasm;${typedefBlock}`);
  }
  return `${typedefBlock}${content}`;
}

function patchJsDoc(targetPath, typeSpecifier) {
  if (!fs.existsSync(targetPath)) {
    return;
  }
  let jsContent = fs.readFileSync(targetPath, 'utf8');
  const originalContent = jsContent;

  jsContent = injectTypedef(jsContent, typeSpecifier);

  const optionsPattern = /@param\s+\{any\}\s+options/g;
  const optionsReplacement = '@param {WasmConversionOptions | null | undefined} [options]';
  jsContent = jsContent.replace(optionsPattern, optionsReplacement);

  const returnsPattern = /@returns\s+\{any\}/g;
  const returnsReplacement = '@returns {Record<string, string>}';
  jsContent = jsContent.replace(returnsPattern, returnsReplacement);

  if (jsContent !== originalContent) {
    fs.writeFileSync(targetPath, jsContent, 'utf8');
  }
}

if (!typesOnly) {
  if (!fs.existsSync(entryPath)) {
    console.error(`[patch-bundler-entry] Missing entry file at ${entryPath}`);
    process.exit(1);
  }

  const wrapper = `import * as wasmModule from "./html_to_markdown_wasm_bg.wasm";
export * from "./html_to_markdown_wasm_bg.js";
import * as imports_mod from "./html_to_markdown_wasm_bg.js";

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

imports_mod.__wbg_set_wasm(notReadyProxy);

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
  imports_mod.__wbg_set_wasm(exports);
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

async function ensureInitPromise() {
  if (initialized) {
    return Promise.resolve(wasmExports);
  }
  if (!initPromise) {
    initPromise = (async () => {
      let module = wasmModule;

      // Handle promise-wrapped modules
      if (module && typeof module.then === "function") {
        module = await module;
      }

      // Handle function loaders (like @rollup/plugin-wasm)
      if (module && typeof module.default === "function") {
        module = await module.default(module);
      }

      // Handle WebAssembly.Module (Wrangler/esbuild)
      if (module && module.default instanceof WebAssembly.Module) {
        const imports = {};
        imports["./html_to_markdown_wasm_bg.js"] = {};
        for (const key in imports_mod) {
          if ((key.startsWith('__wbg_') || key.startsWith('__wbindgen_')) && key !== '__wbg_set_wasm' && typeof imports_mod[key] === 'function') {
            imports["./html_to_markdown_wasm_bg.js"][key] = imports_mod[key];
          }
        }
        const instance = await WebAssembly.instantiate(module.default, imports);
        return finalize(instance.exports);
      }

      // Try standard export detection
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
}

if (!fs.existsSync(dtsPath)) {
  console.error(`[patch-bundler-entry] Missing type definitions at ${dtsPath}`);
  process.exit(1);
}

let content = fs.readFileSync(dtsPath, 'utf8');

if (!typesOnly && !content.includes('initWasm():')) {
  const additions = `\nexport declare function initWasm(): Promise<void>;\nexport declare const wasmReady: Promise<void>;\n`;
  content += additions;
}

if (content.includes('options: any')) {
  content = content.replace(/options: any/g, 'options?: WasmConversionOptions | null');
}

content = content.replace('readonly attributes: any;', 'readonly attributes: Record<string, string>;');

if (!content.includes('interface WasmConversionOptions')) {
  content += `\n${typeDefinitions}`;
}

fs.writeFileSync(dtsPath, content, 'utf8');

const jsDocTarget = fs.existsSync(bgPath) ? bgPath : entryPath;
const typeImportSpecifier = './html_to_markdown_wasm';
patchJsDoc(jsDocTarget, typeImportSpecifier);

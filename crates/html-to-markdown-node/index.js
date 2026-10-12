// HAND-MAINTAINED SOURCE FILE — NOT generated, and NOT rebuilt at publish time.
// The publish pipeline copies this file verbatim into the npm tarball, so whatever is
// committed here is exactly what ships.
//
// Do NOT overwrite it with `napi build` output. A build run from the repo root emits a
// loader for `html-to-markdown-monorepo-*` (the root manifest name) requiring
// `./index.*.node`, none of which is ever published — that is a broken package on every
// platform. Run builds from this crate directory, and if this file shows up modified
// after a build, restore it:
//   git show HEAD:crates/html-to-markdown-node/index.js > crates/html-to-markdown-node/index.js
//
// scripts/verify-loader-manifest.mjs checks this file against package.json and index.d.ts.
"use strict";

const { platform, arch } = process;
const isMusl = () => {
  // Prefer the report-header `glibcVersion` string when present — fastest and
  // unambiguous on Node builds that populate it. On Node 22+, certain CI
  // environments leave `glibcVersion` undefined even on glibc systems, so the
  // `=== undefined` branch from older napi-rs templates produces a false
  // "is musl" positive. Fall through to the filesystem heuristic instead: on
  // glibc systems `/lib64/ld-musl-x86_64.so.1` does not exist; on musl systems
  // it always does. statSync errors → not musl.
  if (
    typeof process.report === "object" &&
    typeof process.report.getReport === "function"
  ) {
    const report = process.report.getReport();
    if (
      report &&
      report.header &&
      typeof report.header.glibcVersion === "string"
    ) {
      return false;
    }
  }
  try {
    require("fs").statSync("/lib64/ld-musl-x86_64.so.1");
    return true;
  } catch {
    return false;
  }
};

let nativeBinding = null;
const loadErrors = [];

function requireOptionalDependency(name) {
  try {
    return require(name);
  } catch (e) {
    loadErrors.push(`Optional dependency ${name}: ${e.message}`);
    return null;
  }
}

const tryLoadBinding = () => {
  // Local `.node` files are named after `napi.binaryName` (binary file name on disk).
  // Optional-dep packages are named after `napi.packageName` (npm subpackage names),
  // which inherits any scope prefix from the parent package.
  const targets = [
    ["linux", "x64", "gnu", "./html-to-markdown-node.linux-x64-gnu.node", "@xberg-io/html-to-markdown-linux-x64-gnu"],
    ["linux", "arm64", "gnu", "./html-to-markdown-node.linux-arm64-gnu.node", "@xberg-io/html-to-markdown-linux-arm64-gnu"],
    ["linux", "x64", "musl", "./html-to-markdown-node.linux-x64-musl.node", "@xberg-io/html-to-markdown-linux-x64-musl"],
    ["linux", "arm64", "musl", "./html-to-markdown-node.linux-arm64-musl.node", "@xberg-io/html-to-markdown-linux-arm64-musl"],
    ["darwin", "x64", null, "./html-to-markdown-node.darwin-x64.node", "@xberg-io/html-to-markdown-darwin-x64"],
    ["darwin", "arm64", null, "./html-to-markdown-node.darwin-arm64.node", "@xberg-io/html-to-markdown-darwin-arm64"],
    ["win32", "x64", null, "./html-to-markdown-node.win32-x64-msvc.node", "@xberg-io/html-to-markdown-win32-x64-msvc"],
    ["win32", "arm64", null, "./html-to-markdown-node.win32-arm64-msvc.node", "@xberg-io/html-to-markdown-win32-arm64-msvc"],
  ];

  for (const [plat, a, abi, localPath, optionalDep] of targets) {
    if (platform !== plat || arch !== a) {
      continue;
    }

    if (plat === "linux" && abi) {
      const isCurMusl = isMusl();
      if ((abi === "musl") !== isCurMusl) {
        continue;
      }
    }

    try {
      nativeBinding = require(localPath);
      if (nativeBinding) {
        return;
      }
    } catch (e) {
      loadErrors.push(e.message);
    }

    try {
      const optBinding = requireOptionalDependency(optionalDep);
      if (optBinding) {
        nativeBinding = optBinding;
        return;
      }
    } catch (e) {
      loadErrors.push(e.message);
    }
  }
};

tryLoadBinding();

if (!nativeBinding) {
  throw new Error(
    `Failed to load native binding for ${platform}-${arch}. Errors: ${loadErrors.join(", ")}`
  );
}

module.exports = nativeBinding;

// --- BEGIN cjs named re-exports (issue #450) ---
// Hand-maintained, and deliberately limited to the surface index.d.ts declares.
// `module.exports = nativeBinding` alone gives Node's CJS named-export detection
// nothing to see, so `import { convert } from ...` fails without these.
// Keep in sync with index.d.ts; scripts/verify-loader-manifest.mjs enforces it.
module.exports.convert = nativeBinding.convert;
module.exports.CodeBlockStyle = nativeBinding.CodeBlockStyle;
module.exports.HeadingStyle = nativeBinding.HeadingStyle;
module.exports.HiddenContent = nativeBinding.HiddenContent;
module.exports.HighlightStyle = nativeBinding.HighlightStyle;
module.exports.ImageType = nativeBinding.ImageType;
module.exports.InlineDataMedia = nativeBinding.InlineDataMedia;
module.exports.LinkStyle = nativeBinding.LinkStyle;
module.exports.LinkType = nativeBinding.LinkType;
module.exports.ListIndentType = nativeBinding.ListIndentType;
module.exports.NewlineStyle = nativeBinding.NewlineStyle;
module.exports.NodeType = nativeBinding.NodeType;
module.exports.OutputFormat = nativeBinding.OutputFormat;
module.exports.PreprocessingPreset = nativeBinding.PreprocessingPreset;
module.exports.StructuredDataType = nativeBinding.StructuredDataType;
module.exports.TextDirection = nativeBinding.TextDirection;
module.exports.TierStrategy = nativeBinding.TierStrategy;
module.exports.UrlEscapeStyle = nativeBinding.UrlEscapeStyle;
module.exports.VisitorHandle = nativeBinding.VisitorHandle;
module.exports.VisitResult = nativeBinding.VisitResult;
module.exports.WarningKind = nativeBinding.WarningKind;
module.exports.WhitespaceMode = nativeBinding.WhitespaceMode;

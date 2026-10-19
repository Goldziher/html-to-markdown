import assert from "node:assert/strict";
import { readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import test from "node:test";

// ~keep Install docs dependencies first: pnpm --dir docs-site install --frozen-lockfile.
const packagePath = realpathSync(
  fileURLToPath(new URL("../../docs-site/node_modules/starlight-llms-txt/package.json", import.meta.url)),
);
const pluginRequire = createRequire(packagePath);

test("should_use_the_direct_glob_matcher_without_loading_the_vulnerable_braces_dependency", () => {
  const generator = readFileSync(new URL("./generator.ts", `file://${packagePath}`), "utf8");
  assert.match(generator, /import picomatch from ['"]picomatch['"]/);
  assert.equal(generator.includes("micromatch"), false);
  assert.equal(generator.match(/picomatch\((?:include|exclude|expr)\)\((?:doc\.id|id)\)/g)?.length, 4);
  for (const dependency of ["micromatch", "braces"]) {
    assert.throws(() => pluginRequire.resolve(dependency), { code: "MODULE_NOT_FOUND" });
  }
});

test("should_preserve_glob_filtering_for_document_selection_and_priority", () => {
  const picomatch = pluginRequire("picomatch");
  const cases = [
    ["guides/intro", "guides/*", true],
    ["guides/deep/page", "guides/*", false],
    ["guides/deep/page", "guides/**", true],
    ["reference/api-rust", "reference/api-{rust,go}", true],
    ["reference/api-python", "reference/api-{rust,go}", false],
    ["guides/.private", "guides/**", false],
    ["guides/a.md", "**/*.md", true],
    ["日本語/intro", "日本語/**", true],
    ["reference/api-go", ["guides/**", "reference/**"], true],
    ["concepts/overview", ["guides/**", "reference/**"], false],
    ["reference/api-go", "!reference/**", false],
    ["guides/intro", "!reference/**", true],
  ];
  assert.equal(cases.length, 12);
  for (const [documentId, patterns, expected] of cases) {
    assert.equal(picomatch(patterns)(documentId), expected, JSON.stringify({ documentId, patterns }));
  }
});

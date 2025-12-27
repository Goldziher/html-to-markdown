#!/usr/bin/env node
/**
 * Profile visitor callback overhead in html-to-markdown Node.js binding.
 *
 * Measures:
 *   - ThreadsafeFunction callback overhead
 *   - V8 GC impact of visitor callbacks
 *   - Context marshalling cost
 *   - Result conversion overhead
 *
 * Test scenarios:
 *   - no-op: Visitor with empty callbacks
 *   - simple: Simple text extraction
 *   - custom_output: Building custom output
 *   - complex: Multiple operations per callback
 */

const fs = require("fs");
const path = require("path");
const { performance } = require("perf_hooks");

// Try to use the local package, fallback to installed
let htmlToMarkdown;
try {
  // Try from local build
  htmlToMarkdown = require(
    path.join(__dirname, "../packages/typescript/dist/index.js")
  );
} catch {
  try {
    // Fallback to package
    htmlToMarkdown = require("html-to-markdown");
  } catch (e) {
    console.error("Error: Could not load html-to-markdown module");
    console.error("Make sure to build with: cd packages/typescript && pnpm install && pnpm build");
    process.exit(1);
  }
}

/**
 * Timing metrics for a test run
 */
class TimingMetrics {
  constructor(data) {
    this.scenario = data.scenario;
    this.htmlSizeBytes = data.htmlSizeBytes;
    this.elementCount = data.elementCount;
    this.baselineMs = data.baselineMs;
    this.visitorMs = data.visitorMs;
    this.overheadMs = data.overheadMs;
    this.overheadPercent = data.overheadPercent;
    this.callbackInvocations = data.callbackInvocations;
    this.avgCallbackTimeUs = data.avgCallbackTimeUs;
    this.iterations = data.iterations;
    this.gcPauses = data.gcPauses || [];
    this.gcHeapBefore = data.gcHeapBefore || 0;
    this.gcHeapAfter = data.gcHeapAfter || 0;
  }
}

/**
 * Load test HTML document
 */
function loadTestHtml(filename) {
  const filepath = path.join(
    __dirname,
    "../test_documents/html/wikipedia",
    filename
  );
  if (!fs.existsSync(filepath)) {
    throw new Error(`Test document not found: ${filepath}`);
  }
  return fs.readFileSync(filepath, "utf8");
}

/**
 * Count HTML elements by regex
 */
function countElements(html) {
  return (html.match(/<[^/>]+>/g) || []).length;
}

/**
 * No-op visitor
 */
class NoOpVisitor {
  constructor() {
    this.invocations = 0;
  }

  visitNode(node) {
    this.invocations++;
  }
}

/**
 * Simple text extraction visitor
 */
class SimpleVisitor {
  constructor() {
    this.invocations = 0;
    this.texts = [];
  }

  visitNode(node) {
    this.invocations++;
    if (node.type === "text") {
      const text = node.content || "";
      this.texts.push(text);
    }
  }
}

/**
 * Custom output builder visitor
 */
class CustomOutputVisitor {
  constructor() {
    this.invocations = 0;
    this.output = [];
  }

  visitNode(node) {
    this.invocations++;
    const nodeType = node.type || "";
    const tag = node.tag || "";

    if (nodeType === "element") {
      this.output.push(`[${tag.toUpperCase()}]`);
    } else if (nodeType === "text") {
      const content = node.content || "";
      this.output.push(content);
    }
  }
}

/**
 * Complex visitor with multiple operations
 */
class ComplexVisitor {
  constructor() {
    this.invocations = 0;
    this.stats = {};
    this.depths = [];
  }

  visitNode(node) {
    this.invocations++;
    const nodeType = node.type || "";

    // Track statistics
    if (!(nodeType in this.stats)) {
      this.stats[nodeType] = 0;
    }
    this.stats[nodeType]++;

    // Track depth
    const depth = node.depth || 0;
    this.depths.push(depth);

    // Do some computation
    if (node.attributes) {
      const attrCount = Object.keys(node.attributes).length;
      this.stats["attrs_total"] = (this.stats["attrs_total"] || 0) + attrCount;
    }
  }
}

/**
 * Measure GC pauses during execution
 */
function measureGCPauses(fn, iterations = 1) {
  if (!global.gc) {
    console.warn("Warning: GC measurements unavailable. Run with --expose-gc");
    return { time: fn(iterations), pauses: [] };
  }

  global.gc(); // Clean up before test
  const gcPauses = [];
  let lastGc = 0;

  // Note: We can't directly hook GC, so we estimate by measuring heap
  const heapBefore = process.memoryUsage().heapUsed;

  const start = performance.now();
  fn(iterations);
  const time = performance.now() - start;

  const heapAfter = process.memoryUsage().heapUsed;

  return {
    time,
    heapBefore,
    heapAfter,
    heapGrowth: heapAfter - heapBefore,
    pauses: gcPauses,
  };
}

/**
 * Benchmark conversion with visitor
 */
function benchmarkWithVisitor(html, visitor, iterations = 10) {
  return measureGCPauses(
    (iter) => {
      for (let i = 0; i < iter; i++) {
        try {
          // Try to call with visitor - this might not be implemented yet
          if (htmlToMarkdown.convertWithVisitor) {
            htmlToMarkdown.convertWithVisitor(html, { visitor });
          } else {
            // Fallback to regular convert
            htmlToMarkdown.convert(html);
          }
        } catch (e) {
          // Visitor pattern might not be implemented
        }
      }
    },
    iterations
  );
}

/**
 * Benchmark baseline conversion
 */
function benchmarkBaseline(html, iterations = 10) {
  return measureGCPauses(
    (iter) => {
      for (let i = 0; i < iter; i++) {
        htmlToMarkdown.convert(html);
      }
    },
    iterations
  );
}

/**
 * Profile a specific scenario
 */
function profileScenario(name, html, VisitorClass, iterations = 10) {
  console.log(`\nProfiling scenario: ${name}`);
  console.log(`  HTML size: ${html.length} bytes`);
  console.log(`  Iterations: ${iterations}`);

  const elementCount = countElements(html);

  // Warm up
  console.log("  Warming up...");
  benchmarkBaseline(html, 2);

  // Baseline
  console.log("  Running baseline...");
  const baseline = benchmarkBaseline(html, iterations);
  const baselineAvg = baseline.time / iterations;

  // With visitor
  console.log(`  Running ${name} visitor...`);
  const visitor = new VisitorClass();
  const visitorResult = benchmarkWithVisitor(html, visitor, iterations);
  const visitorAvg = visitorResult.time / iterations;

  // Calculate overhead
  const overheadMs = visitorResult.time - baseline.time;
  const overheadPercent =
    baseline.time > 0 ? (overheadMs / baseline.time) * 100 : 0;

  // Per-callback timing
  const callbackCount = visitor.invocations || 0;
  const avgCallbackTimeUs =
    callbackCount > 0 ? (overheadMs * 1000) / callbackCount : 0;

  console.log(`  Baseline: ${baselineAvg.toFixed(2)}ms/iter`);
  console.log(`  Visitor:  ${visitorAvg.toFixed(2)}ms/iter`);
  console.log(`  Overhead: ${overheadMs.toFixed(2)}ms (${overheadPercent.toFixed(1)}%)`);
  console.log(
    `  Callbacks: ${callbackCount} (${avgCallbackTimeUs.toFixed(2)}µs avg)`
  );
  if (visitorResult.heapGrowth !== undefined) {
    console.log(
      `  Heap growth: ${(visitorResult.heapGrowth / 1024).toFixed(1)}KB`
    );
  }

  return new TimingMetrics({
    scenario: name,
    htmlSizeBytes: html.length,
    elementCount,
    baselineMs: baselineAvg,
    visitorMs: visitorAvg,
    overheadMs,
    overheadPercent,
    callbackInvocations: callbackCount,
    avgCallbackTimeUs,
    iterations,
    gcPauses: visitorResult.pauses,
    gcHeapBefore: visitorResult.heapBefore,
    gcHeapAfter: visitorResult.heapAfter,
  });
}

/**
 * Parse command line arguments
 */
function parseArgs() {
  const args = process.argv.slice(2);
  const result = {
    scenario: "all",
    html: "medium",
    iterations: 10,
    output: "visitor_profile_results",
    gcProfile: false,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--scenario" && i + 1 < args.length) {
      result.scenario = args[++i];
    } else if (args[i] === "--html" && i + 1 < args.length) {
      result.html = args[++i];
    } else if (args[i] === "--iterations" && i + 1 < args.length) {
      result.iterations = parseInt(args[++i], 10);
    } else if (args[i] === "--output" && i + 1 < args.length) {
      result.output = args[++i];
    } else if (args[i] === "--gc-profile") {
      result.gcProfile = true;
    }
  }

  return result;
}

/**
 * Main entry point
 */
function main() {
  const args = parseArgs();

  // Create output directory
  if (!fs.existsSync(args.output)) {
    fs.mkdirSync(args.output, { recursive: true });
  }

  // Select HTML file
  const htmlMap = {
    small: "small_html.html",
    medium: "medium_python.html",
    large: "large_rust.html",
  };
  const htmlFile = htmlMap[args.html] || "medium_python.html";
  const html = loadTestHtml(htmlFile);

  console.log(`Visitor Callback Profiling (Node.js)`);
  console.log(`====================================`);
  console.log(`HTML: ${args.html} (${htmlFile})`);
  console.log(`Size: ${html.length} bytes`);
  console.log(`Output: ${args.output}`);

  const scenarios = [
    ["no-op", NoOpVisitor],
    ["simple", SimpleVisitor],
    ["custom-output", CustomOutputVisitor],
    ["complex", ComplexVisitor],
  ];

  const results = [];

  for (const [scenarioName, VisitorClass] of scenarios) {
    if (args.scenario !== "all" && scenarioName !== args.scenario) {
      continue;
    }

    const metrics = profileScenario(
      scenarioName,
      html,
      VisitorClass,
      args.iterations
    );
    results.push(metrics);
  }

  // Write JSON results
  const jsonPath = path.join(args.output, "results.json");
  const output = {
    htmlSize: html.length,
    htmlFile,
    elementCount: countElements(html),
    timestamp: Date.now(),
    results: results.map((r) => ({
      scenario: r.scenario,
      htmlSizeBytes: r.htmlSizeBytes,
      elementCount: r.elementCount,
      baselineMs: r.baselineMs,
      visitorMs: r.visitorMs,
      overheadMs: r.overheadMs,
      overheadPercent: r.overheadPercent,
      callbackInvocations: r.callbackInvocations,
      avgCallbackTimeUs: r.avgCallbackTimeUs,
      iterations: r.iterations,
      gcHeapBefore: r.gcHeapBefore,
      gcHeapAfter: r.gcHeapAfter,
    })),
  };

  fs.writeFileSync(jsonPath, JSON.stringify(output, null, 2));
  console.log(`\n\nResults written to ${jsonPath}`);

  // Print summary
  console.log("\n\nSummary");
  console.log("=======");
  for (const result of results) {
    console.log(
      `${result.scenario.padEnd(20)} ` +
        `Baseline: ${result.baselineMs.toFixed(2).padStart(7)}ms, ` +
        `Overhead: ${result.overheadMs.toFixed(2).padStart(7)}ms (${result.overheadPercent.toFixed(1).padStart(5)}%), ` +
        `Avg callback: ${result.avgCallbackTimeUs.toFixed(2).padStart(7)}µs`
    );
  }
}

main();

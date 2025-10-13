import { readFileSync } from "node:fs";
import { join } from "node:path";
import { Bench } from "tinybench";
import { convert } from "./index.js";

// Helper to load test documents
const loadTestDoc = (path: string): string => {
  const fullPath = join(__dirname, "../../test_documents", path);
  return readFileSync(fullPath, "utf-8");
};

// Generate complex HTML (matching Python benchmark)
function generateComplexHtml(sizeFactor: number): string {
  const paragraphs: string[] = [];

  for (let i = 0; i < sizeFactor; i++) {
    paragraphs.push(
      `<p>Paragraph ${i} with <strong>bold</strong>, <em>italic</em>, ` +
        `<code>code</code>, <a href="#link${i}">links</a>, and <mark>highlights</mark>.</p>`,
    );
  }

  return `<html><head><title>Test Document</title></head><body>\n${paragraphs.join("\n")}\n</body></html>`;
}

async function runBenchmarks() {
  console.log("@html-to-markdown/node - Native NAPI-RS Benchmarks");
  console.log("=".repeat(60));
  console.log("");

  // Small documents benchmark
  const benchSmall = new Bench({ name: "Small Documents", time: 1000 });
  const smallHtml = generateComplexHtml(5);

  benchSmall
    .add("small document (5 paragraphs)", () => {
      convert(smallHtml);
    })
    .add("small document with options", () => {
      convert(smallHtml, {
        headingStyle: "Atx",
        codeBlockStyle: "Backticks",
        listIndentWidth: 2,
      });
    });

  await benchSmall.run();
  console.log("Small Documents:");
  console.table(
    benchSmall.tasks.map((task) => ({
      Task: task.name,
      "ops/sec": task.result?.hz?.toFixed(0),
      "Average (ms)": task.result?.mean ? (task.result.mean * 1000).toFixed(3) : "N/A",
      Margin: task.result?.rme ? `±${task.result.rme.toFixed(2)}%` : "N/A",
    })),
  );
  console.log("");

  // Medium documents benchmark
  const benchMedium = new Bench({ name: "Medium Documents", time: 1000 });
  const mediumHtml = generateComplexHtml(25);

  benchMedium.add("medium document (25 paragraphs)", () => {
    convert(mediumHtml);
  });

  await benchMedium.run();
  console.log("Medium Documents:");
  console.table(
    benchMedium.tasks.map((task) => ({
      Task: task.name,
      "ops/sec": task.result?.hz?.toFixed(0),
      "Average (ms)": task.result?.mean ? (task.result.mean * 1000).toFixed(3) : "N/A",
      Margin: task.result?.rme ? `±${task.result.rme.toFixed(2)}%` : "N/A",
    })),
  );
  console.log("");

  // Large documents benchmark
  const benchLarge = new Bench({ name: "Large Documents", time: 1000 });
  const largeHtml = generateComplexHtml(100);

  benchLarge.add("large document (100 paragraphs)", () => {
    convert(largeHtml);
  });

  await benchLarge.run();
  console.log("Large Documents:");
  console.table(
    benchLarge.tasks.map((task) => ({
      Task: task.name,
      "ops/sec": task.result?.hz?.toFixed(0),
      "Average (ms)": task.result?.mean ? (task.result.mean * 1000).toFixed(3) : "N/A",
      Margin: task.result?.rme ? `±${task.result.rme.toFixed(2)}%` : "N/A",
    })),
  );
  console.log("");

  // Tables benchmark
  const benchTables = new Bench({ name: "Tables", time: 1000 });
  const tableHtml =
    "<html><body>" +
    Array.from({ length: 20 }, (_, i) => {
      const rows = Array.from(
        { length: 10 },
        (_, j) =>
          `<tr><td>Data${i}-${j}</td><td>Value${i}-${j}</td><td>Info${i}-${j}</td><td>Result${i}-${j}</td></tr>`,
      ).join("");
      return `<table><tr><th>Col1</th><th>Col2</th><th>Col3</th><th>Col4</th></tr>${rows}</table>`;
    }).join("") +
    "</body></html>";

  benchTables.add("20 tables with 10 rows each", () => {
    convert(tableHtml);
  });

  await benchTables.run();
  console.log("Tables:");
  console.table(
    benchTables.tasks.map((task) => ({
      Task: task.name,
      "ops/sec": task.result?.hz?.toFixed(0),
      "Average (ms)": task.result?.mean ? (task.result.mean * 1000).toFixed(3) : "N/A",
      Margin: task.result?.rme ? `±${task.result.rme.toFixed(2)}%` : "N/A",
    })),
  );
  console.log("");

  // Lists benchmark
  const benchLists = new Bench({ name: "Lists", time: 1000 });
  const listHtml =
    "<html><body>" +
    Array.from({ length: 10 }, (_, i) => {
      const items = Array.from(
        { length: 50 },
        (_, j) => `<li>List item ${i}-${j} with <strong>formatting</strong> and <a href="#">links</a></li>`,
      ).join("");
      return `<ul>${items}</ul>`;
    }).join("") +
    "</body></html>";

  benchLists.add("10 lists with 50 items each", () => {
    convert(listHtml);
  });

  await benchLists.run();
  console.log("Lists:");
  console.table(
    benchLists.tasks.map((task) => ({
      Task: task.name,
      "ops/sec": task.result?.hz?.toFixed(0),
      "Average (ms)": task.result?.mean ? (task.result.mean * 1000).toFixed(3) : "N/A",
      Margin: task.result?.rme ? `±${task.result.rme.toFixed(2)}%` : "N/A",
    })),
  );
  console.log("");

  // Real-world Wikipedia documents
  const benchWikipedia = new Bench({ name: "Wikipedia Documents", time: 1000 });
  const wikiTimeline = loadTestDoc("html/wikipedia/lists_timeline.html");
  const wikiCountries = loadTestDoc("html/wikipedia/tables_countries.html");
  const wikiPython = loadTestDoc("html/wikipedia/medium_python.html");

  benchWikipedia
    .add(`Wikipedia Timeline (${Math.round(wikiTimeline.length / 1024)}KB)`, () => {
      convert(wikiTimeline);
    })
    .add(`Wikipedia Countries Table (${Math.round(wikiCountries.length / 1024)}KB)`, () => {
      convert(wikiCountries);
    })
    .add(`Wikipedia Python Article (${Math.round(wikiPython.length / 1024)}KB)`, () => {
      convert(wikiPython);
    });

  await benchWikipedia.run();
  console.log("Wikipedia Real-World Documents:");
  console.table(
    benchWikipedia.tasks.map((task) => ({
      Task: task.name,
      "ops/sec": task.result?.hz?.toFixed(0),
      "Average (ms)": task.result?.mean ? (task.result.mean * 1000).toFixed(3) : "N/A",
      Margin: task.result?.rme ? `±${task.result.rme.toFixed(2)}%` : "N/A",
    })),
  );
  console.log("");

  // Preprocessing benchmark
  const benchPreprocessing = new Bench({ name: "Preprocessing", time: 1000 });
  const complexHtml = generateComplexHtml(20);

  benchPreprocessing
    .add("no preprocessing", () => {
      convert(complexHtml);
    })
    .add("standard preprocessing", () => {
      convert(complexHtml, {
        preprocessing: {
          enabled: true,
          preset: "Standard",
        },
      });
    })
    .add("aggressive preprocessing", () => {
      convert(complexHtml, {
        preprocessing: {
          enabled: true,
          preset: "Aggressive",
          removeNavigation: true,
          removeForms: true,
        },
      });
    });

  await benchPreprocessing.run();
  console.log("Preprocessing:");
  console.table(
    benchPreprocessing.tasks.map((task) => ({
      Task: task.name,
      "ops/sec": task.result?.hz?.toFixed(0),
      "Average (ms)": task.result?.mean ? (task.result.mean * 1000).toFixed(3) : "N/A",
      Margin: task.result?.rme ? `±${task.result.rme.toFixed(2)}%` : "N/A",
    })),
  );
  console.log("");

  // Summary statistics
  const allTasks = [
    ...benchSmall.tasks,
    ...benchMedium.tasks,
    ...benchLarge.tasks,
    ...benchTables.tasks,
    ...benchLists.tasks,
    ...benchWikipedia.tasks,
    ...benchPreprocessing.tasks,
  ];

  const avgOpsPerSec = allTasks.reduce((sum, task) => sum + (task.result?.hz || 0), 0) / allTasks.length;

  console.log("=".repeat(60));
  console.log("Summary:");
  console.log(`Total benchmarks: ${allTasks.length}`);
  console.log(`Average throughput: ${Math.round(avgOpsPerSec).toLocaleString()} ops/sec`);
  console.log("=".repeat(60));
}

// Run benchmarks
runBenchmarks().catch(console.error);

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { convert, convertWithInlineImages, convertWithMetadata, startProfiling, stopProfiling } from "html-to-markdown-node";

type Scenario =
	| "convert-default"
	| "convert-options"
	| "inline-images-default"
	| "inline-images-options"
	| "metadata-default"
	| "metadata-options";

const args = process.argv.slice(2);
const options: {
	file?: string;
	iterations: number;
	format: "html" | "hocr";
	scenario: Scenario;
} = {
	iterations: 50,
	format: "html",
	scenario: "convert-default",
};

for (let i = 0; i < args.length; i += 1) {
	const arg = args[i];
	if (arg === "--file" && args[i + 1]) {
		options.file = args[i + 1];
		i += 1;
	} else if (arg === "--iterations" && args[i + 1]) {
		options.iterations = Math.max(1, Number.parseInt(args[i + 1] ?? "1", 10) || 1);
		i += 1;
	} else if (arg === "--format" && args[i + 1]) {
		const format = args[i + 1]?.toLowerCase();
		if (format === "html" || format === "hocr") {
			options.format = format;
		}
		i += 1;
	} else if (arg === "--scenario" && args[i + 1]) {
		options.scenario = args[i + 1] as Scenario;
		i += 1;
	}
}

if (!options.file) {
	console.error("Error: --file is required");
	process.exit(1);
}

const supportedScenarios: Scenario[] = [
	"convert-default",
	"convert-options",
	"inline-images-default",
	"inline-images-options",
	"metadata-default",
	"metadata-options",
];
if (!supportedScenarios.includes(options.scenario)) {
	console.error(`Unsupported scenario: ${options.scenario}`);
	process.exit(1);
}

const filePath = resolve(options.file);
const html = readFileSync(filePath, "utf8");
const bytesProcessedPerIteration = Buffer.byteLength(html, "utf8");

const conversionOptions = options.format === "hocr" ? { hocrSpatialTables: false } : undefined;

const runScenario = (): void => {
	switch (options.scenario) {
		case "convert-default":
			convert(html);
			break;
		case "convert-options":
			convert(html, conversionOptions);
			break;
		case "inline-images-default":
			convertWithInlineImages(html, undefined, undefined);
			break;
		case "inline-images-options":
			convertWithInlineImages(html, conversionOptions, undefined);
			break;
		case "metadata-default":
			convertWithMetadata(html, undefined, undefined);
			break;
		case "metadata-options":
			convertWithMetadata(html, conversionOptions, undefined);
			break;
		default:
			throw new Error(`Unsupported scenario: ${options.scenario}`);
	}
};

const warmup = Math.max(
	0,
	Number.parseInt(process.env.HTML_TO_MARKDOWN_BENCH_WARMUP ?? "0", 10) || 0,
);
for (let i = 0; i < warmup; i += 1) {
	runScenario();
}

const profileOutput = process.env.HTML_TO_MARKDOWN_PROFILE_OUTPUT;
if (profileOutput) {
	const frequency = Math.max(
		1,
		Number.parseInt(process.env.HTML_TO_MARKDOWN_PROFILE_FREQUENCY ?? "1000", 10) || 1000,
	);
	startProfiling(profileOutput, frequency);
}

const start = process.hrtime.bigint();
for (let i = 0; i < options.iterations; i += 1) {
	runScenario();
}
const elapsedSeconds = Number(process.hrtime.bigint() - start) / 1_000_000_000;

if (profileOutput) {
	stopProfiling();
}

const bytesProcessed = bytesProcessedPerIteration * options.iterations;
const opsPerSec = options.iterations / elapsedSeconds;
const mbPerSec = bytesProcessed / (1024 * 1024) / elapsedSeconds;

const result = {
	language: "node",
	fixture: filePath.split("/").pop() ?? filePath,
	fixture_path: filePath,
	scenario: options.scenario,
	iterations: options.iterations,
	elapsed_seconds: elapsedSeconds,
	ops_per_sec: opsPerSec,
	mb_per_sec: mbPerSec,
	bytes_processed: bytesProcessed,
};

console.log(JSON.stringify(result));

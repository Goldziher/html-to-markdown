import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { convert, convertWithInlineImages, convertWithMetadata, convertWithVisitor, startProfiling, stopProfiling } from "html-to-markdown-node";

type Scenario =
	| "convert-default"
	| "convert-options"
	| "inline-images-default"
	| "inline-images-options"
	| "metadata-default"
	| "metadata-options";

type VisitorType = "noop" | "simple" | "custom" | "complex";

const args = process.argv.slice(2);
const options: {
	file?: string;
	iterations: number;
	format: "html" | "hocr";
	scenario: Scenario;
	visitor?: VisitorType;
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
	} else if (arg === "--visitor" && args[i + 1]) {
		const visitor = args[i + 1];
		if (["noop", "simple", "custom", "complex"].includes(visitor)) {
			options.visitor = visitor as VisitorType;
		}
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

// Visitor factory functions
function createNoopVisitor(): object {
	return {
		visitText: () => "continue",
		visitHeading: () => "continue",
		visitParagraph: () => "continue",
		visitLink: () => "continue",
		visitImage: () => "continue",
		visitStrong: () => "continue",
		visitEm: () => "continue",
		visitCode: () => "continue",
		visitBr: () => "continue",
	};
}

function createSimpleVisitor(): object {
	return {
		textCount: 0,
		linkCount: 0,
		imageCount: 0,
		visitText: () => "continue",
		visitHeading: () => "continue",
		visitParagraph: () => "continue",
		visitLink: () => "continue",
		visitImage: () => "continue",
		visitStrong: () => "continue",
		visitEm: () => "continue",
		visitCode: () => "continue",
		visitBr: () => "continue",
	};
}

function createCustomVisitor(): object {
	return {
		visitText: () => "continue",
		visitHeading: () => "continue",
		visitParagraph: () => "continue",
		visitLink: (_ctx: unknown, href: string, text: string) => ["custom", `LINK[${text}](${href})`],
		visitImage: (_ctx: unknown, src: string, alt: string) => ["custom", `![${alt}](${src})`],
		visitStrong: () => "continue",
		visitEm: () => "continue",
		visitCode: () => "continue",
		visitBr: () => "continue",
	};
}

function createComplexVisitor(): object {
	return {
		texts: 0,
		links: 0,
		images: 0,
		headings: 0,
		visitText: () => "continue",
		visitHeading: () => "continue",
		visitParagraph: () => "continue",
		visitLink: (_ctx: unknown, href: string, text: string) => ["custom", `[${text}](${href})`],
		visitImage: () => "skip",
		visitStrong: () => "continue",
		visitEm: () => "continue",
		visitCode: () => "continue",
		visitBr: () => "continue",
	};
}

// Create visitor if specified
let visitor: object | undefined;
if (options.visitor) {
	const visitorCreators: Record<VisitorType, () => object> = {
		noop: createNoopVisitor,
		simple: createSimpleVisitor,
		custom: createCustomVisitor,
		complex: createComplexVisitor,
	};
	visitor = visitorCreators[options.visitor]();
}

const runScenario = (): void => {
	if (visitor) {
		convertWithVisitor(html, undefined, visitor);
		return;
	}

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

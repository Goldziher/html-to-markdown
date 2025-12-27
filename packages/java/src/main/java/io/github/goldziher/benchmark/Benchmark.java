package io.github.goldziher.benchmark;

import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;

/**
 * Benchmark utility for HTML-to-Markdown conversion performance testing.
 *
 * <p>This program measures the performance of the HTML-to-Markdown converter
 * on real HTML files with configurable iterations and warmup runs.
 *
 * @since 2.17.0
 */
public final class Benchmark {
    /**
     * Default number of benchmark iterations.
     */
    private static final int DEFAULT_ITERATIONS = 50;

    /**
     * Default number of warmup iterations.
     */
    private static final int DEFAULT_WARMUP = 3;

    /**
     * Default profiling frequency in Hz.
     */
    private static final int DEFAULT_PROFILING_FREQUENCY = 1000;

    /**
     * Conversion factor from nanoseconds to seconds.
     */
    private static final double NANOS_TO_SECONDS = 1_000_000_000.0;

    /**
     * Conversion factor from bytes to megabytes.
     */
    private static final double BYTES_TO_MB = 1024.0 * 1024.0;

    /**
     * Main entry point for the benchmark.
     *
     * @param args command-line arguments
     */
    public static void main(final String[] args) {
        String filePath = null;
        int iterations = DEFAULT_ITERATIONS;
        int warmup = DEFAULT_WARMUP;
        String format = "html";
        String scenario = "convert-default";

        for (int i = 0; i < args.length; i++) {
            switch (args[i]) {
                case "--file":
                    if (i + 1 < args.length) {
                        filePath = args[++i];
                    }
                    break;
                case "--iterations":
                    if (i + 1 < args.length) {
                        iterations = Integer.parseInt(args[++i]);
                    }
                    break;
                case "--warmup":
                    if (i + 1 < args.length) {
                        warmup = Integer.parseInt(args[++i]);
                    }
                    break;
                case "--format":
                    if (i + 1 < args.length) {
                        format = args[++i];
                    }
                    break;
                case "--scenario":
                    if (i + 1 < args.length) {
                        scenario = args[++i];
                    }
                    break;
            }
        }

        if (filePath == null) {
            System.err.println("Error: --file is required");
            System.exit(1);
        }

        if (!scenario.equals("convert-default")
                && !scenario.equals("metadata-default")) {
            System.err.println("Unsupported scenario: " + scenario);
            System.exit(1);
        }

        try {
            Path path = Paths.get(filePath);
            String html = Files.readString(path);

            String warmupEnv =
                System.getenv("HTML_TO_MARKDOWN_BENCH_WARMUP");
            if (warmupEnv != null && !warmupEnv.isBlank()) {
                try {
                    warmup = Integer.parseInt(warmupEnv);
                } catch (NumberFormatException ignored) {
                    warmup = DEFAULT_WARMUP;
                }
            }
            if (warmup < 0) {
                warmup = 0;
            }

            for (int i = 0; i < warmup; i++) {
                runScenario(html, scenario);
            }

            String profileOutput =
                System.getenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT");
            if (profileOutput != null && !profileOutput.isBlank()) {
                int frequency = DEFAULT_PROFILING_FREQUENCY;
                String freqEnv =
                    System.getenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY");
                if (freqEnv != null && !freqEnv.isBlank()) {
                    try {
                        frequency = Integer.parseInt(freqEnv);
                    } catch (NumberFormatException ignored) {
                        frequency = DEFAULT_PROFILING_FREQUENCY;
                    }
                }
                HtmlToMarkdown.startProfiling(profileOutput, frequency);
            }

            long startNanos = System.nanoTime();
            for (int i = 0; i < iterations; i++) {
                runScenario(html, scenario);
            }
            long endNanos = System.nanoTime();

            if (profileOutput != null && !profileOutput.isBlank()) {
                HtmlToMarkdown.stopProfiling();
            }

            double elapsedSeconds = (endNanos - startNanos) / NANOS_TO_SECONDS;
            int bytesProcessed = html.getBytes().length * iterations;
            double opsPerSec = iterations / elapsedSeconds;
            double mbPerSec = (bytesProcessed / BYTES_TO_MB) / elapsedSeconds;

            Map<String, Object> result = new HashMap<>();
            result.put("language", "java");
            result.put("fixture", path.getFileName().toString());
            result.put("fixture_path", filePath);
            result.put("scenario", scenario);
            result.put("iterations", iterations);
            result.put("elapsed_seconds", elapsedSeconds);
            result.put("ops_per_sec", opsPerSec);
            result.put("mb_per_sec", mbPerSec);
            result.put("bytes_processed", bytesProcessed);

            System.out.println(toJson(result));

        } catch (IOException e) {
            System.err.println("Error reading file: " + e.getMessage());
            System.exit(1);
        }
    }

    /**
     * Convert a map to a JSON string representation.
     *
     * @param map the map to convert
     * @return JSON string representation
     */
    private static String toJson(final Map<String, Object> map) {
        StringBuilder sb = new StringBuilder("{");
        boolean first = true;
        for (Map.Entry<String, Object> entry : map.entrySet()) {
            if (!first) {
                sb.append(",");
            }
            first = false;
            sb.append("\"").append(entry.getKey()).append("\":");
            Object value = entry.getValue();
            if (value instanceof String) {
                sb.append("\"").append(value).append("\"");
            } else {
                sb.append(value);
            }
        }
        sb.append("}");
        return sb.toString();
    }

    /**
     * Run a benchmark scenario.
     *
     * @param html the HTML content to process
     * @param scenario the scenario name
     */
    private static void runScenario(final String html,
            final String scenario) {
        if ("metadata-default".equals(scenario)) {
            HtmlToMarkdown.convertWithMetadata(html);
        } else {
            HtmlToMarkdown.convert(html);
        }
    }

    /**
     * Private constructor to prevent instantiation.
     */
    private Benchmark() {
        throw new UnsupportedOperationException("Utility class");
    }
}

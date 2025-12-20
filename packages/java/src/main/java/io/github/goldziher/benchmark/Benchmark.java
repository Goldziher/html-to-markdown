package io.github.goldziher.benchmark;

import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;

public class Benchmark {
    private static final int DEFAULT_ITERATIONS = 50;
    private static final int DEFAULT_PROFILING_FREQUENCY = 1000;
    private static final double NANOS_TO_SECONDS = 1_000_000_000.0;
    private static final double BYTES_TO_MB = 1024.0 * 1024.0;

    public static void main(String[] args) {
        String filePath = null;
        int iterations = DEFAULT_ITERATIONS;
        String format = "html";

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
                case "--format":
                    if (i + 1 < args.length) {
                        format = args[++i];
                    }
                    break;
            }
        }

        if (filePath == null) {
            System.err.println("Error: --file is required");
            System.exit(1);
        }

        try {
            Path path = Paths.get(filePath);
            String html = Files.readString(path);

            HtmlToMarkdown.convert(html);

            String profileOutput = System.getenv("HTML_TO_MARKDOWN_PROFILE_OUTPUT");
            if (profileOutput != null && !profileOutput.isBlank()) {
                int frequency = DEFAULT_PROFILING_FREQUENCY;
                String freqEnv = System.getenv("HTML_TO_MARKDOWN_PROFILE_FREQUENCY");
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
                HtmlToMarkdown.convert(html);
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

    private static String toJson(Map<String, Object> map) {
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
}

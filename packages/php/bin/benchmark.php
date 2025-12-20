#!/usr/bin/env php
<?php

declare(strict_types=1);

require __DIR__ . '/../vendor/autoload.php';

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Service\Converter;

$options = getopt('', ['file:', 'iterations:', 'format:']);

$file = $options['file'] ?? null;
$iterations = isset($options['iterations']) ? max(1, (int) $options['iterations']) : 50;
$format = isset($options['format']) ? strtolower((string) $options['format']) : 'html';

if ($file === null) {
    fwrite(STDERR, "Missing --file parameter\n");
    exit(1);
}

if ($format !== 'html' && $format !== 'hocr') {
    fwrite(STDERR, "Unsupported format: {$format}. Expected 'html' or 'hocr'.\n");
    exit(1);
}

if (!file_exists($file)) {
    fwrite(STDERR, "Fixture not found: {$file}\n");
    exit(1);
}

if (!extension_loaded('html_to_markdown')) {
    fwrite(STDERR, "The html_to_markdown extension is not loaded. Run `pie install goldziher/html-to-markdown` or build the extension locally.\n");
    exit(1);
}

$html = file_get_contents($file);
if ($html === false) {
    fwrite(STDERR, "Unable to read fixture: {$file}\n");
    exit(1);
}

$converter = Converter::create();
$optionPayload = [];
if ($format === 'hocr') {
    $optionPayload['hocr_spatial_tables'] = false;
}
$conversionOptions = $optionPayload === [] ? null : ConversionOptions::fromArray($optionPayload);
$payloadSizeBytes = strlen($html);

// Warmup once to ensure caches/allocations are primed
$converter->convert($html, $conversionOptions);

$profileOutput = getenv('HTML_TO_MARKDOWN_PROFILE_OUTPUT') ?: null;
if ($profileOutput !== null && function_exists('html_to_markdown_profile_start')) {
    $frequency = getenv('HTML_TO_MARKDOWN_PROFILE_FREQUENCY');
    $freqValue = $frequency !== false ? (int) $frequency : 1000;
    html_to_markdown_profile_start($profileOutput, $freqValue);
}

$start = hrtime(true);
for ($i = 0; $i < $iterations; $i++) {
    $converter->convert($html, $conversionOptions);
}
$durationNs = hrtime(true) - $start;
$durationSeconds = $durationNs / 1_000_000_000;

if ($profileOutput !== null && function_exists('html_to_markdown_profile_stop')) {
    html_to_markdown_profile_stop();
}

$bytesProcessed = $payloadSizeBytes * $iterations;
$opsPerSecond = $iterations / $durationSeconds;
$mbPerSecond = $bytesProcessed / (1024 * 1024) / $durationSeconds;

$result = [
    'language' => 'php',
    'fixture' => basename($file),
    'fixture_path' => $file,
    'iterations' => $iterations,
    'elapsed_seconds' => $durationSeconds,
    'ops_per_sec' => $opsPerSecond,
    'mb_per_sec' => $mbPerSecond,
    'bytes_processed' => $bytesProcessed,
];

echo json_encode($result, JSON_THROW_ON_ERROR) . PHP_EOL;

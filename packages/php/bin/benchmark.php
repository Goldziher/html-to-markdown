#!/usr/bin/env php
<?php

declare(strict_types=1);

require __DIR__ . '/../vendor/autoload.php';

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter;

$options = getopt('', ['file:', 'iterations:', 'format:', 'scenario:']);

$file = $options['file'] ?? null;
$iterations = isset($options['iterations']) ? max(1, (int) $options['iterations']) : 50;
$format = isset($options['format']) ? strtolower((string) $options['format']) : 'html';
$scenario = isset($options['scenario']) ? strtolower((string) $options['scenario']) : 'convert-default';

if ($file === null) {
    fwrite(STDERR, "Missing --file parameter\n");
    exit(1);
}

if ($format !== 'html' && $format !== 'hocr') {
    fwrite(STDERR, "Unsupported format: {$format}. Expected 'html' or 'hocr'.\n");
    exit(1);
}

$supportedScenarios = [
    'convert-default',
    'convert-options',
    'inline-images-default',
    'inline-images-options',
    'metadata-default',
    'metadata-options',
];
if (!in_array($scenario, $supportedScenarios, true)) {
    fwrite(STDERR, "Unsupported scenario: {$scenario}.\n");
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
$conversionOptions = $optionPayload === [] ? new ConversionOptions() : ConversionOptions::fromArray($optionPayload);
$inlineConfig = new InlineImageConfig();
$metadataConfig = [
    'extract_headers' => true,
    'extract_links' => true,
    'extract_images' => true,
    'extract_structured_data' => true,
    'extract_document' => true,
];
$payloadSizeBytes = strlen($html);

// Warmup once to ensure caches/allocations are primed
runScenario($converter, $html, $scenario, $conversionOptions, $inlineConfig, $metadataConfig);

$profileOutput = getenv('HTML_TO_MARKDOWN_PROFILE_OUTPUT') ?: null;
if ($profileOutput !== null && function_exists('html_to_markdown_profile_start')) {
    $frequency = getenv('HTML_TO_MARKDOWN_PROFILE_FREQUENCY');
    $freqValue = $frequency !== false ? (int) $frequency : 1000;
    html_to_markdown_profile_start($profileOutput, $freqValue);
}

$start = hrtime(true);
for ($i = 0; $i < $iterations; $i++) {
    runScenario($converter, $html, $scenario, $conversionOptions, $inlineConfig, $metadataConfig);
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
    'scenario' => $scenario,
    'iterations' => $iterations,
    'elapsed_seconds' => $durationSeconds,
    'ops_per_sec' => $opsPerSecond,
    'mb_per_sec' => $mbPerSecond,
    'bytes_processed' => $bytesProcessed,
];

echo json_encode($result, JSON_THROW_ON_ERROR) . PHP_EOL;

/**
 * @param array<string, bool|int|string> $metadataConfig
 */
function runScenario(
    Converter $converter,
    string $html,
    string $scenario,
    ConversionOptions $options,
    InlineImageConfig $inlineConfig,
    array $metadataConfig,
): void {
    switch ($scenario) {
        case 'convert-default':
            $converter->convert($html);
            return;
        case 'convert-options':
            $converter->convert($html, $options);
            return;
        case 'inline-images-default':
            $converter->convertWithInlineImages($html, null, $inlineConfig);
            return;
        case 'inline-images-options':
            $converter->convertWithInlineImages($html, $options, $inlineConfig);
            return;
        case 'metadata-default':
            $converter->convertWithMetadata($html, null, $metadataConfig);
            return;
        case 'metadata-options':
            $converter->convertWithMetadata($html, $options, $metadataConfig);
            return;
        default:
            throw new RuntimeException("Unsupported scenario: {$scenario}");
    }
}

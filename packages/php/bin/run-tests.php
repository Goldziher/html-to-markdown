#!/usr/bin/env php
<?php

declare(strict_types=1);

$root = dirname(__DIR__, 3);

$extension = detectExtension($root);

runPhpunit($root, $extension);

function detectExtension(string $root): string
{
    $targetDir = $root . DIRECTORY_SEPARATOR . 'target' . DIRECTORY_SEPARATOR . 'release';
    $filename = match (PHP_OS_FAMILY) {
        'Windows' => 'html_to_markdown_php.dll',
        'Darwin' => 'libhtml_to_markdown_php.dylib',
        default => 'libhtml_to_markdown_php.so',
    };

    $path = $targetDir . DIRECTORY_SEPARATOR . $filename;

    // Always (re)build the extension with the metadata feature enabled. Other tasks in this repo
    // may have built the workspace with `--no-default-features`, leaving behind an extension
    // binary that is missing `html_to_markdown_convert_with_metadata`.
    $command = ['cargo', 'build', '-p', 'html-to-markdown-php', '--release', '--features', 'metadata'];
    runProcess($command, $root);

    if (file_exists($path)) {
        return realpath($path) ?: $path;
    }

    fwrite(STDERR, "Unable to locate compiled html-to-markdown PHP extension at {$path}.\n");
    exit(1);
}

function runPhpunit(string $root, string $extension): void
{
    $phpunit = $root . DIRECTORY_SEPARATOR . 'packages' . DIRECTORY_SEPARATOR . 'php' . DIRECTORY_SEPARATOR . 'vendor' . DIRECTORY_SEPARATOR . 'bin' . DIRECTORY_SEPARATOR . 'phpunit';
    if (!file_exists($phpunit)) {
        fwrite(STDERR, "phpunit executable not found. Did you run composer install?\n");
        exit(1);
    }

    $phpunitConfig = $root . DIRECTORY_SEPARATOR . 'packages' . DIRECTORY_SEPARATOR . 'php' . DIRECTORY_SEPARATOR . 'phpunit.xml';

    $command = [
        'php',
        '-d',
        'extension=' . $extension,
        $phpunit,
        '--configuration',
        $phpunitConfig,
    ];

    runProcess($command, $root . DIRECTORY_SEPARATOR . 'packages' . DIRECTORY_SEPARATOR . 'php');
}

/**
 * @param string[] $command
 */
function runProcess(array $command, string $cwd): void
{
    $descriptorSpec = [
        0 => STDIN,
        1 => STDOUT,
        2 => STDERR,
    ];

    $process = proc_open($command, $descriptorSpec, $pipes, $cwd, null);
    if (!is_resource($process)) {
        fwrite(STDERR, "Failed to start process: " . implode(' ', $command) . "\n");
        exit(1);
    }

    $status = proc_close($process);
    if ($status !== 0) {
        exit($status);
    }
}

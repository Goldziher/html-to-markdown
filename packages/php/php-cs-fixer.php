<?php

declare(strict_types=1);

use PhpCsFixer\Config;
use PhpCsFixer\Finder;

$finder = Finder::create()
    ->files()
    ->in(__DIR__)
    ->path('#^src/#')
    ->path('#^tests/#')
    ->notName('*.blade.php');

return (new Config())
    ->setRiskyAllowed(true)
    ->setRules([
        '@PSR12' => true,
        '@PSR12:risky' => true,
        'array_syntax' => ['syntax' => 'short'],
        'declare_strict_types' => true,
        'final_class' => false,
        'final_public_method_for_abstract_class' => false,
        'single_quote' => true,
        'phpdoc_to_comment' => false,
        'php_unit_method_casing' => false,
        'ordered_imports' => true,
        'no_superfluous_phpdoc_tags' => true,
        'native_function_invocation' => ['include' => ['@internal']],
    ])
    ->setFinder($finder);

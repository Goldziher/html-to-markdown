---
priority: medium
---

# PHP 8.2+ - ext-php-rs Extension + Composer Package

**PHP 8.2+ · ext-php-rs extension · PHPStan level 9 · PSR-12 · PHPUnit · 80%+ coverage**

- PHP 8.2+ with declare(strict_types=1); typed properties, union types, enums
- ext-php-rs extension crate (crates/html-to-markdown-php) compiled to Rust .so/.dll
- PIE packaging metadata in packages/php-ext for distribution
- Composer package in packages/php wraps the extension with typed interfaces
- PHPStan level 9; never suppress warnings
- PSR-12 code standards: phpcbf auto-fix, max 120 char lines, 4-space indent
- PHPUnit tests in packages/php/tests; ClassName → ClassNameTest, 80%+ coverage
- Composer: lock dependencies (composer.lock committed), ^version constraints
- Build flow: cargo build -p html-to-markdown-php --release → composer run test
- Never: business logic in PHP; all conversion logic lives in Rust
- Use Haiku 4.5 for PHP binding engineering and Composer issues

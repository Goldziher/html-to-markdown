import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

interface TestCase {
  name: string;
  html: string;
  expectedMarkdown: string;
  options?: Record<string, any>;
}

function loadFixtures(filename: string): TestCase[] {
  const fixturePath = join(__dirname, '../fixtures', filename);
  const content = readFileSync(fixturePath, 'utf-8');
  return JSON.parse(content);
}

describe('comprehensive html-to-markdown tests', () => {
  const basicFixtures = loadFixtures('basic-html.json');

  basicFixtures.forEach((testCase) => {
    it(testCase.name, async () => {
      const { convertHtmlToMarkdown } = await import('@kreuzberg/html-to-markdown');
      const result = convertHtmlToMarkdown(testCase.html, testCase.options);
      expect(result.trim()).toBe(testCase.expectedMarkdown.trim());
    });
  });
});

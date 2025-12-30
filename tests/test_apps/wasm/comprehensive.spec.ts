import { describe, it, expect, beforeAll } from 'vitest';
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

describe('comprehensive html-to-markdown WASM tests', () => {
  let wasmModule: any;

  beforeAll(async () => {
    try {
      wasmModule = await import('html-to-markdown-wasm');
    } catch (error) {
      console.error('Failed to load WASM module:', error);
      throw error;
    }
  });

  const basicFixtures = loadFixtures('basic-html.json');

  basicFixtures.forEach((testCase) => {
    it(testCase.name, () => {
      const result = wasmModule.convert(testCase.html, testCase.options);
      expect(result.trim()).toBe(testCase.expectedMarkdown.trim());
    });
  });

  describe('edge cases', () => {
    it('should handle nested HTML', () => {
      const html = '<div><p>Nested <strong>content</strong></p></div>';
      const result = wasmModule.convert(html);
      expect(result).toContain('Nested');
      expect(result).toContain('content');
    });

    it('should handle mixed content', () => {
      const html = '<h1>Title</h1><p>Paragraph with <em>emphasis</em> and <strong>strong</strong></p>';
      const result = wasmModule.convert(html);
      expect(result).toContain('Title');
      expect(result).toContain('Paragraph');
      expect(result).toContain('emphasis');
      expect(result).toContain('strong');
    });

    it('should handle special characters', () => {
      const html = '<p>Test & special < characters ></p>';
      const result = wasmModule.convert(html);
      expect(result).toContain('Test');
    });

    it('should handle whitespace', () => {
      const html = '<p>Text with    multiple     spaces</p>';
      const result = wasmModule.convert(html);
      expect(result.length).toBeGreaterThan(0);
    });
  });
});

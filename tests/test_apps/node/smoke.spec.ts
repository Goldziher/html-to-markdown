import { describe, it, expect } from 'vitest';

describe('html-to-markdown smoke tests', () => {
  it('should import the package', async () => {
    const htmlToMarkdown = await import('html-to-markdown');
    expect(htmlToMarkdown).toBeDefined();
  });

  it('should convert basic HTML', async () => {
    const { convertHtmlToMarkdown } = await import('html-to-markdown');
    const html = '<p>Hello World</p>';
    const result = convertHtmlToMarkdown(html);
    expect(result).toContain('Hello World');
  });

  it('should handle options', async () => {
    const { convertHtmlToMarkdown } = await import('html-to-markdown');
    const html = '<h1>Title</h1>';
    const result = convertHtmlToMarkdown(html);
    expect(result).toMatch(/^#/);
  });

  it('should handle empty input', async () => {
    const { convertHtmlToMarkdown } = await import('html-to-markdown');
    const result = convertHtmlToMarkdown('');
    expect(result).toBe('');
  });
});

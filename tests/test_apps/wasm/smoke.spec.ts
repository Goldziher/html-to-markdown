import { describe, it, expect, beforeAll } from 'vitest';

describe('html-to-markdown WASM smoke tests', () => {
  let wasmModule: any;

  beforeAll(async () => {
    try {
      wasmModule = await import('html-to-markdown-wasm');
    } catch (error) {
      console.error('Failed to load WASM module:', error);
      throw error;
    }
  });

  it('should import the WASM package', () => {
    expect(wasmModule).toBeDefined();
  });

  it('should expose convert function', () => {
    expect(typeof wasmModule.convert).toBe('function');
  });

  it('should convert basic HTML', () => {
    const html = '<p>Hello World</p>';
    const result = wasmModule.convert(html);
    expect(result).toContain('Hello World');
  });

  it('should handle heading HTML', () => {
    const html = '<h1>Title</h1>';
    const result = wasmModule.convert(html);
    expect(result).toMatch(/^#/);
  });

  it('should handle empty input', () => {
    const result = wasmModule.convert('');
    expect(result).toBe('');
  });

  it('should handle list HTML', () => {
    const html = '<ul><li>Item 1</li><li>Item 2</li></ul>';
    const result = wasmModule.convert(html);
    expect(result).toContain('Item 1');
    expect(result).toContain('Item 2');
  });

  it('should handle link HTML', () => {
    const html = '<a href="https://example.com">Example</a>';
    const result = wasmModule.convert(html);
    expect(result).toContain('Example');
    expect(result).toContain('example.com');
  });
});

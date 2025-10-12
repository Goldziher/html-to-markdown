import type {
  ConversionOptions,
  InlineImageConfig,
  HtmlExtraction,
} from "./types.js";

export * from "./types.js";

/**
 * Backend type used for conversion
 */
export type Backend = "native" | "wasm";

/**
 * Convert function signature
 */
type ConvertFn = (html: string, options?: ConversionOptions) => string;

/**
 * Convert with inline images function signature
 */
type ConvertWithInlineImagesFn = (
  html: string,
  options?: ConversionOptions,
  imageConfig?: InlineImageConfig,
) => HtmlExtraction;

/**
 * Converter instance that uses either native or WASM backend
 */
class Converter {
  private backend: Backend | null = null;
  private convertFn: ConvertFn | null = null;
  private convertWithInlineImagesFn: ConvertWithInlineImagesFn | null = null;

  /**
   * Initialize the converter with the best available backend
   * Tries native first, falls back to WASM
   */
  private async initialize(): Promise<void> {
    if (this.convertFn !== null) {
      return; // Already initialized
    }

    // Try native bindings first (best performance)
    try {
      const nativeModule = await import("@html-to-markdown/node");
      this.convertFn = nativeModule.convert;
      this.convertWithInlineImagesFn = nativeModule.convertWithInlineImages;
      this.backend = "native";
      return;
    } catch (error) {
      // Native bindings not available, try WASM
      if (process.env.NODE_ENV === "development") {
        console.debug(
          "Native bindings not available, falling back to WASM:",
          error,
        );
      }
    }

    // Fall back to WASM
    try {
      const wasmModule = await import("@html-to-markdown/wasm");
      this.convertFn = wasmModule.convert;
      this.convertWithInlineImagesFn = wasmModule.convertWithInlineImages;
      this.backend = "wasm";
      return;
    } catch (error) {
      throw new Error(
        "Failed to initialize html-to-markdown: Neither native nor WASM backend available",
        { cause: error },
      );
    }
  }

  /**
   * Get the current backend type
   */
  getBackend(): Backend | null {
    return this.backend;
  }

  /**
   * Convert HTML to Markdown
   *
   * @param html - The HTML string to convert
   * @param options - Optional conversion options
   * @returns The converted Markdown string
   *
   * @example
   * ```typescript
   * const converter = new Converter();
   * const markdown = await converter.convert('<h1>Hello World</h1>');
   * console.log(markdown); // # Hello World
   * ```
   *
   * @example
   * ```typescript
   * const converter = new Converter();
   * const markdown = await converter.convert('<h1>Hello</h1>', {
   *   headingStyle: HeadingStyle.Atx,
   *   wrap: true,
   *   wrapWidth: 80,
   * });
   * ```
   */
  async convert(html: string, options?: ConversionOptions): Promise<string> {
    await this.initialize();

    if (!this.convertFn) {
      throw new Error("Converter not initialized");
    }

    return this.convertFn(html, options);
  }

  /**
   * Convert HTML to Markdown synchronously
   * Only works if the converter is already initialized
   *
   * @param html - The HTML string to convert
   * @param options - Optional conversion options
   * @returns The converted Markdown string
   *
   * @throws {Error} If the converter is not initialized
   */
  convertSync(html: string, options?: ConversionOptions): string {
    if (!this.convertFn) {
      throw new Error(
        "Converter not initialized. Call convert() first or use initializeSync()",
      );
    }

    return this.convertFn(html, options);
  }

  /**
   * Convert HTML to Markdown while collecting inline images
   *
   * @param html - The HTML string to convert
   * @param options - Optional conversion options
   * @param imageConfig - Optional inline image configuration
   * @returns Promise with extraction result containing markdown, images, and warnings
   *
   * @example
   * ```typescript
   * const converter = new Converter();
   * const result = await converter.convertWithInlineImages(
   *   '<img src="data:image/png;base64,..." alt="test">',
   *   null,
   *   { maxDecodedSizeBytes: 1024 * 1024, inferDimensions: true }
   * );
   * console.log(result.markdown);
   * console.log(result.inlineImages.length);
   * console.log(result.warnings);
   * ```
   */
  async convertWithInlineImages(
    html: string,
    options?: ConversionOptions,
    imageConfig?: InlineImageConfig,
  ): Promise<HtmlExtraction> {
    await this.initialize();

    if (!this.convertWithInlineImagesFn) {
      throw new Error("Converter not initialized");
    }

    return this.convertWithInlineImagesFn(html, options, imageConfig);
  }

  /**
   * Convert HTML to Markdown with inline images synchronously
   * Only works if the converter is already initialized
   *
   * @param html - The HTML string to convert
   * @param options - Optional conversion options
   * @param imageConfig - Optional inline image configuration
   * @returns Extraction result containing markdown, images, and warnings
   *
   * @throws {Error} If the converter is not initialized
   */
  convertWithInlineImagesSync(
    html: string,
    options?: ConversionOptions,
    imageConfig?: InlineImageConfig,
  ): HtmlExtraction {
    if (!this.convertWithInlineImagesFn) {
      throw new Error(
        "Converter not initialized. Call convert() first or use initializeSync()",
      );
    }

    return this.convertWithInlineImagesFn(html, options, imageConfig);
  }

  /**
   * Initialize the converter synchronously
   * This will throw if initialization fails
   */
  initializeSync(): void {
    if (this.convertFn !== null) {
      return; // Already initialized
    }

    // Try native bindings first
    try {
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const nativeModule = require("@html-to-markdown/node");
      this.convertFn = nativeModule.convert;
      this.convertWithInlineImagesFn = nativeModule.convertWithInlineImages;
      this.backend = "native";
      return;
    } catch (error) {
      // Native bindings not available, try WASM
      if (process.env.NODE_ENV === "development") {
        console.debug(
          "Native bindings not available, falling back to WASM:",
          error,
        );
      }
    }

    // Fall back to WASM (note: WASM initialization might still be async in browsers)
    try {
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const wasmModule = require("@html-to-markdown/wasm");
      this.convertFn = wasmModule.convert;
      this.convertWithInlineImagesFn = wasmModule.convertWithInlineImages;
      this.backend = "wasm";
    } catch (error) {
      throw new Error(
        "Failed to initialize html-to-markdown: Neither native nor WASM backend available",
        { cause: error },
      );
    }
  }
}

// Singleton instance for convenience
const defaultConverter = new Converter();

/**
 * Convert HTML to Markdown using the default converter instance
 *
 * @param html - The HTML string to convert
 * @param options - Optional conversion options
 * @returns The converted Markdown string
 *
 * @example
 * ```typescript
 * import { convert } from 'html-to-markdown';
 *
 * const markdown = await convert('<h1>Hello World</h1>');
 * console.log(markdown); // # Hello World
 * ```
 */
export async function convert(
  html: string,
  options?: ConversionOptions,
): Promise<string> {
  return defaultConverter.convert(html, options);
}

/**
 * Convert HTML to Markdown synchronously using the default converter
 * The converter must be initialized first by calling convert() or initializeSync()
 *
 * @param html - The HTML string to convert
 * @param options - Optional conversion options
 * @returns The converted Markdown string
 *
 * @throws {Error} If the converter is not initialized
 *
 * @example
 * ```typescript
 * import { initializeSync, convertSync } from 'html-to-markdown';
 *
 * initializeSync();
 * const markdown = convertSync('<h1>Hello World</h1>');
 * console.log(markdown); // # Hello World
 * ```
 */
export function convertSync(html: string, options?: ConversionOptions): string {
  return defaultConverter.convertSync(html, options);
}

/**
 * Initialize the default converter synchronously
 *
 * @example
 * ```typescript
 * import { initializeSync, convertSync } from 'html-to-markdown';
 *
 * initializeSync();
 * // Now you can use convertSync
 * ```
 */
export function initializeSync(): void {
  defaultConverter.initializeSync();
}

/**
 * Get the backend type being used by the default converter
 *
 * @returns The backend type ('native' or 'wasm'), or null if not initialized
 *
 * @example
 * ```typescript
 * import { convert, getBackend } from 'html-to-markdown';
 *
 * await convert('<h1>Hello</h1>');
 * console.log('Using backend:', getBackend()); // 'native' or 'wasm'
 * ```
 */
export function getBackend(): Backend | null {
  return defaultConverter.getBackend();
}

/**
 * Convert HTML to Markdown while collecting inline images using the default converter
 *
 * @param html - The HTML string to convert
 * @param options - Optional conversion options
 * @param imageConfig - Optional inline image configuration
 * @returns Promise with extraction result
 *
 * @example
 * ```typescript
 * import { convertWithInlineImages } from 'html-to-markdown';
 *
 * const result = await convertWithInlineImages(
 *   '<img src="data:image/png;base64,..." alt="test">',
 *   null,
 *   { maxDecodedSizeBytes: 1024 * 1024, inferDimensions: true }
 * );
 * console.log(result.markdown);
 * console.log(result.inlineImages);
 * ```
 */
export async function convertWithInlineImages(
  html: string,
  options?: ConversionOptions,
  imageConfig?: InlineImageConfig,
): Promise<HtmlExtraction> {
  return defaultConverter.convertWithInlineImages(html, options, imageConfig);
}

/**
 * Convert HTML to Markdown with inline images synchronously using the default converter
 * The converter must be initialized first
 *
 * @param html - The HTML string to convert
 * @param options - Optional conversion options
 * @param imageConfig - Optional inline image configuration
 * @returns Extraction result
 *
 * @throws {Error} If the converter is not initialized
 *
 * @example
 * ```typescript
 * import { initializeSync, convertWithInlineImagesSync } from 'html-to-markdown';
 *
 * initializeSync();
 * const result = convertWithInlineImagesSync('<img src="data:..." alt="test">');
 * ```
 */
export function convertWithInlineImagesSync(
  html: string,
  options?: ConversionOptions,
  imageConfig?: InlineImageConfig,
): HtmlExtraction {
  return defaultConverter.convertWithInlineImagesSync(
    html,
    options,
    imageConfig,
  );
}

/**
 * Create a new converter instance
 * Useful if you want multiple converters with different backends or configurations
 *
 * @returns A new Converter instance
 *
 * @example
 * ```typescript
 * import { createConverter } from 'html-to-markdown';
 *
 * const converter = createConverter();
 * const markdown = await converter.convert('<h1>Hello</h1>');
 * ```
 */
export function createConverter(): Converter {
  return new Converter();
}

// Default export for convenience
export default {
  convert,
  convertSync,
  convertWithInlineImages,
  convertWithInlineImagesSync,
  initializeSync,
  getBackend,
  createConverter,
  Converter,
};

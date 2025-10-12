/**
 * Heading style options for Markdown output
 */
export enum HeadingStyle {
  /** Underlined style (=== for h1, --- for h2) */
  Underlined = "Underlined",
  /** ATX style (# for h1, ## for h2, etc.) */
  Atx = "Atx",
  /** ATX closed style (# title #) */
  AtxClosed = "AtxClosed",
}

/**
 * List indentation type
 */
export enum ListIndentType {
  Spaces = "Spaces",
  Tabs = "Tabs",
}

/**
 * Whitespace handling mode
 */
export enum WhitespaceMode {
  /** Normalize whitespace */
  Normalized = "Normalized",
  /** Preserve strict whitespace */
  Strict = "Strict",
}

/**
 * Newline style for line breaks
 */
export enum NewlineStyle {
  /** Two spaces at end of line */
  Spaces = "Spaces",
  /** Backslash at end of line */
  Backslash = "Backslash",
}

/**
 * Code block style
 */
export enum CodeBlockStyle {
  /** Indented code blocks (4 spaces) - CommonMark default */
  Indented = "Indented",
  /** Fenced code blocks with backticks (```) */
  Backticks = "Backticks",
  /** Fenced code blocks with tildes (~~~) */
  Tildes = "Tildes",
}

/**
 * Highlight style for `<mark>` elements
 */
export enum HighlightStyle {
  /** ==text== */
  DoubleEqual = "DoubleEqual",
  /** <mark>text</mark> */
  Html = "Html",
  /** **text** */
  Bold = "Bold",
  /** Plain text (no formatting) */
  None = "None",
}

/**
 * Preprocessing preset levels
 */
export enum PreprocessingPreset {
  Minimal = "Minimal",
  Standard = "Standard",
  Aggressive = "Aggressive",
}

/**
 * HTML preprocessing options
 */
export interface PreprocessingOptions {
  /** Enable preprocessing (default: false) */
  enabled?: boolean;
  /** Preprocessing preset (default: Standard) */
  preset?: PreprocessingPreset;
  /** Remove navigation elements (default: true) */
  removeNavigation?: boolean;
  /** Remove form elements (default: true) */
  removeForms?: boolean;
}

/**
 * Main conversion options for HTML to Markdown conversion
 */
export interface ConversionOptions {
  /** Heading style (default: Atx) */
  headingStyle?: HeadingStyle;
  /** List indentation type (default: Spaces) */
  listIndentType?: ListIndentType;
  /** List indentation width in spaces (default: 2) */
  listIndentWidth?: number;
  /** Bullet characters for unordered lists (default: "-") */
  bullets?: string;
  /** Symbol for strong/emphasis, either * or _ (default: "*") */
  strongEmSymbol?: string;
  /** Escape asterisks in text (default: false) */
  escapeAsterisks?: boolean;
  /** Escape underscores in text (default: false) */
  escapeUnderscores?: boolean;
  /** Escape miscellaneous markdown characters (default: false) */
  escapeMisc?: boolean;
  /** Escape all ASCII punctuation for strict CommonMark compliance (default: false) */
  escapeAscii?: boolean;
  /** Default code language for code blocks (default: "") */
  codeLanguage?: string;
  /** Use autolinks for bare URLs (default: true) */
  autolinks?: boolean;
  /** Add default title if none exists (default: false) */
  defaultTitle?: boolean;
  /** Use <br> in tables instead of spaces (default: false) */
  brInTables?: boolean;
  /** Enable spatial table reconstruction in hOCR documents (default: true) */
  hocrSpatialTables?: boolean;
  /** Highlight style for <mark> elements (default: DoubleEqual) */
  highlightStyle?: HighlightStyle;
  /** Extract metadata from HTML (default: true) */
  extractMetadata?: boolean;
  /** Whitespace handling mode (default: Normalized) */
  whitespaceMode?: WhitespaceMode;
  /** Strip newlines from HTML before processing (default: false) */
  stripNewlines?: boolean;
  /** Enable text wrapping (default: false) */
  wrap?: boolean;
  /** Text wrap width in characters (default: 80) */
  wrapWidth?: number;
  /** Treat block elements as inline (default: false) */
  convertAsInline?: boolean;
  /** Symbol for subscript text (default: "") */
  subSymbol?: string;
  /** Symbol for superscript text (default: "") */
  supSymbol?: string;
  /** Newline style (default: Spaces) */
  newlineStyle?: NewlineStyle;
  /** Code block style (default: Indented) */
  codeBlockStyle?: CodeBlockStyle;
  /** Elements where images should remain as markdown (default: []) */
  keepInlineImagesIn?: string[];
  /** Preprocessing options */
  preprocessing?: PreprocessingOptions;
  /** Source encoding, informational only (default: "utf-8") */
  encoding?: string;
  /** Enable debug mode with diagnostic warnings (default: false) */
  debug?: boolean;
  /** List of HTML tags to strip (output only text content) (default: []) */
  stripTags?: string[];
}

/**
 * Inline image format
 */
export type InlineImageFormat =
  | "png"
  | "jpeg"
  | "gif"
  | "bmp"
  | "webp"
  | "svg"
  | string;

/**
 * Inline image source type
 */
export type InlineImageSource = "img_data_uri" | "svg_element";

/**
 * Inline image data
 */
export interface InlineImage {
  /** Raw image data as Buffer (Node) or Uint8Array (browser) */
  data: Buffer | Uint8Array;
  /** Image format (png, jpeg, gif, etc.) */
  format: InlineImageFormat;
  /** Generated or provided filename */
  filename?: string;
  /** Alt text / description */
  description?: string;
  /** Image dimensions [width, height] if available */
  dimensions?: [number, number];
  /** Source type (img_data_uri or svg_element) */
  source: InlineImageSource;
  /** HTML attributes from the source element */
  attributes: Record<string, string>;
}

/**
 * Warning about inline image processing
 */
export interface InlineImageWarning {
  /** Index of the image that caused the warning */
  index: number;
  /** Warning message */
  message: string;
}

/**
 * Configuration for inline image extraction
 */
export interface InlineImageConfig {
  /** Maximum decoded size in bytes (default: 5MB) */
  maxDecodedSizeBytes?: number;
  /** Filename prefix for generated filenames */
  filenamePrefix?: string;
  /** Capture inline SVG elements (default: true) */
  captureSvg?: boolean;
  /** Infer image dimensions (default: false) */
  inferDimensions?: boolean;
}

/**
 * Result of HTML extraction with inline images
 */
export interface HtmlExtraction {
  /** Converted markdown */
  markdown: string;
  /** Extracted inline images */
  inlineImages: InlineImage[];
  /** Warnings encountered during extraction */
  warnings: InlineImageWarning[];
}

/**
 * Error thrown when conversion fails
 */
export class ConversionError extends Error {
  constructor(
    message: string,
    public readonly cause?: unknown,
  ) {
    super(message);
    this.name = "ConversionError";
    Object.setPrototypeOf(this, ConversionError.prototype);
  }
}

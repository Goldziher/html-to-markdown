//! hOCR 1.2 document processing.
//!
//! Complete hOCR 1.2 specification support for extracting structured content from OCR documents.
//!
//! ## Features
//!
//! - **Full Element Support**: All 40+ hOCR 1.2 element types
//! - **Complete Property Parsing**: All 20+ hOCR properties (bbox, baseline, fonts, etc.)
//! - **Document Structure**: Logical hierarchy (paragraphs, sections, chapters)
//! - **Spatial Table Reconstruction**: Automatic table detection from bbox coordinates
//! - **Metadata Extraction**: OCR system info, capabilities, languages
//!
//! ## Modules
//!
//! - [`types`]: Core hOCR element and property types
//! - [`parser`]: Property parsing from title attributes
//! - [`extractor`]: DOM to hOCR element tree extraction
//! - [`converter`]: hOCR to Markdown conversion
//! - [`spatial`]: Spatial table reconstruction from bounding boxes

pub mod converter;
pub mod extractor;
pub mod parser;
pub mod spatial;
pub mod types;

pub use converter::{convert_to_markdown, convert_to_markdown_with_options};
pub use extractor::extract_hocr_document;
pub use spatial::{HocrWord, extract_hocr_words, reconstruct_table, table_to_markdown};
pub use types::{BBox, Baseline, HocrElement, HocrElementType, HocrMetadata, HocrProperties};

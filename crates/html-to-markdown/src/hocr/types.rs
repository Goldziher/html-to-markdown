#![allow(clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::unused_self)]
//! hOCR 1.2 type definitions
//!
//! Complete type system for hOCR 1.2 specification elements and properties.

use std::collections::HashMap;

/// All hOCR 1.2 element types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HocrElementType {
    /// Document abstract or summary
    OcrAbstract,
    /// Author attribution
    OcrAuthor,
    /// Block quotation
    OcrBlockquote,
    /// Image caption
    OcrCaption,
    /// Chapter division
    OcrChapter,
    /// Document root element
    OcrDocument,
    /// Paragraph text
    OcrPar,
    /// Major part or section
    OcrPart,
    /// Section heading
    OcrSection,
    /// Subsection heading
    OcrSubsection,
    /// Subsubsection heading
    OcrSubsubsection,
    /// Document title
    OcrTitle,

    /// Column area of multi-column layout
    OcrCarea,
    /// Column within a page
    OcrColumn,
    /// Text line
    OcrLine,
    /// Linearization element
    OcrLinear,
    /// Page element
    OcrPage,
    /// Separator or divider
    OcrSeparator,

    /// Chemical formula
    OcrChem,
    /// Display equation or complex content
    OcrDisplay,
    /// Float element (typically with caption)
    OcrFloat,
    /// Footer area of page
    OcrFooter,
    /// Header area of page
    OcrHeader,
    /// Image element
    OcrImage,
    /// Line drawing or diagram
    OcrLinedrawing,
    /// Mathematical formula
    OcrMath,
    /// Page number marker
    OcrPageno,
    /// Photograph element
    OcrPhoto,
    /// Table element
    OcrTable,
    /// Text float (floating text box)
    OcrTextfloat,
    /// Text image (text rendered as image)
    OcrTextimage,

    /// Character information
    OcrCinfo,
    /// Decorative capital letter
    OcrDropcap,
    /// Glyph element
    OcrGlyph,
    /// Multiple glyphs
    OcrGlyphs,
    /// Noise or artifacts
    OcrNoise,
    /// `XyZut` analysis segment
    OcrXycut,

    /// Block-level element
    OcrxBlock,
    /// OCR word line
    OcrxLine,
    /// Individual word element
    OcrxWord,
}

impl HocrElementType {
    /// Get element type from class name
    #[must_use]
    pub fn from_class(class: &str) -> Option<Self> {
        match class {
            "ocr_abstract" => Some(Self::OcrAbstract),
            "ocr_author" => Some(Self::OcrAuthor),
            "ocr_blockquote" => Some(Self::OcrBlockquote),
            "ocr_caption" => Some(Self::OcrCaption),
            "ocr_chapter" => Some(Self::OcrChapter),
            "ocr_document" => Some(Self::OcrDocument),
            "ocr_par" => Some(Self::OcrPar),
            "ocr_part" => Some(Self::OcrPart),
            "ocr_section" => Some(Self::OcrSection),
            "ocr_subsection" => Some(Self::OcrSubsection),
            "ocr_subsubsection" => Some(Self::OcrSubsubsection),
            "ocr_title" => Some(Self::OcrTitle),

            "ocr_carea" => Some(Self::OcrCarea),
            "ocr_column" => Some(Self::OcrColumn),
            "ocr_line" => Some(Self::OcrLine),
            "ocr_linear" => Some(Self::OcrLinear),
            "ocr_page" => Some(Self::OcrPage),
            "ocr_separator" => Some(Self::OcrSeparator),

            "ocr_chem" => Some(Self::OcrChem),
            "ocr_display" => Some(Self::OcrDisplay),
            "ocr_float" => Some(Self::OcrFloat),
            "ocr_footer" => Some(Self::OcrFooter),
            "ocr_header" => Some(Self::OcrHeader),
            "ocr_image" => Some(Self::OcrImage),
            "ocr_linedrawing" => Some(Self::OcrLinedrawing),
            "ocr_math" => Some(Self::OcrMath),
            "ocr_pageno" => Some(Self::OcrPageno),
            "ocr_photo" => Some(Self::OcrPhoto),
            "ocr_table" => Some(Self::OcrTable),
            "ocr_textfloat" => Some(Self::OcrTextfloat),
            "ocr_textimage" => Some(Self::OcrTextimage),

            "ocr_cinfo" => Some(Self::OcrCinfo),
            "ocr_dropcap" => Some(Self::OcrDropcap),
            "ocr_glyph" => Some(Self::OcrGlyph),
            "ocr_glyphs" => Some(Self::OcrGlyphs),
            "ocr_noise" => Some(Self::OcrNoise),
            "ocr_xycut" => Some(Self::OcrXycut),

            "ocrx_block" => Some(Self::OcrxBlock),
            "ocrx_line" => Some(Self::OcrxLine),
            "ocrx_word" => Some(Self::OcrxWord),

            _ => None,
        }
    }
}

/// Bounding box with corner coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BBox {
    /// Left edge x-coordinate (pixels)
    pub x1: u32,
    /// Top edge y-coordinate (pixels)
    pub y1: u32,
    /// Right edge x-coordinate (pixels)
    pub x2: u32,
    /// Bottom edge y-coordinate (pixels)
    pub y2: u32,
}

impl BBox {
    /// Calculate the width from left to right edge
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.x2.saturating_sub(self.x1)
    }

    /// Calculate the height from top to bottom edge
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.y2.saturating_sub(self.y1)
    }
}

/// Baseline property for text alignment in OCR elements
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baseline {
    /// Baseline slope relative to horizontal
    pub slope: f64,
    /// Baseline vertical offset in pixels
    pub constant: i32,
}

/// All hOCR properties extracted from element attributes
#[derive(Debug, Clone, Default)]
pub struct HocrProperties {
    /// Bounding box (left, top, right, bottom) coordinates
    pub bbox: Option<BBox>,
    /// Baseline properties (slope, constant offset)
    pub baseline: Option<Baseline>,
    /// Text rotation angle in degrees
    pub textangle: Option<f64>,
    /// Polygon coordinates for non-rectangular regions
    pub poly: Option<Vec<(i32, i32)>>,

    /// Word-level confidence score (0-100)
    pub x_wconf: Option<f64>,
    /// Per-character confidence scores
    pub x_confs: Vec<f64>,
    /// Natural language processing results
    pub nlp: Vec<f64>,

    /// Font name or family
    pub x_font: Option<String>,
    /// Font size in points
    pub x_fsize: Option<u32>,

    /// Reading order index for document structure
    pub order: Option<u32>,
    /// Column flow direction (ltr, rtl, etc.)
    pub cflow: Option<String>,
    /// Hard line break indicator
    pub hardbreak: bool,

    /// Cut lines for layout analysis
    pub cuts: Vec<Vec<u32>>,
    /// Alternative bounding boxes for multi-part elements
    pub x_bboxes: Vec<BBox>,

    /// Image path or data URI
    pub image: Option<String>,
    /// MD5 hash of image content
    pub imagemd5: Option<String>,
    /// Physical page number
    pub ppageno: Option<u32>,
    /// Logical page number/label
    pub lpageno: Option<String>,
    /// Scanner resolution (`dpi_x`, `dpi_y`)
    pub scan_res: Option<(u32, u32)>,
    /// Image source file paths
    pub x_source: Vec<String>,
    /// Scanner device identifier
    pub x_scanner: Option<String>,

    /// Additional custom properties
    pub other: HashMap<String, String>,
}

/// A complete hOCR element with type, properties, text content, and child elements
#[derive(Debug, Clone)]
pub struct HocrElement {
    /// The semantic type of this hOCR element
    pub element_type: HocrElementType,
    /// All extracted properties (bbox, confidence, etc.)
    pub properties: HocrProperties,
    /// Text content of this element
    pub text: String,
    /// Child elements in the document tree
    pub children: Vec<Self>,
}

/// hOCR document metadata extracted from document properties
#[derive(Debug, Clone, Default)]
pub struct HocrMetadata {
    /// Name and version of the OCR system used
    pub ocr_system: Option<String>,
    /// OCR capabilities supported (e.g., "`ocr_page`", "`ocr_carea`")
    pub ocr_capabilities: Vec<String>,
    /// Total number of pages in the OCR'd document
    pub ocr_number_of_pages: Option<u32>,
    /// Languages detected in the document (ISO 639 codes)
    pub ocr_langs: Vec<String>,
    /// Scripts used in the document (ISO 15924 codes)
    pub ocr_scripts: Vec<String>,
}

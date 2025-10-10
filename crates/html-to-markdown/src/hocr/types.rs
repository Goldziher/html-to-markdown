//! hOCR 1.2 type definitions
//!
//! Complete type system for hOCR 1.2 specification elements and properties.

use std::collections::HashMap;

/// All hOCR 1.2 element types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HocrElementType {
    // Logical structure
    OcrAbstract,
    OcrAuthor,
    OcrBlockquote,
    OcrCaption,
    OcrChapter,
    OcrDocument,
    OcrPar,
    OcrPart,
    OcrSection,
    OcrSubsection,
    OcrSubsubsection,
    OcrTitle,

    // Typesetting
    OcrCarea,
    OcrColumn, // deprecated
    OcrLine,
    OcrLinear,
    OcrPage,
    OcrSeparator,

    // Float elements
    OcrChem,
    OcrDisplay,
    OcrFloat,
    OcrFooter,
    OcrHeader,
    OcrImage,
    OcrLinedrawing,
    OcrMath,
    OcrPageno,
    OcrPhoto,
    OcrTable,
    OcrTextfloat,
    OcrTextimage,

    // Inline elements
    OcrCinfo,
    OcrDropcap,
    OcrGlyph,
    OcrGlyphs,
    OcrNoise,
    OcrXycut,

    // Engine-specific
    OcrxBlock,
    OcrxLine,
    OcrxWord,
}

impl HocrElementType {
    /// Get element type from class name
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

/// Bounding box (x1, y1, x2, y2)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

impl BBox {
    pub fn width(&self) -> u32 {
        self.x2.saturating_sub(self.x1)
    }

    pub fn height(&self) -> u32 {
        self.y2.saturating_sub(self.y1)
    }
}

/// Baseline property (slope, constant)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baseline {
    pub slope: f64,
    pub constant: i32,
}

/// All hOCR properties
#[derive(Debug, Clone, Default)]
pub struct HocrProperties {
    // Layout
    pub bbox: Option<BBox>,
    pub baseline: Option<Baseline>,
    pub textangle: Option<f64>,
    pub poly: Option<Vec<(i32, i32)>>,

    // Confidence
    pub x_wconf: Option<f64>,
    pub x_confs: Vec<f64>,
    pub nlp: Vec<f64>,

    // Font
    pub x_font: Option<String>,
    pub x_fsize: Option<u32>,

    // Content flow
    pub order: Option<u32>,
    pub cflow: Option<String>,
    pub hardbreak: bool,

    // Character-level
    pub cuts: Vec<Vec<u32>>,
    pub x_bboxes: Vec<BBox>,

    // Page
    pub image: Option<String>,
    pub imagemd5: Option<String>,
    pub ppageno: Option<u32>,
    pub lpageno: Option<String>,
    pub scan_res: Option<(u32, u32)>,
    pub x_source: Vec<String>,
    pub x_scanner: Option<String>,

    // Other
    pub other: HashMap<String, String>,
}

/// A complete hOCR element
#[derive(Debug, Clone)]
pub struct HocrElement {
    pub element_type: HocrElementType,
    pub properties: HocrProperties,
    pub text: String,
    pub children: Vec<HocrElement>,
}

/// hOCR document metadata
#[derive(Debug, Clone, Default)]
pub struct HocrMetadata {
    pub ocr_system: Option<String>,
    pub ocr_capabilities: Vec<String>,
    pub ocr_number_of_pages: Option<u32>,
    pub ocr_langs: Vec<String>,
    pub ocr_scripts: Vec<String>,
}

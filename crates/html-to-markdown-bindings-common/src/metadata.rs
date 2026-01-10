//! Metadata conversion utilities for language bindings.
//!
//! Provides intermediate representations for metadata types that can
//! be easily converted to language-specific structures (PyDict, JsObject, etc.).

use html_to_markdown_rs::metadata::{
    DocumentMetadata, ExtendedMetadata, HeaderMetadata, ImageMetadata, LinkMetadata, StructuredData, TextDirection,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Intermediate representation for `DocumentMetadata`.
///
/// Uses simple serde-compatible types that can be easily converted
/// to language-specific structures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadataIntermediate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub author: Option<String>,
    pub canonical_url: Option<String>,
    pub base_href: Option<String>,
    pub language: Option<String>,
    pub text_direction: Option<String>,
    pub open_graph: BTreeMap<String, String>,
    pub twitter_card: BTreeMap<String, String>,
    pub meta_tags: BTreeMap<String, String>,
}

impl From<DocumentMetadata> for DocumentMetadataIntermediate {
    fn from(metadata: DocumentMetadata) -> Self {
        Self {
            title: metadata.title,
            description: metadata.description,
            keywords: metadata.keywords,
            author: metadata.author,
            canonical_url: metadata.canonical_url,
            base_href: metadata.base_href,
            language: metadata.language,
            text_direction: metadata.text_direction.map(|dir| match dir {
                TextDirection::LeftToRight => "ltr".to_string(),
                TextDirection::RightToLeft => "rtl".to_string(),
                TextDirection::Auto => "auto".to_string(),
            }),
            open_graph: metadata.open_graph,
            twitter_card: metadata.twitter_card,
            meta_tags: metadata.meta_tags,
        }
    }
}

/// Intermediate representation for `HeaderMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderMetadataIntermediate {
    pub level: u8,
    pub text: String,
    pub id: Option<String>,
}

impl From<HeaderMetadata> for HeaderMetadataIntermediate {
    fn from(header: HeaderMetadata) -> Self {
        Self {
            level: header.level,
            text: header.text,
            id: header.id,
        }
    }
}

/// Intermediate representation for `LinkMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMetadataIntermediate {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
    pub link_type: String,
}

impl From<LinkMetadata> for LinkMetadataIntermediate {
    fn from(link: LinkMetadata) -> Self {
        Self {
            href: link.href,
            text: link.text,
            title: link.title,
            link_type: link.link_type.to_string(),
        }
    }
}

/// Intermediate representation for `ImageMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadataIntermediate {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub image_type: String,
    pub attributes: BTreeMap<String, String>,
}

impl From<ImageMetadata> for ImageMetadataIntermediate {
    fn from(image: ImageMetadata) -> Self {
        let (width, height) = image.dimensions.unzip();
        Self {
            src: image.src,
            alt: image.alt,
            title: image.title,
            width,
            height,
            image_type: image.image_type.to_string(),
            attributes: image.attributes,
        }
    }
}

/// Intermediate representation for `StructuredData`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredDataIntermediate {
    pub data_type: String,
    pub raw_json: String,
    pub schema_type: Option<String>,
}

impl From<StructuredData> for StructuredDataIntermediate {
    fn from(data: StructuredData) -> Self {
        Self {
            data_type: data.data_type.to_string(),
            raw_json: data.raw_json,
            schema_type: data.schema_type,
        }
    }
}

/// Intermediate representation for `ExtendedMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedMetadataIntermediate {
    pub document: DocumentMetadataIntermediate,
    pub headers: Vec<HeaderMetadataIntermediate>,
    pub links: Vec<LinkMetadataIntermediate>,
    pub images: Vec<ImageMetadataIntermediate>,
    pub structured_data: Vec<StructuredDataIntermediate>,
}

impl From<ExtendedMetadata> for ExtendedMetadataIntermediate {
    fn from(metadata: ExtendedMetadata) -> Self {
        Self {
            document: metadata.document.into(),
            headers: metadata.headers.into_iter().map(Into::into).collect(),
            links: metadata.links.into_iter().map(Into::into).collect(),
            images: metadata.images.into_iter().map(Into::into).collect(),
            structured_data: metadata.structured_data.into_iter().map(Into::into).collect(),
        }
    }
}

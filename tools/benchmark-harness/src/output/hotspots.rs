//! Flamegraph hotspot extraction and analysis.

use crate::{Error, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A performance hotspot from a flamegraph.
#[derive(serde::Serialize)]
pub struct Hotspot {
    pub name: String,
    pub samples: usize,
}

/// Extract hotspots from a flamegraph file (SVG or 0x profile output).
pub fn extract_flamegraph_hotspots(path: &Path, limit: usize) -> Result<Vec<Hotspot>> {
    let data = fs::read_to_string(path).map_err(Error::Io)?;

    // Try 0x profile format first
    if let Some(hotspots) = extract_0x_hotspots(&data, limit) {
        return Ok(hotspots);
    }

    // Fall back to flamegraph SVG format
    extract_svg_hotspots(&data, limit)
}

/// Extract hotspots from SVG flamegraph format.
fn extract_svg_hotspots(data: &str, limit: usize) -> Result<Vec<Hotspot>> {
    let mut totals: HashMap<String, usize> = HashMap::new();

    for chunk in data.split("<title>").skip(1) {
        let Some(end) = chunk.find("</title>") else {
            continue;
        };
        let title = &chunk[..end];
        let (name, meta) = match title.rsplit_once(" (") {
            Some((name, meta)) => (name, meta.trim_end_matches(')')),
            None => continue,
        };

        let samples = parse_samples(meta);
        if samples == 0 {
            continue;
        }
        if should_ignore_frame(name) {
            continue;
        }

        *totals.entry(name.to_string()).or_insert(0) += samples;
    }

    let mut entries = totals
        .into_iter()
        .map(|(name, samples)| Hotspot { name, samples })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.samples.cmp(&a.samples));
    entries.truncate(limit);
    Ok(entries)
}

/// Extract hotspots from 0x profiler output format (JSON).
fn extract_0x_hotspots(data: &str, limit: usize) -> Option<Vec<Hotspot>> {
    let start = data.find("visualizer(")? + "visualizer(".len();
    let tail = &data[start..];
    let end = tail.find(");")?;
    let payload = tail[..end].trim();

    if payload.is_empty() {
        return None;
    }

    let mut totals: HashMap<String, usize> = HashMap::new();
    let mut index = 0;

    while let Some(pos) = payload[index..].find("\"name\":\"") {
        let name_start = index + pos + "\"name\":\"".len();
        let mut name = String::new();
        let mut escaped = false;
        let mut cursor = name_start;

        // Parse the name string, handling escapes
        for (offset, ch) in payload[name_start..].char_indices() {
            cursor = name_start + offset;
            if escaped {
                name.push(ch);
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                cursor += 1;
                break;
            }
            name.push(ch);
        }

        if name.is_empty() || should_ignore_frame(&name) {
            index = cursor;
            continue;
        }

        // Extract the value/sample count
        if let Some(value_pos) = payload[cursor..].find("\"value\":") {
            let mut value_index = cursor + value_pos + "\"value\":".len();

            // Skip whitespace
            while let Some(ch) = payload[value_index..].chars().next() {
                if ch.is_whitespace() {
                    value_index += ch.len_utf8();
                } else {
                    break;
                }
            }

            // Extract digits and decimal point
            let mut value_str = String::new();
            for ch in payload[value_index..].chars() {
                if ch.is_ascii_digit() || ch == '.' {
                    value_str.push(ch);
                } else {
                    break;
                }
            }

            if let Ok(value) = value_str.parse::<f64>() {
                if value > 0.0 {
                    *totals.entry(name).or_insert(0) += value as usize;
                }
            }
        }

        index = cursor;
    }

    let mut entries = totals
        .into_iter()
        .map(|(name, samples)| Hotspot { name, samples })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.samples.cmp(&a.samples));
    entries.truncate(limit);
    Some(entries)
}

/// Parse sample count from flamegraph metadata string (e.g., "1,234 samples").
fn parse_samples(meta: &str) -> usize {
    let mut digits = String::new();
    for ch in meta.chars() {
        if ch.is_ascii_digit() || ch == ',' {
            digits.push(ch);
        } else {
            break;
        }
    }
    digits.replace(',', "").parse::<usize>().unwrap_or(0)
}

/// Check if a frame name should be ignored in hotspot analysis.
fn should_ignore_frame(name: &str) -> bool {
    matches!(name, "all" | "all stacks" | "__libc_start_main" | "_start")
        || name.starts_with("benchmark-harne")
        || name.starts_with("benchmark-harness")
}

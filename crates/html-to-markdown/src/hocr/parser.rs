//! hOCR property parser
//!
//! Parses hOCR title attributes into structured properties.

use super::types::{BBox, Baseline, HocrProperties};
use crate::text::decode_html_entities_cow;

/// Parse all properties from hOCR title attribute
pub fn parse_properties(title: &str, debug: bool) -> HocrProperties {
    let mut props = HocrProperties::default();

    let title = decode_html_entities_cow(title);

    for part in title.as_ref().split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let mut tokens = part.split_whitespace();
        if let Some(key) = tokens.next() {
            match key {
                "bbox" => {
                    if let Some(bbox) = parse_bbox_coords(&mut tokens) {
                        props.bbox = Some(bbox);
                    }
                }
                "baseline" => {
                    if let Some(baseline) = parse_baseline(&mut tokens) {
                        props.baseline = Some(baseline);
                    }
                }
                "textangle" => {
                    if let Some(angle_str) = tokens.next() {
                        if let Ok(angle) = angle_str.parse::<f64>() {
                            props.textangle = Some(angle);
                        }
                    }
                }
                "poly" => {
                    props.poly = parse_poly(&mut tokens);
                }
                "x_wconf" => {
                    if let Some(conf_str) = tokens.next() {
                        if let Ok(conf) = conf_str.parse::<f64>() {
                            props.x_wconf = Some(conf);
                        }
                    }
                }
                "x_confs" => {
                    props.x_confs = parse_float_list(&mut tokens);
                }
                "nlp" => {
                    props.nlp = parse_float_list(&mut tokens);
                }
                "x_font" => {
                    if let Some(font) = parse_quoted_string(part) {
                        props.x_font = Some(font);
                    }
                }
                "x_fsize" => {
                    if let Some(size_str) = tokens.next() {
                        if let Ok(size) = size_str.parse::<u32>() {
                            props.x_fsize = Some(size);
                        }
                    }
                }
                "order" => {
                    if let Some(order_str) = tokens.next() {
                        if let Ok(order) = order_str.parse::<u32>() {
                            props.order = Some(order);
                        }
                    }
                }
                "cflow" => {
                    if let Some(flow) = parse_quoted_string(part) {
                        props.cflow = Some(flow);
                    }
                }
                "hardbreak" => {
                    if let Some(val) = tokens.next() {
                        props.hardbreak = val == "1";
                    }
                }
                "cuts" => {
                    props.cuts = parse_cuts(&mut tokens);
                }
                "x_bboxes" => {
                    props.x_bboxes = parse_bboxes_list(&mut tokens);
                }
                "image" => {
                    if let Some(img) = parse_quoted_string(part) {
                        props.image = Some(img);
                    }
                }
                "imagemd5" => {
                    if let Some(md5) = parse_quoted_string(part) {
                        props.imagemd5 = Some(md5);
                    }
                }
                "ppageno" => {
                    if let Some(page_str) = tokens.next() {
                        if let Ok(page) = page_str.parse::<u32>() {
                            props.ppageno = Some(page);
                        }
                    }
                }
                "lpageno" => {
                    let rest: Vec<&str> = tokens.collect();
                    if !rest.is_empty() {
                        let lpageno_str = rest.join(" ");
                        if let Some(quoted) = parse_quoted_string(part) {
                            props.lpageno = Some(quoted);
                        } else {
                            props.lpageno = Some(lpageno_str);
                        }
                    }
                }
                "scan_res" => {
                    let coords: Vec<&str> = tokens.collect();
                    if coords.len() >= 2 {
                        if let (Ok(x), Ok(y)) = (coords[0].parse::<u32>(), coords[1].parse::<u32>()) {
                            props.scan_res = Some((x, y));
                        }
                    }
                }
                "x_source" => {
                    let sources = parse_all_quoted_strings(part);
                    if !sources.is_empty() {
                        props.x_source = sources;
                    }
                }
                "x_scanner" => {
                    if let Some(scanner) = parse_quoted_string(part) {
                        props.x_scanner = Some(scanner);
                    }
                }
                "x_size" | "x_descenders" | "x_ascenders" => {
                    let value: Vec<&str> = tokens.collect();
                    if !value.is_empty() {
                        props.other.insert(key.to_string(), value.join(" "));
                    }
                }
                _ => {
                    if debug {
                        eprintln!("[hOCR] Unknown property: {}", key);
                    }
                    let value: Vec<&str> = tokens.collect();
                    if !value.is_empty() {
                        props.other.insert(key.to_string(), value.join(" "));
                    }
                }
            }
        }
    }

    props
}

fn parse_bbox_coords<'a, I>(tokens: &mut I) -> Option<BBox>
where
    I: Iterator<Item = &'a str>,
{
    let x1 = tokens.next()?.parse::<u32>().ok()?;
    let y1 = tokens.next()?.parse::<u32>().ok()?;
    let x2 = tokens.next()?.parse::<u32>().ok()?;
    let y2 = tokens.next()?.parse::<u32>().ok()?;
    Some(BBox { x1, y1, x2, y2 })
}

fn parse_baseline<'a, I>(tokens: &mut I) -> Option<Baseline>
where
    I: Iterator<Item = &'a str>,
{
    let slope = tokens.next()?.parse::<f64>().ok()?;
    let constant = tokens.next()?.parse::<i32>().ok()?;
    Some(Baseline { slope, constant })
}

fn parse_poly<'a, I>(tokens: &mut I) -> Option<Vec<(i32, i32)>>
where
    I: Iterator<Item = &'a str>,
{
    let coords: Vec<&str> = tokens.collect();
    if coords.len() >= 4 && coords.len() % 2 == 0 {
        let mut points = Vec::new();
        for chunk in coords.chunks(2) {
            if let (Ok(x), Ok(y)) = (chunk[0].parse::<i32>(), chunk[1].parse::<i32>()) {
                points.push((x, y));
            } else {
                return None;
            }
        }
        return Some(points);
    }
    None
}

fn parse_float_list<'a, I>(tokens: &mut I) -> Vec<f64>
where
    I: Iterator<Item = &'a str>,
{
    tokens.filter_map(|s| s.parse::<f64>().ok()).collect()
}

fn parse_cuts<'a, I>(tokens: &mut I) -> Vec<Vec<u32>>
where
    I: Iterator<Item = &'a str>,
{
    let mut cuts = Vec::new();
    for token in tokens {
        if token.contains(',') {
            let parts: Vec<u32> = token.split(',').filter_map(|s| s.parse::<u32>().ok()).collect();
            cuts.push(parts);
        } else if let Ok(val) = token.parse::<u32>() {
            cuts.push(vec![val]);
        }
    }
    cuts
}

fn parse_bboxes_list<'a, I>(tokens: &mut I) -> Vec<BBox>
where
    I: Iterator<Item = &'a str>,
{
    let coords: Vec<u32> = tokens.filter_map(|s| s.parse::<u32>().ok()).collect();

    coords
        .chunks(4)
        .filter_map(|chunk| {
            if chunk.len() == 4 {
                Some(BBox {
                    x1: chunk[0],
                    y1: chunk[1],
                    x2: chunk[2],
                    y2: chunk[3],
                })
            } else {
                None
            }
        })
        .collect()
}

fn parse_quoted_string(s: &str) -> Option<String> {
    if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            return Some(s[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

fn parse_all_quoted_strings(s: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut remaining = s;

    while let Some(start) = remaining.find('"') {
        if let Some(end) = remaining[start + 1..].find('"') {
            results.push(remaining[start + 1..start + 1 + end].to_string());
            remaining = &remaining[start + 1 + end + 1..];
        } else {
            break;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bbox() {
        let props = parse_properties("bbox 100 50 200 150", false);
        assert_eq!(
            props.bbox,
            Some(BBox {
                x1: 100,
                y1: 50,
                x2: 200,
                y2: 150
            })
        );
    }

    #[test]
    fn test_parse_baseline() {
        let props = parse_properties("baseline 0.015 -18", false);
        assert_eq!(
            props.baseline,
            Some(Baseline {
                slope: 0.015,
                constant: -18
            })
        );
    }

    #[test]
    fn test_parse_multiple_properties() {
        let props = parse_properties("bbox 0 0 100 50; x_wconf 95.5; textangle 7.2", false);
        assert_eq!(
            props.bbox,
            Some(BBox {
                x1: 0,
                y1: 0,
                x2: 100,
                y2: 50
            })
        );
        assert_eq!(props.x_wconf, Some(95.5));
        assert_eq!(props.textangle, Some(7.2));
    }

    #[test]
    fn test_parse_quoted_strings() {
        let props = parse_properties("x_font \"Comic Sans MS\"; x_fsize 12", false);
        assert_eq!(props.x_font, Some("Comic Sans MS".to_string()));
        assert_eq!(props.x_fsize, Some(12));
    }

    #[test]
    fn test_parse_poly() {
        let props = parse_properties("poly 0 0 0 10 10 10 10 0", false);
        assert_eq!(props.poly, Some(vec![(0, 0), (0, 10), (10, 10), (10, 0)]));
    }

    #[test]
    fn test_parse_x_confs() {
        let props = parse_properties("x_confs 37.3 51.23 100", false);
        assert_eq!(props.x_confs, vec![37.3, 51.23, 100.0]);
    }
}

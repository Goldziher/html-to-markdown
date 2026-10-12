#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! A code block deep in block quotes must not cost the square of the depth.
//!
//! Each block quote read every line of its content twice to learn which lines are indented code.
//! A code block that is `n` quotes deep was read `2n` times, each time through more quote markers.
//! With a fenced style each block quote read the markers of every line to find the fences.
//! The library converts untrusted HTML, so such growth is a denial-of-service vector.
//!
//! The bound is on the growth between two depths, not on a time: twice the depth may take at
//! most three times as long. One read for the whole document doubles the time, a read at each
//! level quadruples it.

use std::time::{Duration, Instant};

use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, TierStrategy, convert};

/// ~keep Same instrument as `list_item_complexity.rs`: 3.0 separates linear growth from quadratic
/// ~keep growth with room for noise.
const MAX_DOUBLING_RATIO: f64 = 3.0;

/// ~keep A failure must reproduce on every independent attempt; one that does not is noise.
const MAX_MEASUREMENT_ATTEMPTS: usize = 3;

/// ~keep The fastest of this many runs is one sample.
const REPEATS_PER_SAMPLE: usize = 3;

/// ~keep The converter stops at a depth limit past these, and writes no code block there.
const DEPTHS: [usize; 4] = [5, 10, 20, 40];

/// ~keep Enough lines that the reads of the code outweigh the fixed cost of a conversion.
const CODE_LINES: usize = 400;

/// A page that makes every block quote ask which of its lines are code.
#[derive(Clone, Copy, Debug)]
enum Shape {
    /// Only the code block.
    CodeBlock,
    /// The last line of the code ends in spaces: each quote trims its content, and keeps the
    /// white space at the end of a line of code.
    LastLineEndsInSpaces,
    /// Each quote also holds a small code block with a line of spaces: a quote writes a blank
    /// line as its marker alone, and keeps a line of code.
    LineOfSpacesInEachQuote,
}

const SHAPES: [Shape; 3] = [
    Shape::CodeBlock,
    Shape::LastLineEndsInSpaces,
    Shape::LineOfSpacesInEachQuote,
];

/// A code block that is `depth` block quotes deep.
fn page(shape: Shape, depth: usize) -> String {
    let mut html = "<blockquote>".repeat(depth);
    html.push_str("<pre>");
    html.push_str(&"value = 1\n".repeat(CODE_LINES));
    if matches!(shape, Shape::LastLineEndsInSpaces) {
        html.push_str("end = 1  ");
    }
    html.push_str("</pre>");
    for _ in 0..depth {
        if matches!(shape, Shape::LineOfSpacesInEachQuote) {
            html.push_str("<pre>a\n  \nb</pre>");
        }
        html.push_str("</blockquote>");
    }
    html
}

/// The white space that `style` writes before each line of code, after the marker of a quote.
const fn code_indent(style: CodeBlockStyle) -> &'static str {
    match style {
        CodeBlockStyle::Indented => "    ",
        CodeBlockStyle::Backticks | CodeBlockStyle::Tildes => "",
    }
}

/// ~keep A page that is not written as its shape says measures nothing.
fn assert_shape(shape: Shape, depth: usize, style: CodeBlockStyle, output: &str) {
    let indent = code_indent(style);
    match shape {
        Shape::CodeBlock => {}
        Shape::LastLineEndsInSpaces => assert!(
            output.contains(&format!(" {indent}end = 1  \n")),
            "the last line of the code keeps its spaces"
        ),
        Shape::LineOfSpacesInEachQuote => assert_eq!(
            output
                .lines()
                .filter(|line| line.ends_with(&format!("> {indent}  ")))
                .count(),
            depth,
            "each quote keeps its line of spaces"
        ),
    }
}

fn fastest_convert(shape: Shape, depth: usize, style: CodeBlockStyle, tier_strategy: TierStrategy) -> Duration {
    let html = &page(shape, depth);
    let options = ConversionOptions {
        extract_metadata: false,
        code_block_style: style,
        tier_strategy,
        ..ConversionOptions::default()
    };
    let code_line = format!(" {}value = 1", code_indent(style));
    (0..REPEATS_PER_SAMPLE)
        .map(|_| {
            let start = Instant::now();
            let output = convert(std::hint::black_box(html), Some(options.clone()))
                .expect("conversion must succeed")
                .content
                .unwrap_or_default();
            let elapsed = start.elapsed();
            // ~keep A conversion that stops at the depth limit is fast and proves nothing.
            assert_eq!(
                output.lines().filter(|line| line.contains(&code_line)).count(),
                CODE_LINES,
                "every line of the code is a line of code in the output"
            );
            assert_shape(shape, depth, style, &output);
            elapsed
        })
        .min()
        .expect("at least one run")
}

/// The doubling ratios of one converter for one shape, or a description of why they look quadratic.
fn measure(shape: Shape, style: CodeBlockStyle, tier_strategy: TierStrategy) -> Result<(), String> {
    let seconds: Vec<f64> = DEPTHS
        .into_iter()
        .map(|depth| {
            fastest_convert(shape, depth, style, tier_strategy)
                .as_secs_f64()
                .max(1e-6)
        })
        .collect();
    let ratios: Vec<f64> = seconds.windows(2).map(|pair| pair[1] / pair[0]).collect();
    if ratios.iter().all(|ratio| *ratio < MAX_DOUBLING_RATIO) {
        return Ok(());
    }
    Err(format!(
        "{shape:?}, {style:?}, {tier_strategy:?}: depths {DEPTHS:?} took {seconds:.4?} seconds, ratios {ratios:.2?}; \
         expected under {MAX_DOUBLING_RATIO}x for each doubling"
    ))
}

/// ~keep One measurement at a time: a second one on another thread changes both.
static ONE_MEASUREMENT: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn an_indented_code_block_in_nested_block_quotes_scales_linearly_in_both_tiers() {
    assert_linear_growth(CodeBlockStyle::Indented);
}

#[test]
fn a_fenced_code_block_in_nested_block_quotes_scales_linearly_in_both_tiers() {
    assert_linear_growth(CodeBlockStyle::Backticks);
}

/// ~keep Enough quotes that the reads of their lines outweigh the fixed cost of a conversion.
const SIBLINGS: [usize; 3] = [500, 1000, 2000];

/// One block quote that holds `count` block quotes, each with a short code block, and after each
/// of them a code block of its own with a line of spaces.
///
/// ~keep The outer quote asks about each line of spaces, so it reads its lines once for each
/// ~keep inner quote: the lines between two lines of its inner quotes. A read that starts at the
/// ~keep first line, or that goes on to the last one, costs the square of the count.
fn fastest_convert_of_siblings(count: usize, style: CodeBlockStyle, tier_strategy: TierStrategy) -> Duration {
    let html = &format!(
        "<blockquote>{}</blockquote>",
        "<blockquote><pre>value = 1</pre></blockquote><pre>a\n  \nb</pre>".repeat(count)
    );
    let options = ConversionOptions {
        extract_metadata: false,
        code_block_style: style,
        tier_strategy,
        ..ConversionOptions::default()
    };
    let code_line = format!("> > {}value = 1", code_indent(style));
    let line_of_spaces = format!("> {}  ", code_indent(style));
    (0..REPEATS_PER_SAMPLE)
        .map(|_| {
            let start = Instant::now();
            let output = convert(std::hint::black_box(html), Some(options.clone()))
                .expect("conversion must succeed")
                .content
                .unwrap_or_default();
            let elapsed = start.elapsed();
            assert_eq!(
                output.lines().filter(|line| *line == code_line).count(),
                count,
                "every inner quote holds its line of code"
            );
            // ~keep Without the lines of spaces the outer quote asks nothing and reads nothing.
            assert_eq!(
                output.lines().filter(|line| *line == line_of_spaces).count(),
                count,
                "the outer quote keeps each line of spaces"
            );
            elapsed
        })
        .min()
        .expect("at least one run")
}

#[test]
fn code_blocks_in_sibling_block_quotes_scale_linearly_in_both_tiers() {
    let _alone = ONE_MEASUREMENT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut slow = Vec::new();
    for style in [CodeBlockStyle::Indented, CodeBlockStyle::Backticks] {
        for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
            let mut failures = Vec::with_capacity(MAX_MEASUREMENT_ATTEMPTS);
            for attempt in 1..=MAX_MEASUREMENT_ATTEMPTS {
                let seconds: Vec<f64> = SIBLINGS
                    .into_iter()
                    .map(|count| {
                        fastest_convert_of_siblings(count, style, tier_strategy)
                            .as_secs_f64()
                            .max(1e-6)
                    })
                    .collect();
                let ratios: Vec<f64> = seconds.windows(2).map(|pair| pair[1] / pair[0]).collect();
                if ratios.iter().all(|ratio| *ratio < MAX_DOUBLING_RATIO) {
                    break;
                }
                failures.push(format!(
                    "attempt {attempt}: {style:?}, {tier_strategy:?}: {SIBLINGS:?} quotes took {seconds:.4?} seconds, \
                     ratios {ratios:.2?}; expected under {MAX_DOUBLING_RATIO}x for each doubling"
                ));
            }
            if failures.len() == MAX_MEASUREMENT_ATTEMPTS {
                slow.extend(failures);
            }
        }
    }
    assert!(
        slow.is_empty(),
        "the time grew faster than the count of quotes on every attempt:\n{}",
        slow.join("\n")
    );
}

fn assert_linear_growth(style: CodeBlockStyle) {
    let _alone = ONE_MEASUREMENT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // ~keep Every shape is measured on both converters before the test fails, so one red run
    // ~keep names each slow pair.
    let mut slow = Vec::new();
    for shape in SHAPES {
        for tier_strategy in [TierStrategy::Tier1, TierStrategy::Tier2] {
            let mut failures = Vec::with_capacity(MAX_MEASUREMENT_ATTEMPTS);
            for attempt in 1..=MAX_MEASUREMENT_ATTEMPTS {
                match measure(shape, style, tier_strategy) {
                    Ok(()) => break,
                    Err(reason) => failures.push(format!("attempt {attempt}: {reason}")),
                }
            }
            if failures.len() == MAX_MEASUREMENT_ATTEMPTS {
                slow.extend(failures);
            }
        }
    }
    assert!(
        slow.is_empty(),
        "the time grew faster than the depth on every attempt:\n{}",
        slow.join("\n")
    );
}

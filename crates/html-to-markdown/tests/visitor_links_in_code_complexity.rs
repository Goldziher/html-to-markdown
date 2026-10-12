#![allow(missing_docs)]
#![cfg(feature = "visitor")]

//! A link in code with a visitor must not start a walk in each walk that it starts.
//!
//! A link in code whose text is empty there asks what its content writes outside code: the
//! converter walks that content once more, and no visitor of the caller sees that walk. A link in
//! `<code>` inside that content asked the same again, so each level of links doubled the walks:
//! 12 levels made 4,095 walks. The library converts untrusted HTML, so such growth is a
//! denial-of-service vector.
//!
//! A link in a link in code still walks its own content once more, so the time for links nested
//! in links grows faster than the square of the nesting depth, up to the depth limit. That cost
//! is accepted and pinned here: it must not grow faster than the cube, and a page of many such
//! chains must stay linear in the count of chains.
//!
//! The bound is on the growth between two sizes. The nested shape has a time limit too, with a
//! wide margin, so that the accepted cost cannot grow unseen.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
use html_to_markdown_rs::{ConversionOptions, convert};

/// ~keep Same instrument as `list_item_complexity.rs`: 3.0 separates linear growth from quadratic
/// ~keep growth with room for noise.
const MAX_DOUBLING_RATIO: f64 = 3.0;

/// ~keep Each link in a link walks its own content once more, and each link in that walk reads
/// ~keep the text below it, so twice the depth is eight times the work at most: 5 to 6 times
/// ~keep from depth 20 to depth 40. A walk in each walk is 256 times the work from depth 8 to
/// ~keep depth 16.
const MAX_NESTED_DOUBLING_RATIO: f64 = 8.0;

/// ~keep 40 links, each directly in the one before, take 2.4 ms in a release build and 45 ms in
/// ~keep a debug build on a loaded host; before the walk they took 0.3 ms. The limit is 20 times
/// ~keep the debug time, so a slow host passes and a walk in each walk does not.
const MAX_SECONDS_AT_DEPTH_40: f64 = 1.0;

/// ~keep A failure must reproduce on every independent attempt; one that does not is noise.
const MAX_MEASUREMENT_ATTEMPTS: usize = 3;

/// ~keep The fastest of this many runs is one sample.
const REPEATS_PER_SAMPLE: usize = 5;

#[derive(Debug, Default)]
struct CountsCalls {
    calls: usize,
}

impl HtmlVisitor for CountsCalls {
    fn visit_link(&mut self, _ctx: &NodeContext<'_>, _href: &str, _text: &str, _title: Option<&str>) -> VisitResult {
        self.calls += 1;
        VisitResult::Continue
    }

    fn visit_image(&mut self, _ctx: &NodeContext<'_>, _src: &str, _alt: &str, _title: Option<&str>) -> VisitResult {
        self.calls += 1;
        VisitResult::Continue
    }
}

/// A page of links in code whose text is empty there.
#[derive(Clone, Copy, Debug)]
enum Shape {
    /// A link in `<code>` in a link in `<code>`, in one `<pre>`.
    LinksInLinks,
    /// A link directly in a link, in one `<pre>`, around one image.
    LinksDirectlyInLinks,
    /// Chains of 10 links, each directly in the one before, side by side in one `<pre>`.
    ChainsOfLinksInLinks,
    /// Links side by side in one `<pre>`, each around one image.
    SiblingImageLinks,
    /// One such link in `<code>` in each item of a list in a list.
    LinkInNestedListItems,
}

impl Shape {
    const fn sizes(self) -> [usize; 2] {
        match self {
            Self::LinksInLinks => [8, 16],
            Self::LinksDirectlyInLinks => [20, 40],
            Self::ChainsOfLinksInLinks => [500, 1000],
            Self::SiblingImageLinks => [2000, 4000],
            Self::LinkInNestedListItems => [10, 20],
        }
    }

    const fn max_doubling_ratio(self) -> f64 {
        match self {
            Self::LinksInLinks | Self::LinksDirectlyInLinks => MAX_NESTED_DOUBLING_RATIO,
            Self::ChainsOfLinksInLinks | Self::SiblingImageLinks | Self::LinkInNestedListItems => MAX_DOUBLING_RATIO,
        }
    }

    /// The most seconds that the larger size may take, for a shape whose cost is accepted.
    const fn max_seconds(self) -> Option<f64> {
        match self {
            Self::LinksDirectlyInLinks => Some(MAX_SECONDS_AT_DEPTH_40),
            _ => None,
        }
    }

    /// `depth` links into their own page, each directly in the one before, around one image.
    fn chain(depth: usize) -> String {
        format!(
            "{}<img src=\"i.png\">{}",
            "<a href=\"#x\">".repeat(depth),
            "</a>".repeat(depth)
        )
    }

    fn page(self, size: usize) -> String {
        const LINK: &str = "<a href=\"#x\"><img src=\"i.png\"></a>";
        match self {
            Self::LinksInLinks => format!(
                "<pre>{}<img src=\"i.png\">{}</pre>",
                "<a href=\"#x\"><code>".repeat(size),
                "</code></a>".repeat(size)
            ),
            Self::LinksDirectlyInLinks => format!("<pre>{}</pre>", Self::chain(size)),
            Self::ChainsOfLinksInLinks => format!("<pre>{}</pre>", format!("{}\n", Self::chain(10)).repeat(size)),
            Self::SiblingImageLinks => format!("<pre>{}</pre>", format!("{LINK}\n").repeat(size)),
            Self::LinkInNestedListItems => format!(
                "{}{}",
                format!("<ul><li><code>{LINK}</code>").repeat(size),
                "</li></ul>".repeat(size)
            ),
        }
    }
}

fn fastest_convert(shape: Shape, size: usize) -> Duration {
    let html = &shape.page(size);
    (0..REPEATS_PER_SAMPLE)
        .map(|_| {
            let visitor = Arc::new(Mutex::new(CountsCalls::default()));
            let handle: VisitorHandle = visitor.clone();
            let options = ConversionOptions {
                extract_metadata: false,
                visitor: Some(handle),
                ..ConversionOptions::default()
            };
            let start = Instant::now();
            let result = convert(std::hint::black_box(html), Some(options));
            let elapsed = start.elapsed();
            assert!(result.is_ok(), "{shape:?} of size {size} converts");
            // ~keep A conversion that stops at the depth limit is fast and proves nothing.
            assert!(
                visitor.lock().expect("the visitor lock").calls > 0,
                "{shape:?} of size {size}: the visitor is called"
            );
            elapsed
        })
        .min()
        .expect("at least one run")
}

/// The doubling ratio of one shape, or a description of why it is too large.
fn measure(shape: Shape) -> Result<(), String> {
    let sizes = shape.sizes();
    let seconds = sizes.map(|size| fastest_convert(shape, size).as_secs_f64().max(1e-6));
    let ratio = seconds[1] / seconds[0];
    let limit = shape.max_doubling_ratio();
    let max_seconds = shape.max_seconds().unwrap_or(f64::INFINITY);
    if ratio < limit && seconds[1] < max_seconds {
        return Ok(());
    }
    Err(format!(
        "{shape:?}: sizes {sizes:?} took {seconds:.4?} seconds, ratio {ratio:.2}; expected under {limit}x and under {max_seconds} seconds"
    ))
}

fn assert_bounded_growth(shape: Shape) {
    let mut failures = Vec::with_capacity(MAX_MEASUREMENT_ATTEMPTS);
    for attempt in 1..=MAX_MEASUREMENT_ATTEMPTS {
        match measure(shape) {
            Ok(()) => return,
            Err(reason) => failures.push(format!("attempt {attempt}: {reason}")),
        }
    }
    panic!("the time grew too fast on every attempt:\n{}", failures.join("\n"));
}

#[test]
fn links_in_links_in_code_do_not_double_the_walks_at_each_level() {
    assert_bounded_growth(Shape::LinksInLinks);
}

#[test]
fn links_directly_in_links_in_code_grow_with_the_cube_of_the_depth_at_most() {
    assert_bounded_growth(Shape::LinksDirectlyInLinks);
}

#[test]
fn chains_of_links_in_links_in_code_scale_linearly_in_their_count() {
    assert_bounded_growth(Shape::ChainsOfLinksInLinks);
}

#[test]
fn sibling_image_links_in_code_scale_linearly() {
    assert_bounded_growth(Shape::SiblingImageLinks);
}

#[test]
fn a_link_in_code_in_nested_list_items_scales_linearly() {
    assert_bounded_growth(Shape::LinkInNestedListItems);
}

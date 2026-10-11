// ~keep The inner attribute below is a crate-level Rust attribute, not a shell shebang.
#![allow(missing_docs)]
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)] // ~keep: tests print by design

//! Robustness oracle: conversion must not panic, hang, or blow up in size.
//!
//! This library's job is parsing untrusted input, and its failure history is dominated by
//! that: issues #216 and #217 were reported panics ("byte index N is out of bounds") caused
//! by a buffer index going stale across handlers, and `deep_nesting_overflow.rs` exists
//! because a depth-guard reset let native recursion overflow the stack.
//!
//! No ground truth is needed to catch that class. The oracle is behavioural:
//!
//! - conversion returns rather than panicking (an `Err` is a fine outcome; a panic is not),
//! - it terminates inside a wall-clock budget,
//! - output stays within a sane multiple of the input, so a quadratic or runaway expansion
//!   shows up as a failure rather than as an out-of-memory kill in CI.
//!
//! Inputs come from the in-repo fixture corpus plus a deterministic generator. The generator
//! is seeded and written here rather than pulled from a property-testing crate so that a
//! failure reproduces exactly from the seed printed in the assertion message, instead of
//! depending on a shrinking strategy that varies between runs.
//!
//! A stack overflow aborts the process and cannot be caught here, by design of the platform;
//! that case is covered separately by `deep_nesting_overflow.rs`.

use html_to_markdown_rs::ConversionError;
use html_to_markdown_rs::options::{ConversionOptions, NewlineStyle};
use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// Wall-clock ceiling for a single conversion.
///
/// ~keep Deliberately generous. This is a hang detector, not a benchmark: the largest
/// ~keep fixture converts in milliseconds, so a case that needs seconds has already stopped
/// ~keep being linear in its input. `tools/benchmark-harness` is where throughput is tracked.
const CONVERSION_BUDGET: Duration = Duration::from_secs(20);

/// Ceiling on output size as a multiple of input size, plus a fixed floor for tiny inputs.
///
/// ~keep Markdown is normally smaller than the HTML it came from. Growth is legitimate for
/// ~keep pathological input (entity expansion, deeply nested emphasis re-emitting
/// ~keep delimiters), so this only has to be tight enough to catch runaway expansion.
const MAX_GROWTH_FACTOR: usize = 64;
const MAX_GROWTH_FLOOR: usize = 64 * 1024;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/benchmark-harness/fixtures")
}

/// Additional real-world HTML from the `test_documents` corpus.
///
/// ~keep Optional on purpose, and kept as a filtered `is_dir` check rather than an
/// ~keep unconditional path (even though `test_documents` is tracked inside this repo, two
/// ~keep levels up from `CARGO_MANIFEST_DIR`, same as `fixture_root`'s
/// ~keep `tools/benchmark-harness/fixtures`): a checkout that is missing this directory for
/// ~keep any reason must still run this test at full strength over the in-repo fixtures
/// ~keep instead of failing or, worse, silently covering nothing. The sweep asserts a floor on
/// ~keep the number of documents it keeps, and the floor counts this folder only when it is there.
/// ~keep
/// ~keep Was `../../../test_documents/html` (three levels up, landing outside the repo)
/// ~keep until this was found and fixed. In CI, and in any clean checkout, nothing exists at
/// ~keep that path, so the `is_dir` filter above silently dropped it and this corpus
/// ~keep contributed zero fixtures with no test failure to signal it. It went unnoticed
/// ~keep because a developer machine with the polyrepo checked out DOES have a
/// ~keep `test_documents` directory one level above this repository, so the path resolved
/// ~keep locally -- to the wrong corpus -- while covering nothing wherever it mattered.
fn optional_extra_roots() -> Vec<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    vec![manifest.join("../../test_documents/html")]
        .into_iter()
        .filter(|p| p.is_dir())
        .collect()
}

/// The number of distinct documents that the fixture folder holds.
///
/// ~keep Source: `git ls-files tools/benchmark-harness/fixtures test_documents/html` with the
/// ~keep SHA-256 of each `.html` file. The fixture folder holds 29 files, all distinct.
/// ~keep `test_documents/html` holds 70 files; 16 are copies of a fixture, so it adds 54. If a
/// ~keep change removes a document, lower the number in that change. A new document needs no
/// ~keep edit, because the sweep checks a floor.
const FIXTURE_DOCUMENTS: usize = 29;

/// The number of distinct documents that `test_documents/html` adds to the fixture folder.
const EXTRA_DOCUMENTS: usize = 54;

fn collect_html(dir: &PathBuf, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_html(&path, out);
        } else if path.extension().is_some_and(|e| e == "html")
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            out.push((path.display().to_string(), text));
        }
    }
}

/// Run `body` on a worker thread with a wall-clock ceiling, reporting which input it was
/// on if it never finishes.
///
/// ~keep One thread for the whole sweep rather than one per conversion: spawning ~6000
/// ~keep threads cost more than the conversions themselves. Panic attribution does not need a
/// ~keep thread (`catch_unwind` is per input, inline), and hang attribution is preserved by
/// ~keep having the worker publish the label it is currently on, so a timeout still names the
/// ~keep exact input instead of only the test.
fn with_hang_guard(budget: Duration, body: impl FnOnce(&Mutex<String>) + Send + 'static) {
    let current: Arc<Mutex<String>> = Arc::new(Mutex::new("<none>".to_owned()));
    let worker_view = Arc::clone(&current);
    let (tx, rx) = mpsc::channel();
    let started = Instant::now();
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            body(&worker_view);
            let _ = tx.send(());
        })
        .expect("spawn conversion thread");

    let finished = rx.recv_timeout(budget);
    println!("sweep took {:?} of its {budget:?} budget", started.elapsed());
    if finished.is_err() {
        let stuck = current.lock().map_or_else(|e| e.into_inner().clone(), |g| g.clone());
        // ~keep The worker is left running on purpose: it is wedged by definition, and
        // ~keep detaching it lets the failure be reported instead of deadlocking the suite.
        panic!("conversion did not finish within {budget:?}; last input started: {stuck}");
    }
}

/// Convert, treating only a panic as failure.
///
/// ~keep A conversion `Err` is a fine outcome -- refusing malformed input is correct
/// ~keep behaviour, crashing on it is not.
/// ~keep `convert` catches a panic of the DOM walk and returns it as `ConversionError::Panic`,
/// ~keep so that variant is the crash and not a refusal. While this function accepted every
/// ~keep `Err`, it could not see the panic of issue #749.
fn convert_guarded(html: &str, options: ConversionOptions) -> Result<usize, String> {
    match catch_unwind(AssertUnwindSafe(|| html_to_markdown_rs::convert(html, Some(options)))) {
        Ok(Ok(result)) => Ok(result.content.unwrap_or_default().len()),
        Ok(Err(ConversionError::Panic(message))) => Err(format!("panicked: {message}")),
        Ok(Err(_)) => Ok(0),
        Err(_) => Err("panicked".to_owned()),
    }
}

fn option_matrix() -> Vec<(&'static str, ConversionOptions)> {
    vec![
        ("default", ConversionOptions::default()),
        (
            "backslash",
            ConversionOptions {
                newline_style: NewlineStyle::Backslash,
                ..Default::default()
            },
        ),
    ]
}

/// The document structure records text during the same walk, with its own indexes into it.
///
/// ~keep A sweep of its own and not a third entry of `option_matrix`: each sweep has one
/// ~keep wall-clock budget for all its conversions, and the fixture sweep already uses most of
/// ~keep it on a loaded runner. In the fixture test it runs after the first sweep on the same
/// ~keep thread, so it takes no core from that sweep. In the two other tests it overlaps the
/// ~keep fixture sweep for a few seconds, and those sweeps are the cheap ones.
fn structure_matrix() -> Vec<(&'static str, ConversionOptions)> {
    vec![(
        "document structure",
        ConversionOptions {
            include_document_structure: true,
            ..Default::default()
        },
    )]
}

type OptionMatrix = fn() -> Vec<(&'static str, ConversionOptions)>;

fn assert_survives(label: &str, html: &str, options: ConversionOptions, option_label: &str, current: &Mutex<String>) {
    if let Ok(mut slot) = current.lock() {
        slot.clear();
        slot.push_str(label);
        slot.push_str(" [");
        slot.push_str(option_label);
        slot.push(']');
    }
    match convert_guarded(html, options) {
        Ok(out_len) => {
            let ceiling = html.len().saturating_mul(MAX_GROWTH_FACTOR).max(MAX_GROWTH_FLOOR);
            assert!(
                out_len <= ceiling,
                "{label} [{option_label}]: output grew to {out_len} bytes from {} bytes of input \
                 (ceiling {ceiling})",
                html.len()
            );
        }
        Err(reason) => panic!("{label} [{option_label}]: conversion {reason}"),
    }
}

fn sweep_fixtures(matrix: OptionMatrix) {
    let mut corpus = Vec::new();
    collect_html(&fixture_root(), &mut corpus);
    let root = fixture_root();
    let required = corpus.len();
    let extra_roots = optional_extra_roots();
    for extra in &extra_roots {
        collect_html(extra, &mut corpus);
    }
    // ~keep `test_documents/html` holds byte for byte copies of 16 of the in-repo fixtures, among
    // ~keep them the five largest pages. A copy converts to the same result, so the second
    // ~keep conversion proves nothing and costs the time of the first. Dropping it cut the corpus
    // ~keep from 12.6 MB to 7.1 MB, and each sweep of it by the same share, which is what keeps a
    // ~keep sweep with the document structure on inside its budget beside the sweep without it.
    let collected = corpus.len();
    let mut seen = HashSet::new();
    corpus.retain(|(_, html)| seen.insert(html.clone()));
    println!(
        "corpus: {required} in-repo fixture(s) + {} from optional sibling corpora, {} distinct documents",
        collected - required,
        corpus.len()
    );

    // ~keep A corpus that silently resolves to nothing, or to a few documents, is how this kind
    // ~keep of test rots into a no-op that passes forever. Fail loudly instead. The floor counts
    // ~keep `test_documents/html` only when that folder is there, because the folder is optional.
    let floor = FIXTURE_DOCUMENTS + if extra_roots.is_empty() { 0 } else { EXTRA_DOCUMENTS };
    assert!(
        corpus.len() >= floor,
        "the sweep keeps {} distinct document(s), fewer than the {floor} that the corpus holds \
         ({FIXTURE_DOCUMENTS} under {}, {} more under test_documents/html) -- the corpus path is \
         wrong, or the sweep lost documents",
        corpus.len(),
        root.display(),
        floor - FIXTURE_DOCUMENTS
    );

    with_hang_guard(CONVERSION_BUDGET, move |current| {
        for (path, html) in &corpus {
            for (option_label, options) in matrix() {
                assert_survives(path, html, options, option_label, current);
            }
        }
    });
}

#[test]
fn should_survive_every_fixture_in_the_corpus() {
    sweep_fixtures(option_matrix);
    sweep_fixtures(structure_matrix);
}

/// Deterministic 64-bit PRNG (`SplitMix64`), written here so a failing seed reproduces
/// exactly without depending on a property-testing crate's shrinking behaviour.
struct Rng(u64);

impl Rng {
    const fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    const fn below(&mut self, n: usize) -> usize {
        // ~keep The modulus is `n`, so the result is always < n and fits a usize on every
        // ~keep target regardless of pointer width; the cast cannot truncate.
        #[expect(clippy::cast_possible_truncation, reason = "value is reduced mod n first")]
        {
            (self.next_u64() % n as u64) as usize
        }
    }
}

/// Fragments chosen for the shapes that have actually broken this crate: unclosed and
/// mis-nested tags, fresh-buffer handlers, table structure, code contexts, `<br>` runs,
/// entities, and multibyte text that can put a stale byte index mid-character.
const FRAGMENTS: &[&str] = &[
    "<p>",
    "</p>",
    "<div>",
    "</div>",
    "<span>",
    "</span>",
    "<br>",
    "<br/>",
    "<table>",
    "<tr>",
    "<td>",
    "</td>",
    "</tr>",
    "</table>",
    "<ul>",
    "<li>",
    "</li>",
    "</ul>",
    "<blockquote>",
    "</blockquote>",
    "<pre>",
    "<code>",
    "</code>",
    "</pre>",
    "<strong>",
    "</strong>",
    "<em>",
    "</em>",
    "<h1>",
    "</h1>",
    "<figure>",
    "<figcaption>",
    "</figcaption>",
    "</figure>",
    "<details>",
    "<summary>",
    "</summary>",
    "</details>",
    "<dl>",
    "<dt>",
    "</dt>",
    "<dd>",
    "</dd>",
    "</dl>",
    "<a href=\"x\">",
    "</a>",
    "<img src=\"i.png\" alt=\"a\">",
    "<!-- c -->",
    "<![CDATA[x]]>",
    // ~keep Bogus-comment shapes. The pre-pass that removes these runs on every document
    // ~keep and must step over real comments and CDATA without eating their terminators,
    // ~keep so the interesting inputs interleave both kinds.
    "<?php echo 1; ?>",
    "<?",
    "<!bogus>",
    "</3>",
    "<![if !vml]>",
    "<![endif]>",
    "<!--[if gte mso 9]>",
    "<![endif]-->",
    "<!doctype html>",
    "&amp;",
    "&#x1F600;",
    "&nbsp;",
    "&lt;",
    "\u{1F600}",
    "\u{65E5}\u{672C}\u{8A9E}",
    "text",
    "  ",
    "\n",
    "\t",
    "\\",
    "`",
    "*",
    "_",
    "|",
    "#",
    ">",
    "<script>x</script>",
    "<style>y</style>",
    "<hr>",
    "<custom-el>",
    "</custom-el>",
];

fn generate(seed: u64, max_fragments: usize) -> String {
    let mut rng = Rng(seed);
    let count = 1 + rng.below(max_fragments);
    let mut html = String::new();
    for _ in 0..count {
        html.push_str(FRAGMENTS[rng.below(FRAGMENTS.len())]);
    }
    html
}

fn sweep_generated(matrix: OptionMatrix) {
    // ~keep Seeds are the reproducer: a failure names the exact seed, and `generate(seed, n)`
    // ~keep rebuilds that input byte for byte.
    const CASES: u64 = 3_000;
    const MAX_FRAGMENTS: usize = 60;

    with_hang_guard(CONVERSION_BUDGET, move |current| {
        for seed in 0..CASES {
            let html = generate(seed, MAX_FRAGMENTS);
            for (option_label, options) in matrix() {
                assert_survives(&format!("seed {seed}"), &html, options, option_label, current);
            }
        }
    });
}

#[test]
fn should_survive_generated_adversarial_markup() {
    sweep_generated(option_matrix);
    sweep_generated(structure_matrix);
}

fn sweep_pathological(matrix: OptionMatrix) {
    // ~keep Named shapes rather than random ones, for the degenerate inputs a fragment
    // ~keep shuffler is unlikely to build but a hostile document trivially contains.
    let cases: Vec<(&str, String)> = vec![
        (
            "deeply nested emphasis",
            "<em>".repeat(5_000) + "x" + &"</em>".repeat(5_000),
        ),
        ("unclosed nested emphasis", "<em>".repeat(5_000) + "x"),
        (
            "many attributes",
            format!("<p {}>x</p>", "data-a=\"1\" ".repeat(20_000)),
        ),
        ("entity storm", "&amp;".repeat(200_000)),
        // ~keep Sized to stay fast, not to probe the limit. Unterminated `<` was quadratic
        // ~keep when this file was written; asserting on that scaling is the job of
        // ~keep `bare_lt_complexity.rs`, which pins the growth ratio. Here the only question
        // ~keep is whether the shape crashes.
        ("bare angle brackets", "<".repeat(20_000)),
        ("null and control bytes", "a\u{0}b\u{1}c\u{7}d".repeat(10_000)),
        ("unterminated comment", format!("<!-- {}", "a".repeat(50_000))),
        ("unterminated tag", format!("<p {}", "a".repeat(50_000))),
        ("multibyte boundary spam", "\u{1F600}<br>".repeat(20_000)),
        (
            "table without rows",
            format!("<table>{}</table>", "<td>x".repeat(20_000)),
        ),
        ("interleaved mis-nesting", "<b><i></b></i>".repeat(20_000)),
        ("br run", "<p>a".to_owned() + &"<br>".repeat(20_000) + "b</p>"),
    ];

    with_hang_guard(CONVERSION_BUDGET, move |current| {
        for (label, html) in &cases {
            for (option_label, options) in matrix() {
                assert_survives(label, html, options, option_label, current);
            }
        }
    });
}

#[test]
fn should_survive_pathological_shapes() {
    sweep_pathological(option_matrix);
    sweep_pathological(structure_matrix);
}

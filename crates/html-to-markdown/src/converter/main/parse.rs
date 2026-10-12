use super::{preprocess_initial_html, preprocess_repaired_html, trace_parse_complete};
use crate::converter::DomContext;
use crate::converter::main_helpers::{has_custom_element_tags, preserves_menu, repair_with_html5ever_bounded};
use crate::converter::preprocessing_helpers::{
    has_discarded_end_tag, has_inline_block_misnest, has_omitted_end_tag, readable_text_len, written_title_len,
};
use crate::converter::utility::caching::build_dom_context;
use crate::converter::utility::preprocessing::normalize_inert_bodies_with_preservation;
use crate::error::{ConversionError, Result};
use crate::options::{ConversionOptions, HiddenContent};

#[cfg(test)]
thread_local! {
    static PARSE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FORCE_INVALID_LENGTH: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn parse_html(input: &str) -> Result<tl::VDom<'_>> {
    #[cfg(test)]
    PARSE_CALLS.with(|calls| calls.set(calls.get() + 1));
    #[cfg(test)]
    if FORCE_INVALID_LENGTH.with(std::cell::Cell::get) {
        return Err(invalid_length(input));
    }
    tl::parse(input, tl::ParserOptions::default()).map_err(|tl::ParseError::InvalidLength| invalid_length(input))
}

fn invalid_length(input: &str) -> ConversionError {
    tracing::error!(
        target: "html_to_markdown::convert",
        input_len = input.len(),
        "failed to parse HTML; input length exceeds parser capacity"
    );
    ConversionError::ParseError("Failed to parse HTML".to_string())
}

/// The tree that the converter reads, the length of the page that the tree is made of, and the
/// limit that stopped the tree builder when the first parse was kept for that reason.
pub(super) struct ParsedPage<'a> {
    pub(super) dom: tl::VDom<'a>,
    pub(super) dom_ctx: DomContext,
    pub(super) len: usize,
    pub(super) repair_limit: Option<crate::types::WarningKind>,
}

impl<'a> ParsedPage<'a> {
    const fn first_parse(dom: tl::VDom<'a>, dom_ctx: DomContext, len: usize) -> Self {
        Self {
            dom,
            dom_ctx,
            len,
            repair_limit: None,
        }
    }
}

/// Parse `input`, and parse it a second time through the HTML tree builder when it has an
/// omitted end tag or a misnest. `repaired_page` holds the page of the second parse.
///
/// ~keep The repair must never lose text. The tree builder drops text that the first parse
/// ~keep keeps (all that follows `</frameset>` in a frameset document), and no list of such
/// ~keep cases is kept here: both pages are read by `tl` and measured by
/// ~keep [`readable_text_len`], and a repaired page with a smaller count is not used. The
/// ~keep page then converts as it did before issue #772.
///
/// ~keep `as_written` is the page before the passes of this conversion, when the tree builder
/// ~keep has not written the page of `input`. With it, a page that asks for no second parse
/// ~keep takes one all the same when a narrower choice of `hidden_content` takes one:
/// ~keep [`narrower_choice_asks`].
pub(super) fn parse_for_conversion<'a>(
    input: &'a str,
    repaired_page: &'a std::cell::OnceCell<String>,
    options: &ConversionOptions,
    as_written: Option<&str>,
) -> Result<ParsedPage<'a>> {
    let (preserve_menu, hidden_content) = (preserves_menu(options), options.hidden_content);
    let dom = parse_html(input)?;
    let parser = dom.parser();
    let dom_ctx = build_dom_context(&dom, parser, input.len());
    let len = input.len();
    let asked = Asked::of(&dom_ctx, parser, input);
    let misnest = asked.misnest;
    let for_narrower_choice = match as_written {
        Some(html) if !asked.any() => narrower_choice_asks(html, input, options, hidden_content)?,
        _ => false,
    };
    // ~keep The second parse for a narrower choice has the text rule and the limits of the one
    // ~keep for an omitted end tag: the first parse holds every word of the page.
    let omitted_end_tag = asked.omitted_end_tag || for_narrower_choice;
    if !omitted_end_tag && !misnest {
        return Ok(ParsedPage::first_parse(dom, dom_ctx, len));
    }
    // ~keep Every route to the tree builder has the node budget and the depth limit (issues
    // ~keep #808 and #809). When only an omitted end tag asked for the repair, a tree that
    // ~keep reached a limit is not used: the first parse holds every word of the input and is
    // ~keep kept, with the warning. The first parse of a misnest can hide content, so a misnest
    // ~keep takes the tree up to the depth limit and the rest of the input as written.
    let Some((repaired, repair_limit)) = repair_with_html5ever_bounded(input) else {
        tracing::warn!(
            target: "html_to_markdown::convert",
            "html5ever repair gave no tree (the document cannot be written back); proceeding with original structure"
        );
        return Ok(ParsedPage::first_parse(dom, dom_ctx, len));
    };
    if repair_limit.is_some() && !misnest {
        return Ok(ParsedPage {
            dom,
            dom_ctx,
            len,
            repair_limit,
        });
    }
    let page = repaired_page.get_or_init(|| preprocess_repaired_html(&repaired, preserve_menu, hidden_content));
    let second = parse_html(page)?;
    // ~keep The text rule is for the second parse that an omitted end tag asks for. The first
    // ~keep parse of a misnest can hide content, so its count is not a measure.
    let loses_text = omitted_end_tag
        && (readable_text_len(&second) < readable_text_len(&dom)
            || loses_a_title_of_a_wider_choice(&dom, &second, as_written, options, hidden_content)?);
    if loses_text && !misnest {
        // ~keep A frameset page with every end tag written comes here, so this is no warning.
        tracing::debug!(
            target: "html_to_markdown::convert",
            "html5ever repair would lose text; proceeding with original structure"
        );
        return Ok(ParsedPage::first_parse(dom, dom_ctx, len));
    }
    let for_omitted_end_tag = omitted_end_tag && !loses_text && repair_limit.is_none();
    log_the_second_parse(for_narrower_choice, asked.discarded_end_tag, for_omitted_end_tag);
    let dom_ctx = build_dom_context(&second, second.parser(), page.len());
    Ok(ParsedPage {
        dom: second,
        dom_ctx,
        len: page.len(),
        repair_limit,
    })
}

/// Log why the conversion reads the second parse.
fn log_the_second_parse(for_narrower_choice: bool, discarded_end_tag: bool, for_omitted_end_tag: bool) {
    if for_narrower_choice {
        tracing::debug!(
            target: "html_to_markdown::convert",
            "a narrower choice of hidden_content takes the html5ever repair; re-parsed with html5ever repair"
        );
    } else if discarded_end_tag {
        tracing::debug!(
            target: "html_to_markdown::convert",
            "end tag of an open element discarded by the first parse; re-parsed with html5ever repair"
        );
    } else if for_omitted_end_tag {
        // ~keep An omitted end tag is HTML that the standard permits, so this route is logged
        // ~keep at debug, like the choice of a tier. A misnest is an authoring error and warns.
        // ~keep A page that reached a limit is here only as a misnest.
        tracing::debug!(
            target: "html_to_markdown::convert",
            "element with no end tag; re-parsed with html5ever repair"
        );
    } else {
        tracing::warn!(
            target: "html_to_markdown::convert",
            "misnested HTML elements detected; re-parsed with html5ever repair"
        );
    }
}

/// What in the first parse of a page asks for the second parse, through the HTML tree builder.
#[derive(Clone, Copy)]
struct Asked {
    /// An element with content is open at the end of the page, or the first parse discarded
    /// the end tag of an open element.
    omitted_end_tag: bool,
    /// The first parse discarded the end tag of an open element.
    discarded_end_tag: bool,
    /// An element is where the HTML tree builder does not leave it.
    misnest: bool,
}

impl Asked {
    fn of(dom_ctx: &DomContext, parser: &tl::Parser<'_>, input: &str) -> Self {
        let open_at_the_end = has_omitted_end_tag(dom_ctx, parser);
        let misnest = has_inline_block_misnest(dom_ctx, parser);
        // ~keep Both rules ask where the tree builder ends an element, so both have the text rule.
        let discarded_end_tag = !open_at_the_end && !misnest && has_discarded_end_tag(dom_ctx, parser, input);
        Self {
            omitted_end_tag: open_at_the_end || discarded_end_tag,
            discarded_end_tag,
            misnest,
        }
    }

    const fn any(self) -> bool {
        self.omitted_end_tag || self.misnest
    }
}

/// Whether `html` goes to the HTML tree builder under a choice of `hidden_content` that is
/// narrower than the choice of the caller. `page` is the page of the caller's choice.
///
/// ~keep The tree builder moves content between the head and the body, and the walk writes the
/// ~keep two differently: an `<img>` in a `<head>` is written only after the tree builder moved
/// ~keep it to the body. A narrower choice keeps tags that a wider choice removes, and such a
/// ~keep tag can be all that asks for the tree builder: in `<head><img><template></head><p>`
/// ~keep the `<template>` has no end tag. So a wider choice takes the tree builder wherever a
/// ~keep narrower one does, and writes what the narrower one writes. A narrower page that is
/// ~keep equal to the wider page asks for nothing new and is not parsed.
///
/// ~keep The inert-body pass ran on `html` under the choice of the caller and left as written
/// ~keep each `<template>` and `<noscript>` that the choice keeps. The narrower choice empties
/// ~keep those that it does not keep, so the pass runs again under the narrower choice.
fn narrower_choice_asks(
    html: &str,
    page: &str,
    options: &ConversionOptions,
    hidden_content: HiddenContent,
) -> Result<bool> {
    let narrower = match hidden_content {
        HiddenContent::Drop => return Ok(false),
        HiddenContent::Reachable => HiddenContent::Drop,
        HiddenContent::All => HiddenContent::Reachable,
    };
    let (narrower_html, _) = normalize_inert_bodies_with_preservation(html, false, narrower);
    let narrower_page = preprocess_initial_html(&narrower_html, options, narrower)?;
    if narrower_page != page && asks_for_the_tree_builder(&narrower_page) {
        return Ok(true);
    }
    narrower_choice_asks(html, &narrower_page, options, narrower)
}

/// Whether the second parse of a choice that keeps hidden content loses a title that the first
/// parse writes, on a page that `drop` does not send to the tree builder. `as_written` is the
/// page before the passes of this conversion.
///
/// ~keep The walk writes a `<title>` of a `<head>` that holds a paragraph as text, and the tree
/// ~keep builder puts that title in the head, where it is the title of the document. Kept content
/// ~keep can be all that asks for the tree builder: in `<head><title>t</title><p>a</p></head>`
/// ~keep `<template><div>b</template>` the `<div>` has no end tag. `drop` then writes the title
/// ~keep from its first parse, so a wider choice keeps its first parse too. Where `drop` takes
/// ~keep the tree builder, each choice takes it and no choice writes the title.
fn loses_a_title_of_a_wider_choice(
    first: &tl::VDom<'_>,
    second: &tl::VDom<'_>,
    as_written: Option<&str>,
    options: &ConversionOptions,
    hidden_content: HiddenContent,
) -> Result<bool> {
    let Some(html) = as_written else {
        return Ok(false);
    };
    if hidden_content == HiddenContent::Drop || written_title_len(second) >= written_title_len(first) {
        return Ok(false);
    }
    let (emptied, _) = normalize_inert_bodies_with_preservation(html, false, HiddenContent::Drop);
    let page_of_drop = preprocess_initial_html(&emptied, options, HiddenContent::Drop)?;
    Ok(!asks_for_the_tree_builder(&page_of_drop))
}

/// Whether the conversion of `page`, a page before its first parse, runs the HTML tree builder.
fn asks_for_the_tree_builder(page: &str) -> bool {
    has_custom_element_tags(page)
        || parse_html(page).is_ok_and(|dom| {
            let parser = dom.parser();
            Asked::of(&build_dom_context(&dom, parser, page.len()), parser, page).any()
        })
}

pub(super) fn finish_dom_parse(
    dom: &mut tl::VDom<'_>,
    dom_ctx: &mut DomContext,
    inert_bodies: &std::collections::HashMap<String, String>,
    input_len: usize,
) -> Result<()> {
    if !inert_bodies.is_empty() {
        crate::converter::utility::preprocessing::restore_inert_bodies(dom, inert_bodies)?;
        *dom_ctx = build_dom_context(dom, dom.parser(), input_len);
    }
    trace_parse_complete(dom, input_len);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FORCE_INVALID_LENGTH, PARSE_CALLS};
    use crate::converter::utility::caching::DOM_CONTEXT_BUILDS;
    use crate::{ConversionError, ConversionOptions, TierStrategy, convert};

    #[test]
    fn should_parse_and_build_dom_context_once_for_well_formed_tier2_input() {
        let options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        };
        PARSE_CALLS.with(|calls| calls.set(0));
        DOM_CONTEXT_BUILDS.with(|builds| builds.set(0));

        let result = convert("<p>Hello <strong>world</strong></p>", options).expect("convert HTML");

        assert_eq!(result.content.as_deref(), Some("Hello **world**\n"));
        PARSE_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        DOM_CONTEXT_BUILDS.with(|builds| assert_eq!(builds.get(), 1));
    }

    fn parse_calls_for(html: &str) -> (usize, Option<String>) {
        let options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        };
        PARSE_CALLS.with(|calls| calls.set(0));
        let result = convert(html, options).expect("convert HTML");
        (PARSE_CALLS.with(std::cell::Cell::get), result.content)
    }

    #[test]
    fn should_parse_the_page_of_a_narrower_choice_only_when_it_is_another_page() {
        // ~keep A wider choice asks the page of each narrower choice whether it takes the tree
        // ~keep builder. A page equal to the wider page is not parsed, and no narrower page is
        // ~keep parsed when the page of the choice asks for the second parse itself.
        let choices = [
            crate::HiddenContent::Drop,
            crate::HiddenContent::Reachable,
            crate::HiddenContent::All,
        ];
        for (html, expected) in [
            ("<p>one</p>", [1, 1, 1]),
            ("<p>one</p><div hidden>two</div>", [1, 2, 2]),
            ("<p>one</p><template><p>two</p></template>", [1, 1, 2]),
            (
                "<p>one</p><div hidden>two</div><template><p>three</p></template>",
                [1, 2, 3],
            ),
            ("<div><p>one</div><template><p>two</p></template>tail", [2, 2, 2]),
        ] {
            let calls = choices.map(|hidden_content| {
                let options = ConversionOptions {
                    tier_strategy: TierStrategy::Tier2,
                    hidden_content,
                    ..ConversionOptions::default()
                };
                PARSE_CALLS.with(|calls| calls.set(0));
                convert(html, options).expect("convert HTML");
                PARSE_CALLS.with(std::cell::Cell::get)
            });
            assert_eq!(calls, expected, "{html}");
        }
    }

    #[test]
    fn should_parse_twice_when_an_element_with_content_has_no_end_tag() {
        let (calls, content) = parse_calls_for("<div><p>one</div>tail");

        assert_eq!(content.as_deref(), Some("one\n\ntail\n"));
        assert_eq!(calls, 2);
    }

    #[test]
    fn should_repair_a_form_nested_in_a_form_without_an_omitted_end_tag() {
        let mut options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        };
        options.preprocessing.remove_forms = false;
        PARSE_CALLS.with(|calls| calls.set(0));
        let result = convert("<form>a<form>b</form>c</form>d", options).expect("retained nested form");

        assert_eq!(result.content.as_deref(), Some("ab\n\ncd\n"));
        PARSE_CALLS.with(|calls| assert_eq!(calls.get(), 2));
    }

    #[test]
    fn should_parse_once_after_source_removal_of_closed_nested_forms() {
        let (calls, content) = parse_calls_for("<form>a<form>b</form>c</form>d");

        assert_eq!(content.as_deref(), Some("cd\n"));
        assert_eq!(calls, 1);
    }

    #[test]
    fn should_not_repair_form_shaped_text_or_forms_inside_a_template() {
        for html in [
            "<form>a</form><form>b</form><p>tail</p>",
            "<form title='<form>'>a</form><p>tail</p>",
            "<form><!-- <form>b</form> -->a</form><p>tail</p>",
            "<form><script>const text = '<form><form>';</script>a</form><p>tail</p>",
            "<form><style>a::after { content: '<form><form>'; }</style>a</form><p>tail</p>",
            "<form><template><form>b</form></template>a</form><p>tail</p>",
            "<template><form>a<form>b</form>c</form></template><p>tail</p>",
            "<form><noscript><form>a<form>b</form>c</form></noscript></form><p>tail</p>",
        ] {
            let (calls, content) = parse_calls_for(html);

            assert_eq!(calls, 1, "{html:?}");
            assert_eq!(content.as_deref(), Some("tail\n"), "{html:?}");
        }
    }

    #[test]
    fn should_parse_once_when_only_html_and_body_have_no_end_tag() {
        for html in ["<html><body><p>one</p><p>two</p>", "<HTML><BODY><p>one</p><p>two</p>"] {
            let (calls, content) = parse_calls_for(html);

            assert_eq!(content.as_deref(), Some("one\n\ntwo\n"), "{html:?}");
            assert_eq!(calls, 1, "{html:?}");
        }
    }

    #[test]
    fn should_keep_the_first_parse_when_the_second_parse_nests_past_its_limit() {
        let html = format!("<p>kept</p>{}deep", "<div>".repeat(600));
        let (calls, content) = parse_calls_for(&html);

        assert_eq!(content.as_deref(), Some("kept\n"));
        assert_eq!(calls, 1);
    }

    #[test]
    fn should_parse_twice_when_a_misnest_nests_past_the_repair_depth_limit() {
        // ~keep A cell with no row is a misnest and has no end tag. The first parse hides it,
        // ~keep so the page takes the tree up to the depth limit (issue #808) and the rest of
        // ~keep the input as written, in a second parse.
        let html = format!("{}x", "<table><td>".repeat(300));
        let (calls, content) = parse_calls_for(&html);

        // ~keep The page is past the default depth of the walk, so the test reads the parse
        // ~keep count. The words of such a page are a test of `parser_repair_safeguards.rs`,
        // ~keep which raises the depth of the walk.
        assert!(content.is_some());
        assert_eq!(calls, 2);
    }

    #[test]
    fn should_parse_twice_when_the_first_parse_discarded_the_end_tag_of_an_open_element() {
        // ~keep The list pass writes the end tag of an `li` and of a `dd` before the first
        // ~keep parse, so in a list no element is open at the end tag of the list: one parse.
        let mut failures = Vec::new();
        let cases = [
            ("div", "p", 2),
            ("select", "option", 2),
            ("ul", "li", 1),
            ("dl", "dd", 1),
        ];
        let words = ["one", "two", "rest", "tail"];
        for (outer, inner, expected) in cases {
            let html = format!("<{outer}><{inner}>one</{outer}>two</{inner}>rest</{outer}>tail");
            let (calls, content) = parse_calls_for(&html);
            let content = content.unwrap_or_default();
            let has_every_word = words.into_iter().all(|word| content.contains(word));
            if calls != expected || !has_every_word {
                failures.push(format!("{html:?}: {calls} parse(s), not {expected}: {content:?}"));
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
        let (calls, content) = parse_calls_for("<div><p>one</div>two</p>rest</div>tail");
        assert_eq!(content.as_deref(), Some("one\n\ntwo\n\nresttail\n"));
        assert_eq!(calls, 2);
    }

    #[test]
    fn should_parse_once_when_a_discarded_end_tag_names_no_open_element() {
        for html in [
            "<p>a</b>b</p>",
            "<div title='</div>'><p>x</p></div>",
            "<div><p>x<!-- </div> --></p></div>",
        ] {
            let (calls, content) = parse_calls_for(html);

            assert_eq!(calls, 1, "{html:?}");
            assert!(content.is_some(), "{html:?}");
        }
    }

    #[test]
    fn should_parse_once_when_the_open_element_has_no_content() {
        for html in ["<p>one</p><br>", "<p>one</p><span>", "<html><body><p>one</p><hr>"] {
            let (calls, content) = parse_calls_for(html);

            assert_eq!(calls, 1, "{html:?}");
            assert!(content.is_some_and(|content| content.starts_with("one\n")), "{html:?}");
        }
    }

    #[test]
    fn should_parse_once_when_an_attribute_value_holds_an_end_tag() {
        let (calls, content) = parse_calls_for("<p>one</p><div title='</div>'></div>");

        assert_eq!(content.as_deref(), Some("one\n"));
        assert_eq!(calls, 1);
    }

    #[test]
    fn should_convert_the_first_parse_when_the_repaired_page_holds_less_text() {
        // ~keep `tl` leaves a `<frame>` open, so the first page looks like one with no end tag.
        // ~keep The second call is the parse of the repaired page, which is measured and left.
        for (html, expected) in [
            (
                "<frameset><frame></frameset><div><p>one</p></div><p>tail</p>",
                "one\n\ntail\n",
            ),
            ("<FRAMESET></FRAMESET><div><p>one</div>tail", "onetail\n"),
            ("<noframes><div></noframes><div><p>one</p></div>tail", "one\n\ntail\n"),
        ] {
            let (calls, content) = parse_calls_for(html);

            assert_eq!(content.as_deref(), Some(expected), "{html:?}");
            assert_eq!(calls, 2, "{html:?}");
        }
    }

    #[test]
    fn should_log_at_debug_that_a_repair_that_loses_text_is_not_used() {
        assert_eq!(
            repair_events("<frameset><frame></frameset><div><p>one</p></div><p>tail</p>"),
            [(
                tracing::Level::DEBUG,
                "html5ever repair would lose text; proceeding with original structure".to_string()
            )]
        );
    }

    type Events = std::sync::Arc<std::sync::Mutex<Vec<(tracing::Level, String)>>>;

    struct EventLog(Events);

    struct Message<'a>(&'a mut String);

    impl tracing::field::Visit for Message<'_> {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                *self.0 = format!("{value:?}");
            }
        }
    }

    impl tracing::Subscriber for EventLog {
        fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, _attributes: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }
        fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}
        fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}
        fn event(&self, event: &tracing::Event<'_>) {
            let mut message = String::new();
            event.record(&mut Message(&mut message));
            self.0
                .lock()
                .expect("event log")
                .push((*event.metadata().level(), message));
        }
        fn enter(&self, _span: &tracing::span::Id) {}
        fn exit(&self, _span: &tracing::span::Id) {}
    }

    fn repair_events(html: &str) -> Vec<(tracing::Level, String)> {
        let events = Events::default();
        tracing::subscriber::with_default(EventLog(Events::clone(&events)), || {
            // ~keep A callsite keeps the interest of the thread that reaches it first, and a
            // ~keep test on another thread reaches these with no subscriber. So the first
            // ~keep conversion only reaches them, and the interest is then asked again.
            parse_calls_for(html);
            tracing::callsite::rebuild_interest_cache();
            events.lock().expect("event log").clear();
            parse_calls_for(html);
        });
        let events = events.lock().expect("event log");
        events
            .iter()
            .filter(|(_, message)| message.contains("html5ever repair"))
            .cloned()
            .collect()
    }

    #[test]
    fn should_log_the_second_parse_at_debug_for_an_omitted_end_tag_and_warn_for_a_misnest() {
        assert_eq!(
            repair_events("<div><p>one</div>tail"),
            [(
                tracing::Level::DEBUG,
                "element with no end tag; re-parsed with html5ever repair".to_string()
            )]
        );
        assert_eq!(
            repair_events("<b><p>one</p></b>"),
            [(
                tracing::Level::WARN,
                "misnested HTML elements detected; re-parsed with html5ever repair".to_string()
            )]
        );
    }

    #[test]
    fn should_warn_of_a_misnest_when_it_nests_past_the_repair_depth_limit() {
        // ~keep No cell has an end tag, and the tree builder nests past its depth limit. The
        // ~keep repair that the page then takes is the one for a misnest.
        let html = format!("{}x", "<table><td>".repeat(300));

        assert_eq!(
            repair_events(&html),
            [(
                tracing::Level::WARN,
                "misnested HTML elements detected; re-parsed with html5ever repair".to_string()
            )]
        );
    }

    #[test]
    fn should_fail_once_when_input_exceeds_parser_capacity() {
        let options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        };
        PARSE_CALLS.with(|calls| calls.set(0));
        FORCE_INVALID_LENGTH.with(|forced| forced.set(true));

        let result = convert("<p>oversized</p>", options);

        FORCE_INVALID_LENGTH.with(|forced| forced.set(false));
        assert!(matches!(
            result,
            Err(ConversionError::ParseError(message)) if message == "Failed to parse HTML"
        ));
        PARSE_CALLS.with(|calls| assert_eq!(calls.get(), 1));
    }
}

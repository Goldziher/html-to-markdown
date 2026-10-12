use std::borrow::Cow;

use crate::error::{ConversionError, Result};
use crate::options::HiddenContent;

pub(super) fn before_unwrapping<'a>(
    html: &'a str,
    hidden_content: HiddenContent,
    selectors: &[String],
) -> Result<Cow<'a, str>> {
    if hidden_content == HiddenContent::Drop || selectors.is_empty() {
        return Ok(Cow::Borrowed(html));
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ConversionError::ParseError("Failed to parse HTML".to_owned()))?;
    let parser = dom.parser();
    let mut ranges = Vec::new();
    for selector in selectors {
        if let Some(matches) = dom.query_selector(selector) {
            ranges.extend(matches.filter_map(|handle| {
                handle.get(parser)?.as_tag().map(|tag| {
                    let (start, end) = tag.boundaries(parser);
                    (start, end + 1)
                })
            }));
        }
    }
    if ranges.is_empty() {
        return Ok(Cow::Borrowed(html));
    }
    ranges.sort_unstable();
    let mut output = String::with_capacity(html.len());
    let mut cursor = 0;
    for (start, end) in ranges {
        if start > cursor {
            output.push_str(&html[cursor..start]);
        }
        cursor = cursor.max(end);
    }
    output.push_str(&html[cursor..]);
    Ok(Cow::Owned(output))
}

use std::collections::{HashMap, HashSet};

use super::preserved_bodies::script_marker_attribute;

type ScriptOwners = (HashSet<usize>, Vec<(usize, String)>);

fn script_owners(dom: &tl::VDom<'_>, originals: &HashMap<String, String>) -> ScriptOwners {
    let parser = dom.parser();
    let mut stack: Vec<_> = dom.children().to_vec();
    let mut bodies = HashSet::new();
    let mut attributes = Vec::new();
    while let Some(handle) = stack.pop() {
        let Some(tag) = handle.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let name = tag.name().as_utf8_str();
        // ~keep tl can mis-nest an HTML-source script under SVG after mixed-case closes; the allocated owner proves origin.
        if name.eq_ignore_ascii_case("script") {
            for child in tag.children().top().iter() {
                let Some(raw) = child.get(parser).and_then(tl::Node::as_raw) else {
                    continue;
                };
                let marker = raw.as_utf8_str();
                let Some(attribute) = script_marker_attribute(&marker) else {
                    continue;
                };
                let owned = tag
                    .attributes()
                    .get(attribute.as_str())
                    .flatten()
                    .is_some_and(|value| value.as_utf8_str() == marker);
                if !owned || !originals.contains_key(marker.as_ref()) {
                    continue;
                }
                let (Ok(body), Ok(script_id)) =
                    (usize::try_from(child.get_inner()), usize::try_from(handle.get_inner()))
                else {
                    continue;
                };
                bodies.insert(body);
                attributes.push((script_id, attribute));
            }
        }
        stack.extend(tag.children().top().iter().copied());
    }
    (bodies, attributes)
}

/// Restore opaque source bytes while retaining exact repaired-script ownership. ~keep
pub fn restore_inert_bodies(
    dom: &mut tl::VDom<'_>,
    originals: &HashMap<String, String>,
) -> crate::error::Result<HashSet<usize>> {
    let (script_bodies, attributes) = script_owners(dom, originals);
    let mut restored = HashSet::new();
    for (id, node) in dom.nodes_mut().iter_mut().enumerate() {
        let Some(raw) = node.as_raw_mut() else { continue };
        let marker = raw.as_utf8_str();
        let Some(original) = originals.get(marker.as_ref()) else {
            continue;
        };
        if script_marker_attribute(&marker).is_some() && !script_bodies.contains(&id) {
            continue;
        }
        raw.set(original.as_bytes().to_vec()).map_err(|error| {
            crate::error::ConversionError::Other(format!("restoring inert HTML source bytes: {error:?}"))
        })?;
        restored.insert(id);
    }
    for (id, attribute) in attributes {
        if let Some(tag) = dom.nodes_mut().get_mut(id).and_then(tl::Node::as_tag_mut) {
            let key = tl::Bytes::try_from(attribute).map_err(|error| {
                crate::error::ConversionError::Other(format!("removing repaired script ownership attribute: {error:?}"))
            })?;
            tag.attributes_mut().remove(key);
        }
    }
    Ok(restored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_require_the_owner_value_to_equal_its_direct_body_marker() {
        let marker = "h2m_script_body_0_end";
        let source = format!(
            "<script data-h2m-script-0='h2m_script_body_1_end'>{marker}</script><script data-h2m-script-0='{marker}'>{marker}</script>"
        );
        let mut dom = tl::parse(&source, tl::ParserOptions::default()).expect("two scripts");
        let originals = HashMap::from([(marker.into(), "JSON".into())]);
        let ids = restore_inert_bodies(&mut dom, &originals).expect("restoration");
        assert_eq!(ids.len(), 1);
        let scripts: Vec<_> = dom.query_selector("script").expect("selector").collect();
        let unowned = scripts[0]
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .expect("unowned script");
        assert_eq!(unowned.inner_text(dom.parser()), marker);
        assert_eq!(
            unowned
                .attributes()
                .get("data-h2m-script-0")
                .flatten()
                .expect("authored attribute")
                .as_utf8_str(),
            "h2m_script_body_1_end"
        );
        let owned = scripts[1]
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .expect("owned script");
        assert_eq!(owned.inner_text(dom.parser()), "JSON");
        assert_eq!(owned.attributes().len(), 0);
    }

    #[test]
    fn should_restore_only_the_bound_direct_script_body_and_remove_only_its_attribute() {
        let marker = "h2m_script_body_0_end";
        let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
        let source = format!(
            "<p data-h2m-script-0='{marker}'>{marker}</p><script data-h2m-script-0='{marker}'>{marker}</script><script>{marker}</script>"
        );
        let mut dom = tl::parse(&source, tl::ParserOptions::default()).expect("DOM");
        let originals = HashMap::from([(marker.into(), json.into())]);
        let restored = restore_inert_bodies(&mut dom, &originals).expect("owned restoration");
        assert_eq!(restored.len(), 1);
        let scripts: Vec<_> = dom.query_selector("script").expect("selector").collect();
        let owned = scripts[0]
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .expect("owned tag");
        assert_eq!(owned.inner_text(dom.parser()), json);
        assert_eq!(owned.attributes().len(), 0);
        let other = scripts[1]
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .expect("unowned tag");
        assert_eq!(other.inner_text(dom.parser()), marker);
        let paragraph = dom.query_selector("p").expect("selector").next().expect("paragraph");
        assert_eq!(
            paragraph
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .expect("paragraph tag")
                .inner_text(dom.parser()),
            marker
        );
    }

    #[test]
    fn should_leave_foreign_script_markers_and_authored_owner_attributes_untouched() {
        use super::super::preserved_bodies::PreservedBodies;

        let authored_marker = "h2m_script_body_0_end";
        let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
        for namespace in ["svg", "math"] {
            let authored = format!(
                "<{namespace}><script DATA-H2M-SCRIPT-0='{authored_marker}'>{authored_marker}</script></{namespace}>"
            );
            let mut preserved = PreservedBodies::new(&authored);
            let (marker, attribute) = preserved.protect_script(json);
            assert_eq!(marker, "h2m_script_body_1_end");
            let source = format!("{authored}<script {attribute}='{marker}'>{marker}</script>");
            let mut dom = tl::parse(&source, tl::ParserOptions::default()).expect("foreign DOM and owned sibling");
            let ids = restore_inert_bodies(&mut dom, &preserved.originals).expect("source-proven restoration");
            assert_eq!(ids.len(), 1);
            let handles: Vec<_> = dom.query_selector("script").expect("selector").collect();
            assert_eq!(handles.len(), 2);
            let foreign = handles[0]
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .expect("authored foreign script");
            assert_eq!(foreign.inner_text(dom.parser()), authored_marker);
            assert_eq!(
                foreign
                    .attributes()
                    .get("data-h2m-script-0")
                    .flatten()
                    .expect("authored attribute")
                    .as_utf8_str(),
                authored_marker
            );
            let owned = handles[1]
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .expect("owned sibling");
            assert_eq!(owned.inner_text(dom.parser()), json);
            assert_eq!(owned.attributes().len(), 0);
        }
    }
}

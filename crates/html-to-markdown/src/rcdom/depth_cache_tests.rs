use super::*;
use html5ever::{local_name, ns};

fn element(dom: &RcDom) -> Handle {
    dom.create_element(
        QualName::new(None, ns!(html), local_name!(div)),
        Vec::new(),
        ElementFlags::default(),
    )
}

fn chain(dom: &RcDom, depth: usize) -> Handle {
    let mut parent = dom.document.clone();
    for _ in 0..depth {
        let child = element(dom);
        dom.append(&parent, NodeOrText::AppendNode(child.clone()));
        parent = child;
    }
    parent
}

#[test]
fn should_bound_ancestor_reads_for_repeated_deep_appends() {
    let dom = RcDom::bounded(20_000, 512);
    let parent = chain(&dom, 500);
    dom.depth_reads.set(0);
    for _ in 0..2_000 {
        dom.append(&parent, NodeOrText::AppendNode(element(&dom)));
    }
    assert_eq!(parent.children.borrow().len(), 2_000);
    assert_eq!(dom.repair_limit.get(), None);
    assert!(dom.depth_reads.get() <= 501, "{} ancestry reads", dom.depth_reads.get());
}

#[test]
fn should_detect_depth_after_moving_a_cached_descendant() {
    let dom = RcDom::bounded(100, 4);
    let original = element(&dom);
    dom.append(&dom.document, NodeOrText::AppendNode(original.clone()));
    let descendant = element(&dom);
    dom.append(&original, NodeOrText::AppendNode(descendant.clone()));
    dom.observe_depth(&descendant);
    let destination = chain(&dom, 2);
    dom.remove_from_parent(&original);
    dom.append(&destination, NodeOrText::AppendNode(original.clone()));
    assert!(Rc::ptr_eq(
        &depth_ancestor(&original).expect("attached parent"),
        &destination
    ));
    dom.repair_limit.set(None);
    dom.observe_depth(&descendant);
    assert_eq!(
        dom.repair_limit.get(),
        Some(crate::types::WarningKind::DepthLimitExceeded)
    );
}

#[test]
fn should_refresh_detached_and_sibling_inserted_subtrees() {
    let dom = RcDom::bounded(100, 4);
    let parent = chain(&dom, 2);
    let sibling = element(&dom);
    dom.append(&parent, NodeOrText::AppendNode(sibling.clone()));
    let subtree = element(&dom);
    let child = element(&dom);
    dom.append(&subtree, NodeOrText::AppendNode(child.clone()));
    dom.observe_depth(&child);
    dom.append_before_sibling(&sibling, NodeOrText::AppendNode(subtree.clone()));
    dom.observe_depth(&child);
    assert_eq!(
        dom.repair_limit.get(),
        Some(crate::types::WarningKind::DepthLimitExceeded)
    );
    dom.remove_from_parent(&subtree);
    dom.repair_limit.set(None);
    dom.observe_depth(&child);
    assert_eq!(dom.repair_limit.get(), None);
}

#[test]
fn should_refresh_cached_children_when_reparented() {
    let dom = RcDom::bounded(100, 4);
    let original = element(&dom);
    dom.append(&dom.document, NodeOrText::AppendNode(original.clone()));
    let child = element(&dom);
    dom.append(&original, NodeOrText::AppendNode(child.clone()));
    let descendant = element(&dom);
    dom.append(&child, NodeOrText::AppendNode(descendant.clone()));
    dom.observe_depth(&descendant);
    let destination = chain(&dom, 2);
    dom.reparent_children(&original, &destination);
    dom.observe_depth(&descendant);
    assert_eq!(
        dom.repair_limit.get(),
        Some(crate::types::WarningKind::DepthLimitExceeded)
    );
}

#[test]
fn should_refresh_template_content_depth_when_host_is_attached() {
    let dom = RcDom::bounded(100, 4);
    let mut flags = ElementFlags::default();
    flags.template = true;
    let host = dom.create_element(QualName::new(None, ns!(html), local_name!(template)), Vec::new(), flags);
    let contents = dom.get_template_contents(&host);
    let child = element(&dom);
    dom.append(&contents, NodeOrText::AppendNode(child.clone()));
    dom.observe_depth(&child);
    let parent = chain(&dom, 2);
    dom.append(&parent, NodeOrText::AppendNode(host));
    dom.observe_depth(&child);
    assert_eq!(
        dom.repair_limit.get(),
        Some(crate::types::WarningKind::DepthLimitExceeded)
    );
}

#[test]
fn should_keep_the_equal_and_over_depth_boundaries() {
    for depth in [3, 4, 5] {
        let dom = RcDom::bounded(100, 4);
        let parent = chain(&dom, depth);
        dom.repair_limit.set(None);
        dom.observe_depth(&parent);
        assert_eq!(dom.repair_limit.get().is_some(), depth >= 4);
    }
}

#[test]
fn should_stop_a_cyclic_ancestry_at_the_depth_ceiling() {
    let dom = RcDom::bounded(100, 4);
    let node = element(&dom);
    node.parent.set(Some(Rc::downgrade(&node)));
    dom.observe_depth(&node);
    assert_eq!(
        dom.repair_limit.get(),
        Some(crate::types::WarningKind::DepthLimitExceeded)
    );
}

#[test]
fn should_scale_actual_omitted_paragraph_repair_reads_linearly() {
    use html5ever::tendril::TendrilSink;
    for depth in [0, 100, 500] {
        for paragraphs in [500, 2_000] {
            let html = format!("{}{}", "<div>".repeat(depth), "<p>x ".repeat(paragraphs));
            let dom = html5ever::parse_document(RcDom::bounded(20_000, 512), Default::default()).one(html);
            assert_eq!(dom.repair_limit.get(), None);
            assert!(
                dom.depth_reads.get() <= paragraphs + depth + 10,
                "depth={depth}, paragraphs={paragraphs}, reads={}",
                dom.depth_reads.get()
            );
        }
    }
}

#[test]
fn should_preserve_closed_and_omitted_paragraph_bytes_near_the_repair_limit() {
    use crate::{ConversionOptions, TierStrategy, convert};
    let options = ConversionOptions {
        max_depth: Some(1024),
        tier_strategy: TierStrategy::Tier2,
        ..Default::default()
    };
    for depth in [0, 100, 500] {
        for paragraphs in [500, 2_000] {
            let expected = format!("{}x\n", "x\n\n".repeat(paragraphs - 1));
            for paragraph in ["<p>x ", "<p>x </p>"] {
                let html = format!("{}{}", "<div>".repeat(depth), paragraph.repeat(paragraphs));
                let result = convert(&html, options.clone()).expect("conversion");
                assert_eq!(result.content.as_deref(), Some(expected.as_str()));
                assert!(result.warnings.is_empty());
            }
        }
    }
}

#[test]
fn should_refresh_cached_depth_when_a_detached_component_root_drops() {
    let dom = RcDom::bounded(100, 1);
    let parent = element(&dom);
    let child = element(&dom);
    dom.append(&parent, NodeOrText::AppendNode(child.clone()));
    dom.observe_depth(&child);
    assert_eq!(
        dom.repair_limit.get(),
        Some(crate::types::WarningKind::DepthLimitExceeded)
    );
    dom.repair_limit.set(None);
    drop(parent);
    dom.observe_depth(&child);
    assert_eq!(dom.repair_limit.get(), None);
}

#[test]
fn should_disable_cache_reuse_when_the_topology_epoch_saturates() {
    let dom = RcDom::bounded(100, 4);
    let parent = chain(&dom, 2);
    dom.topology_epoch.set(usize::MAX);
    dom.invalidate_depths();
    assert_eq!(dom.topology_epoch.get(), usize::MAX);
    dom.depth_reads.set(0);
    for _ in 0..10 {
        dom.observe_depth(&parent);
    }
    assert_eq!(dom.depth_reads.get(), 30);
    assert_eq!(dom.repair_limit.get(), None);
}

#[test]
fn should_detect_actual_repair_depth_on_both_sides_of_the_ceiling() {
    use html5ever::tendril::TendrilSink;
    for depth in [508, 509, 600] {
        let html = format!("{}{}", "<div>".repeat(depth), "<p>x ".repeat(20));
        let dom = html5ever::parse_document(RcDom::bounded(20000, 512), Default::default()).one(html);
        assert_eq!(dom.repair_limit.get().is_some(), depth >= 509);
    }
}

#[test]
fn should_bound_self_attachment_without_borrowing_the_cache_twice() {
    let dom = RcDom::bounded(100, 4);
    let node = element(&dom);
    dom.append(&node, NodeOrText::AppendNode(node.clone()));
    dom.observe_depth(&node);
    let limit = dom.repair_limit.get();
    node.parent.set(None);
    node.children.borrow_mut().clear();
    assert_eq!(limit, Some(crate::types::WarningKind::DepthLimitExceeded));
}

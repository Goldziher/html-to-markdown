use std::collections::HashSet;

use super::ConversionParameters;
use crate::converter::context::{Context, ContextParameters};
use crate::options::ConversionOptions;

/// The context of the walk over the page `html`, the normalized input of the conversion.
pub(super) fn create_conversion_context(
    html: &str,
    options: &ConversionOptions,
    parameters: ConversionParameters<'_>,
    reference_collector: Option<&crate::converter::reference_collector::ReferenceCollectorHandle>,
    literal_body_ids: HashSet<usize>,
) -> Context {
    let ctx = Context::new(
        options,
        ContextParameters {
            inline_collector: parameters.inline_collector,
            #[cfg(feature = "metadata")]
            metadata_collector: parameters.metadata_collector,
            #[cfg(feature = "visitor")]
            visitor: parameters.visitor,
            structure_collector: parameters.structure_collector,
            reference_collector: reference_collector.map(std::rc::Rc::clone),
            base_url: parameters.base_url,
            own_page: crate::converter::url_resolve::OwnPage::of(html, options.base_url.as_deref()),
        },
    );
    #[cfg(feature = "metadata")]
    {
        let mut ctx = ctx;
        if !literal_body_ids.is_empty() {
            ctx.literal_body_ids = std::rc::Rc::new(literal_body_ids);
        }
        ctx
    }
    #[cfg(not(feature = "metadata"))]
    {
        drop(literal_body_ids);
        ctx
    }
}

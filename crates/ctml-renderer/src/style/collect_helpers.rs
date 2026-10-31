use crate::css_style::ElementIdentity;
use crate::font_metrics::FontRelativeUnitsContext;
use crate::{StyleRule, css_style, style::TextStyle};
use ctml_css::{SelectorPosition, SelectorSibling, SelectorTreeElement};

pub(super) fn element_identity(
    tag: &str,
    class: &Option<String>,
    id: &Option<String>,
) -> ElementIdentity {
    ElementIdentity {
        tag: tag.to_owned(),
        class: class.clone(),
        id: id.clone(),
    }
}

pub(super) fn inline_style(attributes: &[(String, Option<String>)]) -> Option<&str> {
    attributes
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("style"))
        .and_then(|(_, value)| value.as_deref())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_element_style(
    inherited: &TextStyle,
    identity: &ElementIdentity,
    attributes: &[(String, Option<String>)],
    path: &[ElementIdentity],
    path_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    path_siblings: &[Vec<SelectorSibling>],
    position: SelectorPosition,
    path_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    inline_style: Option<&str>,
    stylesheet: &[StyleRule],
    font_metrics: Option<&FontRelativeUnitsContext<'_>>,
) -> TextStyle {
    let mut inherited_style = inherited.clone();
    inherited_style.counter_reset.clear();
    inherited_style.counter_increment.clear();
    inherited_style.counter_set.clear();
    if let Some(font_metrics) = font_metrics {
        inherited_style.font_relative_units = font_metrics.resolve(inherited);
    }
    let initial_style = css_style::resolve_style_with_selector_tree_context(
        &inherited_style,
        identity,
        path,
        attributes,
        path_attributes,
        siblings_before_current,
        path_siblings,
        position,
        path_positions,
        selector_tree,
        inline_style,
        stylesheet,
    );
    let local_font_units = font_metrics
        .map(|font_metrics| font_metrics.resolve(&initial_style))
        .unwrap_or(initial_style.font_relative_units);
    css_style::resolve_style_with_selector_tree_context_and_font_units(
        &inherited_style,
        identity,
        path,
        attributes,
        path_attributes,
        siblings_before_current,
        path_siblings,
        position,
        path_positions,
        selector_tree,
        inline_style,
        stylesheet,
        local_font_units,
    )
}

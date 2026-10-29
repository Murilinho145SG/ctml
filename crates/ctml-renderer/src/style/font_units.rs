use crate::{StyleRule, css_style, font_metrics::FontRelativeUnitsContext};
use ctml_css::{ElementIdentity, SelectorPosition, SelectorTreeElement};

use super::TextStyle;

pub(super) fn resolve_root_style(
    inherited: &TextStyle,
    body: &ElementIdentity,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
    font_metrics: Option<&FontRelativeUnitsContext<'_>>,
) -> TextStyle {
    let initial_style = css_style::resolve_style_with_selector_tree_context(
        inherited,
        body,
        &[],
        &[],
        &[],
        &[],
        &[],
        position,
        &[],
        selector_tree,
        None,
        stylesheet,
    );
    let local_font_units = font_metrics
        .map(|font_metrics| font_metrics.resolve(&initial_style))
        .unwrap_or(initial_style.font_relative_units);
    css_style::resolve_style_with_selector_tree_context_and_font_units(
        &TextStyle::default(),
        body,
        &[],
        &[],
        &[],
        &[],
        &[],
        position,
        &[],
        selector_tree,
        None,
        stylesheet,
        local_font_units,
    )
}

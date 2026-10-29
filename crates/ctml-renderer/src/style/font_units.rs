use crate::{StyleRule, css_style, font_metrics::FontRelativeUnitsContext};
use ctml_css::{ElementIdentity, SelectorPosition, SelectorTreeElement, ViewportSize};

use super::TextStyle;

pub(super) fn resolve_root_style(
    body: &ElementIdentity,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
    font_metrics: Option<&FontRelativeUnitsContext<'_>>,
    viewport: ViewportSize,
) -> TextStyle {
    let mut inherited = TextStyle::default();
    inherited.viewport_size = viewport;
    if let Some(font_metrics) = font_metrics {
        inherited.font_relative_units = font_metrics.resolve(&inherited);
    }
    let initial_style = css_style::resolve_style_with_selector_tree_context(
        &inherited,
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
    let mut initial_values = TextStyle::default();
    initial_values.font_relative_units = inherited.font_relative_units;
    initial_values.viewport_size = viewport;
    css_style::resolve_style_with_selector_tree_context_and_font_units(
        &initial_values,
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

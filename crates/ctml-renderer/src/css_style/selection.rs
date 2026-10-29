use ctml_css::{SelectorPosition, SelectorTreeElement};

use crate::{StyleRule, style::TextStyle};

use super::{ElementIdentity, resolve_style_with_selector_tree_context};

pub(crate) fn selection_background_color(stylesheet: &[StyleRule]) -> [u8; 4] {
    let declarations = stylesheet
        .iter()
        .filter_map(|rule| {
            let selectors = rule
                .selectors
                .iter()
                .filter(|selector| {
                    matches!(
                        selector.trim().to_ascii_lowercase().as_str(),
                        "::selection" | "*::selection" | "body::selection"
                    )
                })
                .map(|_| "body".to_owned())
                .collect::<Vec<_>>();
            (!selectors.is_empty()).then(|| StyleRule {
                selectors,
                layer: rule.layer,
                declarations: rule
                    .declarations
                    .iter()
                    .filter(|declaration| {
                        declaration
                            .property
                            .eq_ignore_ascii_case("background-color")
                    })
                    .cloned()
                    .collect(),
            })
        })
        .collect::<Vec<_>>();
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let style = resolve_style_with_selector_tree_context(
        &TextStyle::default(),
        &body,
        &[],
        &[],
        &[],
        &[],
        &[],
        SelectorPosition::default(),
        &[],
        &[],
        None,
        &declarations,
    );
    style.background_color.unwrap_or([173, 216, 230, 255])
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_selection_style(
    inherited: &TextStyle,
    current: &ElementIdentity,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
    default_background: [u8; 4],
) -> TextStyle {
    let declarations = stylesheet
        .iter()
        .filter_map(|rule| {
            let selectors = rule
                .selectors
                .iter()
                .filter_map(|selector| {
                    let (base, suffix) = selector.rsplit_once("::")?;
                    if !suffix.eq_ignore_ascii_case("selection") {
                        return None;
                    }
                    let base = base.trim();
                    Some(if base.is_empty() || base == "*" {
                        "*".to_owned()
                    } else if base.eq_ignore_ascii_case("body") {
                        "body *".to_owned()
                    } else {
                        base.to_owned()
                    })
                })
                .collect::<Vec<_>>();
            (!selectors.is_empty()).then(|| StyleRule {
                selectors,
                layer: rule.layer,
                declarations: rule
                    .declarations
                    .iter()
                    .filter(|declaration| {
                        declaration
                            .property
                            .eq_ignore_ascii_case("background-color")
                            || declaration.property.eq_ignore_ascii_case("color")
                    })
                    .cloned()
                    .collect(),
            })
        })
        .collect::<Vec<_>>();
    let mut inherited = inherited.clone();
    inherited.background_color = Some(default_background);
    super::resolve_style_with_selector_tree_context(
        &inherited,
        current,
        &[],
        &[],
        &[],
        &[],
        &[],
        position,
        &[],
        selector_tree,
        None,
        &declarations,
    )
}

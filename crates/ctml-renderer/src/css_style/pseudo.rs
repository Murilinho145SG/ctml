use ctml_css::{SelectorPosition, SelectorTreeElement};

use crate::{StyleRule, css_style::ElementIdentity, style::TextStyle};

pub(crate) struct PseudoStyle {
    pub(crate) foreground: Option<[u8; 4]>,
    pub(crate) background: Option<[u8; 4]>,
    pub(crate) resolved: TextStyle,
}

pub(crate) fn first_line_style(
    inherited: &TextStyle,
    current: &ElementIdentity,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
) -> PseudoStyle {
    resolve_pseudo_style(
        "first-line",
        inherited,
        current,
        position,
        selector_tree,
        stylesheet,
    )
}

pub(crate) fn first_letter_style(
    inherited: &TextStyle,
    current: &ElementIdentity,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
) -> PseudoStyle {
    resolve_pseudo_style(
        "first-letter",
        inherited,
        current,
        position,
        selector_tree,
        stylesheet,
    )
}

fn resolve_pseudo_style(
    pseudo_element: &str,
    inherited: &TextStyle,
    current: &ElementIdentity,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
) -> PseudoStyle {
    let rules = stylesheet
        .iter()
        .filter_map(|rule| {
            let selectors = rule
                .selectors
                .iter()
                .filter_map(|selector| {
                    let (base, suffix) = selector.rsplit_once("::")?;
                    if !suffix.eq_ignore_ascii_case(pseudo_element) {
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
                .filter(|selector| !selector.is_empty())
                .collect::<Vec<_>>();
            (!selectors.is_empty()).then(|| StyleRule {
                selectors,
                layer: rule.layer,
                declarations: rule
                    .declarations
                    .iter()
                    .filter(|declaration| {
                        [
                            "color",
                            "background-color",
                            "font",
                            "font-family",
                            "font-size",
                            "font-weight",
                            "font-style",
                            "font-stretch",
                        ]
                        .iter()
                        .any(|property| declaration.property.eq_ignore_ascii_case(property))
                    })
                    .cloned()
                    .collect(),
            })
        })
        .collect::<Vec<_>>();
    let mut inherited = inherited.clone();
    inherited.background_color = None;
    let resolved = super::resolve_style_with_selector_tree_context(
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
        &rules,
    );
    PseudoStyle {
        foreground: (resolved.color != inherited.color).then_some(resolved.color),
        background: resolved.background_color,
        resolved,
    }
}

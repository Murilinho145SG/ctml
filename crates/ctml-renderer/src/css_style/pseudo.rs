use ctml_css::{SelectorPosition, SelectorTreeElement};

use crate::{StyleRule, css_style::ElementIdentity, style::TextStyle};

pub(crate) struct FirstLineStyle {
    pub(crate) foreground: Option<[u8; 4]>,
    pub(crate) background: Option<[u8; 4]>,
}

pub(crate) fn first_line_style(
    inherited: &TextStyle,
    current: &ElementIdentity,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
) -> FirstLineStyle {
    let rules = stylesheet
        .iter()
        .filter_map(|rule| {
            let selectors = rule
                .selectors
                .iter()
                .filter_map(|selector| {
                    let (base, suffix) = selector.rsplit_once("::")?;
                    if !suffix.eq_ignore_ascii_case("first-line") {
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
                        declaration.property.eq_ignore_ascii_case("color")
                            || declaration
                                .property
                                .eq_ignore_ascii_case("background-color")
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
    FirstLineStyle {
        foreground: (resolved.color != inherited.color).then_some(resolved.color),
        background: resolved.background_color,
    }
}

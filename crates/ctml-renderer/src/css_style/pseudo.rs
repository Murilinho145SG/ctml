use ctml_css::{SelectorPosition, SelectorTreeElement};

use crate::{StyleRule, css_style::ElementIdentity, style::TextStyle};

pub(crate) struct PseudoStyle {
    pub(crate) foreground: Option<[u8; 4]>,
    pub(crate) background: Option<[u8; 4]>,
    pub(crate) resolved: TextStyle,
    pub(crate) decoration: Option<PseudoDecoration>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PseudoDecoration {
    pub(crate) underline: bool,
    pub(crate) overline: bool,
    pub(crate) line_through: bool,
    pub(crate) color: [u8; 4],
    pub(crate) thickness: f32,
    pub(crate) style: crate::style::TextDecorationStyle,
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
                            "text-decoration",
                            "text-decoration-line",
                            "text-decoration-color",
                            "text-decoration-thickness",
                            "text-decoration-style",
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
    let has_decoration_declaration =
        rules
            .iter()
            .flat_map(|rule| &rule.declarations)
            .any(|declaration| {
                [
                    "text-decoration",
                    "text-decoration-line",
                    "text-decoration-color",
                    "text-decoration-thickness",
                    "text-decoration-style",
                ]
                .iter()
                .any(|property| declaration.property.eq_ignore_ascii_case(property))
            });
    let decoration = (has_decoration_declaration
        && (resolved.underline || resolved.overline || resolved.strikethrough))
        .then_some(PseudoDecoration {
            underline: resolved.underline,
            overline: resolved.overline,
            line_through: resolved.strikethrough,
            color: resolved.text_decoration_color.unwrap_or(resolved.color),
            thickness: resolved.decoration_thickness.unwrap_or(1.0),
            style: resolved.decoration_style,
        });
    PseudoStyle {
        foreground: (resolved.color != inherited.color).then_some(resolved.color),
        background: resolved.background_color,
        resolved,
        decoration,
    }
}

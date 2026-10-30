use ctml_css::{SelectorPosition, SelectorTreeElement};

use crate::{StyleRule, css_style::ElementIdentity, style::TextStyle};

pub(crate) struct PseudoStyle {
    pub(crate) foreground: Option<[u8; 4]>,
    pub(crate) background: Option<[u8; 4]>,
    pub(crate) resolved: TextStyle,
    pub(crate) decoration: Option<PseudoDecoration>,
    pub(crate) opacity: Option<f32>,
    pub(crate) has_rendered_text_style: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PseudoDecoration {
    pub(crate) underline: bool,
    pub(crate) overline: bool,
    pub(crate) line_through: bool,
    pub(crate) color: [u8; 4],
    pub(crate) thickness: f32,
    pub(crate) style: crate::style::TextDecorationStyle,
    pub(crate) skip_ink: crate::style::TextDecorationSkipInk,
    pub(crate) underline_offset: Option<f32>,
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
                            "opacity",
                            "background",
                            "background-color",
                            "font",
                            "font-family",
                            "font-size",
                            "font-weight",
                            "font-style",
                            "font-stretch",
                            "font-kerning",
                            "font-variant",
                            "font-variant-caps",
                            "font-variant-ligatures",
                            "font-variant-position",
                            "font-variant-numeric",
                            "font-variant-east-asian",
                            "font-feature-settings",
                            "font-optical-sizing",
                            "font-variation-settings",
                            "line-height",
                            "vertical-align",
                            "letter-spacing",
                            "word-spacing",
                            "text-shadow",
                            "text-emphasis",
                            "text-emphasis-style",
                            "text-emphasis-color",
                            "text-emphasis-position",
                            "text-rendering",
                            "text-transform",
                            "text-decoration",
                            "text-decoration-line",
                            "text-decoration-color",
                            "text-decoration-thickness",
                            "text-decoration-style",
                            "text-decoration-skip-ink",
                            "text-underline-offset",
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
                    "text-decoration-skip-ink",
                    "text-underline-offset",
                ]
                .iter()
                .any(|property| declaration.property.eq_ignore_ascii_case(property))
            });
    let has_rendered_text_style =
        rules
            .iter()
            .flat_map(|rule| &rule.declarations)
            .any(|declaration| {
                [
                    "font",
                    "font-family",
                    "font-size",
                    "font-weight",
                    "font-style",
                    "font-stretch",
                    "font-kerning",
                    "font-variant",
                    "font-variant-caps",
                    "font-variant-ligatures",
                    "font-variant-position",
                    "font-variant-numeric",
                    "font-variant-east-asian",
                    "font-feature-settings",
                    "font-optical-sizing",
                    "font-variation-settings",
                    "line-height",
                    "vertical-align",
                    "letter-spacing",
                    "word-spacing",
                    "text-shadow",
                    "text-emphasis",
                    "text-emphasis-style",
                    "text-emphasis-color",
                    "text-emphasis-position",
                    "text-rendering",
                    "text-transform",
                    "opacity",
                ]
                .iter()
                .any(|property| declaration.property.eq_ignore_ascii_case(property))
            });
    let has_opacity_declaration = rules
        .iter()
        .flat_map(|rule| &rule.declarations)
        .any(|declaration| declaration.property.eq_ignore_ascii_case("opacity"));
    let pseudo_opacity = if has_opacity_declaration {
        resolved.opacity
    } else {
        1.0
    };
    let effective_pseudo_opacity = pseudo_opacity * inherited.opacity;
    let decoration = (has_decoration_declaration
        && (resolved.underline || resolved.overline || resolved.strikethrough))
        .then_some(PseudoDecoration {
            underline: resolved.underline,
            overline: resolved.overline,
            line_through: resolved.strikethrough,
            color: scale_alpha(
                resolved.text_decoration_color.unwrap_or(resolved.color),
                effective_pseudo_opacity,
            ),
            thickness: resolved.decoration_thickness.unwrap_or(1.0),
            style: resolved.decoration_style,
            skip_ink: resolved.decoration_skip_ink,
            underline_offset: resolved.underline_offset,
        });
    PseudoStyle {
        foreground: (resolved.color != inherited.color).then_some(resolved.color),
        background: resolved
            .background_color
            .map(|color| scale_alpha(color, pseudo_opacity)),
        opacity: has_opacity_declaration.then_some(pseudo_opacity),
        resolved,
        decoration,
        has_rendered_text_style,
    }
}

fn scale_alpha(mut color: [u8; 4], opacity: f32) -> [u8; 4] {
    color[3] = (f32::from(color[3]) * opacity).round().clamp(0.0, 255.0) as u8;
    color
}

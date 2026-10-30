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
                            "text-orientation",
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
                            "border",
                            "border-color",
                            "border-width",
                            "border-style",
                            "border-top",
                            "border-right",
                            "border-bottom",
                            "border-left",
                            "border-top-color",
                            "border-right-color",
                            "border-bottom-color",
                            "border-left-color",
                            "border-top-width",
                            "border-right-width",
                            "border-bottom-width",
                            "border-left-width",
                            "border-top-style",
                            "border-right-style",
                            "border-bottom-style",
                            "border-left-style",
                            "border-inline",
                            "border-inline-start",
                            "border-inline-end",
                            "border-block",
                            "border-block-start",
                            "border-block-end",
                            "border-inline-color",
                            "border-inline-start-color",
                            "border-inline-end-color",
                            "border-block-color",
                            "border-block-start-color",
                            "border-block-end-color",
                            "border-inline-width",
                            "border-inline-start-width",
                            "border-inline-end-width",
                            "border-block-width",
                            "border-block-start-width",
                            "border-block-end-width",
                            "border-inline-style",
                            "border-inline-start-style",
                            "border-inline-end-style",
                            "border-block-style",
                            "border-block-start-style",
                            "border-block-end-style",
                            "box-shadow",
                            "border-radius",
                            "border-top-left-radius",
                            "border-top-right-radius",
                            "border-bottom-right-radius",
                            "border-bottom-left-radius",
                            "outline",
                            "outline-width",
                            "outline-style",
                            "outline-color",
                            "outline-offset",
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
                    "text-orientation",
                    "text-shadow",
                    "text-emphasis",
                    "text-emphasis-style",
                    "text-emphasis-color",
                    "text-emphasis-position",
                    "text-rendering",
                    "text-transform",
                    "opacity",
                    "border",
                    "border-color",
                    "border-width",
                    "border-style",
                    "border-top",
                    "border-right",
                    "border-bottom",
                    "border-left",
                    "border-top-color",
                    "border-right-color",
                    "border-bottom-color",
                    "border-left-color",
                    "border-top-width",
                    "border-right-width",
                    "border-bottom-width",
                    "border-left-width",
                    "border-top-style",
                    "border-right-style",
                    "border-bottom-style",
                    "border-left-style",
                    "border-inline",
                    "border-inline-start",
                    "border-inline-end",
                    "border-block",
                    "border-block-start",
                    "border-block-end",
                    "border-inline-color",
                    "border-inline-start-color",
                    "border-inline-end-color",
                    "border-block-color",
                    "border-block-start-color",
                    "border-block-end-color",
                    "border-inline-width",
                    "border-inline-start-width",
                    "border-inline-end-width",
                    "border-block-width",
                    "border-block-start-width",
                    "border-block-end-width",
                    "border-inline-style",
                    "border-inline-start-style",
                    "border-inline-end-style",
                    "border-block-style",
                    "border-block-start-style",
                    "border-block-end-style",
                    "box-shadow",
                    "border-radius",
                    "border-top-left-radius",
                    "border-top-right-radius",
                    "border-bottom-right-radius",
                    "border-bottom-left-radius",
                    "outline",
                    "outline-width",
                    "outline-style",
                    "outline-color",
                    "outline-offset",
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

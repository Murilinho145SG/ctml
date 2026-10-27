use cosmic_text::{Stretch, Style, Weight};
use winit::window::CursorIcon;

use crate::{
    StyleRule,
    css_color::parse_color,
    style::{FontVariantCaps, TextStyle, TextTransform},
};

pub(super) struct ElementIdentity {
    pub(super) tag: String,
    pub(super) class: Option<String>,
    pub(super) id: Option<String>,
}

pub(super) fn resolve_style(
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    stylesheet: &[StyleRule],
) -> TextStyle {
    let mut style = inherited.clone();
    apply_matching_styles(&mut style, current, ancestors, stylesheet);
    style
}

pub(super) fn apply_matching_styles(
    style: &mut TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    stylesheet: &[StyleRule],
) {
    let mut matches = Vec::new();
    for (rule_index, rule) in stylesheet.iter().enumerate() {
        let specificity = rule
            .selectors
            .iter()
            .filter(|selector| selector_matches(selector, current, ancestors))
            .map(|selector| selector_specificity(selector))
            .max();
        if let Some(specificity) = specificity {
            for (declaration_index, declaration) in rule.declarations.iter().enumerate() {
                matches.push((specificity, rule_index, declaration_index, declaration));
            }
        }
    }
    matches.sort_by_key(|(specificity, rule, declaration, _)| (*specificity, *rule, *declaration));

    let inherited_color = style.color;
    for (_, _, _, declaration) in matches
        .iter()
        .filter(|(_, _, _, declaration)| declaration.property == "color")
    {
        apply_color_declaration(style, &declaration.value, inherited_color);
    }
    for (_, _, _, declaration) in matches
        .iter()
        .filter(|(_, _, _, declaration)| declaration.property != "color")
    {
        apply_declaration(style, &declaration.property, &declaration.value);
    }
}

fn selector_matches(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
) -> bool {
    let compounds = selector.split_whitespace().collect::<Vec<_>>();
    let Some(last) = compounds.last() else {
        return false;
    };
    if !compound_matches(last, current) {
        return false;
    }
    let mut ancestor_index = ancestors.len();
    for compound in compounds[..compounds.len() - 1].iter().rev() {
        let Some(found) = ancestors[..ancestor_index]
            .iter()
            .rposition(|ancestor| compound_matches(compound, ancestor))
        else {
            return false;
        };
        ancestor_index = found;
    }
    true
}

fn compound_matches(compound: &str, element: &ElementIdentity) -> bool {
    if compound == "*" {
        return true;
    }
    let mut cursor = 0;
    let bytes = compound.as_bytes();
    if bytes.first() == Some(&b'*') {
        cursor = 1;
    } else if bytes
        .first()
        .is_some_and(|byte| *byte != b'.' && *byte != b'#')
    {
        let start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'.' && bytes[cursor] != b'#' {
            cursor += 1;
        }
        if !compound[start..cursor].eq_ignore_ascii_case(&element.tag) {
            return false;
        }
    }
    while cursor < bytes.len() {
        let marker = bytes[cursor];
        if marker != b'.' && marker != b'#' {
            return false;
        }
        cursor += 1;
        let start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'.' && bytes[cursor] != b'#' {
            cursor += 1;
        }
        if start == cursor {
            return false;
        }
        let name = &compound[start..cursor];
        if marker == b'#' && element.id.as_deref() != Some(name) {
            return false;
        }
        if marker == b'.'
            && !element
                .class
                .as_deref()
                .is_some_and(|classes| classes.split_ascii_whitespace().any(|class| class == name))
        {
            return false;
        }
    }
    true
}

fn selector_specificity(selector: &str) -> (usize, usize, usize) {
    let ids = selector.bytes().filter(|byte| *byte == b'#').count();
    let classes = selector.bytes().filter(|byte| *byte == b'.').count();
    let tags = selector
        .split_whitespace()
        .filter(|compound| !compound.starts_with(['.', '#', '*']))
        .count();
    (ids, classes, tags)
}

fn apply_color_declaration(style: &mut TextStyle, value: &str, inherited_color: [u8; 4]) {
    let value = value.trim();
    style.color = if value.eq_ignore_ascii_case("currentcolor") {
        inherited_color
    } else {
        parse_color(value).unwrap_or(style.color)
    };
}

fn apply_declaration(style: &mut TextStyle, property: &str, value: &str) {
    let value = value.trim();
    match property {
        "background-color" => {
            let color = if value.eq_ignore_ascii_case("currentcolor") {
                Some(style.color)
            } else {
                parse_color(value)
            };
            if let Some(color) = color {
                style.background = [
                    composite_over_white(color[0], color[3]),
                    composite_over_white(color[1], color[3]),
                    composite_over_white(color[2], color[3]),
                ];
            }
        }
        "font-size" => {
            if let Some(size) = parse_length(value, style.font_size) {
                style.line_height *= size / style.font_size;
                style.font_size = size;
            }
        }
        "line-height" => {
            if let Ok(multiplier) = value.parse::<f32>() {
                style.line_height = style.font_size * multiplier;
            } else if let Some(height) = parse_length(value, style.font_size) {
                style.line_height = height;
            }
        }
        "font-family" => {
            if let Some(family) = value.split(',').next() {
                style.family = family.trim().trim_matches(['\'', '"']).to_owned();
            }
        }
        "font-weight" => {
            style.weight = match value.to_ascii_lowercase().as_str() {
                "normal" => Weight::NORMAL,
                "bold" => Weight::BOLD,
                _ => value
                    .parse::<u16>()
                    .ok()
                    .filter(|weight| (1..=1000).contains(weight))
                    .map_or(style.weight, Weight),
            };
        }
        "font-style" => {
            style.font_style = match value.to_ascii_lowercase().as_str() {
                "normal" => Style::Normal,
                "italic" => Style::Italic,
                "oblique" => Style::Oblique,
                _ => style.font_style,
            };
        }
        "font-stretch" => {
            style.stretch = match value.to_ascii_lowercase().as_str() {
                "ultra-condensed" => Stretch::UltraCondensed,
                "extra-condensed" => Stretch::ExtraCondensed,
                "condensed" => Stretch::Condensed,
                "semi-condensed" => Stretch::SemiCondensed,
                "normal" => Stretch::Normal,
                "semi-expanded" => Stretch::SemiExpanded,
                "expanded" => Stretch::Expanded,
                "extra-expanded" => Stretch::ExtraExpanded,
                "ultra-expanded" => Stretch::UltraExpanded,
                _ => style.stretch,
            };
        }
        "font-variant" | "font-variant-caps" => {
            style.font_variant_caps = match value.to_ascii_lowercase().as_str() {
                "normal" => FontVariantCaps::Normal,
                "small-caps" => FontVariantCaps::SmallCaps,
                "all-small-caps" => FontVariantCaps::AllSmallCaps,
                _ => style.font_variant_caps,
            };
        }
        "letter-spacing" => apply_letter_spacing(style, value),
        "text-transform" => {
            style.text_transform = match value.to_ascii_lowercase().as_str() {
                "uppercase" => TextTransform::Uppercase,
                "lowercase" => TextTransform::Lowercase,
                "capitalize" => TextTransform::Capitalize,
                "none" => TextTransform::None,
                _ => style.text_transform,
            };
        }
        "text-align" => {
            style.text_align = match value.to_ascii_lowercase().as_str() {
                "left" | "start" => cosmic_text::Align::Left,
                "right" => cosmic_text::Align::Right,
                "end" => cosmic_text::Align::End,
                "center" => cosmic_text::Align::Center,
                "justify" => cosmic_text::Align::Justified,
                _ => style.text_align,
            };
        }
        "text-decoration" | "text-decoration-line" => apply_text_decoration(style, value),
        "text-decoration-color" => {
            style.text_decoration_color = if value.eq_ignore_ascii_case("currentcolor") {
                None
            } else {
                parse_color(value)
            };
        }
        "cursor" => {
            if let Some(cursor) = parse_cursor(value) {
                style.cursor = cursor;
            }
        }
        _ => {}
    }
}

fn composite_over_white(channel: u8, alpha: u8) -> u8 {
    ((channel as u32 * alpha as u32 + 255 * (255 - alpha) as u32 + 127) / 255) as u8
}

fn apply_letter_spacing(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        style.letter_spacing = 0.0;
        style.letter_spacing_px = false;
    } else if let Some(px) = value.strip_suffix("px")
        && let Ok(px) = px.trim().parse::<f32>()
    {
        style.letter_spacing = px;
        style.letter_spacing_px = true;
    } else if let Some(em) = value.strip_suffix("em")
        && let Ok(em) = em.trim().parse::<f32>()
    {
        style.letter_spacing = em;
        style.letter_spacing_px = false;
    }
}

fn apply_text_decoration(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        style.underline = false;
        style.overline = false;
        style.strikethrough = false;
        return;
    }
    for decoration in value.split_ascii_whitespace() {
        match decoration.to_ascii_lowercase().as_str() {
            "underline" => style.underline = true,
            "overline" => style.overline = true,
            "line-through" => style.strikethrough = true,
            _ => {}
        }
    }
}

fn parse_cursor(value: &str) -> Option<CursorIcon> {
    Some(match value.to_ascii_lowercase().as_str() {
        "auto" | "text" => CursorIcon::Text,
        "default" => CursorIcon::Default,
        "pointer" => CursorIcon::Pointer,
        "crosshair" => CursorIcon::Crosshair,
        "move" => CursorIcon::Move,
        "wait" => CursorIcon::Wait,
        "progress" => CursorIcon::Progress,
        "help" => CursorIcon::Help,
        "not-allowed" => CursorIcon::NotAllowed,
        "no-drop" => CursorIcon::NoDrop,
        "grab" => CursorIcon::Grab,
        "grabbing" => CursorIcon::Grabbing,
        "copy" => CursorIcon::Copy,
        "alias" => CursorIcon::Alias,
        "context-menu" => CursorIcon::ContextMenu,
        "cell" => CursorIcon::Cell,
        "all-scroll" => CursorIcon::AllScroll,
        "vertical-text" => CursorIcon::VerticalText,
        "ew-resize" | "col-resize" => CursorIcon::EwResize,
        "ns-resize" | "row-resize" => CursorIcon::NsResize,
        "nesw-resize" => CursorIcon::NeswResize,
        "nwse-resize" => CursorIcon::NwseResize,
        "n-resize" => CursorIcon::NResize,
        "e-resize" => CursorIcon::EResize,
        "s-resize" => CursorIcon::SResize,
        "w-resize" => CursorIcon::WResize,
        "ne-resize" => CursorIcon::NeResize,
        "nw-resize" => CursorIcon::NwResize,
        "se-resize" => CursorIcon::SeResize,
        "sw-resize" => CursorIcon::SwResize,
        "zoom-in" => CursorIcon::ZoomIn,
        "zoom-out" => CursorIcon::ZoomOut,
        _ => return None,
    })
}

fn parse_length(value: &str, relative_to: f32) -> Option<f32> {
    let value = value.trim();
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse::<f32>().ok().filter(|value| *value > 0.0);
    }
    if let Some(em) = value.strip_suffix("em") {
        return em
            .trim()
            .parse::<f32>()
            .ok()
            .map(|value| value * relative_to)
            .filter(|value| *value > 0.0);
    }
    None
}

#[cfg(test)]
mod tests {
    use cosmic_text::{Stretch, Style, Weight};

    use super::{ElementIdentity, resolve_style};
    use crate::{
        StyleDeclaration, StyleRule,
        style::{FontVariantCaps, TextStyle},
    };

    fn style_for_declarations(declarations: &[(&str, &str)]) -> TextStyle {
        let rule = StyleRule {
            selectors: vec![".sample".to_owned()],
            declarations: declarations
                .iter()
                .map(|(property, value)| StyleDeclaration {
                    property: (*property).to_owned(),
                    value: (*value).to_owned(),
                })
                .collect(),
        };
        let body = ElementIdentity {
            tag: "body".to_owned(),
            class: None,
            id: None,
        };
        let paragraph = ElementIdentity {
            tag: "p".to_owned(),
            class: Some("sample".to_owned()),
            id: None,
        };
        resolve_style(&TextStyle::default(), &paragraph, &[body], &[rule])
    }

    #[test]
    fn resolves_numeric_weight_font_style_and_stretch() {
        let style = style_for_declarations(&[
            ("font-weight", "350"),
            ("font-style", "oblique"),
            ("font-stretch", "condensed"),
        ]);

        assert_eq!(style.weight, Weight(350));
        assert_eq!(style.font_style, Style::Oblique);
        assert_eq!(style.stretch, Stretch::Condensed);
    }

    #[test]
    fn resolves_standard_named_css_colors() {
        let style = style_for_declarations(&[("color", "rebeccapurple")]);

        assert_eq!(style.color, [102, 51, 153, 255]);
    }

    #[test]
    fn resolves_alpha_text_color_and_blends_page_background_with_white() {
        let style = style_for_declarations(&[
            ("color", "#00000080"),
            ("background-color", "rgba(0, 0, 0, 50%)"),
        ]);

        assert_eq!(style.color, [0, 0, 0, 128]);
        assert_eq!(style.background, [127, 127, 127]);
    }

    #[test]
    fn resolves_transparent_text_and_page_background() {
        let style = style_for_declarations(&[
            ("color", "transparent"),
            ("background-color", "transparent"),
        ]);

        assert_eq!(style.color, [0, 0, 0, 0]);
        assert_eq!(style.background, [255, 255, 255]);
    }

    #[test]
    fn resolves_current_color_after_cascade() {
        let style =
            style_for_declarations(&[("background-color", "currentColor"), ("color", "#123456")]);

        assert_eq!(style.color, [18, 52, 86, 255]);
        assert_eq!(style.background, [18, 52, 86]);
    }

    #[test]
    fn current_color_for_color_uses_the_inherited_color() {
        let rule = StyleRule {
            selectors: vec![".sample".to_owned()],
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "currentColor".to_owned(),
            }],
        };
        let inherited = TextStyle {
            color: [12, 34, 56, 255],
            ..TextStyle::default()
        };
        let body = ElementIdentity {
            tag: "body".to_owned(),
            class: None,
            id: None,
        };
        let paragraph = ElementIdentity {
            tag: "p".to_owned(),
            class: Some("sample".to_owned()),
            id: None,
        };

        let style = resolve_style(&inherited, &paragraph, &[body], &[rule]);

        assert_eq!(style.color, [12, 34, 56, 255]);
    }

    #[test]
    fn resolves_supported_text_alignment_values() {
        assert_eq!(
            style_for_declarations(&[("text-align", "center")]).text_align,
            cosmic_text::Align::Center
        );
        assert_eq!(
            style_for_declarations(&[("text-align", "justify")]).text_align,
            cosmic_text::Align::Justified
        );
    }

    #[test]
    fn ignores_invalid_numeric_font_weight() {
        let style = style_for_declarations(&[("font-weight", "1001")]);

        assert_eq!(style.weight, Weight::NORMAL);
    }

    #[test]
    fn resolves_small_caps_variants() {
        let style = style_for_declarations(&[("font-variant-caps", "all-small-caps")]);

        assert_eq!(style.font_variant_caps, FontVariantCaps::AllSmallCaps);
    }

    #[test]
    fn applies_text_decoration_color_to_active_lines() {
        let style = style_for_declarations(&[
            ("text-decoration", "underline line-through"),
            ("text-decoration-color", "#ff008080"),
        ]);

        assert!(style.underline);
        assert!(style.strikethrough);
        assert_eq!(style.text_decoration_color, Some([255, 0, 128, 128]));
    }
}

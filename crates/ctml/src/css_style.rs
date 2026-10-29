use cosmic_text::{Stretch, Style, Weight};
use winit::window::CursorIcon;

pub(super) use crate::css_selector::ElementIdentity;
use crate::{
    StyleRule,
    css_color::{parse_color, parse_color_with_opacity},
    css_selector::{selector_matches, selector_specificity},
    style::{FontVariantCaps, FontVariantPosition, TextStyle, TextTransform},
};

const ROOT_FONT_SIZE: f32 = 16.0;

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

    let inherited_color = (style.color, style.color_opacity);
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

fn apply_color_declaration(style: &mut TextStyle, value: &str, inherited_color: ([u8; 4], f32)) {
    let value = value.trim();
    let color = if value.eq_ignore_ascii_case("currentcolor") {
        Some(inherited_color)
    } else {
        parse_color_with_opacity(value).map(|(channels, opacity)| {
            (
                [
                    channels[0],
                    channels[1],
                    channels[2],
                    (opacity * 255.0).round() as u8,
                ],
                opacity,
            )
        })
    };
    if let Some((color, opacity)) = color {
        style.color = color;
        style.color_opacity = opacity;
    }
}

fn apply_declaration(style: &mut TextStyle, property: &str, value: &str) {
    let value = value.trim();
    match property {
        "background-color" => {
            let color = if value.eq_ignore_ascii_case("currentcolor") {
                Some((
                    [style.color[0], style.color[1], style.color[2]],
                    style.color_opacity,
                ))
            } else {
                parse_color_with_opacity(value)
            };
            if let Some((channels, alpha)) = color {
                style.background = [
                    composite_over_white(channels[0], alpha),
                    composite_over_white(channels[1], alpha),
                    composite_over_white(channels[2], alpha),
                ];
            }
        }
        "font-size" => {
            if let Some(size) = parse_font_size(value, style.font_size) {
                style.font_size = size;
                if let Some(multiplier) = style.line_height_multiplier {
                    style.line_height = size * multiplier;
                }
            }
        }
        "line-height" => {
            if let Ok(multiplier) = value.parse::<f32>() {
                style.line_height = style.font_size * multiplier;
                style.line_height_multiplier = Some(multiplier);
            } else if let Some(height) = parse_length(value, style.font_size) {
                style.line_height = height;
                style.line_height_multiplier = None;
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
                "bolder" => relative_font_weight(style.weight, true),
                "lighter" => relative_font_weight(style.weight, false),
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
            if let Some(stretch) = parse_font_stretch(value) {
                style.stretch = stretch;
            }
        }
        "font-kerning" => {
            style.kerning = match value.to_ascii_lowercase().as_str() {
                "auto" | "normal" => Some(true),
                "none" => Some(false),
                _ => style.kerning,
            };
        }
        "font-variant-ligatures" => apply_font_variant_ligatures(style, value),
        "font-variant" | "font-variant-caps" => {
            style.font_variant_caps = match value.to_ascii_lowercase().as_str() {
                "normal" => FontVariantCaps::Normal,
                "small-caps" => FontVariantCaps::SmallCaps,
                "all-small-caps" => FontVariantCaps::AllSmallCaps,
                "petite-caps" => FontVariantCaps::PetiteCaps,
                "all-petite-caps" => FontVariantCaps::AllPetiteCaps,
                "unicase" => FontVariantCaps::Unicase,
                "titling-caps" => FontVariantCaps::TitlingCaps,
                _ => style.font_variant_caps,
            };
        }
        "font-variant-position" => {
            style.font_variant_position = match value.to_ascii_lowercase().as_str() {
                "normal" => FontVariantPosition::Normal,
                "sub" => FontVariantPosition::Sub,
                "super" => FontVariantPosition::Super,
                _ => style.font_variant_position,
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
        "text-decoration-style" => {
            style.underline_style = match value.to_ascii_lowercase().as_str() {
                "solid" => cosmic_text::UnderlineStyle::Single,
                "double" => cosmic_text::UnderlineStyle::Double,
                _ => style.underline_style,
            };
        }
        "text-decoration-color" => {
            style.text_decoration_color = if value.eq_ignore_ascii_case("currentcolor") {
                None
            } else {
                parse_color(value)
            };
        }
        "cursor" => {
            if value.eq_ignore_ascii_case("none") {
                style.cursor = CursorIcon::Default;
                style.cursor_hidden = true;
            } else if let Some(cursor) = parse_cursor(value) {
                style.cursor = cursor;
                style.cursor_hidden = false;
            }
        }
        _ => {}
    }
}

fn composite_over_white(channel: u8, alpha: f32) -> u8 {
    (channel as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8
}

fn relative_font_weight(inherited: Weight, bolder: bool) -> Weight {
    let weight = inherited.0;
    let resolved = match (bolder, weight) {
        (true, weight) if weight < 100 => 400,
        (true, 100..350) => 400,
        (true, 350..550) => 700,
        (true, 550..750) => 900,
        (true, 750..900) => 900,
        (true, _) => weight,
        (false, weight) if weight < 100 => weight,
        (false, 100..350) => 100,
        (false, 350..550) => 100,
        (false, 550..750) => 400,
        (false, 750..900) => 700,
        (false, _) => 700,
    };
    Weight(resolved)
}

fn parse_font_stretch(value: &str) -> Option<Stretch> {
    let keyword = match value.to_ascii_lowercase().as_str() {
        "ultra-condensed" => Some(Stretch::UltraCondensed),
        "extra-condensed" => Some(Stretch::ExtraCondensed),
        "condensed" => Some(Stretch::Condensed),
        "semi-condensed" => Some(Stretch::SemiCondensed),
        "normal" => Some(Stretch::Normal),
        "semi-expanded" => Some(Stretch::SemiExpanded),
        "expanded" => Some(Stretch::Expanded),
        "extra-expanded" => Some(Stretch::ExtraExpanded),
        "ultra-expanded" => Some(Stretch::UltraExpanded),
        _ => None,
    };
    if keyword.is_some() {
        return keyword;
    }

    let percentage = value.strip_suffix('%')?.trim().parse::<f32>().ok()?;
    if !percentage.is_finite() || percentage < 0.0 {
        return None;
    }
    [
        (50.0, Stretch::UltraCondensed),
        (62.5, Stretch::ExtraCondensed),
        (75.0, Stretch::Condensed),
        (87.5, Stretch::SemiCondensed),
        (100.0, Stretch::Normal),
        (112.5, Stretch::SemiExpanded),
        (125.0, Stretch::Expanded),
        (150.0, Stretch::ExtraExpanded),
        (200.0, Stretch::UltraExpanded),
    ]
    .into_iter()
    .min_by(|(left, _), (right, _)| {
        (left - percentage)
            .abs()
            .total_cmp(&(right - percentage).abs())
    })
    .map(|(_, stretch)| stretch)
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

fn apply_font_variant_ligatures(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        style.common_ligatures = None;
        style.discretionary_ligatures = None;
        style.contextual_alternates = None;
        return;
    }
    if value.eq_ignore_ascii_case("none") {
        style.common_ligatures = Some(false);
        style.discretionary_ligatures = Some(false);
        style.contextual_alternates = Some(false);
        return;
    }
    for feature in value.split_ascii_whitespace() {
        match feature.to_ascii_lowercase().as_str() {
            "common-ligatures" => style.common_ligatures = Some(true),
            "no-common-ligatures" => style.common_ligatures = Some(false),
            "discretionary-ligatures" => style.discretionary_ligatures = Some(true),
            "no-discretionary-ligatures" => style.discretionary_ligatures = Some(false),
            "contextual" => style.contextual_alternates = Some(true),
            "no-contextual" => style.contextual_alternates = Some(false),
            _ => {}
        }
    }
}

fn apply_text_decoration(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        style.underline = false;
        style.overline = false;
        style.strikethrough = false;
        style.underline_style = cosmic_text::UnderlineStyle::Single;
        return;
    }
    for decoration in value.split_ascii_whitespace() {
        match decoration.to_ascii_lowercase().as_str() {
            "underline" => style.underline = true,
            "overline" => style.overline = true,
            "line-through" => style.strikethrough = true,
            "solid" => style.underline_style = cosmic_text::UnderlineStyle::Single,
            "double" => style.underline_style = cosmic_text::UnderlineStyle::Double,
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
    let value = value.trim().to_ascii_lowercase();
    if let Some(percentage) = value.strip_suffix('%') {
        return percentage
            .trim()
            .parse::<f32>()
            .ok()
            .map(|percentage| relative_to * percentage / 100.0)
            .filter(|value| value.is_finite() && *value > 0.0);
    }
    for (unit, pixels_per_unit) in [
        ("rem", ROOT_FONT_SIZE),
        ("px", 1.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("q", 96.0 / 101.6),
        ("pt", 96.0 / 72.0),
        ("pc", 16.0),
        ("em", relative_to),
    ] {
        if let Some(number) = value.strip_suffix(unit) {
            return number
                .trim()
                .parse::<f32>()
                .ok()
                .map(|number| number * pixels_per_unit)
                .filter(|value| value.is_finite() && *value > 0.0);
        }
    }
    None
}

fn parse_font_size(value: &str, relative_to: f32) -> Option<f32> {
    let absolute_size = match value.trim().to_ascii_lowercase().as_str() {
        "xx-small" => Some(9.0),
        "x-small" => Some(10.0),
        "small" => Some(13.0),
        "medium" => Some(16.0),
        "large" => Some(18.0),
        "x-large" => Some(24.0),
        "xx-large" => Some(32.0),
        "xxx-large" => Some(48.0),
        _ => None,
    };
    absolute_size.or_else(|| parse_length(value, relative_to))
}

#[cfg(test)]
#[path = "css_style_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "css_style_matrix_tests.rs"]
mod matrix_tests;

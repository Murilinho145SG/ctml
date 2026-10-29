use cosmic_text::{Stretch, Style, Weight};
use winit::window::CursorIcon;

use crate::style::{FontVariantPosition, TextStyle, TextTransform};
use ctml_css::{
    parse_color, parse_color_with_opacity, parse_length, parse_nonnegative_length,
    parse_signed_length_percentage,
};

use super::font_variant::{
    apply_font_variant, apply_font_variant_caps, apply_font_variant_ligatures,
    apply_font_variant_numeric,
};
use cursor::parse_cursor;
pub(super) use font::parse_font_size;
use font::{apply_font_shorthand, inherit_font, reset_font};
use text_decoration::{
    apply_text_decoration_line, apply_text_decoration_shorthand, copy_text_decoration_lines,
    inherit_text_decoration, reset_text_decoration,
};

mod cursor;
mod font;
mod text_decoration;

pub(super) fn apply_color_declaration(
    style: &mut TextStyle,
    value: &str,
    inherited_color: ([u8; 4], f32),
) -> bool {
    let value = value.trim();
    let color = if value.eq_ignore_ascii_case("currentcolor") {
        Some(inherited_color)
    } else if matches!(value.to_ascii_lowercase().as_str(), "initial" | "revert") {
        Some(([0, 0, 0, 255], 1.0))
    } else if matches!(value.to_ascii_lowercase().as_str(), "inherit" | "unset") {
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
        true
    } else {
        false
    }
}

pub(super) fn apply_declaration_with_inherited(
    style: &mut TextStyle,
    property: &str,
    value: &str,
    inherited: &TextStyle,
) {
    let Some(keyword) = css_wide_keyword(value) else {
        apply_declaration(style, property, value);
        return;
    };
    let inherit_value =
        keyword == "inherit" || (keyword == "unset" && is_inherited_property(property));
    if inherit_value {
        inherit_property(style, inherited, property);
    } else {
        initial_property(style, property);
    }
}

fn css_wide_keyword(value: &str) -> Option<&str> {
    let value = value.trim();
    ["inherit", "initial", "unset", "revert"]
        .into_iter()
        .find(|keyword| value.eq_ignore_ascii_case(keyword))
}

fn is_inherited_property(property: &str) -> bool {
    matches!(
        property,
        "font"
            | "font-family"
            | "font-size"
            | "font-weight"
            | "font-style"
            | "font-stretch"
            | "font-kerning"
            | "font-variant"
            | "font-variant-caps"
            | "font-variant-ligatures"
            | "font-variant-position"
            | "font-variant-numeric"
            | "line-height"
            | "letter-spacing"
            | "text-transform"
            | "text-align"
            | "text-decoration-skip-ink"
            | "text-underline-offset"
            | "cursor"
    )
}

fn inherit_property(style: &mut TextStyle, inherited: &TextStyle, property: &str) {
    match property {
        "background-color" => style.background = inherited.background,
        "font" => inherit_font(style, inherited),
        "font-family" => style.family.clone_from(&inherited.family),
        "font-size" => {
            style.font_size = inherited.font_size;
            style.font_scale = inherited.font_scale;
            style.line_height = inherited.line_height;
            style.line_height_multiplier = inherited.line_height_multiplier;
            style.line_height_normal = inherited.line_height_normal;
        }
        "font-weight" => style.weight = inherited.weight,
        "font-style" => style.font_style = inherited.font_style,
        "font-stretch" => style.stretch = inherited.stretch,
        "font-kerning" => style.kerning = inherited.kerning,
        "font-variant" => {
            style.font_variant_caps = inherited.font_variant_caps;
            style.font_variant_position = inherited.font_variant_position;
            style.common_ligatures = inherited.common_ligatures;
            style.discretionary_ligatures = inherited.discretionary_ligatures;
            style.historical_ligatures = inherited.historical_ligatures;
            style.contextual_alternates = inherited.contextual_alternates;
            style
                .font_variant_numeric
                .clone_from(&inherited.font_variant_numeric);
        }
        "font-variant-caps" => style.font_variant_caps = inherited.font_variant_caps,
        "font-variant-ligatures" => {
            style.common_ligatures = inherited.common_ligatures;
            style.discretionary_ligatures = inherited.discretionary_ligatures;
            style.historical_ligatures = inherited.historical_ligatures;
            style.contextual_alternates = inherited.contextual_alternates;
        }
        "font-variant-position" => style.font_variant_position = inherited.font_variant_position,
        "font-variant-numeric" => style
            .font_variant_numeric
            .clone_from(&inherited.font_variant_numeric),
        "line-height" => {
            style.line_height = inherited.line_height;
            style.line_height_multiplier = inherited.line_height_multiplier;
            style.line_height_normal = inherited.line_height_normal;
        }
        "letter-spacing" => {
            style.letter_spacing = inherited.letter_spacing;
            style.letter_spacing_px = inherited.letter_spacing_px;
        }
        "text-transform" => style.text_transform = inherited.text_transform,
        "text-align" => style.text_align = inherited.text_align,
        "text-decoration" => {
            inherit_text_decoration(style, inherited);
        }
        "text-decoration-line" => copy_text_decoration_lines(style, inherited),
        "text-decoration-color" => style.text_decoration_color = inherited.text_decoration_color,
        "text-decoration-skip-ink" => style.decoration_skip_ink = inherited.decoration_skip_ink,
        "text-decoration-thickness" => style.decoration_thickness = inherited.decoration_thickness,
        "text-underline-offset" => style.underline_offset = inherited.underline_offset,
        "cursor" => {
            style.cursor = inherited.cursor;
            style.cursor_hidden = inherited.cursor_hidden;
        }
        _ => {}
    }
}

fn initial_property(style: &mut TextStyle, property: &str) {
    match property {
        "background-color" => style.background = [255, 255, 255],
        "font" => reset_font(style),
        "font-family" => style.family = "serif".to_owned(),
        "font-size" => {
            style.font_size = 16.0;
            if let Some(multiplier) = style.line_height_multiplier {
                style.line_height = style.font_size * multiplier;
            }
        }
        "font-weight" => style.weight = Weight::NORMAL,
        "font-style" => style.font_style = Style::Normal,
        "font-stretch" => style.stretch = Stretch::Normal,
        "font-kerning" => style.kerning = None,
        "font-variant" => {
            style.font_variant_caps = crate::style::FontVariantCaps::Normal;
            style.font_variant_position = FontVariantPosition::Normal;
            style.common_ligatures = None;
            style.discretionary_ligatures = None;
            style.historical_ligatures = None;
            style.contextual_alternates = None;
            style.font_variant_numeric.clear();
        }
        "font-variant-caps" => style.font_variant_caps = crate::style::FontVariantCaps::Normal,
        "font-variant-ligatures" => {
            style.common_ligatures = None;
            style.discretionary_ligatures = None;
            style.historical_ligatures = None;
            style.contextual_alternates = None;
        }
        "font-variant-position" => style.font_variant_position = FontVariantPosition::Normal,
        "font-variant-numeric" => style.font_variant_numeric.clear(),
        "line-height" => {
            style.line_height_normal = true;
            style.line_height_multiplier = None;
        }
        "letter-spacing" => {
            style.letter_spacing = 0.0;
            style.letter_spacing_px = false;
        }
        "text-transform" => style.text_transform = TextTransform::None,
        "text-align" => style.text_align = cosmic_text::Align::Left,
        "text-decoration" | "text-decoration-line" => {
            if property == "text-decoration" {
                reset_text_decoration(style);
            } else {
                apply_text_decoration_line(style, "none");
            }
        }
        "text-decoration-style" => {
            style.decoration_style = crate::style::TextDecorationStyle::Solid
        }
        "text-decoration-color" => style.text_decoration_color = None,
        "text-decoration-skip-ink" => {
            style.decoration_skip_ink = crate::style::TextDecorationSkipInk::Auto;
        }
        "text-decoration-thickness" => style.decoration_thickness = None,
        "text-underline-offset" => style.underline_offset = None,
        "cursor" => {
            style.cursor = CursorIcon::Text;
            style.cursor_hidden = false;
        }
        _ => {}
    }
}

fn apply_declaration(style: &mut TextStyle, property: &str, value: &str) {
    let value = value.trim();
    match property {
        "font" => apply_font_shorthand(style, value),
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
            if value.eq_ignore_ascii_case("normal") {
                style.line_height_normal = true;
                style.line_height_multiplier = None;
            } else if let Ok(multiplier) = value.parse::<f32>() {
                style.line_height = style.font_size * multiplier;
                style.line_height_multiplier = Some(multiplier);
                style.line_height_normal = false;
            } else if let Some(height) = parse_length(value, style.font_size) {
                style.line_height = height;
                style.line_height_multiplier = None;
                style.line_height_normal = false;
            }
        }
        "font-family" => {
            style.family = value.to_owned();
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
        "font-variant" => apply_font_variant(style, value),
        "font-variant-caps" => apply_font_variant_caps(style, value),
        "font-variant-numeric" => apply_font_variant_numeric(style, value),
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
        "text-decoration" => {
            apply_text_decoration_shorthand(style, value);
        }
        "text-decoration-line" => apply_text_decoration_line(style, value),
        "text-decoration-style" => {
            style.decoration_style = match value.to_ascii_lowercase().as_str() {
                "solid" => crate::style::TextDecorationStyle::Solid,
                "double" => crate::style::TextDecorationStyle::Double,
                "dotted" => crate::style::TextDecorationStyle::Dotted,
                "dashed" => crate::style::TextDecorationStyle::Dashed,
                "wavy" => crate::style::TextDecorationStyle::Wavy,
                _ => style.decoration_style,
            };
        }
        "text-decoration-skip-ink" => {
            style.decoration_skip_ink = match value.to_ascii_lowercase().as_str() {
                "auto" => crate::style::TextDecorationSkipInk::Auto,
                "none" => crate::style::TextDecorationSkipInk::None,
                "all" => crate::style::TextDecorationSkipInk::All,
                _ => style.decoration_skip_ink,
            };
        }
        "text-decoration-thickness" => {
            if matches!(value.to_ascii_lowercase().as_str(), "auto" | "from-font") {
                style.decoration_thickness = None;
            } else if let Some(thickness) = parse_nonnegative_length(value, style.font_size) {
                style.decoration_thickness = Some(thickness);
            }
        }
        "text-underline-offset" => {
            if value.eq_ignore_ascii_case("auto") {
                style.underline_offset = None;
            } else if let Some(offset) = parse_signed_length_percentage(value, style.font_size) {
                style.underline_offset = Some(offset);
            }
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

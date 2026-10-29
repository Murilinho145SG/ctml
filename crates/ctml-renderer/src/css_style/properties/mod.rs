use cosmic_text::{Stretch, Style, Weight};
use winit::window::CursorIcon;

use crate::style::{FontVariantPosition, TextStyle, TextTransform};
use ctml_css::parse_nonnegative_font_relative_length_with_viewport;
use ctml_css::parse_signed_font_relative_length_with_viewport;

use super::font_variant::{
    apply_font_variant, apply_font_variant_caps, apply_font_variant_east_asian,
    apply_font_variant_ligatures, apply_font_variant_numeric,
};
use background::{apply_background_color, apply_background_shorthand};
pub(super) use color::apply_color_declaration;
use cursor::parse_cursor;
use display::{apply_display, inherit_display, reset_display, revert_display};
#[cfg(test)]
pub(super) use font::parse_font_size;
use font::{
    apply_font_shorthand, inherit_font, parse_font_size_with_font_units, parse_font_stretch,
    relative_font_weight, reset_font,
};
use font_feature_settings::apply_font_feature_settings;
use list_style::{apply_list_style_shorthand, apply_list_style_type, reset_list_style_type};
use size::{
    apply_box_sizing, apply_height, apply_max_height, apply_max_width, apply_min_height,
    apply_min_width, apply_width, inherit_width,
};
use spacing::{apply_letter_spacing, apply_word_spacing};
use text_decoration::{
    apply_text_decoration_line, apply_text_decoration_property, copy_text_decoration_lines,
    inherit_text_decoration, reset_text_decoration,
};
use text_emphasis::{apply_text_emphasis_property, inherit_text_emphasis, reset_text_emphasis};
use text_shadow::apply_text_shadow;

mod background;
mod color;
mod cursor;
mod display;
mod font;
mod font_feature_settings;
mod list_style;
mod size;
mod spacing;
mod text_decoration;
mod text_emphasis;
mod text_shadow;

pub(super) fn apply_declaration_with_inherited(
    style: &mut TextStyle,
    property: &str,
    value: &str,
    inherited: &TextStyle,
) {
    let Some(keyword) = css_wide_keyword(value) else {
        apply_declaration(
            style,
            property,
            value,
            inherited.line_height,
            inherited.font_relative_units,
        );
        return;
    };
    let inherit_value =
        keyword == "inherit" || (keyword == "unset" && is_inherited_property(property));
    if inherit_value {
        inherit_property(style, inherited, property);
    } else if keyword == "revert" && property == "display" {
        revert_display(style);
    } else if keyword == "revert" && matches!(property, "list-style" | "list-style-type") {
        style.list_style_type = None;
    } else if keyword == "revert" && matches!(property, "width" | "inline-size") {
        style.width = None;
    } else if keyword == "revert" && matches!(property, "min-width" | "min-inline-size") {
        style.min_width = None;
    } else if keyword == "revert" && matches!(property, "max-width" | "max-inline-size") {
        style.max_width = None;
    } else if keyword == "revert" && matches!(property, "height" | "block-size") {
        style.height = None;
    } else if keyword == "revert" && matches!(property, "min-height" | "min-block-size") {
        style.min_height = None;
    } else if keyword == "revert" && matches!(property, "max-height" | "max-block-size") {
        style.max_height = None;
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
            | "font-variant-east-asian"
            | "font-feature-settings"
            | "list-style"
            | "list-style-type"
            | "text-shadow"
            | "text-emphasis"
            | "text-emphasis-style"
            | "text-emphasis-color"
            | "text-emphasis-position"
            | "line-height"
            | "letter-spacing"
            | "word-spacing"
            | "text-transform"
            | "text-align"
            | "text-decoration-skip-ink"
            | "text-underline-offset"
            | "cursor"
    )
}

fn inherit_property(style: &mut TextStyle, inherited: &TextStyle, property: &str) {
    match property {
        "opacity" => style.opacity = inherited.opacity,
        "background-color" => {
            style.background = inherited.background;
            style.background_color = inherited.background_color;
        }
        "display" => inherit_display(style, inherited),
        "width" | "inline-size" => inherit_width(style, inherited),
        "box-sizing" => style.border_box = inherited.border_box,
        "min-width" | "min-inline-size" => style.min_width.clone_from(&inherited.min_width),
        "max-width" | "max-inline-size" => style.max_width.clone_from(&inherited.max_width),
        "height" | "block-size" => style.height.clone_from(&inherited.height),
        "min-height" | "min-block-size" => {
            style.min_height.clone_from(&inherited.min_height);
        }
        "max-height" | "max-block-size" => {
            style.max_height.clone_from(&inherited.max_height);
        }
        "list-style" | "list-style-type" => {
            style.list_style_type.clone_from(&inherited.list_style_type);
        }
        "background" => {
            style.background = inherited.background;
            style.background_color = inherited.background_color;
        }
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
            style
                .font_variant_east_asian
                .clone_from(&inherited.font_variant_east_asian);
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
        "font-variant-east-asian" => style
            .font_variant_east_asian
            .clone_from(&inherited.font_variant_east_asian),
        "font-feature-settings" => style
            .font_feature_settings
            .clone_from(&inherited.font_feature_settings),
        "text-shadow" => style.text_shadows.clone_from(&inherited.text_shadows),
        "text-emphasis"
        | "text-emphasis-style"
        | "text-emphasis-color"
        | "text-emphasis-position" => inherit_text_emphasis(style, inherited, property),
        "line-height" => {
            style.line_height = inherited.line_height;
            style.line_height_multiplier = inherited.line_height_multiplier;
            style.line_height_normal = inherited.line_height_normal;
        }
        "vertical-align" => style.vertical_align_offset = inherited.vertical_align_offset,
        "letter-spacing" => {
            style.letter_spacing = inherited.letter_spacing;
            style.letter_spacing_px = inherited.letter_spacing_px;
        }
        "word-spacing" => {
            style.word_spacing_px = inherited.word_spacing_px;
            style.word_spacing_percent = inherited.word_spacing_percent;
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
        "opacity" => style.opacity = 1.0,
        "background-color" => {
            style.background = [255, 255, 255];
            style.background_color = None;
        }
        "display" => reset_display(style),
        "width" | "inline-size" => style.width = None,
        "box-sizing" => style.border_box = false,
        "min-width" | "min-inline-size" => style.min_width = None,
        "max-width" | "max-inline-size" => style.max_width = None,
        "height" | "block-size" => style.height = None,
        "min-height" | "min-block-size" => style.min_height = None,
        "max-height" | "max-block-size" => style.max_height = None,
        "list-style" | "list-style-type" => reset_list_style_type(style),
        "background" => {
            style.background = [255, 255, 255];
            style.background_color = None;
        }
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
            style.font_variant_east_asian.clear();
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
        "font-variant-east-asian" => style.font_variant_east_asian.clear(),
        "font-feature-settings" => style.font_feature_settings.clear(),
        "text-shadow" => style.text_shadows.clear(),
        "text-emphasis"
        | "text-emphasis-style"
        | "text-emphasis-color"
        | "text-emphasis-position" => reset_text_emphasis(style, property),
        "line-height" => {
            style.line_height_normal = true;
            style.line_height_multiplier = None;
        }
        "vertical-align" => style.vertical_align_offset = 0.0,
        "letter-spacing" => {
            style.letter_spacing = 0.0;
            style.letter_spacing_px = false;
        }
        "word-spacing" => {
            style.word_spacing_px = 0.0;
            style.word_spacing_percent = None;
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

fn apply_declaration(
    style: &mut TextStyle,
    property: &str,
    value: &str,
    parent_line_height: f32,
    parent_font_units: ctml_css::FontRelativeUnits,
) {
    let value = value.trim();
    match property {
        "opacity" => {
            let value = value.trim();
            let opacity = ctml_css::parse_math_number(value).or_else(|| {
                if let Some(percent) = value.strip_suffix('%') {
                    percent
                        .trim()
                        .parse::<f32>()
                        .ok()
                        .map(|value| value / 100.0)
                } else {
                    value.parse::<f32>().ok()
                }
            });
            if let Some(opacity) = opacity.filter(|opacity| opacity.is_finite()) {
                style.opacity = opacity.clamp(0.0, 1.0);
            }
        }
        "font" => apply_font_shorthand(style, value, parent_line_height, parent_font_units),
        "display" => apply_display(style, value),
        "width" | "inline-size" => apply_width(style, value),
        "box-sizing" => apply_box_sizing(style, value),
        "min-width" | "min-inline-size" => apply_min_width(style, value),
        "max-width" | "max-inline-size" => apply_max_width(style, value),
        "height" | "block-size" => apply_height(style, value),
        "min-height" | "min-block-size" => apply_min_height(style, value),
        "max-height" | "max-block-size" => apply_max_height(style, value),
        "list-style-type" => apply_list_style_type(style, value),
        "list-style" => apply_list_style_shorthand(style, value),
        "background" => apply_background_shorthand(style, value),
        "background-color" => apply_background_color(style, value),
        "font-size" => {
            if let Some(size) = parse_font_size_with_font_units(
                value,
                style.font_size,
                parent_line_height,
                parent_font_units,
                style.viewport_size,
            ) {
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
            } else if let Some(multiplier) =
                ctml_css::parse_math_number(value).or_else(|| value.parse::<f32>().ok())
            {
                style.line_height = style.font_size * multiplier;
                style.line_height_multiplier = Some(multiplier);
                style.line_height_normal = false;
            } else if let Some(height) = parse_nonnegative_font_relative_length_with_viewport(
                value,
                style.font_size,
                parent_line_height,
                style.font_relative_units,
                style.viewport_size,
            ) {
                style.line_height = height;
                style.line_height_multiplier = None;
                style.line_height_normal = false;
            }
        }
        "vertical-align" => {
            style.vertical_align_offset = match value.to_ascii_lowercase().as_str() {
                "baseline" => 0.0,
                "sub" => -style.font_size * 0.2,
                "super" => style.font_size * 0.2,
                _ if value.ends_with('%') => value
                    .strip_suffix('%')
                    .and_then(|percent| percent.trim().parse::<f32>().ok())
                    .filter(|percent| percent.is_finite())
                    .map_or(style.vertical_align_offset, |percent| {
                        style.line_height * percent / 100.0
                    }),
                _ => parse_signed_font_relative_length_with_viewport(
                    value,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
                .unwrap_or(style.vertical_align_offset),
            };
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
                _ => ctml_css::parse_math_number(value)
                    .filter(|number| (1.0..=1000.0).contains(number))
                    .filter(|number| number.fract() == 0.0)
                    .map(|number| number as u16)
                    .or_else(|| value.parse::<u16>().ok())
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
        "font-variant-east-asian" => apply_font_variant_east_asian(style, value),
        "font-feature-settings" => apply_font_feature_settings(style, value),
        "text-shadow" => apply_text_shadow(style, value),
        "text-emphasis"
        | "text-emphasis-style"
        | "text-emphasis-color"
        | "text-emphasis-position" => apply_text_emphasis_property(style, property, value),
        "font-variant-position" => {
            style.font_variant_position = match value.to_ascii_lowercase().as_str() {
                "normal" => FontVariantPosition::Normal,
                "sub" => FontVariantPosition::Sub,
                "super" => FontVariantPosition::Super,
                _ => style.font_variant_position,
            };
        }
        "letter-spacing" => apply_letter_spacing(style, value),
        "word-spacing" => apply_word_spacing(style, value),
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
        "text-decoration"
        | "text-decoration-line"
        | "text-decoration-style"
        | "text-decoration-skip-ink"
        | "text-decoration-thickness"
        | "text-underline-offset"
        | "text-decoration-color" => apply_text_decoration_property(style, property, value),
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

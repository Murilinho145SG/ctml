use cosmic_text::{Align, Stretch, Style, Weight};
use winit::window::CursorIcon;

use crate::style::{
    Direction, FontVariantPosition, Hyphens, LineBreak, OverflowWrap, TextOrientation,
    TextOverflow, TextRendering, TextStyle, TextTransform, UnicodeBidi, UserSelect, WhiteSpace,
    WordBreak, WritingMode,
};
use ctml_css::parse_nonnegative_font_relative_length_with_viewport;
use ctml_css::parse_signed_font_relative_length_with_viewport;

use super::font_variant::{
    apply_font_variant, apply_font_variant_caps, apply_font_variant_east_asian,
    apply_font_variant_ligatures, apply_font_variant_numeric,
};
use background::{apply_background_color, apply_background_shorthand};
pub(super) use color::apply_color_declaration;
use counters::{CounterProperty, apply_counter_property};
use cursor::parse_cursor;
use display::{apply_display, inherit_display, reset_display, revert_display};
#[cfg(test)]
pub(super) use font::parse_font_size;
use font::{
    apply_font_shorthand, apply_font_style, inherit_font, parse_font_size_with_font_units,
    parse_font_stretch, relative_font_weight, reset_font,
};
use font_feature_settings::apply_font_feature_settings;
use font_variation_settings::apply_font_variation_settings;
use list_style::{apply_list_style_shorthand, apply_list_style_type, reset_list_style_type};
use quotes::apply_quotes;
use size::{
    apply_block_size, apply_box_sizing, apply_height, apply_inline_size, apply_max_block_size,
    apply_max_height, apply_max_inline_size, apply_max_width, apply_min_block_size,
    apply_min_height, apply_min_inline_size, apply_min_width, apply_width, inherit_inline_size,
    inherit_width,
};
use spacing::{apply_letter_spacing, apply_word_spacing};
use text_decoration::{
    apply_text_decoration_line, apply_text_decoration_property, copy_text_decoration_lines,
    inherit_text_decoration, reset_text_decoration,
};
use text_emphasis::{apply_text_emphasis_property, inherit_text_emphasis, reset_text_emphasis};
use text_indent::apply_text_indent;
use text_shadow::apply_text_shadow;

mod background;
mod color;
mod counters;
mod cursor;
mod display;
mod font;
mod font_feature_settings;
mod font_variation_settings;
mod list_style;
mod quotes;
mod size;
mod spacing;
mod text_decoration;
mod text_emphasis;
mod text_indent;
mod text_shadow;
use size::is_valid_size_declaration;

pub(super) fn apply_declaration_with_inherited(
    style: &mut TextStyle,
    property: &str,
    value: &str,
    inherited: &TextStyle,
) {
    let Some(keyword) = css_wide_keyword(value) else {
        if property == "user-select" && value.trim().eq_ignore_ascii_case("auto") {
            style.user_select_none = inherited.user_select_none;
            style.user_select_all_owner = inherited.user_select_all_owner;
        }
        if is_valid_size_declaration(style, property, value) {
            style.record_size_property_order(property);
        }
        apply_declaration(
            style,
            property,
            value,
            inherited.line_height,
            inherited.font_relative_units,
            inherited.font_size,
        );
        return;
    };
    if is_valid_size_declaration(style, property, value) || is_size_property(property) {
        style.record_size_property_order(property);
    }
    let inherit_value =
        keyword == "inherit" || (keyword == "unset" && is_inherited_property(property));
    if inherit_value {
        inherit_property(style, inherited, property);
    } else if keyword == "revert" && property == "display" {
        revert_display(style);
    } else if keyword == "revert" && matches!(property, "list-style" | "list-style-type") {
        style.list_style_type = None;
    } else if keyword == "revert" && property == "width" {
        style.width = None;
    } else if keyword == "revert" && property == "inline-size" {
        style.inline_size = None;
    } else if keyword == "revert" && property == "min-width" {
        style.min_width = None;
    } else if keyword == "revert" && property == "min-inline-size" {
        style.min_inline_size = None;
    } else if keyword == "revert" && property == "max-width" {
        style.max_width = None;
    } else if keyword == "revert" && property == "max-inline-size" {
        style.max_inline_size = None;
    } else if keyword == "revert" && property == "height" {
        style.height = None;
    } else if keyword == "revert" && property == "block-size" {
        style.block_size = None;
    } else if keyword == "revert" && property == "min-height" {
        style.min_height = None;
    } else if keyword == "revert" && property == "min-block-size" {
        style.min_block_size = None;
    } else if keyword == "revert" && property == "max-height" {
        style.max_height = None;
    } else if keyword == "revert" && property == "max-block-size" {
        style.max_block_size = None;
    } else {
        initial_property(style, property);
        if property == "user-select" {
            style.user_select_none = inherited.user_select_none;
            style.user_select_all_owner = inherited.user_select_all_owner;
        }
    }
}

fn is_size_property(property: &str) -> bool {
    matches!(
        property,
        "width"
            | "min-width"
            | "max-width"
            | "inline-size"
            | "min-inline-size"
            | "max-inline-size"
            | "height"
            | "min-height"
            | "max-height"
            | "block-size"
            | "min-block-size"
            | "max-block-size"
    )
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
            | "font-optical-sizing"
            | "font-variation-settings"
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
            | "text-align-last"
            | "text-justify"
            | "text-indent"
            | "white-space"
            | "overflow-wrap"
            | "word-break"
            | "line-break"
            | "hyphens"
            | "text-rendering"
            | "text-orientation"
            | "writing-mode"
            | "visibility"
            | "pointer-events"
            | "direction"
            | "quotes"
            | "text-decoration-skip-ink"
            | "text-underline-offset"
            | "cursor"
    )
}

fn inherit_property(style: &mut TextStyle, inherited: &TextStyle, property: &str) {
    match property {
        "opacity" => style.opacity = inherited.opacity,
        "visibility" => style.visibility_hidden = inherited.visibility_hidden,
        "pointer-events" => style.pointer_events_none = inherited.pointer_events_none,
        "user-select" => {
            style.user_select = inherited.user_select;
            style.user_select_none = inherited.user_select_none;
            style.user_select_all_owner = inherited.user_select_all_owner;
        }
        "background-color" => {
            style.background = inherited.background;
            style.background_color = inherited.background_color;
        }
        "display" => inherit_display(style, inherited),
        "width" => inherit_width(style, inherited),
        "inline-size" => inherit_inline_size(style, inherited),
        "box-sizing" => style.border_box = inherited.border_box,
        "min-width" => style.min_width.clone_from(&inherited.min_width),
        "min-inline-size" => style.min_inline_size.clone_from(&inherited.min_inline_size),
        "max-width" => style.max_width.clone_from(&inherited.max_width),
        "max-inline-size" => style.max_inline_size.clone_from(&inherited.max_inline_size),
        "height" => style.height.clone_from(&inherited.height),
        "block-size" => style.block_size.clone_from(&inherited.block_size),
        "min-height" => style.min_height.clone_from(&inherited.min_height),
        "min-block-size" => style.min_block_size.clone_from(&inherited.min_block_size),
        "max-height" => style.max_height.clone_from(&inherited.max_height),
        "max-block-size" => style.max_block_size.clone_from(&inherited.max_block_size),
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
        "font-style" => {
            style.font_style = inherited.font_style;
            style.font_oblique_angle = inherited.font_oblique_angle;
        }
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
        "font-optical-sizing" => style.font_optical_sizing = inherited.font_optical_sizing,
        "font-variation-settings" => style
            .font_variation_settings
            .clone_from(&inherited.font_variation_settings),
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
        "text-align-last" => style.text_align_last = inherited.text_align_last,
        "text-justify" => style.text_justify = inherited.text_justify,
        "white-space" => style.white_space = inherited.white_space,
        "overflow-wrap" => style.overflow_wrap = inherited.overflow_wrap,
        "word-break" => style.word_break = inherited.word_break,
        "line-break" => style.line_break = inherited.line_break,
        "hyphens" => style.hyphens = inherited.hyphens,
        "text-rendering" => style.text_rendering = inherited.text_rendering,
        "text-orientation" => style.text_orientation = inherited.text_orientation,
        "writing-mode" => style.writing_mode = inherited.writing_mode,
        "direction" => style.direction = inherited.direction,
        "unicode-bidi" => style.unicode_bidi = inherited.unicode_bidi,
        "quotes" => style.quotes = inherited.quotes.clone(),
        "text-indent" => {
            style.text_indent = inherited.text_indent;
            style.text_indent_percent = inherited.text_indent_percent;
            style.text_indent_hanging = inherited.text_indent_hanging;
            style.text_indent_each_line = inherited.text_indent_each_line;
        }
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
        "visibility" => style.visibility_hidden = false,
        "pointer-events" => style.pointer_events_none = false,
        "user-select" => {
            style.user_select = UserSelect::Auto;
            style.user_select_none = false;
            style.user_select_all_owner = None;
        }
        "background-color" => {
            style.background = [255, 255, 255];
            style.background_color = None;
        }
        "display" => reset_display(style),
        "width" => style.width = None,
        "inline-size" => style.inline_size = None,
        "box-sizing" => style.border_box = false,
        "min-width" => style.min_width = None,
        "min-inline-size" => style.min_inline_size = None,
        "max-width" => style.max_width = None,
        "max-inline-size" => style.max_inline_size = None,
        "height" => style.height = None,
        "block-size" => style.block_size = None,
        "min-height" => style.min_height = None,
        "min-block-size" => style.min_block_size = None,
        "max-height" => style.max_height = None,
        "max-block-size" => style.max_block_size = None,
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
        "font-style" => {
            style.font_style = Style::Normal;
            style.font_oblique_angle = None;
        }
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
        "font-optical-sizing" => style.font_optical_sizing = true,
        "font-variation-settings" => style.font_variation_settings.clear(),
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
        "text-align-last" => style.text_align_last = None,
        "text-justify" => style.text_justify = cosmic_text::TextJustify::Auto,
        "white-space" => style.white_space = WhiteSpace::Normal,
        "overflow-wrap" => style.overflow_wrap = OverflowWrap::Normal,
        "word-break" => style.word_break = WordBreak::Normal,
        "line-break" => style.line_break = LineBreak::Auto,
        "hyphens" => style.hyphens = Hyphens::Manual,
        "text-rendering" => style.text_rendering = TextRendering::Auto,
        "text-orientation" => style.text_orientation = TextOrientation::Mixed,
        "writing-mode" => style.writing_mode = WritingMode::HorizontalTb,
        "direction" => style.direction = Direction::Ltr,
        "unicode-bidi" => style.unicode_bidi = UnicodeBidi::Normal,
        "quotes" => style.quotes = quotes::default_quotes(),
        "counter-reset" => style.counter_reset.clear(),
        "counter-increment" => style.counter_increment.clear(),
        "counter-set" => style.counter_set.clear(),
        "text-overflow" => style.text_overflow = TextOverflow::Clip,
        "line-clamp" => style.line_clamp = None,
        "text-indent" => {
            style.text_indent = 0.0;
            style.text_indent_percent = 0.0;
            style.text_indent_hanging = false;
            style.text_indent_each_line = false;
        }
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
    parent_font_size: f32,
) {
    let value = value.trim();
    match property {
        "opacity" => {
            let opacity = ctml_css::parse_math_number(value)
                .or_else(|| ctml_css::parse_css_number(value))
                .or_else(|| ctml_css::parse_css_percentage(value).map(|value| value / 100.0));
            if let Some(opacity) = opacity.filter(|opacity| opacity.is_finite()) {
                style.opacity = opacity.clamp(0.0, 1.0);
            }
        }
        "visibility" => match value.to_ascii_lowercase().as_str() {
            "visible" => style.visibility_hidden = false,
            "hidden" | "collapse" => style.visibility_hidden = true,
            _ => {}
        },
        "pointer-events" => match value.to_ascii_lowercase().as_str() {
            "auto" => style.pointer_events_none = false,
            "none" => style.pointer_events_none = true,
            _ => {}
        },
        "user-select" => match value.to_ascii_lowercase().as_str() {
            "auto" => style.user_select = UserSelect::Auto,
            "text" => {
                style.user_select = UserSelect::Text;
                style.user_select_none = false;
            }
            "none" => {
                style.user_select = UserSelect::None;
                style.user_select_none = true;
            }
            "all" => {
                style.user_select = UserSelect::All;
                style.user_select_none = false;
            }
            _ => {}
        },
        "font" => apply_font_shorthand(style, value, parent_line_height, parent_font_units),
        "display" => apply_display(style, value),
        "width" => apply_width(style, value),
        "inline-size" => apply_inline_size(style, value),
        "box-sizing" => apply_box_sizing(style, value),
        "min-width" => apply_min_width(style, value),
        "min-inline-size" => apply_min_inline_size(style, value),
        "max-width" => apply_max_width(style, value),
        "max-inline-size" => apply_max_inline_size(style, value),
        "height" => apply_height(style, value),
        "block-size" => apply_block_size(style, value),
        "min-height" => apply_min_height(style, value),
        "min-block-size" => apply_min_block_size(style, value),
        "max-height" => apply_max_height(style, value),
        "max-block-size" => apply_max_block_size(style, value),
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
                ctml_css::parse_math_number(value).or_else(|| ctml_css::parse_css_number(value))
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
                "sub" => -parent_font_size * 0.2,
                "super" => parent_font_size / 3.0,
                "middle" => parent_font_units.ex * parent_font_size / 2.0,
                "text-top" => {
                    parent_font_units.cap * parent_font_size
                        - style.font_relative_units.cap * style.font_size
                }
                "text-bottom" => (style.font_size - parent_font_size) * 0.2,
                "top" => (parent_line_height - style.line_height) / 2.0,
                "bottom" => (style.line_height - parent_line_height) / 2.0,
                _ if value.ends_with('%') => ctml_css::parse_css_percentage(value)
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
                    .or_else(|| ctml_css::parse_css_number(value))
                    .filter(|number| (1.0..=1000.0).contains(number))
                    .filter(|number| number.fract() == 0.0)
                    .map(|number| number as u16)
                    .filter(|weight| (1..=1000).contains(weight))
                    .map_or(style.weight, Weight),
            };
        }
        "font-style" => apply_font_style(style, value),
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
        "font-optical-sizing" => match value.to_ascii_lowercase().as_str() {
            "auto" => style.font_optical_sizing = true,
            "none" => style.font_optical_sizing = false,
            _ => {}
        },
        "font-variation-settings" => apply_font_variation_settings(style, value),
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
                "left" => cosmic_text::Align::Left,
                "start" => cosmic_text::Align::Start,
                "right" => cosmic_text::Align::Right,
                "end" => cosmic_text::Align::End,
                "center" => cosmic_text::Align::Center,
                "justify" => cosmic_text::Align::Justified,
                _ => style.text_align,
            };
        }
        "text-align-last" => {
            style.text_align_last = match value.to_ascii_lowercase().as_str() {
                "auto" => None,
                "left" => Some(Align::Left),
                "start" => Some(Align::Start),
                "right" => Some(Align::Right),
                "end" => Some(Align::End),
                "center" => Some(Align::Center),
                "justify" => Some(Align::JustifiedLast),
                _ => style.text_align_last,
            };
        }
        "text-justify" => {
            style.text_justify = match value.to_ascii_lowercase().as_str() {
                "auto" => cosmic_text::TextJustify::Auto,
                "none" => cosmic_text::TextJustify::None,
                "inter-word" => cosmic_text::TextJustify::InterWord,
                "inter-character" | "distribute" => cosmic_text::TextJustify::InterCharacter,
                _ => style.text_justify,
            };
        }
        "white-space" => {
            style.white_space = match value.to_ascii_lowercase().as_str() {
                "normal" => WhiteSpace::Normal,
                "nowrap" => WhiteSpace::NoWrap,
                "pre" => WhiteSpace::Pre,
                "pre-wrap" => WhiteSpace::PreWrap,
                "pre-line" => WhiteSpace::PreLine,
                "break-spaces" => WhiteSpace::BreakSpaces,
                _ => style.white_space,
            };
        }
        "overflow-wrap" => {
            style.overflow_wrap = match value.to_ascii_lowercase().as_str() {
                "normal" => OverflowWrap::Normal,
                "break-word" => OverflowWrap::BreakWord,
                "anywhere" => OverflowWrap::Anywhere,
                _ => style.overflow_wrap,
            };
        }
        "word-break" => {
            style.word_break = match value.to_ascii_lowercase().as_str() {
                "normal" => WordBreak::Normal,
                "break-all" => WordBreak::BreakAll,
                "keep-all" => WordBreak::KeepAll,
                "break-word" => WordBreak::BreakWord,
                _ => style.word_break,
            };
        }
        "line-break" => {
            style.line_break = match value.to_ascii_lowercase().as_str() {
                "auto" => LineBreak::Auto,
                "loose" => LineBreak::Loose,
                "normal" => LineBreak::Normal,
                "strict" => LineBreak::Strict,
                "anywhere" => LineBreak::Anywhere,
                _ => style.line_break,
            };
        }
        "hyphens" => {
            style.hyphens = match value.to_ascii_lowercase().as_str() {
                "none" => Hyphens::None,
                "manual" => Hyphens::Manual,
                "auto" => Hyphens::Auto,
                _ => style.hyphens,
            };
        }
        "text-rendering" => {
            style.text_rendering = match value.to_ascii_lowercase().as_str() {
                "auto" => TextRendering::Auto,
                "optimizespeed" => TextRendering::OptimizeSpeed,
                "optimizelegibility" => TextRendering::OptimizeLegibility,
                "geometricprecision" => TextRendering::GeometricPrecision,
                _ => style.text_rendering,
            };
        }
        "text-orientation" => {
            style.text_orientation = match value.to_ascii_lowercase().as_str() {
                "mixed" => TextOrientation::Mixed,
                "upright" => TextOrientation::Upright,
                "sideways" => TextOrientation::Sideways,
                _ => style.text_orientation,
            };
        }
        "writing-mode" => {
            style.writing_mode = match value.to_ascii_lowercase().as_str() {
                "horizontal-tb" => WritingMode::HorizontalTb,
                "vertical-rl" => WritingMode::VerticalRl,
                "vertical-lr" => WritingMode::VerticalLr,
                _ => style.writing_mode,
            }
        }
        "direction" => {
            style.direction = match value.to_ascii_lowercase().as_str() {
                "ltr" => Direction::Ltr,
                "rtl" => Direction::Rtl,
                _ => style.direction,
            };
        }
        "unicode-bidi" => {
            style.unicode_bidi = match value.to_ascii_lowercase().as_str() {
                "normal" => UnicodeBidi::Normal,
                "isolate" => UnicodeBidi::Isolate,
                _ => style.unicode_bidi,
            };
        }
        "quotes" => apply_quotes(style, value),
        "counter-reset" => apply_counter_property(style, CounterProperty::Reset, value),
        "counter-increment" => apply_counter_property(style, CounterProperty::Increment, value),
        "counter-set" => apply_counter_property(style, CounterProperty::Set, value),
        "text-overflow" => {
            style.text_overflow = match value.to_ascii_lowercase().as_str() {
                "clip" => TextOverflow::Clip,
                "ellipsis" => TextOverflow::Ellipsis,
                _ => style.text_overflow,
            };
        }
        "line-clamp" => {
            style.line_clamp = if value.eq_ignore_ascii_case("none") {
                None
            } else {
                value
                    .parse::<usize>()
                    .ok()
                    .filter(|lines| *lines > 0)
                    .or(style.line_clamp)
            };
        }
        "text-indent" => apply_text_indent(style, value),
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

use cosmic_text::{Align, Stretch, Style, Weight};
use winit::window::CursorIcon;

use crate::style::{
    Direction, FontVariantPosition, Hyphens, LineBreak, Overflow, OverflowWrap, TextOrientation,
    TextOverflow, TextRendering, TextStyle, TextTransform, TextWrapMode, UnicodeBidi, UserSelect,
    WhiteSpace, WordBreak, WritingMode,
};
use ctml_css::parse_nonnegative_font_relative_length_with_viewport;
use ctml_css::parse_signed_font_relative_length_with_viewport;

use super::font_variant::{
    apply_font_variant, apply_font_variant_caps, apply_font_variant_east_asian,
    apply_font_variant_ligatures, apply_font_variant_numeric,
};
use apply::apply_declaration;
use background::{apply_background_color, apply_background_shorthand};
use border::{
    apply_border_color, apply_border_color_side, apply_border_shorthand, apply_border_side,
    apply_border_style, apply_border_style_side, apply_border_width, apply_border_width_side,
    apply_logical_border_axis_shorthand, apply_logical_border_color,
    apply_logical_border_color_shorthand, apply_logical_border_shorthand,
    apply_logical_border_style, apply_logical_border_style_shorthand, apply_logical_border_width,
    apply_logical_border_width_shorthand,
};
use border_radius::{apply_border_radius, apply_corner_radius};
use cascade_values::{inherit_property, initial_property};
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
use outline::{
    apply_outline, apply_outline_color, apply_outline_offset, apply_outline_style,
    apply_outline_width,
};
use quotes::apply_quotes;
use size::{
    apply_block_size, apply_box_sizing, apply_height, apply_inline_size, apply_max_block_size,
    apply_max_height, apply_max_inline_size, apply_max_width, apply_min_block_size,
    apply_min_height, apply_min_inline_size, apply_min_width, apply_width, inherit_inline_size,
    inherit_width,
};
use spacing::{
    apply_letter_spacing, apply_logical_margin, apply_logical_margin_shorthand,
    apply_logical_padding, apply_logical_padding_shorthand, apply_margin, apply_margin_side,
    apply_padding, apply_padding_side, apply_word_spacing, inherit_margin_property,
    is_margin_property, reset_margin_property,
};
use text_decoration::{
    apply_text_decoration_line, apply_text_decoration_property, copy_text_decoration_lines,
    inherit_text_decoration, reset_text_decoration,
};
use text_emphasis::{apply_text_emphasis_property, inherit_text_emphasis, reset_text_emphasis};
use text_indent::apply_text_indent;
use text_shadow::apply_text_shadow;

mod apply;
mod background;
mod border;
mod border_cascade;
mod border_radius;
mod cascade_values;
mod color;
mod counters;
mod cursor;
mod display;
mod font;
mod font_feature_settings;
mod font_variation_settings;
mod list_style;
mod outline;
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
            | "padding"
            | "padding-top"
            | "padding-right"
            | "padding-bottom"
            | "padding-left"
            | "padding-inline"
            | "padding-inline-start"
            | "padding-inline-end"
            | "padding-block"
            | "padding-block-start"
            | "padding-block-end"
            | "margin"
            | "margin-top"
            | "margin-right"
            | "margin-bottom"
            | "margin-left"
            | "margin-inline"
            | "margin-inline-start"
            | "margin-inline-end"
            | "margin-block"
            | "margin-block-start"
            | "margin-block-end"
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
            | "text-wrap"
            | "text-wrap-mode"
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

pub(crate) fn resolve_logical_padding(style: &mut TextStyle) {
    spacing::resolve_logical_padding(style);
}

pub(crate) fn resolve_logical_margin(style: &mut TextStyle) {
    spacing::resolve_logical_margin(style);
}

pub(crate) fn resolve_logical_borders(style: &mut TextStyle) {
    border::resolve_logical_borders(style);
}

fn parse_overflow_value(value: &str) -> Option<(Overflow, Overflow)> {
    let mut values = value.split_ascii_whitespace();
    let first = parse_overflow_keyword(values.next()?)?;
    let second = match values.next() {
        Some(value) => Some(parse_overflow_keyword(value)?),
        None => None,
    };
    if values.next().is_some() {
        return None;
    }
    Some((first, second.unwrap_or(first)))
}

fn parse_overflow_keyword(value: &str) -> Option<Overflow> {
    match value.to_ascii_lowercase().as_str() {
        "visible" => Some(Overflow::Visible),
        "hidden" => Some(Overflow::Hidden),
        "clip" => Some(Overflow::Clip),
        "auto" => Some(Overflow::Auto),
        "scroll" => Some(Overflow::Scroll),
        _ => None,
    }
}

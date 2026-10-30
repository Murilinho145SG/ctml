use crate::style::TextStyle;
use ctml_css::{
    parse_css_dimension, parse_css_percentage, parse_signed_font_relative_length_with_viewport,
};

pub(super) fn apply_letter_spacing(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        style.letter_spacing = 0.0;
        style.letter_spacing_px = false;
    } else if let Some((px, unit)) = parse_css_dimension(value)
        && unit == "px"
    {
        style.letter_spacing = px;
        style.letter_spacing_px = true;
    } else if let Some((em, unit)) = parse_css_dimension(value)
        && unit == "em"
    {
        style.letter_spacing = em;
        style.letter_spacing_px = false;
    } else if !value.contains('%')
        && let Some(length) = parse_signed_font_relative_length_with_viewport(
            value,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    {
        style.letter_spacing = length;
        style.letter_spacing_px = true;
    }
}

pub(super) fn apply_word_spacing(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        style.word_spacing_px = 0.0;
        style.word_spacing_percent = None;
    } else if let Some(percent) = parse_css_percentage(value) {
        style.word_spacing_px = 0.0;
        style.word_spacing_percent = Some(percent);
    } else if let Some(length) = parse_signed_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    ) {
        style.word_spacing_px = length;
        style.word_spacing_percent = None;
    }
}

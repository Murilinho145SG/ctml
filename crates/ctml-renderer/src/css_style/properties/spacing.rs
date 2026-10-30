use crate::style::TextStyle;
use ctml_css::{
    parse_css_dimension, parse_css_percentage,
    parse_nonnegative_font_relative_length_with_viewport,
    parse_signed_font_relative_length_with_viewport,
};

pub(super) fn apply_padding(style: &mut TextStyle, value: &str) {
    let values = value
        .split_whitespace()
        .map(|part| parse_padding_value(style, part))
        .collect::<Option<Vec<_>>>();
    let Some(values) = values.filter(|values| (1..=4).contains(&values.len())) else {
        return;
    };
    let [top, right, bottom, left] = match values.as_slice() {
        [all] => [all.clone(), all.clone(), all.clone(), all.clone()],
        [vertical, horizontal] => [
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ],
        [top, horizontal, bottom] => [
            top.clone(),
            horizontal.clone(),
            bottom.clone(),
            horizontal.clone(),
        ],
        [top, right, bottom, left] => [top.clone(), right.clone(), bottom.clone(), left.clone()],
        _ => return,
    };
    style.padding = [Some(top), Some(right), Some(bottom), Some(left)];
}

pub(super) fn apply_padding_side(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(padding) = parse_padding_value(style, value) {
        style.padding[side] = Some(padding);
    }
}

fn parse_padding_value(style: &TextStyle, value: &str) -> Option<String> {
    let value = value.trim();
    if value == "0" {
        Some("0".to_owned())
    } else if parse_css_percentage(value).is_some_and(|percentage| percentage >= 0.0) {
        Some(value.to_ascii_lowercase())
    } else if parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .is_some()
    {
        Some(value.to_ascii_lowercase())
    } else {
        None
    }
}

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

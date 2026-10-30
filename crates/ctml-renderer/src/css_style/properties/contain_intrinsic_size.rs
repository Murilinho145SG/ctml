use super::text_decoration::split_css_whitespace;
use crate::style::TextStyle;

pub(super) fn apply_contain_intrinsic_size(style: &mut TextStyle, value: &str) {
    let Some((width, height)) = parse_size(style, value) else {
        return;
    };
    style.contain_intrinsic_width = width;
    style.contain_intrinsic_height = height;
}

pub(super) fn apply_contain_intrinsic_width(style: &mut TextStyle, value: &str) {
    if let Some(width) = parse_single_size(style, value) {
        style.contain_intrinsic_width = width;
    }
}

pub(super) fn apply_contain_intrinsic_height(style: &mut TextStyle, value: &str) {
    if let Some(height) = parse_single_size(style, value) {
        style.contain_intrinsic_height = height;
    }
}

fn parse_size(style: &TextStyle, value: &str) -> Option<(Option<String>, Option<String>)> {
    let parts = split_css_whitespace(value);
    match parts.as_slice() {
        [one] => parse_single_size(style, one).map(|size| (size.clone(), size)),
        [first, second] if first.eq_ignore_ascii_case("auto") => {
            parse_component(style, second).map(|size| (size.clone(), size))
        }
        [width, height] => Some((
            parse_component(style, width)?,
            parse_component(style, height)?,
        )),
        ["auto", width, "auto", height] => Some((
            parse_component(style, width)?,
            parse_component(style, height)?,
        )),
        _ => None,
    }
}

fn parse_single_size(style: &TextStyle, value: &str) -> Option<Option<String>> {
    match split_css_whitespace(value).as_slice() {
        [one] => parse_component(style, one),
        ["auto", fallback] => parse_component(style, fallback),
        _ => None,
    }
}

fn parse_component(style: &TextStyle, value: &str) -> Option<Option<String>> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("auto") {
        return Some(None);
    }
    let valid_length = |component: &str| {
        ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            component,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    };
    let is_length = valid_length(value).is_some()
        || ctml_css::parse_math_expression(value, valid_length).is_some();
    is_length.then(|| Some(value.to_ascii_lowercase()))
}

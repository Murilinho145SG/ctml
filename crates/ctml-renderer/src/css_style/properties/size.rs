use crate::style::TextStyle;
use ctml_css::parse_nonnegative_length;

pub(super) fn apply_width(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        style.width = None;
    } else if parse_nonnegative_length(value, style.font_size).is_some() {
        style.width = Some(value.to_ascii_lowercase());
    }
}

pub(super) fn inherit_width(style: &mut TextStyle, inherited: &TextStyle) {
    style.width.clone_from(&inherited.width);
}

pub(super) fn apply_min_width(style: &mut TextStyle, value: &str) {
    apply_dimension_limit(&mut style.min_width, value, "auto", style.font_size);
}

pub(super) fn apply_max_width(style: &mut TextStyle, value: &str) {
    apply_dimension_limit(&mut style.max_width, value, "none", style.font_size);
}

pub(super) fn apply_height(style: &mut TextStyle, value: &str) {
    apply_dimension_limit(&mut style.height, value, "auto", style.font_size);
}

pub(super) fn apply_min_height(style: &mut TextStyle, value: &str) {
    apply_dimension_limit(&mut style.min_height, value, "auto", style.font_size);
}

pub(super) fn apply_max_height(style: &mut TextStyle, value: &str) {
    apply_dimension_limit(&mut style.max_height, value, "none", style.font_size);
}

fn apply_dimension_limit(
    target: &mut Option<String>,
    value: &str,
    reset_keyword: &str,
    font_size: f32,
) {
    let value = value.trim();
    if value.eq_ignore_ascii_case(reset_keyword) {
        *target = None;
    } else if parse_nonnegative_length(value, font_size).is_some() {
        *target = Some(value.to_ascii_lowercase());
    }
}

use crate::style::TextStyle;
use ctml_css::parse_nonnegative_font_relative_length;

pub(super) fn apply_width(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        style.width = None;
    } else if parse_nonnegative_font_relative_length(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
    )
    .is_some()
    {
        style.width = Some(value.to_ascii_lowercase());
    }
}

pub(super) fn apply_box_sizing(style: &mut TextStyle, value: &str) {
    match value.trim().to_ascii_lowercase().as_str() {
        "content-box" => style.border_box = false,
        "border-box" => style.border_box = true,
        _ => {}
    }
}

pub(super) fn inherit_width(style: &mut TextStyle, inherited: &TextStyle) {
    style.width.clone_from(&inherited.width);
}

pub(super) fn apply_min_width(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
    );
    apply_dimension_limit(&mut style.min_width, value, "auto", metrics);
}

pub(super) fn apply_max_width(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
    );
    apply_dimension_limit(&mut style.max_width, value, "none", metrics);
}

pub(super) fn apply_height(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
    );
    apply_dimension_limit(&mut style.height, value, "auto", metrics);
}

pub(super) fn apply_min_height(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
    );
    apply_dimension_limit(&mut style.min_height, value, "auto", metrics);
}

pub(super) fn apply_max_height(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
    );
    apply_dimension_limit(&mut style.max_height, value, "none", metrics);
}

fn apply_dimension_limit(
    target: &mut Option<String>,
    value: &str,
    reset_keyword: &str,
    (font_size, line_height, font_relative_units): (f32, f32, ctml_css::FontRelativeUnits),
) {
    let value = value.trim();
    if value.eq_ignore_ascii_case(reset_keyword) {
        *target = None;
    } else if parse_nonnegative_font_relative_length(
        value,
        font_size,
        line_height,
        font_relative_units,
    )
    .is_some()
    {
        *target = Some(value.to_ascii_lowercase());
    }
}

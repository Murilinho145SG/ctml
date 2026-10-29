use crate::style::TextStyle;
use ctml_css::{parse_math_expression, parse_nonnegative_font_relative_length_with_viewport};

pub(super) fn apply_width(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        style.width = None;
    } else if is_size_expression(style, value) {
        style.width = Some(value.to_ascii_lowercase());
    } else if parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
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
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.min_width, value, "auto", metrics);
}

pub(super) fn apply_max_width(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.max_width, value, "none", metrics);
}

pub(super) fn apply_height(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.height, value, "auto", metrics);
}

pub(super) fn apply_min_height(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.min_height, value, "auto", metrics);
}

pub(super) fn apply_max_height(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.max_height, value, "none", metrics);
}

fn apply_dimension_limit(
    target: &mut Option<String>,
    value: &str,
    reset_keyword: &str,
    (font_size, line_height, font_relative_units, viewport_size): (
        f32,
        f32,
        ctml_css::FontRelativeUnits,
        ctml_css::ViewportSize,
    ),
) {
    let value = value.trim();
    if value.eq_ignore_ascii_case(reset_keyword) {
        *target = None;
    } else if is_size_expression_with_metrics(
        value,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    ) {
        *target = Some(value.to_ascii_lowercase());
    } else if parse_nonnegative_font_relative_length_with_viewport(
        value,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    )
    .is_some()
    {
        *target = Some(value.to_ascii_lowercase());
    }
}

fn is_size_expression(style: &TextStyle, value: &str) -> bool {
    is_size_expression_with_metrics(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
}

fn is_size_expression_with_metrics(
    value: &str,
    font_size: f32,
    line_height: f32,
    font_relative_units: ctml_css::FontRelativeUnits,
    viewport_size: ctml_css::ViewportSize,
) -> bool {
    parse_math_expression(value, |component| {
        component
            .strip_suffix('%')
            .and_then(|percent| percent.trim().parse::<f32>().ok())
            .filter(|percent| percent.is_finite())
            .or_else(|| {
                parse_nonnegative_font_relative_length_with_viewport(
                    component,
                    font_size,
                    line_height,
                    font_relative_units,
                    viewport_size,
                )
            })
    })
    .is_some()
}

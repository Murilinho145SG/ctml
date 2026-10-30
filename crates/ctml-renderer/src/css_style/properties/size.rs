use crate::style::TextStyle;
use ctml_css::{parse_math_expression, parse_nonnegative_font_relative_length_with_viewport};

pub(super) fn apply_width(style: &mut TextStyle, value: &str) {
    if let Some(width) = parse_size_value(style, value) {
        style.width = width;
    }
}

pub(super) fn apply_inline_size(style: &mut TextStyle, value: &str) {
    if let Some(size) = parse_size_value(style, value) {
        style.inline_size = size;
    }
}

pub(super) fn inherit_inline_size(style: &mut TextStyle, inherited: &TextStyle) {
    style.inline_size.clone_from(&inherited.inline_size);
}

fn parse_size_value(style: &TextStyle, value: &str) -> Option<Option<String>> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        Some(None)
    } else if is_size_expression(style, value) || is_intrinsic_size_keyword(style, value) {
        Some(Some(value.to_ascii_lowercase()))
    } else if parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .is_some()
    {
        Some(Some(value.to_ascii_lowercase()))
    } else {
        None
    }
}

pub(super) fn is_valid_size_declaration(style: &TextStyle, property: &str, value: &str) -> bool {
    let value = value.trim();
    match property {
        "width" | "inline-size" => parse_size_value(style, value).is_some(),
        "height" | "block-size" | "min-width" | "min-inline-size" | "max-width"
        | "max-inline-size" | "min-height" | "min-block-size" | "max-height" | "max-block-size" => {
            let reset = if matches!(
                property,
                "max-width" | "max-inline-size" | "max-height" | "max-block-size"
            ) {
                "none"
            } else {
                "auto"
            };
            value.eq_ignore_ascii_case(reset)
                || is_size_expression(style, value)
                || is_intrinsic_size_keyword(style, value)
                || parse_nonnegative_font_relative_length_with_viewport(
                    value,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
                .is_some()
        }
        _ => false,
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

pub(super) fn apply_min_inline_size(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.min_inline_size, value, "auto", metrics);
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

pub(super) fn apply_max_inline_size(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.max_inline_size, value, "none", metrics);
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

pub(super) fn apply_block_size(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.block_size, value, "auto", metrics);
}

pub(super) fn apply_min_block_size(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.min_block_size, value, "auto", metrics);
}

pub(super) fn apply_max_block_size(style: &mut TextStyle, value: &str) {
    let metrics = (
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    apply_dimension_limit(&mut style.max_block_size, value, "none", metrics);
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
    } else if is_intrinsic_size_keyword_with_metrics(
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

fn is_intrinsic_size_keyword(style: &TextStyle, value: &str) -> bool {
    is_intrinsic_size_keyword_with_metrics(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
}

fn is_intrinsic_size_keyword_with_metrics(
    value: &str,
    font_size: f32,
    line_height: f32,
    font_relative_units: ctml_css::FontRelativeUnits,
    viewport_size: ctml_css::ViewportSize,
) -> bool {
    let value = value.trim().to_ascii_lowercase();
    if matches!(
        value.as_str(),
        "min-content" | "max-content" | "fit-content"
    ) {
        return true;
    }
    let Some(argument) = value
        .strip_prefix("fit-content(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return false;
    };
    is_size_expression_with_metrics(
        argument.trim(),
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    ) || parse_nonnegative_font_relative_length_with_viewport(
        argument.trim(),
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    )
    .is_some()
}

fn is_size_expression_with_metrics(
    value: &str,
    font_size: f32,
    line_height: f32,
    font_relative_units: ctml_css::FontRelativeUnits,
    viewport_size: ctml_css::ViewportSize,
) -> bool {
    parse_math_expression(value, |component| {
        ctml_css::parse_css_percentage(component)
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

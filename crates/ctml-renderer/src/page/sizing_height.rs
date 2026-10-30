use ctml_css::{FontRelativeUnits, ViewportSize};

use crate::style::{LayoutSize, WritingMode};

use super::sizing::{block_size_constraints, resolve_width};

pub(super) fn resolve_layout_height(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    available_height: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    resolve_used_height(
        height,
        min_height,
        max_height,
        0.0,
        available_height,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    )
}

pub(super) fn resolve_content_block_height(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    content_height: f32,
    available_height: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    resolve_used_height(
        height,
        min_height,
        max_height,
        content_height,
        available_height,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    )
}

pub(super) fn resolve_layout_height_with_ancestors(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    available_height: f32,
    ancestors: &[LayoutSize],
    page_writing_mode: WritingMode,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    let mut containing_height = available_height;
    for ancestor in ancestors.iter().take(ancestors.len().saturating_sub(1)) {
        let (height, min_height, max_height) = block_size_constraints(ancestor, page_writing_mode);
        if height.is_none() && min_height.is_none() && max_height.is_none() {
            continue;
        }
        containing_height = resolve_content_block_height(
            height,
            min_height,
            max_height,
            0.0,
            containing_height,
            ancestor.font_size,
            ancestor.line_height,
            ancestor.font_relative_units,
            ancestor.viewport_size,
        );
    }
    resolve_layout_height(
        height,
        min_height,
        max_height,
        containing_height,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    )
}

fn resolve_used_height(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    auto_height: f32,
    available_height: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    let resolve = |value: &str| {
        resolve_width(
            value,
            available_height,
            font_size,
            line_height,
            font_relative_units,
            viewport_size,
        )
    };
    let min_height = min_height.and_then(|value| resolve(value));
    let max_height = max_height.and_then(|value| resolve(value));
    let specified_height = height
        .and_then(|value| resolve(value))
        .unwrap_or(auto_height);
    match (min_height, max_height) {
        (Some(min), Some(max)) if min > max => specified_height.max(min),
        (Some(min), Some(max)) => specified_height.max(min).min(max),
        (Some(min), None) => specified_height.max(min),
        (None, Some(max)) => specified_height.min(max),
        (None, None) => specified_height,
    }
}

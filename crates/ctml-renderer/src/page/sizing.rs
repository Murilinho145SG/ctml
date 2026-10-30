use cosmic_text::Cursor;
use ctml_css::{
    FontRelativeUnits, ViewportSize, parse_font_relative_length_with_viewport,
    parse_math_expression, parse_nonnegative_font_relative_length_with_viewport,
};
use std::collections::HashMap;

use crate::style::WritingMode;
use crate::style::{LayoutSize, TextRun};

use super::sizing_intrinsic::resolve_intrinsic_width;
use super::sizing_padding::{resolve_margin, resolve_padding};
pub(super) fn resolve_width(
    value: &str,
    available_width: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> Option<f32> {
    let value = value.trim();
    if let Some(width) = parse_math_expression(value, |component| {
        component
            .strip_suffix('%')
            .and_then(|percentage| percentage.trim().parse::<f32>().ok())
            .map(|percentage| available_width * percentage / 100.0)
            .filter(|width| width.is_finite())
            .or_else(|| {
                parse_nonnegative_font_relative_length_with_viewport(
                    component,
                    font_size,
                    line_height,
                    font_relative_units,
                    viewport_size,
                )
            })
    }) {
        return (width >= 0.0).then_some(width);
    }
    if value == "0" {
        return Some(0.0);
    }
    if let Some(percentage) = value.strip_suffix('%') {
        return percentage
            .trim()
            .parse::<f32>()
            .ok()
            .map(|percentage| available_width * percentage / 100.0)
            .filter(|width| width.is_finite() && *width >= 0.0);
    }
    parse_font_relative_length_with_viewport(
        value,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    )
}

pub(super) fn resolve_constrained_size(
    specified: Option<&str>,
    min: Option<&str>,
    max: Option<&str>,
    available: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    let min = min.and_then(|value| {
        resolve_width(
            value,
            available,
            font_size,
            line_height,
            font_relative_units,
            viewport_size,
        )
    });
    let max = max.and_then(|value| {
        resolve_width(
            value,
            available,
            font_size,
            line_height,
            font_relative_units,
            viewport_size,
        )
    });
    let specified = specified
        .and_then(|value| {
            resolve_width(
                value,
                available,
                font_size,
                line_height,
                font_relative_units,
                viewport_size,
            )
        })
        .unwrap_or(available);
    match (min, max) {
        (Some(min), Some(max)) if min > max => specified.max(min),
        (Some(min), Some(max)) => specified.max(min).min(max),
        (Some(min), None) => specified.max(min),
        (None, Some(max)) => specified.min(max),
        (None, None) => specified,
    }
    .max(1.0)
}

pub(super) fn select_axis_size<'a>(
    logical: Option<&'a String>,
    physical: Option<&'a String>,
    orders: &HashMap<String, usize>,
    logical_property: &str,
    physical_property: &str,
) -> Option<Option<&'a str>> {
    match (orders.get(logical_property), orders.get(physical_property)) {
        (Some(logical_order), Some(physical_order)) if logical_order > physical_order => {
            Some(logical.map(String::as_str))
        }
        (Some(_), Some(_)) => Some(physical.map(String::as_str)),
        (Some(_), None) => Some(logical.map(String::as_str)),
        (None, Some(_)) => Some(physical.map(String::as_str)),
        (None, None) => logical
            .map(String::as_str)
            .or_else(|| physical.map(String::as_str))
            .map(Some),
    }
}

pub(super) fn block_size_constraints(
    size: &LayoutSize,
    page_writing_mode: WritingMode,
) -> (Option<&str>, Option<&str>, Option<&str>) {
    let same_axis_orientation = is_vertical(size.writing_mode) == is_vertical(page_writing_mode);
    let (logical, logical_min, logical_max, logical_name, logical_min_name, logical_max_name) =
        if same_axis_orientation {
            (
                &size.block_size,
                &size.min_block_size,
                &size.max_block_size,
                "block-size",
                "min-block-size",
                "max-block-size",
            )
        } else {
            (
                &size.inline_size,
                &size.min_inline_size,
                &size.max_inline_size,
                "inline-size",
                "min-inline-size",
                "max-inline-size",
            )
        };
    let (physical, physical_min, physical_max, physical_name, physical_min_name, physical_max_name) =
        if is_vertical(page_writing_mode) {
            (
                &size.width,
                &size.min_width,
                &size.max_width,
                "width",
                "min-width",
                "max-width",
            )
        } else {
            (
                &size.height,
                &size.min_height,
                &size.max_height,
                "height",
                "min-height",
                "max-height",
            )
        };
    (
        select_axis_size(
            logical.as_ref(),
            physical.as_ref(),
            &size.size_property_orders,
            logical_name,
            physical_name,
        )
        .flatten(),
        select_axis_size(
            logical_min.as_ref(),
            physical_min.as_ref(),
            &size.size_property_orders,
            logical_min_name,
            physical_min_name,
        )
        .flatten(),
        select_axis_size(
            logical_max.as_ref(),
            physical_max.as_ref(),
            &size.size_property_orders,
            logical_max_name,
            physical_max_name,
        )
        .flatten(),
    )
}

pub(super) fn resolve_layout_width(
    sizes: &[LayoutSize],
    available_width: f32,
    page_writing_mode: WritingMode,
) -> f32 {
    let mut content_width = available_width;
    for size in sizes {
        let padding = resolve_padding(size, content_width);
        let margin = resolve_margin(size, content_width);
        let inline_padding = if is_vertical(page_writing_mode) {
            padding[0] + padding[2]
        } else {
            padding[1] + padding[3]
        };
        let inline_margin = if is_vertical(page_writing_mode) {
            margin[0] + margin[2]
        } else {
            margin[1] + margin[3]
        };
        let available_content_width = (content_width - inline_padding - inline_margin).max(0.0);
        let resolve_inline_size = |value: &str| {
            let width = resolve_intrinsic_width(
                value,
                available_content_width,
                size.intrinsic_min_content_width,
                size.intrinsic_max_content_width,
                size.font_size,
                size.line_height,
                size.font_relative_units,
                size.viewport_size,
            )
            .or_else(|| {
                resolve_width(
                    value,
                    content_width,
                    size.font_size,
                    size.line_height,
                    size.font_relative_units,
                    size.viewport_size,
                )
            })?;
            Some(if size.border_box {
                (width - inline_padding).max(0.0)
            } else {
                width
            })
        };
        let same_axis_orientation =
            is_vertical(size.writing_mode) == is_vertical(page_writing_mode);
        let (
            logical_size,
            logical_min,
            logical_max,
            logical_name,
            logical_min_name,
            logical_max_name,
        ) = if same_axis_orientation {
            (
                &size.inline_size,
                &size.min_inline_size,
                &size.max_inline_size,
                "inline-size",
                "min-inline-size",
                "max-inline-size",
            )
        } else {
            (
                &size.block_size,
                &size.min_block_size,
                &size.max_block_size,
                "block-size",
                "min-block-size",
                "max-block-size",
            )
        };
        let (
            physical_size,
            physical_min,
            physical_max,
            physical_name,
            physical_min_name,
            physical_max_name,
        ) = if is_vertical(page_writing_mode) {
            (
                &size.height,
                &size.min_height,
                &size.max_height,
                "height",
                "min-height",
                "max-height",
            )
        } else {
            (
                &size.width,
                &size.min_width,
                &size.max_width,
                "width",
                "min-width",
                "max-width",
            )
        };
        let inline_min = select_axis_size(
            logical_min.as_ref(),
            physical_min.as_ref(),
            &size.size_property_orders,
            logical_min_name,
            physical_min_name,
        )
        .flatten();
        let inline_max = select_axis_size(
            logical_max.as_ref(),
            physical_max.as_ref(),
            &size.size_property_orders,
            logical_max_name,
            physical_max_name,
        )
        .flatten();
        let inline_size = select_axis_size(
            logical_size.as_ref(),
            physical_size.as_ref(),
            &size.size_property_orders,
            logical_name,
            physical_name,
        )
        .flatten();
        let min_width = inline_min.and_then(resolve_inline_size);
        let max_width = inline_max.and_then(resolve_inline_size);
        let specified_width = inline_size
            .and_then(resolve_inline_size)
            .unwrap_or(available_content_width);
        content_width = match (min_width, max_width) {
            (Some(min), Some(max)) if min > max => specified_width.max(min),
            (Some(min), Some(max)) => specified_width.max(min).min(max),
            (Some(min), None) => specified_width.max(min),
            (None, Some(max)) => specified_width.min(max),
            (None, None) => specified_width,
        };
    }
    content_width.max(1.0)
}

#[cfg(test)]
mod tests {
    use super::{resolve_width, select_axis_size};
    use ctml_css::{FontRelativeUnits, ViewportSize};
    use std::collections::HashMap;

    #[test]
    fn resolves_font_metric_units_for_box_dimensions() {
        let units = FontRelativeUnits {
            ex: 0.4,
            ch: 0.55,
            ic: 1.0,
            cap: 0.7,
        };
        let viewport = ViewportSize {
            width: 800.0,
            height: 600.0,
        };
        assert_eq!(
            resolve_width("2ch", 200.0, 20.0, 24.0, units, viewport),
            Some(22.0)
        );
        assert_eq!(
            resolve_width("0.5lh", 200.0, 20.0, 24.0, units, viewport),
            Some(12.0)
        );
        assert_eq!(
            resolve_width("25%", 200.0, 20.0, 24.0, units, viewport),
            Some(50.0)
        );
        assert_eq!(
            resolve_width("10svw", 200.0, 20.0, 24.0, units, viewport),
            Some(80.0)
        );
        assert_eq!(
            resolve_width("calc(50% - 10px)", 200.0, 20.0, 24.0, units, viewport),
            Some(90.0)
        );
        assert_eq!(
            resolve_width("min(25%, 60px)", 200.0, 20.0, 24.0, units, viewport),
            Some(50.0)
        );
        assert_eq!(
            resolve_width("clamp(10px, 25%, 50px)", 200.0, 20.0, 24.0, units, viewport),
            Some(50.0)
        );
    }

    #[test]
    fn physical_and_logical_axis_sizes_follow_declaration_order() {
        let mut orders = HashMap::new();
        orders.insert("width".to_owned(), 0);
        orders.insert("inline-size".to_owned(), 1);
        assert_eq!(
            select_axis_size(
                Some(&"50px".to_owned()),
                Some(&"100px".to_owned()),
                &orders,
                "inline-size",
                "width",
            ),
            Some(Some("50px"))
        );
        orders.insert("inline-size".to_owned(), 2);
        assert_eq!(
            select_axis_size(
                None,
                Some(&"100px".to_owned()),
                &orders,
                "inline-size",
                "width"
            ),
            Some(None)
        );
    }
}

pub(super) fn is_vertical(mode: WritingMode) -> bool {
    mode != WritingMode::HorizontalTb
}

pub(super) fn line_layout_sizes(runs: &[TextRun], line_count: usize) -> Vec<Vec<LayoutSize>> {
    let mut sizes = vec![None; line_count];
    let mut line = 0;
    let mut line_has_text = false;
    for run in runs {
        for character in run.text.chars() {
            if character == '\n' {
                line += 1;
                line_has_text = false;
            } else if !line_has_text {
                if let Some(line_sizes) = sizes.get_mut(line) {
                    *line_sizes = Some(run.style.layout_sizes.clone());
                }
                line_has_text = true;
            }
        }
    }
    sizes.into_iter().map(Option::unwrap_or_default).collect()
}

pub(super) fn flow_spacer_lines(runs: &[TextRun], line_count: usize) -> Vec<bool> {
    let mut spacer_lines = vec![false; line_count];
    let mut line = 0;
    for run in runs {
        for character in run.text.chars() {
            if character == '\n' {
                line += 1;
            } else if run.style.flow_spacer
                && let Some(is_spacer) = spacer_lines.get_mut(line)
            {
                *is_spacer = true;
            }
        }
    }
    spacer_lines
}

pub(super) fn cursor_without_flow_spacers(cursor: Cursor, spacer_lines: &[bool]) -> Cursor {
    let preceding_spacers = spacer_lines
        .iter()
        .take(cursor.line)
        .filter(|is_spacer| **is_spacer)
        .count();
    let on_spacer = spacer_lines.get(cursor.line).copied().unwrap_or_default();
    Cursor::new_with_affinity(
        cursor.line.saturating_sub(preceding_spacers),
        if on_spacer { 0 } else { cursor.index },
        cursor.affinity,
    )
}

pub(super) fn cursor_with_flow_spacers(cursor: Cursor, spacer_lines: &[bool]) -> Cursor {
    let mut source_line = 0;
    for (line, is_spacer) in spacer_lines.iter().copied().enumerate() {
        if is_spacer {
            continue;
        }
        if source_line == cursor.line {
            return Cursor::new_with_affinity(line, cursor.index, cursor.affinity);
        }
        source_line += 1;
    }
    Cursor::new_with_affinity(
        spacer_lines.len().saturating_sub(1),
        cursor.index,
        cursor.affinity,
    )
}

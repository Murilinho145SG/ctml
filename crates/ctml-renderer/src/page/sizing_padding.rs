use crate::style::{BorderLineStyle, LayoutSize, WritingMode};

use super::sizing::{is_vertical, resolve_layout_width, resolve_width};
use ctml_css::parse_signed_font_relative_length_with_viewport;

pub(super) fn resolve_padding(size: &LayoutSize, containing_width: f32) -> [f32; 4] {
    std::array::from_fn(|index| {
        size.padding[index]
            .as_deref()
            .and_then(|value| {
                resolve_width(
                    value,
                    containing_width,
                    size.font_size,
                    size.line_height,
                    size.font_relative_units,
                    size.viewport_size,
                )
            })
            .unwrap_or_default()
    })
}

pub(super) fn resolve_borders(size: &LayoutSize, containing_width: f32) -> [f32; 4] {
    std::array::from_fn(|index| {
        if size.border_style[index] == BorderLineStyle::None && size.border_image_source.is_none() {
            return 0.0;
        }
        let Some(value) = size.border_width[index].as_deref() else {
            return 3.0;
        };
        match value {
            "thin" => 1.0,
            "medium" => 3.0,
            "thick" => 5.0,
            _ => resolve_width(
                value,
                containing_width,
                size.font_size,
                size.line_height,
                size.font_relative_units,
                size.viewport_size,
            )
            .unwrap_or_default(),
        }
    })
}

pub(super) fn resolve_margin(size: &LayoutSize, containing_width: f32) -> [f32; 4] {
    std::array::from_fn(|index| {
        size.margin[index]
            .as_deref()
            .and_then(|value| {
                value
                    .strip_suffix('%')
                    .and_then(|percentage| percentage.trim().parse::<f32>().ok())
                    .map(|percentage| containing_width * percentage / 100.0)
                    .filter(|margin| margin.is_finite())
                    .or_else(|| {
                        parse_signed_font_relative_length_with_viewport(
                            value,
                            size.font_size,
                            size.line_height,
                            size.font_relative_units,
                            size.viewport_size,
                        )
                    })
            })
            .unwrap_or_default()
    })
}

pub(super) fn inline_margin(
    sizes: &[LayoutSize],
    available_width: f32,
    page_writing_mode: WritingMode,
) -> (f32, f32) {
    sizes
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |(start, extent), (index, size)| {
            let containing_width =
                resolve_layout_width(&sizes[..index], available_width, page_writing_mode);
            let margin = resolve_margin(size, containing_width);
            if is_vertical(page_writing_mode) {
                (start + margin[0], extent + margin[0] + margin[2])
            } else {
                (start + margin[3], extent + margin[1] + margin[3])
            }
        })
}

pub(super) fn inline_padding(
    sizes: &[LayoutSize],
    available_width: f32,
    page_writing_mode: WritingMode,
) -> (f32, f32) {
    sizes
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |(start, extent), (index, size)| {
            let containing_width =
                resolve_layout_width(&sizes[..index], available_width, page_writing_mode);
            let padding = resolve_padding(size, containing_width);
            if is_vertical(page_writing_mode) {
                (start + padding[0], extent + padding[0] + padding[2])
            } else {
                (start + padding[3], extent + padding[1] + padding[3])
            }
        })
}

pub(super) fn inline_borders(
    sizes: &[LayoutSize],
    available_width: f32,
    page_writing_mode: WritingMode,
) -> (f32, f32) {
    sizes
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |(start, extent), (index, size)| {
            let containing_width =
                resolve_layout_width(&sizes[..index], available_width, page_writing_mode);
            let border = resolve_borders(size, containing_width);
            if is_vertical(page_writing_mode) {
                (start + border[0], extent + border[0] + border[2])
            } else {
                (start + border[3], extent + border[1] + border[3])
            }
        })
}

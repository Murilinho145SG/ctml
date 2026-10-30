use crate::style::{LayoutSize, WritingMode};

use super::sizing::{is_vertical, resolve_layout_width, resolve_width};

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

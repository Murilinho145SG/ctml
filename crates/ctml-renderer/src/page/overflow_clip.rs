use std::collections::HashMap;

use cosmic_text::{Buffer, FontSystem};
use std::collections::HashSet;

use crate::style::{LayoutSize, TextRun, WritingMode};

use super::{
    PAGE_PADDING, local_writing,
    sizing::{is_vertical, resolve_layout_width},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ClipRect {
    pub(super) left: f32,
    pub(super) top: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BlockGeometry {
    pub(super) clip: ClipRect,
    pub(super) content_block_extent: f32,
}

impl ClipRect {
    pub(super) fn intersect(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right).max(self.left.max(other.left)),
            bottom: self.bottom.min(other.bottom).max(self.top.max(other.top)),
        }
    }
}

pub(super) fn clip_for_path(path: &[usize], clips: &HashMap<usize, ClipRect>) -> Option<ClipRect> {
    path.iter()
        .filter_map(|node_id| clips.get(node_id).copied())
        .reduce(ClipRect::intersect)
}

pub(super) fn scroll_offset_y_for_path(
    path: &[usize],
    offsets: &HashMap<usize, (f32, f32)>,
) -> f32 {
    path.iter()
        .filter_map(|node_id| offsets.get(node_id).map(|(_, offset_y)| *offset_y))
        .sum()
}

#[derive(Clone, Debug)]
struct BlockExtent {
    size: LayoutSize,
    top: f32,
    content_height: f32,
}

pub(super) fn block_geometries(
    runs: &[TextRun],
    buffer: &mut Buffer,
    font_system: &mut FontSystem,
    width: u32,
    height: u32,
    page_writing_mode: WritingMode,
) -> HashMap<usize, BlockGeometry> {
    let style_ranges = local_writing::text_style_ranges(runs);
    let line_starts = local_writing::text_line_starts(runs);
    let mut extents = HashMap::<usize, BlockExtent>::new();
    for line in buffer.layout_runs() {
        let line_start = line_starts.get(line.line_i).copied().unwrap_or(0);
        let Some(glyph) = line.glyphs.first() else {
            continue;
        };
        let Some(style_index) =
            local_writing::style_index_at(&style_ranges, line_start + glyph.start)
        else {
            continue;
        };
        let Some(style_run) = runs.get(style_index) else {
            continue;
        };
        for size in &style_run.style.layout_sizes {
            if size.overflow_x == crate::style::Overflow::Visible
                && size.overflow_y == crate::style::Overflow::Visible
            {
                continue;
            }
            extents
                .entry(size.node_id)
                .and_modify(|extent| extent.content_height += line.line_height)
                .or_insert(BlockExtent {
                    size: size.clone(),
                    top: line.line_top,
                    content_height: line.line_height,
                });
        }
    }

    if extents.values().any(|extent| {
        super::scrolling::user_scrollable_axes(extent.size.overflow_x, extent.size.overflow_y).1
    }) {
        let mut content_heights = HashMap::<usize, f32>::new();
        let line_starts = local_writing::text_line_starts(runs);
        let metrics = buffer.metrics();
        for line_index in 0..buffer.lines.len() {
            let Some(layout) = buffer.line_layout(font_system, line_index) else {
                continue;
            };
            let line_start = line_starts.get(line_index).copied().unwrap_or(0);
            for layout_line in layout {
                let mut seen_blocks = HashSet::new();
                for glyph in &layout_line.glyphs {
                    let Some(style_index) =
                        local_writing::style_index_at(&style_ranges, line_start + glyph.start)
                    else {
                        continue;
                    };
                    let Some(style_run) = runs.get(style_index) else {
                        continue;
                    };
                    for size in &style_run.style.layout_sizes {
                        if seen_blocks.insert(size.node_id) {
                            *content_heights.entry(size.node_id).or_default() +=
                                layout_line.line_height_opt.unwrap_or(metrics.line_height);
                        }
                    }
                }
            }
        }
        for (node_id, extent) in &mut extents {
            extent.content_height = content_heights
                .get(node_id)
                .copied()
                .unwrap_or(extent.content_height);
        }
    }

    let available_width = (width as f32 - PAGE_PADDING * 2.0).max(0.0);
    let available_height = (height as f32 - PAGE_PADDING * 2.0).max(0.0);
    extents
        .into_iter()
        .map(|(node_id, extent)| {
            let block_height = used_block_height(
                &extent.size,
                extent.content_height,
                available_height,
                page_writing_mode,
            );
            let sizes = runs
                .iter()
                .find(|run| {
                    run.style
                        .layout_sizes
                        .iter()
                        .any(|size| size.node_id == node_id)
                })
                .map(|run| {
                    let end = run
                        .style
                        .layout_sizes
                        .iter()
                        .position(|size| size.node_id == node_id)
                        .unwrap_or(run.style.layout_sizes.len());
                    run.style.layout_sizes[..=end].to_vec()
                })
                .unwrap_or_default();
            let block_width = resolve_layout_width(&sizes, available_width, page_writing_mode);
            let clip = if page_writing_mode == WritingMode::HorizontalTb
                && is_vertical(extent.size.writing_mode)
            {
                let transform = local_writing::LocalVerticalTransform::new(
                    extent.size.writing_mode,
                    extent.top,
                    width,
                    height,
                );
                let (left, top, rect_width, rect_height) = transform.map_rect(
                    PAGE_PADDING,
                    PAGE_PADDING + extent.top,
                    block_height.max(0.0),
                    block_width.max(0.0),
                );
                ClipRect {
                    left,
                    top,
                    right: left + rect_width,
                    bottom: top + rect_height,
                }
            } else {
                ClipRect {
                    left: PAGE_PADDING,
                    top: PAGE_PADDING + extent.top,
                    right: PAGE_PADDING + block_width.max(0.0),
                    bottom: PAGE_PADDING + extent.top + block_height.max(0.0),
                }
            };
            (
                node_id,
                BlockGeometry {
                    clip,
                    content_block_extent: extent.content_height,
                },
            )
        })
        .collect()
}

fn used_block_height(
    size: &LayoutSize,
    content_height: f32,
    available_height: f32,
    page_writing_mode: WritingMode,
) -> f32 {
    use super::{sizing::select_axis_size, sizing_height::resolve_content_block_height};

    let same_axis_orientation = is_vertical(size.writing_mode) == is_vertical(page_writing_mode);
    let (logical_height, logical_min, logical_max, logical_name, min_name, max_name) =
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
    let (height, min_height, max_height, height_name, min_height_name, max_height_name) =
        if !is_vertical(page_writing_mode) {
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
    resolve_content_block_height(
        select_axis_size(
            logical_height.as_ref(),
            height.as_ref(),
            &size.size_property_orders,
            logical_name,
            height_name,
        )
        .flatten(),
        select_axis_size(
            logical_min.as_ref(),
            min_height.as_ref(),
            &size.size_property_orders,
            min_name,
            min_height_name,
        )
        .flatten(),
        select_axis_size(
            logical_max.as_ref(),
            max_height.as_ref(),
            &size.size_property_orders,
            max_name,
            max_height_name,
        )
        .flatten(),
        content_height,
        available_height,
        size.font_size,
        size.line_height,
        size.font_relative_units,
        size.viewport_size,
    )
}

use std::collections::HashMap;

use cosmic_text::{Buffer, FontSystem};
use std::collections::HashSet;

use crate::style::{LayoutSize, TextRun, WritingMode};

use super::{
    PAGE_PADDING, local_writing,
    sizing::{is_vertical, resolve_layout_width},
    sizing_padding::inline_padding,
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
    pub(super) border_box: ClipRect,
    pub(super) content_block_extent: f32,
    pub(super) content_inline_extent: f32,
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

pub(super) fn scroll_offset_x_for_path(
    path: &[usize],
    offsets: &HashMap<usize, (f32, f32)>,
) -> f32 {
    path.iter()
        .filter_map(|node_id| offsets.get(node_id).map(|(offset_x, _)| *offset_x))
        .sum()
}

pub(super) fn scroll_translation_for_path(
    path: &[usize],
    offsets: &HashMap<usize, (f32, f32)>,
    writing_mode: WritingMode,
) -> (f32, f32) {
    let offset_x = scroll_offset_x_for_path(path, offsets);
    let offset_y = scroll_offset_y_for_path(path, offsets);
    match writing_mode {
        WritingMode::HorizontalTb => (offset_x, offset_y),
        WritingMode::VerticalRl => (-offset_y, offset_x),
        WritingMode::VerticalLr => (offset_y, -offset_x),
    }
}

#[derive(Clone, Debug)]
struct BlockExtent {
    size: LayoutSize,
    top: f32,
    content_height: f32,
    content_inline_extent: f32,
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
            extents
                .entry(size.node_id)
                .and_modify(|extent| {
                    extent.content_height += line.line_height;
                    extent.content_inline_extent = extent.content_inline_extent.max(line.line_w);
                })
                .or_insert(BlockExtent {
                    size: size.clone(),
                    top: line.line_top,
                    content_height: line.line_height,
                    content_inline_extent: line.line_w,
                });
        }
    }

    if extents.values().any(|extent| {
        let (x_scrollable, y_scrollable) =
            super::scrolling::user_scrollable_axes(extent.size.overflow_x, extent.size.overflow_y);
        x_scrollable || y_scrollable
    }) {
        let mut content_heights = HashMap::<usize, f32>::new();
        let mut content_inline_extents = HashMap::<usize, f32>::new();
        let line_starts = local_writing::text_line_starts(runs);
        let metrics = buffer.metrics();
        for line_index in 0..buffer.lines.len() {
            let Some(layout) = buffer.line_layout(font_system, line_index) else {
                continue;
            };
            let line_start = line_starts.get(line_index).copied().unwrap_or(0);
            for layout_line in layout {
                let mut line_blocks = HashSet::new();
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
                        if line_blocks.insert(size.node_id) {
                            content_inline_extents
                                .entry(size.node_id)
                                .and_modify(|extent| *extent = extent.max(layout_line.w))
                                .or_insert(layout_line.w);
                        }
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
            extent.content_inline_extent = content_inline_extents
                .get(node_id)
                .copied()
                .unwrap_or(extent.content_inline_extent);
        }
    }

    let available_width = (width as f32 - PAGE_PADDING * 2.0).max(0.0);
    let available_height = (height as f32 - PAGE_PADDING * 2.0).max(0.0);
    extents
        .iter()
        .filter(|(_, extent)| {
            extent.size.overflow_x != crate::style::Overflow::Visible
                || extent.size.overflow_y != crate::style::Overflow::Visible
                || extent.size.paint_contained
                || extent
                    .size
                    .border_style
                    .iter()
                    .any(|style| *style != crate::style::BorderLineStyle::None)
                || extent.size.outline_style != crate::style::BorderLineStyle::None
                || !extent.size.box_shadows.is_empty()
                || (extent.size.empty_box && !extent.size.background_layers.is_empty())
        })
        .map(|(&node_id, extent)| {
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
            let block_height = used_block_height_for_path(
                node_id,
                &sizes,
                &extents,
                available_height,
                available_width,
                page_writing_mode,
            );
            let containing_width = resolve_layout_width(
                &sizes[..sizes.len().saturating_sub(1)],
                available_width,
                page_writing_mode,
            );
            let block_width = resolve_layout_width(&sizes, available_width, page_writing_mode)
                + inline_padding(
                    std::slice::from_ref(&extent.size),
                    containing_width,
                    page_writing_mode,
                )
                .1;
            let border_widths =
                super::sizing_padding::resolve_borders(&extent.size, containing_width);
            let (border_start, border_end) = if is_vertical(page_writing_mode) {
                (border_widths[0], border_widths[2])
            } else {
                (border_widths[3], border_widths[1])
            };
            let border_extent = border_start + border_end;
            let (border_block_start, border_block_end) = match page_writing_mode {
                WritingMode::HorizontalTb => (border_widths[0], border_widths[2]),
                WritingMode::VerticalRl => (border_widths[1], border_widths[3]),
                WritingMode::VerticalLr => (border_widths[3], border_widths[1]),
            };
            let block_border_extent = border_block_start + border_block_end;
            let clipped_block_height = (block_height - block_border_extent).max(0.0);
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
                    clipped_block_height,
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
                    bottom: PAGE_PADDING + extent.top + clipped_block_height,
                }
            };
            let border_box = ClipRect {
                left: clip.left - border_start,
                top: clip.top - border_block_start,
                right: clip.right + (border_extent - border_start),
                bottom: clip.bottom + border_block_end,
            };
            (
                node_id,
                BlockGeometry {
                    clip,
                    border_box,
                    content_block_extent: extent.content_height,
                    content_inline_extent: extent.content_inline_extent,
                },
            )
        })
        .collect()
}

fn used_block_height_for_path(
    target_node_id: usize,
    sizes: &[LayoutSize],
    extents: &HashMap<usize, BlockExtent>,
    available_height: f32,
    available_width: f32,
    page_writing_mode: WritingMode,
) -> f32 {
    let mut containing_height = available_height;
    for (index, size) in sizes.iter().enumerate() {
        let Some(extent) = extents.get(&size.node_id) else {
            continue;
        };
        let containing_width =
            resolve_layout_width(&sizes[..index], available_width, page_writing_mode);
        let used_height = used_block_height(
            size,
            if size.size_contained {
                0.0
            } else {
                extent.content_height
            },
            containing_height,
            containing_width,
            page_writing_mode,
        );
        if size.node_id == target_node_id {
            return used_height;
        }
        if is_vertical(size.writing_mode) == is_vertical(page_writing_mode) {
            containing_height = used_height;
        }
    }
    0.0
}

fn used_block_height(
    size: &LayoutSize,
    content_height: f32,
    available_height: f32,
    available_width: f32,
    page_writing_mode: WritingMode,
) -> f32 {
    super::sizing_height::resolve_block_box_extent(
        size,
        content_height,
        available_height,
        available_width,
        page_writing_mode,
    )
}

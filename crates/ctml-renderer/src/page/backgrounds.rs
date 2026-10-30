use cosmic_text::Buffer;

use crate::{
    page::{
        local_writing,
        overflow_clip::{self, BlockGeometry, ClipRect},
        pixel::fill_rect_alpha_clipped,
    },
    style::{TextRun, WritingMode},
};
use std::collections::{HashMap, HashSet};

pub(super) fn draw_block_boxes(
    runs: &[TextRun],
    geometries: &HashMap<usize, BlockGeometry>,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    page_writing_mode: WritingMode,
    overflow_clips: &HashMap<usize, ClipRect>,
    nested_scroll: &HashMap<usize, (f32, f32)>,
    image_cache: &mut super::background_images::ImageCache,
) {
    let mut drawn = HashSet::new();
    for run in runs {
        for size in &run.style.layout_sizes {
            if !drawn.insert(size.node_id)
                || (size.background_layers.is_empty() && size.background_images.is_empty())
                || size.visibility_hidden
                || size.content_skipped
                || size.opacity <= 0.0
            {
                continue;
            }
            let Some(geometry) = geometries.get(&size.node_id) else {
                continue;
            };
            let Some(path_index) = run
                .element_path
                .iter()
                .position(|node_id| *node_id == size.node_id)
            else {
                continue;
            };
            let parent_path = &run.element_path[..path_index];
            let clip = overflow_clip::clip_for_path(parent_path, overflow_clips);
            let (scroll_x, scroll_y) = overflow_clip::scroll_translation_for_path(
                parent_path,
                nested_scroll,
                page_writing_mode,
            );
            let rect = geometry.border_box;
            for layer in &size.background_layers {
                let mut color = *layer;
                color[3] = (f32::from(color[3]) * size.opacity)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                fill_rect_alpha_clipped(
                    pixels,
                    width,
                    height,
                    rect.left - super::PAGE_PADDING - scroll_x,
                    rect.top - super::PAGE_PADDING - scroll_y,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    color,
                    clip,
                );
            }
            super::background_images::draw_background_image_layers(
                &size.background_images,
                &size.background_repeats,
                &size.background_positions,
                &size.background_sizes,
                size.opacity,
                pixels,
                width,
                height,
                overflow_clip::ClipRect {
                    left: rect.left - super::PAGE_PADDING - scroll_x,
                    top: rect.top - super::PAGE_PADDING - scroll_y,
                    right: rect.right - super::PAGE_PADDING - scroll_x,
                    bottom: rect.bottom - super::PAGE_PADDING - scroll_y,
                },
                clip,
                image_cache,
            );
        }
    }
}

pub(super) fn draw(
    runs: &[TextRun],
    buffer: &Buffer,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    skip_root_background: bool,
    page_writing_mode: WritingMode,
    overflow_clips: &HashMap<usize, ClipRect>,
    nested_scroll: &HashMap<usize, (f32, f32)>,
    image_cache: &mut super::background_images::ImageCache,
) {
    let style_ranges = local_writing::text_style_ranges(runs);
    let line_starts = local_writing::text_line_starts(runs);
    let vertical_block_origins =
        local_writing::vertical_block_origins(runs, buffer, page_writing_mode);
    for line in buffer.layout_runs() {
        let line_start = line_starts.get(line.line_i).copied().unwrap_or(0);
        let mut start = 0;
        while start < line.glyphs.len() {
            let glyph = &line.glyphs[start];
            let style_index =
                local_writing::style_index_at(&style_ranges, line_start + glyph.start);
            let Some(style) = style_index.and_then(|index| runs.get(index).map(|run| &run.style))
            else {
                start += 1;
                continue;
            };
            if style.visibility_hidden || style.content_skipped {
                start += 1;
                continue;
            }
            let is_block_background =
                style_index
                    .and_then(|index| runs.get(index))
                    .is_some_and(|run| {
                        run.style.layout_sizes.iter().any(|size| {
                            run.element_path.last() == Some(&size.node_id)
                                && ((!size.background_layers.is_empty()
                                    && size.background_layers == style.background_layers)
                                    || (!size.background_images.is_empty()
                                        && size.background_images == style.background_images))
                        })
                    });
            if is_block_background
                || (style.inline_box
                    && (style.background_layers.len() > usize::from(skip_root_background)
                        || !style.background_images.is_empty()))
            {
                start += 1;
                continue;
            }
            let layer_start =
                usize::from(skip_root_background && !style.background_layers.is_empty());
            if layer_start == style.background_layers.len() && style.background_images.is_empty() {
                start += 1;
                continue;
            }
            let mut end = start + 1;
            while end < line.glyphs.len() {
                let next = &line.glyphs[end];
                if local_writing::style_index_at(&style_ranges, line_start + next.start)
                    != style_index
                    || next.level != glyph.level
                {
                    break;
                }
                end += 1;
            }
            let fragment = &line.glyphs[start..end];
            let (scroll_offset_x, scroll_offset_y) = style_index
                .and_then(|index| runs.get(index))
                .map_or((0.0, 0.0), |run| {
                    overflow_clip::scroll_translation_for_path(
                        &run.element_path,
                        nested_scroll,
                        page_writing_mode,
                    )
                });
            let left = fragment
                .iter()
                .map(|glyph| glyph.x)
                .fold(f32::INFINITY, f32::min);
            let right = fragment
                .iter()
                .map(|glyph| glyph.x + glyph.w)
                .fold(f32::NEG_INFINITY, f32::max);
            for color in &style.background_layers[layer_start..] {
                if color[3] > 0 && right > left {
                    let mut color = *color;
                    color[3] = (f32::from(color[3]) * style.opacity)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    let local_vertical_mode = (page_writing_mode == WritingMode::HorizontalTb)
                        .then_some(style.writing_mode)
                        .filter(|mode| *mode != WritingMode::HorizontalTb);
                    if let Some(mode) = local_vertical_mode {
                        let origin_y = style_index
                            .and_then(|index| runs.get(index))
                            .and_then(|run| run.first_line_block)
                            .and_then(|block| vertical_block_origins.get(&block).copied())
                            .unwrap_or(line.line_y);
                        let transform = local_writing::LocalVerticalTransform::new(
                            mode, origin_y, width, height,
                        );
                        let (left, top, rect_width, rect_height) = transform.map_rect(
                            left + crate::page::PAGE_PADDING,
                            line.line_top + crate::page::PAGE_PADDING,
                            right - left,
                            line.line_height,
                        );
                        let clip = style_index
                            .and_then(|index| runs.get(index))
                            .and_then(|run| {
                                overflow_clip::clip_for_path(&run.element_path, overflow_clips)
                            });
                        fill_rect_alpha_clipped(
                            pixels,
                            width,
                            height,
                            left - crate::page::PAGE_PADDING - scroll_offset_x,
                            top - crate::page::PAGE_PADDING - scroll_offset_y,
                            rect_width,
                            rect_height,
                            color,
                            clip,
                        );
                    } else {
                        let clip = style_index
                            .and_then(|index| runs.get(index))
                            .and_then(|run| {
                                overflow_clip::clip_for_path(&run.element_path, overflow_clips)
                            });
                        fill_rect_alpha_clipped(
                            pixels,
                            width,
                            height,
                            left - scroll_offset_x,
                            line.line_top - scroll_offset_y,
                            right - left,
                            line.line_height,
                            color,
                            clip,
                        );
                    }
                }
            }
            if !style.background_images.is_empty() {
                let clip = style_index
                    .and_then(|index| runs.get(index))
                    .and_then(|run| {
                        overflow_clip::clip_for_path(&run.element_path, overflow_clips)
                    });
                super::background_images::draw_background_image_layers(
                    &style.background_images,
                    &style.background_repeats,
                    &style.background_positions,
                    &style.background_sizes,
                    style.opacity,
                    pixels,
                    width,
                    height,
                    overflow_clip::ClipRect {
                        left: left - scroll_offset_x,
                        top: line.line_top - scroll_offset_y,
                        right: right - scroll_offset_x,
                        bottom: line.line_top + line.line_height - scroll_offset_y,
                    },
                    clip,
                    image_cache,
                );
            }
            start = end;
        }
    }
}

use std::collections::{HashMap, HashSet};

use crate::style::{TextRun, WritingMode};

use super::overflow_clip::{self, ClipRect};

pub(super) fn draw_borders(
    runs: &[TextRun],
    buffer: &cosmic_text::Buffer,
    spans: &[super::CursorSpan],
    pixels: &mut [u8],
    width: u32,
    height: u32,
    page_writing_mode: WritingMode,
    clips: &HashMap<usize, ClipRect>,
    nested_scroll: &HashMap<usize, (f32, f32)>,
    flex_wrap_line_offsets: &HashMap<(usize, usize), f32>,
) {
    let first_lines = super::highlights::first_formatted_line_ranges(buffer, spans);
    let vertical_origins = super::local_writing::vertical_block_origins(
        runs,
        buffer,
        page_writing_mode,
        flex_wrap_line_offsets,
    );
    let lines = super::adjusted_layout_runs(buffer, flex_wrap_line_offsets).collect::<Vec<_>>();
    let mut drawn = HashSet::new();
    for span in spans {
        let Some(block) = span.first_line_block else {
            continue;
        };
        if !drawn.insert((block, span.run_index)) || span.visibility_hidden {
            continue;
        }
        let Some((line_index, line_end)) = first_lines.get(&block).copied() else {
            continue;
        };
        if span.line != line_index || span.start >= line_end {
            continue;
        }
        let Some((run, style)) = runs
            .get(span.run_index)
            .and_then(|run| run.first_line_font_style.as_ref().map(|style| (run, style)))
        else {
            continue;
        };
        let Some(line) = lines.iter().find(|line| line.line_i == line_index) else {
            continue;
        };
        let end = span.end.min(line_end);
        let (mut left, mut right) = (f32::INFINITY, f32::NEG_INFINITY);
        for glyph in line.glyphs {
            if glyph.start < end && glyph.end > span.start {
                left = left.min(glyph.x);
                right = right.max(glyph.x + glyph.w);
            }
        }
        if !left.is_finite() || !right.is_finite() || right <= left {
            continue;
        }
        let mut rect = ClipRect {
            left: left + super::PAGE_PADDING,
            top: line.line_top + super::PAGE_PADDING,
            right: right + super::PAGE_PADDING,
            bottom: line.line_top + line.line_height + super::PAGE_PADDING,
        };
        if let Some(transform) = super::local_writing::LocalVerticalTransform::from_run(
            runs,
            buffer,
            line_index,
            page_writing_mode,
            width,
            height,
            &vertical_origins,
        ) {
            let (left, top, rect_width, rect_height) = transform.map_rect(
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
            );
            rect = ClipRect {
                left,
                top,
                right: left + rect_width,
                bottom: top + rect_height,
            };
        }
        let (scroll_x, scroll_y) = overflow_clip::scroll_translation_for_path(
            &run.element_path,
            nested_scroll,
            page_writing_mode,
        );
        rect.left -= scroll_x;
        rect.right -= scroll_x;
        rect.top -= scroll_y;
        rect.bottom -= scroll_y;
        let parent_path = run
            .element_path
            .iter()
            .copied()
            .filter(|node_id| run.element_path.last() != Some(node_id))
            .collect::<Vec<_>>();
        let clip = overflow_clip::clip_for_path(&parent_path, clips);
        let border = std::array::from_fn(|side| super::layout::resolve_border_side(style, side));
        let inline_available = if super::sizing::is_vertical(page_writing_mode) {
            height as f32 - super::PAGE_PADDING * 2.0
        } else {
            width as f32 - super::PAGE_PADDING * 2.0
        };
        let padding = super::layout::resolve_first_letter_padding(
            &run.style,
            style,
            block,
            inline_available.max(0.0),
            page_writing_mode,
        );
        rect.left -= padding[3] + border[3];
        rect.right += padding[1] + border[1];
        rect.top -= padding[0] + border[0];
        rect.bottom += padding[2] + border[2];
        let colors = style
            .border_color
            .map(|color| color.unwrap_or(style.color))
            .map(|mut color| {
                color[3] =
                    (f32::from(color[3]) * span.opacity * run.first_line_opacity.unwrap_or(1.0))
                        .round()
                        .clamp(0.0, 255.0) as u8;
                color
            });
        let opacity = span.opacity * run.first_line_opacity.unwrap_or(1.0);
        if style.border_image_source.is_some()
            && super::border_images::draw_inline(
                pixels, width, height, style, rect, border, opacity, clip,
            )
        {
            continue;
        }
        let radii = super::border_radius::resolve_radii_values(
            &style.border_radius,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
            rect,
        );
        super::border_radius::draw_rounded_border(
            pixels,
            width,
            height,
            rect,
            border,
            radii,
            style.border_style,
            colors,
            1.0,
            clip,
        );
    }
}

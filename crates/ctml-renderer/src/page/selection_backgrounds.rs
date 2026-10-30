use std::collections::HashMap;

use cosmic_text::Cursor;

use super::local_writing::{self, LocalVerticalTransform};
use super::{
    PageRasterizer,
    highlights::{first_formatted_letters, first_formatted_line_ranges},
    overflow_clip::{self, ClipRect},
    pixel::fill_rect_alpha_clipped,
};

impl PageRasterizer {
    pub(super) fn draw_selection(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        clips: &HashMap<usize, ClipRect>,
    ) {
        let Some((first, second)) = self.selection else {
            return;
        };
        let (start, end) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };
        let vertical_origins =
            local_writing::vertical_block_origins(&self.runs, &self.buffer, self.writing_mode());
        for run in self.buffer.layout_runs() {
            if run.line_i < start.line || run.line_i > end.line {
                continue;
            }
            let mut groups: Vec<(usize, usize, [u8; 4], Vec<usize>)> = Vec::new();
            for span in self.cursor_spans.iter().filter(|span| {
                span.line == run.line_i && !span.visibility_hidden && !span.user_select_none
            }) {
                if (start.line == run.line_i && span.end <= start.index)
                    || (end.line == run.line_i && span.start >= end.index)
                {
                    continue;
                }
                let span_start = if start.line == run.line_i {
                    span.start.max(start.index)
                } else {
                    span.start
                };
                let span_end = if end.line == run.line_i {
                    span.end.min(end.index)
                } else {
                    span.end
                };
                let mut background = span
                    .selection_background
                    .unwrap_or(self.selection_background_color);
                background[3] = (f32::from(background[3]) * span.opacity)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                if let Some((_, group_end, group_background, group_path)) = groups.last_mut()
                    && *group_end == span_start
                    && *group_background == background
                    && *group_path == span.element_path
                {
                    *group_end = span_end;
                } else {
                    groups.push((span_start, span_end, background, span.element_path.clone()));
                }
            }
            for (group_start, group_end, background, path) in groups {
                let local_transform = LocalVerticalTransform::from_run(
                    &self.runs,
                    &self.buffer,
                    run.line_i,
                    self.writing_mode(),
                    width,
                    height,
                    &vertical_origins,
                );
                fill_highlight(
                    run.line_i,
                    run.line_top,
                    run.line_height,
                    group_start,
                    group_end,
                    background,
                    &run,
                    pixels,
                    width,
                    height,
                    local_transform,
                    overflow_clip::clip_for_path(&path, clips),
                    overflow_clip::scroll_translation_for_path(
                        &path,
                        &self.nested_scroll,
                        self.writing_mode(),
                    )
                    .0,
                    overflow_clip::scroll_translation_for_path(
                        &path,
                        &self.nested_scroll,
                        self.writing_mode(),
                    )
                    .1,
                );
            }
        }
    }

    pub(super) fn draw_first_line_backgrounds(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        clips: &HashMap<usize, ClipRect>,
    ) {
        let first_lines = first_formatted_line_ranges(&self.buffer, &self.cursor_spans);
        let vertical_origins =
            local_writing::vertical_block_origins(&self.runs, &self.buffer, self.writing_mode());
        for run in self.buffer.layout_runs() {
            let mut groups: Vec<(usize, usize, usize, [u8; 4], Vec<usize>)> = Vec::new();
            for span in self
                .cursor_spans
                .iter()
                .filter(|span| span.line == run.line_i && !span.visibility_hidden)
            {
                let Some(block) = span.first_line_block else {
                    continue;
                };
                let Some(mut background) = span.first_line_background else {
                    continue;
                };
                background[3] = (f32::from(background[3]) * span.opacity)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let Some((line, end)) = first_lines.get(&block).copied() else {
                    continue;
                };
                if line != run.line_i || span.start >= end {
                    continue;
                }
                let span_end = span.end.min(end);
                if let Some((_, group_end, group_block, group_background, group_path)) =
                    groups.last_mut()
                    && *group_end == span.start
                    && *group_block == block
                    && *group_background == background
                    && *group_path == span.element_path
                {
                    *group_end = span_end;
                } else {
                    groups.push((
                        span.start,
                        span_end,
                        block,
                        background,
                        span.element_path.clone(),
                    ));
                }
            }
            for (group_start, group_end, _, background, path) in groups {
                let local_transform = LocalVerticalTransform::from_run(
                    &self.runs,
                    &self.buffer,
                    run.line_i,
                    self.writing_mode(),
                    width,
                    height,
                    &vertical_origins,
                );
                fill_highlight(
                    run.line_i,
                    run.line_top,
                    run.line_height,
                    group_start,
                    group_end,
                    background,
                    &run,
                    pixels,
                    width,
                    height,
                    local_transform,
                    overflow_clip::clip_for_path(&path, clips),
                    overflow_clip::scroll_translation_for_path(
                        &path,
                        &self.nested_scroll,
                        self.writing_mode(),
                    )
                    .0,
                    overflow_clip::scroll_translation_for_path(
                        &path,
                        &self.nested_scroll,
                        self.writing_mode(),
                    )
                    .1,
                );
            }
        }
    }

    pub(super) fn draw_first_letter_backgrounds(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        clips: &HashMap<usize, ClipRect>,
    ) {
        let first_letters = first_formatted_letters(&self.cursor_spans);
        let vertical_origins =
            local_writing::vertical_block_origins(&self.runs, &self.buffer, self.writing_mode());
        for run in self.buffer.layout_runs() {
            let mut groups: Vec<(usize, usize, [u8; 4], Vec<usize>)> = Vec::new();
            for span in self
                .cursor_spans
                .iter()
                .filter(|span| span.line == run.line_i && !span.visibility_hidden)
            {
                let Some(block) = span.first_letter_block else {
                    continue;
                };
                let Some(mut background) = span.first_letter_background else {
                    continue;
                };
                background[3] = (f32::from(background[3]) * span.opacity)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                if first_letters.get(&block) != Some(&(span.line, span.start, span.end)) {
                    continue;
                }
                groups.push((span.start, span.end, background, span.element_path.clone()));
            }
            for (group_start, group_end, background, path) in groups {
                let local_transform = LocalVerticalTransform::from_run(
                    &self.runs,
                    &self.buffer,
                    run.line_i,
                    self.writing_mode(),
                    width,
                    height,
                    &vertical_origins,
                );
                fill_highlight(
                    run.line_i,
                    run.line_top,
                    run.line_height,
                    group_start,
                    group_end,
                    background,
                    &run,
                    pixels,
                    width,
                    height,
                    local_transform,
                    overflow_clip::clip_for_path(&path, clips),
                    overflow_clip::scroll_translation_for_path(
                        &path,
                        &self.nested_scroll,
                        self.writing_mode(),
                    )
                    .0,
                    overflow_clip::scroll_translation_for_path(
                        &path,
                        &self.nested_scroll,
                        self.writing_mode(),
                    )
                    .1,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_highlight(
    line: usize,
    line_top: f32,
    line_height: f32,
    start: usize,
    end: usize,
    color: [u8; 4],
    run: &cosmic_text::LayoutRun<'_>,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    local_transform: Option<LocalVerticalTransform>,
    clip: Option<ClipRect>,
    translate_x: f32,
    translate_y: f32,
) {
    let start = Cursor {
        line,
        index: start,
        affinity: cosmic_text::Affinity::Before,
    };
    let end = Cursor {
        line,
        index: end,
        affinity: cosmic_text::Affinity::After,
    };
    for (left, span_width) in run.highlight(start, end) {
        let (left, top, span_width, line_height) =
            local_transform.map_or((left, line_top, span_width, line_height), |transform| {
                transform.map_rect(
                    left + super::PAGE_PADDING,
                    line_top + super::PAGE_PADDING,
                    span_width,
                    line_height,
                )
            });
        fill_rect_alpha_clipped(
            pixels,
            width,
            height,
            left - super::PAGE_PADDING - translate_x,
            top - super::PAGE_PADDING - translate_y,
            span_width,
            line_height,
            color,
            clip,
        );
    }
}

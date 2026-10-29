use std::collections::HashMap;

use cosmic_text::Cursor;

use super::{CursorSpan, PageRasterizer, pixel::fill_rect_alpha};

impl PageRasterizer {
    pub(super) fn draw_selection(&self, pixels: &mut [u8], width: u32, height: u32) {
        let Some((first, second)) = self.selection else {
            return;
        };
        let (start, end) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };
        for run in self.buffer.layout_runs() {
            if run.line_i < start.line || run.line_i > end.line {
                continue;
            }
            let mut groups = Vec::new();
            for span in self
                .cursor_spans
                .iter()
                .filter(|span| span.line == run.line_i)
            {
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
                let background = span
                    .selection_background
                    .unwrap_or(self.selection_background_color);
                if let Some((_, group_end, group_background)) = groups.last_mut()
                    && *group_end == span_start
                    && *group_background == background
                {
                    *group_end = span_end;
                } else {
                    groups.push((span_start, span_end, background));
                }
            }
            for (group_start, group_end, background) in groups {
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
                );
            }
        }
    }

    pub(super) fn draw_first_line_backgrounds(&self, pixels: &mut [u8], width: u32, height: u32) {
        let first_lines = first_formatted_line_ranges(&self.buffer, &self.cursor_spans);
        for run in self.buffer.layout_runs() {
            let mut groups: Vec<(usize, usize, usize, [u8; 4])> = Vec::new();
            for span in self
                .cursor_spans
                .iter()
                .filter(|span| span.line == run.line_i)
            {
                let Some(block) = span.first_line_block else {
                    continue;
                };
                let Some(background) = span.first_line_background else {
                    continue;
                };
                let Some((line, end)) = first_lines.get(&block).copied() else {
                    continue;
                };
                if line != run.line_i || span.start >= end {
                    continue;
                }
                let span_end = span.end.min(end);
                if let Some((_, group_end, group_block, group_background)) = groups.last_mut()
                    && *group_end == span.start
                    && *group_block == block
                    && *group_background == background
                {
                    *group_end = span_end;
                } else {
                    groups.push((span.start, span_end, block, background));
                }
            }
            for (group_start, group_end, _, background) in groups {
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
                );
            }
        }
    }
}

pub(super) fn first_formatted_line_ranges(
    buffer: &cosmic_text::Buffer,
    spans: &[CursorSpan],
) -> HashMap<usize, (usize, usize)> {
    let mut first_lines = HashMap::new();
    for run in buffer.layout_runs() {
        let Some(run_start) = run.glyphs.iter().map(|glyph| glyph.start).min() else {
            continue;
        };
        let Some(run_end) = run.glyphs.iter().map(|glyph| glyph.end).max() else {
            continue;
        };
        let mut block_ends = HashMap::new();
        for span in spans.iter().filter(|span| span.line == run.line_i) {
            let Some(block) = span.first_line_block else {
                continue;
            };
            if span.first_line_foreground.is_none() && span.first_line_background.is_none() {
                continue;
            }
            if span.end > run_start && span.start < run_end {
                let end = span.end.min(run_end);
                block_ends
                    .entry(block)
                    .and_modify(|current: &mut usize| *current = (*current).max(end))
                    .or_insert(end);
            }
        }
        for (block, end) in block_ends {
            first_lines.entry(block).or_insert((run.line_i, end));
        }
    }
    first_lines
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
        fill_rect_alpha(
            pixels,
            width,
            height,
            left,
            line_top,
            span_width,
            line_height,
            color,
        );
    }
}

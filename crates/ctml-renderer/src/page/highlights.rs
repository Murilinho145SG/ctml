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
                let mut background = span
                    .selection_background
                    .unwrap_or(self.selection_background_color);
                background[3] = (f32::from(background[3]) * span.opacity)
                    .round()
                    .clamp(0.0, 255.0) as u8;
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

    pub(super) fn draw_first_letter_backgrounds(&self, pixels: &mut [u8], width: u32, height: u32) {
        let first_letters = first_formatted_letters(&self.cursor_spans);
        for run in self.buffer.layout_runs() {
            let mut groups: Vec<(usize, usize, [u8; 4])> = Vec::new();
            for span in self
                .cursor_spans
                .iter()
                .filter(|span| span.line == run.line_i)
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
                groups.push((span.start, span.end, background));
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

    pub(super) fn draw_pseudo_decorations(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        auto_ink_mask: &[u8],
        all_ink_mask: &[u8],
    ) {
        let first_lines = first_formatted_line_ranges(&self.buffer, &self.cursor_spans);
        let first_letters = first_formatted_letters(&self.cursor_spans);
        for run in self.buffer.layout_runs() {
            let mut first_line_groups: Vec<(
                usize,
                usize,
                usize,
                crate::css_style::PseudoDecoration,
            )> = Vec::new();
            for span in self
                .cursor_spans
                .iter()
                .filter(|span| span.line == run.line_i)
            {
                if let Some(block) = span.first_line_block
                    && let Some((line, end)) = first_lines.get(&block).copied()
                    && line == run.line_i
                    && span.start < end
                    && let Some(decoration) = span.first_line_decoration
                {
                    let span_end = span.end.min(end);
                    if let Some((_, group_end, group_block, group_decoration)) =
                        first_line_groups.last_mut()
                        && *group_end == span.start
                        && *group_block == block
                        && *group_decoration == decoration
                    {
                        *group_end = span_end;
                    } else {
                        first_line_groups.push((span.start, span_end, block, decoration));
                    }
                }
                if let Some(block) = span.first_letter_block
                    && first_letters.get(&block) == Some(&(span.line, span.start, span.end))
                    && let Some(decoration) = span.first_letter_decoration
                {
                    draw_decoration(
                        &run,
                        span.start,
                        span.end,
                        decoration,
                        pixels,
                        width,
                        height,
                        auto_ink_mask,
                        all_ink_mask,
                    );
                }
            }
            for (start, end, _, decoration) in first_line_groups {
                draw_decoration(
                    &run,
                    start,
                    end,
                    decoration,
                    pixels,
                    width,
                    height,
                    auto_ink_mask,
                    all_ink_mask,
                );
            }
        }
    }
}

fn draw_decoration(
    run: &cosmic_text::LayoutRun<'_>,
    start: usize,
    end: usize,
    decoration: crate::css_style::PseudoDecoration,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    auto_ink_mask: &[u8],
    all_ink_mask: &[u8],
) {
    if start >= end || decoration.thickness <= 0.0 {
        return;
    }
    let start = Cursor {
        line: run.line_i,
        index: start,
        affinity: cosmic_text::Affinity::Before,
    };
    let end = Cursor {
        line: run.line_i,
        index: end,
        affinity: cosmic_text::Affinity::After,
    };
    let color = cosmic_text::Color::rgba(
        decoration.color[0],
        decoration.color[1],
        decoration.color[2],
        decoration.color[3],
    );
    let thickness = decoration.thickness.round().max(1.0) as u32;
    let mut renderer = super::pixel::PixelRenderer {
        pixels,
        width,
        height,
        skip_ink_mask: match decoration.skip_ink {
            crate::style::TextDecorationSkipInk::None => None,
            crate::style::TextDecorationSkipInk::Auto => Some(auto_ink_mask),
            crate::style::TextDecorationSkipInk::All => Some(all_ink_mask),
        },
    };
    for (left, span_width) in run.highlight(start, end) {
        let width_exact = span_width.max(0.0);
        let line_width = width_exact as u32;
        let tail_coverage = ((width_exact - line_width as f32) * 255.0).round() as u8;
        if line_width == 0 {
            continue;
        }
        for (enabled, y) in [
            (
                decoration.underline,
                run.line_y + 1.0 + decoration.underline_offset.unwrap_or(0.0),
            ),
            (decoration.overline, run.line_y - run.line_height * 0.8),
            (decoration.line_through, run.line_y - run.line_height * 0.3),
        ] {
            if enabled {
                super::decorations::draw_line(
                    &mut renderer,
                    left as i32,
                    y.round() as i32,
                    line_width,
                    tail_coverage,
                    thickness,
                    decoration.style,
                    color,
                );
            }
        }
    }
}

pub(super) fn first_formatted_letters(
    spans: &[CursorSpan],
) -> HashMap<usize, (usize, usize, usize)> {
    let mut first_letters = HashMap::new();
    for span in spans {
        let Some(block) = span.first_letter_block else {
            continue;
        };
        if span.first_letter_foreground.is_none()
            && span.first_letter_background.is_none()
            && span.first_letter_decoration.is_none()
            && span.first_letter_opacity.is_none()
        {
            continue;
        }
        first_letters
            .entry(block)
            .and_modify(|(line, start, end): &mut (usize, usize, usize)| {
                if *line == span.line {
                    *start = (*start).min(span.start);
                    *end = (*end).max(span.end);
                }
            })
            .or_insert((span.line, span.start, span.end));
    }
    first_letters
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
            if span.first_line_foreground.is_none()
                && span.first_line_background.is_none()
                && span.first_line_decoration.is_none()
                && span.first_line_font_style.is_none()
            {
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

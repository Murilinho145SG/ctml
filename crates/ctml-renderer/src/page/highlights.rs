use std::collections::HashMap;

use cosmic_text::Cursor;

use super::local_writing::{self, LocalVerticalTransform};
use super::{
    CursorSpan, PageRasterizer,
    overflow_clip::{self, ClipRect},
};

impl PageRasterizer {
    pub(super) fn draw_pseudo_decorations(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        auto_ink_mask: &[u8],
        all_ink_mask: &[u8],
        clips: &HashMap<usize, ClipRect>,
    ) {
        let first_lines = first_formatted_line_ranges(&self.buffer, &self.cursor_spans);
        let first_letters = first_formatted_letters(&self.cursor_spans);
        let vertical_origins =
            local_writing::vertical_block_origins(&self.runs, &self.buffer, self.writing_mode());
        for run in self.buffer.layout_runs() {
            let mut first_line_groups: Vec<(
                usize,
                usize,
                usize,
                crate::css_style::PseudoDecoration,
                Vec<usize>,
            )> = Vec::new();
            for span in self
                .cursor_spans
                .iter()
                .filter(|span| span.line == run.line_i && !span.visibility_hidden)
            {
                if let Some(block) = span.first_line_block
                    && let Some((line, end)) = first_lines.get(&block).copied()
                    && line == run.line_i
                    && span.start < end
                    && let Some(decoration) = span.first_line_decoration
                {
                    let span_end = span.end.min(end);
                    if let Some((_, group_end, group_block, group_decoration, group_path)) =
                        first_line_groups.last_mut()
                        && *group_end == span.start
                        && *group_block == block
                        && *group_decoration == decoration
                        && *group_path == span.element_path
                    {
                        *group_end = span_end;
                    } else {
                        first_line_groups.push((
                            span.start,
                            span_end,
                            block,
                            decoration,
                            span.element_path.clone(),
                        ));
                    }
                }
                if let Some(block) = span.first_letter_block
                    && first_letters.get(&block) == Some(&(span.line, span.start, span.end))
                    && let Some(decoration) = span.first_letter_decoration
                {
                    let (scroll_x, scroll_y) = overflow_clip::scroll_translation_for_path(
                        &span.element_path,
                        &self.nested_scroll,
                        self.writing_mode(),
                    );
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
                        overflow_clip::clip_for_path(&span.element_path, clips),
                        LocalVerticalTransform::from_run(
                            &self.runs,
                            &self.buffer,
                            run.line_i,
                            self.writing_mode(),
                            width,
                            height,
                            &vertical_origins,
                        ),
                        scroll_x,
                        scroll_y,
                    );
                }
            }
            for (start, end, _, decoration, path) in first_line_groups {
                let (scroll_x, scroll_y) = overflow_clip::scroll_translation_for_path(
                    &path,
                    &self.nested_scroll,
                    self.writing_mode(),
                );
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
                    overflow_clip::clip_for_path(&path, clips),
                    LocalVerticalTransform::from_run(
                        &self.runs,
                        &self.buffer,
                        run.line_i,
                        self.writing_mode(),
                        width,
                        height,
                        &vertical_origins,
                    ),
                    scroll_x,
                    scroll_y,
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
    clip_rect: Option<ClipRect>,
    local_transform: Option<LocalVerticalTransform>,
    translate_x: f32,
    translate_y: f32,
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
        local_transform,
        clip_rect,
        translate_x,
        translate_y,
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

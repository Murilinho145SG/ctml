use cosmic_text::Buffer;

use crate::{page::pixel::fill_rect_alpha, style::TextRun};

pub(super) fn draw(
    runs: &[TextRun],
    buffer: &Buffer,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    skip_root_background: bool,
) {
    let style_ranges = text_style_ranges(runs);
    let line_starts = text_line_starts(runs);
    for line in buffer.layout_runs() {
        let line_start = line_starts.get(line.line_i).copied().unwrap_or(0);
        let mut start = 0;
        while start < line.glyphs.len() {
            let glyph = &line.glyphs[start];
            let style_index = style_index_at(&style_ranges, line_start + glyph.start);
            let Some(style) = style_index.and_then(|index| runs.get(index).map(|run| &run.style))
            else {
                start += 1;
                continue;
            };
            let layer_start =
                usize::from(skip_root_background && !style.background_layers.is_empty());
            if layer_start == style.background_layers.len() {
                start += 1;
                continue;
            }
            let mut end = start + 1;
            while end < line.glyphs.len() {
                let next = &line.glyphs[end];
                if style_index_at(&style_ranges, line_start + next.start) != style_index
                    || next.level != glyph.level
                {
                    break;
                }
                end += 1;
            }
            let fragment = &line.glyphs[start..end];
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
                    fill_rect_alpha(
                        pixels,
                        width,
                        height,
                        left,
                        line.line_top,
                        right - left,
                        line.line_height,
                        color,
                    );
                }
            }
            start = end;
        }
    }
}

fn text_style_ranges(runs: &[TextRun]) -> Vec<std::ops::Range<usize>> {
    let mut offset = 0;
    runs.iter()
        .map(|run| {
            let range = offset..offset + run.text.len();
            offset = range.end;
            range
        })
        .collect()
}

fn style_index_at(ranges: &[std::ops::Range<usize>], offset: usize) -> Option<usize> {
    ranges
        .iter()
        .position(|range| range.start <= offset && offset < range.end)
}

fn text_line_starts(runs: &[TextRun]) -> Vec<usize> {
    let mut starts = vec![0];
    let mut offset = 0;
    for run in runs {
        for (index, character) in run.text.char_indices() {
            if character == '\n' {
                starts.push(offset + index + character.len_utf8());
            }
        }
        offset += run.text.len();
    }
    starts
}

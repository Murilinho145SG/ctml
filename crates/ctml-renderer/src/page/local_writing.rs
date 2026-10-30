use std::collections::HashMap;

use cosmic_text::Buffer;

use crate::{
    page::sizing::is_vertical,
    style::{TextRun, WritingMode},
};

pub(super) fn vertical_block_origins(
    runs: &[TextRun],
    buffer: &Buffer,
    page_writing_mode: WritingMode,
) -> HashMap<usize, f32> {
    let mut origins = HashMap::new();
    if page_writing_mode != WritingMode::HorizontalTb {
        return origins;
    }

    let style_ranges = text_style_ranges(runs);
    let line_starts = text_line_starts(runs);
    for line in buffer.layout_runs() {
        let line_start = line_starts.get(line.line_i).copied().unwrap_or(0);
        for glyph in line.glyphs {
            let Some(index) = style_index_at(&style_ranges, line_start + glyph.start) else {
                continue;
            };
            let Some(run) = runs.get(index) else {
                continue;
            };
            if !is_vertical(run.style.writing_mode) {
                continue;
            }
            if let Some(block) = run.first_line_block {
                origins
                    .entry(block)
                    .and_modify(|origin: &mut f32| *origin = origin.min(line.line_y))
                    .or_insert(line.line_y);
            }
        }
    }
    origins
}

pub(super) fn text_style_ranges(runs: &[TextRun]) -> Vec<std::ops::Range<usize>> {
    let mut offset = 0;
    runs.iter()
        .map(|run| {
            let range = offset..offset + run.text.len();
            offset = range.end;
            range
        })
        .collect()
}

pub(super) fn style_index_at(ranges: &[std::ops::Range<usize>], offset: usize) -> Option<usize> {
    ranges
        .iter()
        .position(|range| range.start <= offset && offset < range.end)
}

pub(super) fn text_line_starts(runs: &[TextRun]) -> Vec<usize> {
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

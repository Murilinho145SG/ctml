use std::collections::HashMap;

use cosmic_text::Buffer;

use crate::{
    page::sizing::is_vertical,
    style::{TextRun, WritingMode},
};

#[derive(Clone, Copy)]
pub(super) struct LocalVerticalTransform {
    mode: WritingMode,
    origin_y: f32,
    width: u32,
    height: u32,
}

impl LocalVerticalTransform {
    pub(super) fn new(mode: WritingMode, origin_y: f32, width: u32, height: u32) -> Self {
        Self {
            mode,
            origin_y,
            width,
            height,
        }
    }

    pub(super) fn map_rect(
        self,
        left: f32,
        top: f32,
        rect_width: f32,
        rect_height: f32,
    ) -> (f32, f32, f32, f32) {
        let source_x = crate::page::PAGE_PADDING;
        let source_y = self.origin_y + crate::page::PAGE_PADDING;
        match self.mode {
            WritingMode::VerticalRl => {
                let target_x = self.width as f32 - source_y;
                let target_y = source_x;
                (
                    target_x - (top + rect_height - source_y),
                    target_y + (left - source_x),
                    rect_height,
                    rect_width,
                )
            }
            WritingMode::VerticalLr => {
                let target_x = source_y;
                let target_y = self.height as f32 - source_x;
                (
                    target_x + (top - source_y),
                    target_y - (left + rect_width - source_x),
                    rect_height,
                    rect_width,
                )
            }
            WritingMode::HorizontalTb => (left, top, rect_width, rect_height),
        }
    }

    pub(super) fn from_run(
        runs: &[TextRun],
        buffer: &Buffer,
        line_index: usize,
        page_writing_mode: WritingMode,
        width: u32,
        height: u32,
        origins: &HashMap<usize, f32>,
    ) -> Option<Self> {
        if page_writing_mode != WritingMode::HorizontalTb {
            return None;
        }
        let line = buffer
            .layout_runs()
            .find(|line| line.line_i == line_index)?;
        let glyph = line.glyphs.first()?;
        let ranges = text_style_ranges(runs);
        let starts = text_line_starts(runs);
        let index = style_index_at(
            &ranges,
            starts.get(line_index).copied().unwrap_or(0) + glyph.start,
        )?;
        let run = runs.get(index)?;
        if !is_vertical(run.style.writing_mode) {
            return None;
        }
        let block = run.first_line_block?;
        let origin_y = origins.get(&block).copied()?;
        Some(Self::new(run.style.writing_mode, origin_y, width, height))
    }
}

pub(super) fn vertical_block_origins(
    runs: &[TextRun],
    buffer: &Buffer,
    page_writing_mode: WritingMode,
    flex_wrap_line_offsets: &HashMap<(usize, usize), f32>,
) -> HashMap<usize, f32> {
    let mut origins = HashMap::new();
    if page_writing_mode != WritingMode::HorizontalTb {
        return origins;
    }

    let style_ranges = text_style_ranges(runs);
    let line_starts = text_line_starts(runs);
    for line in super::adjusted_layout_runs(buffer, flex_wrap_line_offsets) {
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

use std::collections::HashMap;

use crate::{
    page::{FlexLayoutOffsetKey, PageRasterizer},
    style::FlexAlignment,
};

#[derive(Clone, Copy)]
struct ItemStyle {
    item_id: usize,
    start: usize,
    end: usize,
    line_height: f32,
    alignment: FlexAlignment,
}

#[derive(Default)]
struct ItemLine {
    line_height: f32,
    baseline_offset: f32,
    alignment: FlexAlignment,
    glyph_starts: Vec<usize>,
}

pub(super) fn apply_flex_row_alignment(page: &mut PageRasterizer) {
    if page.writing_mode() != crate::style::WritingMode::HorizontalTb {
        page.flex_layout_offsets
            .retain(|(_, key), _| matches!(key, FlexLayoutOffsetKey::Line(_)));
        return;
    }
    let mut styles_by_paragraph = HashMap::<usize, Vec<ItemStyle>>::new();
    let mut paragraph = 0;
    let mut byte_offset = 0;
    for run in &page.runs {
        let mut range_start = byte_offset;
        for character in run.text.chars() {
            if character == '\n' {
                add_item_style(
                    &mut styles_by_paragraph,
                    paragraph,
                    range_start,
                    byte_offset,
                    run,
                );
                paragraph += 1;
                byte_offset = 0;
                range_start = 0;
                continue;
            }
            byte_offset += character.len_utf8();
        }
        add_item_style(
            &mut styles_by_paragraph,
            paragraph,
            range_start,
            byte_offset,
            run,
        );
    }

    page.flex_layout_offsets
        .retain(|(_, key), _| matches!(key, FlexLayoutOffsetKey::Line(_)));

    for line in page.buffer.layout_runs() {
        let Some(item_styles) = styles_by_paragraph.get(&line.line_i) else {
            continue;
        };
        let mut items = HashMap::<usize, ItemLine>::new();
        for glyph in line.glyphs.iter() {
            let Some(style) = item_styles
                .iter()
                .find(|style| glyph.start >= style.start && glyph.start < style.end)
            else {
                continue;
            };
            let item = items.entry(style.item_id).or_default();
            item.line_height = item.line_height.max(style.line_height);
            item.alignment = style.alignment;
            if let Some(font) = page.font_system.get_font(glyph.font_id, glyph.font_weight) {
                let metrics = font.metrics();
                let units_per_em = f32::from(metrics.units_per_em).max(1.0);
                let ascent = metrics.ascent / units_per_em * glyph.font_size;
                let descent = -metrics.descent / units_per_em * glyph.font_size;
                let leading = (style.line_height - ascent - descent).max(0.0) / 2.0;
                item.baseline_offset = item.baseline_offset.max(leading + ascent);
            }
            item.glyph_starts.push(glyph.start);
        }
        for item in items.values() {
            let free_space = (line.line_height - item.line_height).max(0.0);
            let baseline_offset = item.baseline_offset - (line.line_y - line.line_top);
            let offset = match item.alignment {
                FlexAlignment::Center => free_space / 2.0 + baseline_offset,
                FlexAlignment::FlexEnd | FlexAlignment::End => free_space + baseline_offset,
                FlexAlignment::FlexStart
                | FlexAlignment::Start
                | FlexAlignment::Stretch
                | FlexAlignment::SpaceBetween
                | FlexAlignment::SpaceAround
                | FlexAlignment::SpaceEvenly => baseline_offset,
            };
            if offset.abs() <= 0.01 {
                continue;
            }
            for start in &item.glyph_starts {
                page.flex_layout_offsets
                    .insert((line.line_i, FlexLayoutOffsetKey::Glyph(*start)), offset);
            }
        }
    }
}

fn add_item_style(
    styles_by_paragraph: &mut HashMap<usize, Vec<ItemStyle>>,
    paragraph: usize,
    start: usize,
    end: usize,
    run: &crate::style::TextRun,
) {
    if start >= end {
        return;
    }
    if run.style.writing_mode != crate::style::WritingMode::HorizontalTb {
        return;
    }
    let (Some(item_id), Some(alignment)) = (
        run.style.flex_item_cross_id,
        run.style.flex_item_cross_alignment,
    ) else {
        return;
    };
    styles_by_paragraph
        .entry(paragraph)
        .or_default()
        .push(ItemStyle {
            item_id,
            start,
            end,
            line_height: run.style.line_height.max(0.0),
            alignment,
        });
}

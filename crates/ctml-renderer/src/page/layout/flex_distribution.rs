use std::collections::HashMap;

use crate::page::PageRasterizer;

struct FlexGapLine {
    positions: Vec<usize>,
    available_width: f32,
    container_id: Option<usize>,
    mixed_containers: bool,
}

pub(super) fn apply_distributed_flex_gaps(page: &mut PageRasterizer, content_width: f32) -> bool {
    let mut markers_by_line = HashMap::<usize, FlexGapLine>::new();
    let mut line_index = 0;
    let mut byte_index = 0;
    for run in &page.runs {
        for character in run.text.chars() {
            if character == '\n' {
                line_index += 1;
                byte_index = 0;
            } else {
                if character == '\u{200b}' && run.style.flex_distributed_gap {
                    let container_id = run.element_path.last().copied();
                    let line = markers_by_line.entry(line_index).or_insert_with(|| {
                        let writing_mode = page.writing_mode();
                        let inline_size = super::super::sizing::resolve_layout_width(
                            &run.style.layout_sizes,
                            content_width,
                            writing_mode,
                        );
                        let (padding_start, _) = super::super::sizing_padding::inline_padding(
                            &run.style.layout_sizes,
                            content_width,
                            writing_mode,
                        );
                        let (border_start, _) = super::super::sizing_padding::inline_borders(
                            &run.style.layout_sizes,
                            content_width,
                            writing_mode,
                        );
                        let (margin_start, _) = super::super::sizing_padding::inline_margin(
                            &run.style.layout_sizes,
                            content_width,
                            writing_mode,
                        );
                        FlexGapLine {
                            positions: Vec::new(),
                            available_width: inline_size
                                + padding_start
                                + border_start
                                + margin_start,
                            container_id,
                            mixed_containers: false,
                        }
                    });
                    if line.container_id != container_id {
                        line.mixed_containers = true;
                    }
                    line.positions.push(byte_index);
                }
                byte_index += character.len_utf8();
            }
        }
    }

    let extras = page
        .buffer
        .layout_runs()
        .filter_map(|line| {
            let group = markers_by_line.get(&line.line_i)?;
            if group.mixed_containers {
                return None;
            }
            let marker_count = line
                .glyphs
                .iter()
                .filter(|glyph| group.positions.contains(&glyph.start))
                .count();
            if marker_count == 0 || marker_count != group.positions.len() {
                return None;
            }
            let left = line
                .glyphs
                .iter()
                .map(|glyph| glyph.x)
                .fold(f32::INFINITY, f32::min);
            let right = line
                .glyphs
                .iter()
                .map(|glyph| glyph.x + glyph.w)
                .fold(f32::NEG_INFINITY, f32::max);
            let used_width = right - left;
            let extra = (group.available_width - used_width).max(0.0) / marker_count as f32;
            Some((line.line_i, group.container_id?, extra))
        })
        .map(|(line, container, extra)| ((line, container), extra))
        .collect::<HashMap<_, _>>();

    let mut changed = false;
    let mut line_index = 0;
    for run in &mut page.runs {
        if run.style.flex_distributed_gap && !run.style.flex_distributed_gap_applied {
            if let Some(container_id) = run.element_path.last().copied()
                && let Some(extra) = extras.get(&(line_index, container_id)).copied()
            {
                run.style.letter_spacing += extra;
                run.style.letter_spacing_px = true;
                run.style.flex_distributed_gap_applied = true;
                changed |= extra > 0.0;
            }
        }
        line_index += run.text.matches('\n').count();
    }
    changed
}
